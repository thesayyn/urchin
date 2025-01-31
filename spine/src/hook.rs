use std::{
    alloc::{alloc, Layout},
    ffi::c_void,
    ptr::{null, null_mut},
};

use anyhow::Context;
use dashmap::DashMap;
use jvmti::{
    bytecode::{classfile, printer::ClassfilePrinter, Constant},
    environment::jvmti::JVMTI,
};
use lazy_static::lazy_static;



#[derive(Clone, Debug)]
struct MethodInfo {
    name: String,
    signature: String,
    access_flags: i32,
}

lazy_static! {
    static ref CLASS_FILE_CACHE: DashMap<String, classfile::Classfile> = DashMap::new();
    static ref ORIGINAL_CLASSES: DashMap<String, jni::objects::JClass<'static>> = DashMap::new();
    static ref ORIGINAL_CLASS_FILE_CACHE: DashMap<String, Vec<u8>> = DashMap::new();
}

unsafe fn get_method_info(
    jvmti: jvmti::native::JVMTIEnvPtr,
    method: &jvmti::method::MethodId,
) -> anyhow::Result<MethodInfo> {
    let mut method_name = null_mut();
    let method_ptr = &mut method_name;

    let mut signature: jvmti::native::MutString = null_mut();
    let signature_ptr = &mut signature;

    let mut generic_sig: jvmti::native::MutString = null_mut();
    let generic_sig_ptr = &mut generic_sig;

    (**jvmti)
        .GetMethodName
        .context("failed to get method name")?(
        jvmti,
        method.native_id,
        method_ptr,
        signature_ptr,
        generic_sig_ptr,
    );

    Ok(MethodInfo {
        name: jvmti::util::stringify(*method_ptr),
        signature: jvmti::util::stringify(*signature_ptr),
        access_flags: *signature_ptr as i32,
    })
}

unsafe fn get_class_name(
    env: &mut jni::JNIEnv<'_>,
    class: &jvmti::class::ClassId,
) -> anyhow::Result<String> {
    let klass = env.find_class("java/lang/Class")?;
    let get_name = env.get_method_id(klass, "getName", "()Ljava/lang/String;")?;
    let name = env
        .call_method_unchecked(
            jni::objects::JObject::from_raw(class.native_id as *mut jni::sys::_jobject),
            get_name,
            jni::signature::ReturnType::Object,
            &[],
        )?
        .l()?;

    Ok(env
        .get_string(&jni::objects::JString::from_raw(name.into_raw()))?
        .to_str()?
        .replace(".", "/"))
}

// Technically, we don't even need this as we can just query the class file. (maybe)?
fn class_file_load_hook(event: jvmti::runtime::ClassFileLoadEvent) -> Option<Vec<u8>> {
    if !CLASS_FILE_CACHE.contains_key(&event.class_name) {
        CLASS_FILE_CACHE.insert(event.class_name, event.class);
    }
    None
}

unsafe fn reapply_class(
    jvmti: jvmti::native::JVMTIEnvPtr,
    class: &jvmti::class::ClassId,
    class_file: &mut classfile::Classfile,
) -> anyhow::Result<()> {
    for method in class_file.methods.iter_mut() {
        let name = ClassfilePrinter::resolve_utf8(&method.name_index, &class_file.constant_pool);
        let descriptor =
            ClassfilePrinter::resolve_utf8(&method.descriptor_index, &class_file.constant_pool);

        if name != "getSummary" && name != "getDiff" {
            continue;
        }

        method.access_flags.flags |= classfile::MethodAccessFlags::Native as u16;
        for i in 0..method.attributes.len() {
            if let classfile::Attribute::Code {
                attributes,
                code,
                exception_table,
                max_locals,
                max_stack,
            } = method.attributes.get(i).unwrap()
            {
                method.attributes.remove(i);
                break;
            };
        }
    }

    let mut class_data = Vec::<u8>::new();
    let mut writer = jvmti::bytecode::writer::ClassWriter::new(&mut class_data);
    writer.write_class(class_file)?;

    let class_definition = jvmti::native::jvmti_native::jvmtiClassDefinition {
        klass: class.native_id,
        class_bytes: class_data.as_ptr(),
        class_byte_count: class_data.len() as i32,
    };
    eprintln!("redefined class {:x}", ByteBuf(&class_data));
    let redefine_classes = (**jvmti).RedefineClasses.context("")?;
    let error = redefine_classes(jvmti, 1, &class_definition);
    if error != jvmti::native::jvmti_native::JVMTI_ERROR_NONE {
        anyhow::bail!("error while redefining classes {}", error)
    }
    return Ok(());
}

struct ByteBuf<'a>(&'a [u8]);

impl<'a> std::fmt::LowerHex for ByteBuf<'a> {
    fn fmt(&self, fmtr: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for byte in self.0 {
            fmtr.write_fmt(format_args!("{:02x}", byte))?;
        }
        Ok(())
    }
}


