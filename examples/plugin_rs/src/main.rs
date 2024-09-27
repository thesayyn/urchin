// cargo build --target wasm32-wasip1 && cp target/wasm32-wasip1/debug/plugin_rs.wasm main.wasm

#[link(wasm_import_module = "urchin")]
extern { fn say_hello(); }

fn main() {
    unsafe { say_hello() };
    println!("Hello from Rust!");
}
