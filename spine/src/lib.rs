mod hook;
use anyhow::Context;
use ctor::ctor;
use dashmap::DashMap;
use hook::jnihook_attach;
use jni::{objects::JObject, sys, JNIEnv};
use jvmti::method;
use lazy_static::lazy_static;
use std::{
    ops::Deref,
    ptr::null_mut,
    sync::{LazyLock, Mutex},
    thread,
};

pub type JniGetCreatedJavaVms = unsafe extern "system" fn(
    vmBuf: *mut *mut sys::JavaVM,
    bufLen: sys::jsize,
    nVMs: *mut sys::jsize,
) -> sys::jint;
pub const JNI_GET_JAVA_VMS_NAME: &[u8] = b"JNI_GetCreatedJavaVMs\0";

lazy_static! {
    static ref HOOKED: DashMap<String, (jni::objects::JClass<'static>, jni::objects::JMethodID)> =
        DashMap::new();
}

#[no_mangle]
pub extern "system" fn get_summary<'local>(
    jni: jvmti::native::JNIEnvPtr,
    obj: jvmti::native::JavaObject,
) -> sys::jstring {
    eprintln!("get_summary called");

    let hooked = HOOKED
        .get("com/google/devtools/build/lib/analysis/BlazeVersionInfo")
        .unwrap();

    let v = unsafe {
        let call_object_method = (**jni)
            .CallNonvirtualObjectMethod
            .expect("JNI::CallObjectMethod");

        let v = call_object_method(
            jni,
            obj,
            hooked.0.as_raw() as jvmti::native::jvmti_native::jclass,
            hooked.1.as_ref().into_raw() as jvmti::native::jvmti_native::jmethodID,
        );

        jni::objects::JString::from_raw(v as sys::jobject)
    };

    let mut env = unsafe { JNIEnv::from_raw(jni as *mut sys::JNIEnv) }.unwrap();
    let original_summary = env.get_string(&v).unwrap();
    let original_summary = original_summary.to_str().unwrap();

    let summary = env
        .new_string(format!(
            r#"Urchin Spine for Bazel extensions 🐚

{original_summary}
"#
        ))
        .unwrap();
    summary.into_raw()
}

#[no_mangle]
pub extern "system" fn get_diff<'local>(
    jni: jvmti::native::JNIEnvPtr,
    obj: jvmti::native::JavaObject,
    view1: sys::jobject,
    view2: sys::jobject,
) -> sys::jobject {
    eprintln!("get_diff called");

    let hooked = HOOKED
        .get("com/google/devtools/build/lib/skyframe/LocalDiffAwareness")
        .unwrap();

    unsafe {
        let call_object_method = (**jni)
            .CallNonvirtualObjectMethod
            .expect("JNI::CallObjectMethod");

        let v = call_object_method(
            jni,
            obj,
            hooked.0.as_raw() as jvmti::native::jvmti_native::jclass,
            hooked.1.as_ref().into_raw() as jvmti::native::jvmti_native::jmethodID,
            view1,
            view2
        );

        v as sys::jobject
    }
}

unsafe fn start() -> anyhow::Result<()> {
    eprintln!("Urchin Spine v0.1.0");
    loop {
        // spin in a loop until we find a jvm instance.
        let lib = unsafe {
            libloading::Library::new("/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home/lib/server/libjvm.dylib")
        }?;
        // locate the JNI_GetCreatedJavaVMs function on the JVM library.
        let get_created_java_vms: JniGetCreatedJavaVms =
            *unsafe { lib.get(JNI_GET_JAVA_VMS_NAME) }?;

        // create a buffer to store the JVM instances.
        let mut created_java_vms: [*mut sys::JavaVM; 1] = [null_mut() as *mut sys::JavaVM];
        let mut java_vms_count: i32 = 0;

        // call the JNI_GetCreatedJavaVMs function to get the JVM instances.
        unsafe {
            get_created_java_vms(created_java_vms.as_mut_ptr(), 1, &mut java_vms_count);
        }

        if java_vms_count == 0 {
            continue;
        }

        eprintln!("get_created_java_vms: {:?}", get_created_java_vms);
        eprintln!("java_vms_count: {:?}", java_vms_count);

        eprintln!("created_java_vms: {:?}", created_java_vms);
        let jvm_ptr = *created_java_vms.first().unwrap();
        eprintln!("jvm_ptr: {:?}", jvm_ptr);
        let jvm = unsafe { jni::JavaVM::from_raw(jvm_ptr) }?;
        eprintln!("jvm: {:?}", jvm);

        eprintln!("attaching");
        let mut env = jvm
            .attach_current_thread_permanently()
            .context("Attaching current thread")?;
        eprintln!("attached");

        // let blaze_info = env
        //     .find_class("com/google/devtools/build/lib/analysis/BlazeVersionInfo")
        //     .unwrap();
        // eprintln!("blaze_info: {:?}", blaze_info);

        // let get_summary_method = env
        //     .get_method_id(blaze_info, "getSummary", "()Ljava/lang/String;")
        //     .unwrap();
        // eprintln!("get_summary_method: {:?}", get_summary_method);

        // eprintln!("JVMTI initialized");
        // let (klazz, method) = jnihook_attach(
        //     jvm.get_java_vm_pointer(),
        //     get_summary_method,
        //     get_summary as *mut std::ffi::c_void,
        // )?;

        // HOOKED.insert(
        //     "com/google/devtools/build/lib/analysis/BlazeVersionInfo".to_string(),
        //     (klazz, method),
        // );

         let local_diff_awareness = env
            .find_class("com/google/devtools/build/lib/skyframe/LocalDiffAwareness")
            .unwrap();
        eprintln!("local_diff_awareness: {:?}", local_diff_awareness);

        let get_diff_method = env
            .get_method_id(&local_diff_awareness, "getDiff", "(Lcom/google/devtools/build/lib/skyframe/DiffAwareness$View;Lcom/google/devtools/build/lib/skyframe/DiffAwareness$View;)Lcom/google/devtools/build/lib/vfs/ModifiedFileSet;")
            .unwrap();
        eprintln!("get_diff_method: {:?}", get_diff_method);

        eprintln!("JVMTI initialized");
        let (klazz, method) = jnihook_attach(
            jvm_ptr,
            get_diff_method,
            get_diff as *mut std::ffi::c_void,
        )?;

        HOOKED.insert(
            "com/google/devtools/build/lib/skyframe/LocalDiffAwareness".to_string(),
            (klazz, method),
        );
        
        break;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn is_main_thread() -> bool {
    use objc::runtime::Class;
    use objc::*;
    let nsthread = Class::get("NSThread");
    if nsthread.is_none() {
        return false;
    }
    unsafe { msg_send![nsthread.unwrap(), isMainThread] }
}

#[no_mangle]
#[ctor]
fn dl_entry() {
    eprintln!("current thread {:#?}", is_main_thread());
    if is_main_thread() {
        thread::spawn(|| unsafe {
            let result = start();
            if let Err(e) = result {
                eprintln!("injection failed: {:?}", e);
            }
        });
    }
}