pub unsafe fn jnihook_attach(
    jvm: *mut jni::sys::JavaVM,
    method: jni::objects::JMethodID,
    native_hook_method: *const c_void,
) -> anyhow::Result<(jni::objects::JClass<'static>, jni::objects::JMethodID)> {
    let jvmti_raw =
        alloc(Layout::new::<jvmti::native::jvmti_native::jvmtiEnv>()) as *mut *mut c_void;

    let get_env = (**jvm).GetEnv.context("failed to get JVM::GetEnv")?;
    if get_env(
        jvm,
        jvmti_raw,
        jvmti::native::jvmti_native::JVMTI_VERSION_1_2 as i32,
    ) != jvmti::native::jvmti_native::JVMTI_ERROR_NONE as i32
    {
        anyhow::bail!("Failed to get JVMTI environment");
    }

    let mut jvmti =
        jvmti::environment::jvmti::JVMTIEnvironment::new(*jvmti_raw as jvmti::native::JVMTIEnvPtr);

    let mut capabilities = jvmti.get_capabilities();
    capabilities.can_redefine_classes = true;
    capabilities.can_redefine_any_class = true;
    capabilities.can_retransform_classes = true;
    capabilities.can_retransform_any_class = true;
    capabilities.can_suspend = true;

    let Result::Ok(_) = jvmti.add_capabilities(&capabilities) else {
        anyhow::bail!("failed to add capabilities");
    };

    let callbacks = jvmti::event::EventCallbacks {
        class_file_load_hook: Some(class_file_load_hook),
        ..Default::default()
    };
    jvmti.set_event_callbacks(callbacks);

    let method = jvmti::method::MethodId {
        native_id: method.into_raw() as jvmti::native::JavaMethod,
    };

    let jvm = jni::JavaVM::from_raw(jvm).context("failed to get JavaVM")?;
    let mut env = jvm.get_env().context("failed to get JNI::GetEnv")?;

    let jvmti_raw = *jvmti_raw as jvmti::native::JVMTIEnvPtr;

    let Ok(class) = jvmti.get_method_declaring_class(&method) else {
        anyhow::bail!("Failed to get class");
    };

    let class_name = get_class_name(&mut env, &class)?;

    let method_info = get_method_info(jvmti_raw, &method)?;


    // Disabled for now; in later stages we might want to prevent JVM from doing anything while we are hooking.
    if false {
        let mut current_thread: jvmti::native::jvmti_native::jthread = null_mut();

        (**jvmti_raw)
            .GetCurrentThread
            .context("JVMTI::GetCurrentThread")?(jvmti_raw, &mut current_thread);
    
        let mut all_threads: *mut jvmti::native::jvmti_native::jthread = null_mut();
        let mut thread_count = 0;
    
        (**jvmti_raw)
            .GetAllThreads
            .context("JVMTI::GetAllThreads")?(jvmti_raw, &mut thread_count, &mut all_threads);
        let threads_array = unsafe { std::slice::from_raw_parts(all_threads, thread_count as usize) };
    
        eprintln!("thread count: {:?} {:?}", thread_count, threads_array);
        eprintln!("current thread: {:?}", current_thread);
    
        for (i, th) in threads_array.iter().enumerate() {
            if (i == thread_count as usize - 1) {
                eprintln!("skipping first thread");
                continue;
            }
            (**jvmti_raw)
                .SuspendThread
                .context("JVMTI::SuspendThread")?(jvmti_raw, *th);
            eprintln!("suspended thread: {:?}", th);
        }
    }

    

    // If we don't have the bytecode for the class, we need to retransform the class to get it.
    if !CLASS_FILE_CACHE.contains_key(&class_name) {
        eprintln!("class file not found in cache, retransforming class");

        assert!(
            jvmti
                .set_event_notification_mode(jvmti::event::VMEvent::ClassFileLoadHook, true)
                .is_none(),
            "setting event notification mode failed"
        );
        let retransform_classes = (**jvmti_raw)
            .RetransformClasses
            .context("failed to get RetransformClasses")?;
        let result = retransform_classes(jvmti_raw, 1, &class.native_id);
        if result != jvmti::native::jvmti_native::JVMTI_ERROR_NONE {
            anyhow::bail!("failed to retransform classes");
        }
        assert!(
            jvmti
                .set_event_notification_mode(jvmti::event::VMEvent::ClassFileLoadHook, false)
                .is_none(),
            "setting event notification mode failed"
        );

        if !CLASS_FILE_CACHE.contains_key(&class_name) {
            anyhow::bail!("failed to find the class due to class file load hook not being called");
        }
    }

    eprintln!("Waiting for class file load hook to be called");

    // put the original class file in the cache for later use, in case we need to reapply it
    if !ORIGINAL_CLASSES.contains_key(&class_name) {
        // we need to create a copy of the class by converting it to a byte array first
        let mut class_data = Vec::<u8>::with_capacity(5385);
        let mut writer = jvmti::bytecode::writer::ClassWriter::new(&mut class_data);
        let klass = CLASS_FILE_CACHE.get(&class_name).unwrap();
        writer.write_class(&klass)?;

        // write the class bytecode into the cache
        ORIGINAL_CLASS_FILE_CACHE.insert(class_name.clone(), class_data.clone());

        eprintln!(
            "original class has {} methods and bytecode of {:x}",
            &klass.methods.len(),
            ByteBuf(&class_data)
        );

        // DO NOT REMOVE: its here to prevent deadlocking.
        drop(klass);

        let mut cursor = std::io::Cursor::new(class_data);
        let mut class_file = jvmti::bytecode::reader::ClassReader::read_class(&mut cursor)
            .context("Failed to read class")?;

        eprintln!("done reading class");

        let class_copy_name = format!("{}", class_name);
        let class_copy_source_name =
            format!("{}.java", class_copy_name.split("/").last().unwrap());
        
        let mut constant_mutations = Vec::<(usize, Constant)>::new();

        for attr in class_file.attributes.iter() {
            let classfile::Attribute::SourceFile(idx) = attr else {
                continue;
            };
            constant_mutations.push((idx.idx, classfile::Constant::Utf8(class_copy_source_name.clone().into())));
        }

        for constant in class_file.constant_pool.constants.iter() {
            match constant {
                Constant::Class(idx) =>{
                    let class_ci = ClassfilePrinter::resolve_utf8(&idx, &class_file.constant_pool);
                    if class_ci == class_name {
                        constant_mutations.push((idx.idx, Constant::Utf8(class_copy_name.clone().into())));
                    }
                },
                // Constant::NameAndType { name_index, descriptor_index } => {
                //     let descriptor_ci = ClassfilePrinter::resolve_utf8(&descriptor_index, &class_file.constant_pool);
                //     if descriptor_ci.contains(&class_name) {
                //         let class_desc = format!("L{};", class_name);
                //         let class_copy_desc = format!("L{};", class_copy_name);
                //         let new_desc = descriptor_ci.replace(&class_desc, &class_copy_desc);
                //         constant_mutations.push((descriptor_index.idx, Constant::Utf8(new_desc.into())));
                //     }
                // },
                _ => {}
            }
        }
        
        for (idx, constant) in constant_mutations {
            eprintln!("mutating constant pool entry {}", idx);
            class_file.constant_pool.constants[idx] = constant;
        }

        let mut class_data = Vec::<u8>::new();
        let mut writer = jvmti::bytecode::writer::ClassWriter::new(&mut class_data);
        writer.write_class(&class_file)?;

        let mut class_loader = alloc(Layout::new::<jni::sys::jobject>())
            as *mut jvmti::native::jvmti_native::Struct__jobject;
        let get_class_loader = (**jvmti_raw)
            .GetClassLoader
            .context("failed to get GetClassLoader function")?;

        if get_class_loader(jvmti_raw, class.native_id, &mut class_loader)
            != jvmti::native::jvmti_native::JVMTI_ERROR_NONE
        {
            anyhow::bail!("failed to get class loader");
        }

        let define_class = (**env.get_raw())
            .DefineClass
            .context("failed to get DefineClass function")?;

        let copied_class = define_class(
            env.get_raw(),
            null(),
            class_loader as *mut jni::sys::_jobject,
            class_data.as_ptr() as *const jni::sys::jbyte,
            class_data.len() as i32,
        );

        ORIGINAL_CLASSES.insert(
            class_name.clone(),
            jni::objects::JClass::from_raw(copied_class),
        );
    }

    // ensure class copying was successful
    assert!(
        ORIGINAL_CLASSES.contains_key(&class_name),
        "failed to create a copy of the class"
    );

    reapply_class(
        jvmti_raw,
        &class,
        &mut CLASS_FILE_CACHE.get_mut(&class_name).unwrap(),
    )?;

    let native_method = jni::NativeMethod {
        name: method_info.name.clone().into(),
        sig: method_info.signature.clone().into(),
        fn_ptr: native_hook_method as *mut c_void,
    };

    eprintln!("repplied class");

    // After replacing the original class with the hooked class, we need to register the native method.
    env.register_native_methods(
        jni::objects::JClass::from_raw(class.native_id as *mut jni::sys::_jobject),
        &[native_method],
    )?;

    eprintln!("registered native method");

    // Disabled for now; in later stages we might want to prevent JVM from doing anything while we are hooking.
    // for (i, th) in threads_array.iter().enumerate() {
    //     if (i == thread_count as usize - 1) {
    //         eprintln!("skipping first thread");
    //         continue;
    //     }
    //     (**jvmti_raw)
    //         .ResumeThread
    //         .context("JVMTI::SuspendThread")?(jvmti_raw, *th);
    //     eprintln!("unsuspended thread: {:?}", th);
    // }


    let class = jni::objects::JClass::from_raw(ORIGINAL_CLASSES.get(&class_name).unwrap().as_raw());

    if (method_info.access_flags & jvmti::bytecode::classfile::MethodAccessFlags::Static as i32) == jvmti::bytecode::classfile::MethodAccessFlags::Static as i32 {
        anyhow::bail!("static methods are not supported yet");
    } else {
        let method_id = env.get_method_id(&class, method_info.name, method_info.signature)?;
        return Ok((class, method_id))
    }
    
    anyhow::bail!("failed to get method id");
}
