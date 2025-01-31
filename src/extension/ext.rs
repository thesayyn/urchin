use std::path::Path;


use wasmtime::{Config, Engine, FuncType, Linker, Module, Store, Val, ValType};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;



pub struct Ext {}


impl Ext {
    pub fn new() -> Self {
        Self {} 
    }
    pub async fn run(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let mut config = Config::new();
        config.async_support(true);
        
        let engine = Engine::new(&config)?;
        let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);

        preview1::add_to_linker_async(&mut linker, |t| t)?;
    
   
        let ty = FuncType::new(&engine, [ValType::ANYREF], []);

        linker.func_new("urchin", "exchange", ty, |_, args, _| {
            let ar = args[0].anyref().unwrap().as_ref().unwrap();
            
            println!("Somebody said hello!");
            Ok(())
        })?;

        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build_p1();

        let mut store = Store::new(&engine, wasi_ctx);

        let module = Module::from_file(&engine, path)?;

        linker.module_async(&mut store, "", &module).await?;
        linker
            .get_default(&mut store, "")?
            .typed::<(), ()>(&store)?
            .call_async(&mut store, ()).await?;

        Ok(())
    }
}
