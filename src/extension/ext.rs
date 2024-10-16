use std::path::Path;
use std::fs;

use anyhow::{Context, Result};
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};


// It is important to define the payload struct that will be used to pass data between the host and the guest
// The payload struct must be `#[repr(C)]` and must not be reordered or changed in any way
//
// Binary compatible ABIs usually have the following rules:
// When moving fields around, it is important to keep the same order in binary
// When adding a field, it must be added at the end of the struct
// When removing a field, it must be replaced with a padding field of the same size
// When changing type of a field, it's a breaking change
//
// https://lvc.github.io/abi-compliance-checker/
//
// However, WASM ABIs are not as strict as C ABIs

// This has the following benefits:
//  - It is optimized for local communication rather than a network communication
//  - Canonical ABI guarantees the binary compatibility between different languages
// https://github.com/WebAssembly/component-model
// https://github.com/WebAssembly/component-model/blob/main/design/mvp/CanonicalABI.md
// https://www.fermyon.com/blog/webassembly-component-model

wasmtime::component::bindgen!("v1");


/// This function is only needed until rust can natively output a component.
///
/// Generally embeddings should not be expected to do this programmatically, but instead
/// language specific tooling should be used, for example in Rust `cargo component`
/// is a good way of doing that: https://github.com/bytecodealliance/cargo-component
///
/// In this example we convert the code here to simplify the testing process and build system.
fn convert_to_component(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let bytes = &fs::read(&path).context("failed to read input file")?;
    wit_component::ComponentEncoder::default()
        .module(&bytes)?
        .encode()
}

pub struct Ext {
    store: wasmtime::Store<MyState>,
    linker: wasmtime::component::Linker<MyState>,
}

struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl V1Imports for MyState {
    fn adios(&mut self, input:wasmtime::component::__internal::String,) -> wasmtime::component::__internal::String {
        println!("called");
        format!("Adios back, {}", input)
    }
}

impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl Ext {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_multi_memory(true);
        config.wasm_component_model(true);

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        
        wasmtime_wasi::add_to_linker_sync(&mut linker)?;
        V1::add_to_linker(&mut linker, |state: &mut MyState| state)?;

        let store = Store::new(
            &engine,
            MyState {
                ctx: WasiCtxBuilder::new()
                    .inherit_stderr()
                    .inherit_env()
                    .inherit_stdout()
                    .build(),
                table: ResourceTable::new(),
            },
        );

        Ok(Self { store, linker })
    }
    pub fn run(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        // let bytes = convert_to_component(path.as_ref())?;

        // read file at path into byte array
        let bytes = fs::read(path.as_ref()).context("failed to read input file")?;
        
        let component = Component::from_binary(&mut self.store.engine(), &bytes)
            .context("failed to load plugin")?;

        let instance = self.linker.instantiate(&mut self.store, &component)?;

        let func = instance.get_func(&mut self.store, "run").context("failed to find run function")?;
        func.call(&mut self.store, &[], &mut [])
        // let default = instance
        //     .get_func(&mut self.store, "")
        //     .or_else(|| instance.get_func(&mut self.store, "_start"))
        //     .context("failed to find default function")?
        //     .typed::<(), ()>(&mut self.store)?;

        // default.call(&mut self.store, ())
    }
}
