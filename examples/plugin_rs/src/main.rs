// cargo build --target wasm32-wasip1

mod heap;

use heap::CapnpReusableWriter;
use once_cell::sync::Lazy;
use std::sync::Mutex;


// Host functions
#[link(wasm_import_module = "urchin")]
extern { fn exchange(ptr: *const u8, len: usize); }


pub mod urchin_capnp {
    include!(concat!(env!("OUT_DIR"), "/urchin_capnp.rs"));
}

static CAPNP_WRITER: Lazy<Mutex<CapnpReusableWriter>> =
    Lazy::new(|| Mutex::new(CapnpReusableWriter::new()));


fn mutex_error_to_capnp_error<T>(e: std::sync::PoisonError<T>) -> capnp::Error {
    capnp::Error::failed(e.to_string())
}

pub fn create_person() -> capnp::Result<Vec<u8>> {
    let mut writer = CAPNP_WRITER.lock().map_err(mutex_error_to_capnp_error)?;
    let mut builder = writer.builder();
    let mut root = builder.init_root::<urchin_capnp::person::Builder>();

    root.set_name("Alice");

    let mut data = Vec::new();
    capnp::serialize::write_message(&mut data, &builder)?;

    Ok(data)
}

fn main() -> anyhow::Result<()> {
    
    let person = create_person()?;
    
    unsafe { 
        exchange(person.as_ptr(), person.len());
    }
    println!("Hello from Rust!");
    Ok(())
}
