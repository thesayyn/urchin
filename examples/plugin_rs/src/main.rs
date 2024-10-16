// cargo build --target wasm32-wasip1 && cp target/wasm32-wasip1/debug/plugin_rs.wasm main.wasm

// #[link(wasm_import_module = "urchin")]
// extern { fn say_hello(); }

wit_bindgen::generate!({
    // the name of the world in the `*.wit` input file
    world: "v1",
});

mod bindings;

struct Component;

impl bindings::Guest for Component {
    fn run() {
        bindings::adios("Rust");
        println!("Hello from Rust!");
    }
}

bindings::export!(Component with_types_in bindings);

fn main() {
    // bindings::adios("Rust");
    // println!("Hello from Rust!");
}
