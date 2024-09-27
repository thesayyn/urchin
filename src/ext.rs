use std::{fs, io::Read, path::Path};

use wasmer::{Function, FunctionEnv, FunctionEnvMut, Module, Store};
use wasmer_cache::{Cache, FileSystemCache, Hash};
use wasmer_compiler_llvm::LLVM;
use wasmer_wasix::{Pipe, WasiEnv};

#[derive(Clone)]
struct ExtEnv {}

fn say_hello(env: FunctionEnvMut<ExtEnv>) {
    println!("Somebody said hello!");
}

pub struct Ext {
    store: Store,
    cache: FileSystemCache,
}

impl Ext {
    pub fn new() -> Self {
        fs::create_dir_all(".urchin/cache").expect("Failed to create cache directory");
        let cache = FileSystemCache::new(".urchin/cache").expect("Failed to create cache");
        Self {
            store: Store::new(LLVM::default()),
            cache: cache,
        }
    }
    pub fn run(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        // read the file into a byte array
        let mut file = fs::File::open(&path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        // generate a hash from the byte array
        let key = Hash::generate(&bytes);
        let from_cache = unsafe { self.cache.load(&self.store, key) };
        let module = if from_cache.is_ok() {
            from_cache?
        } else {
            let module = Module::from_binary(&self.store, &bytes)?;
            self.cache.store(key, &module)?;
            module
        };
        let (stdout_tx, mut stdout_rx) = Pipe::channel();
        let (stderr_tx, mut stderr_rx) = Pipe::channel();
        let env = FunctionEnv::new(&mut self.store, ExtEnv {});
        WasiEnv::builder("go")
            .args(&["spawn"])
            .env("URCHIN_VERSION", "0.0.0")
            .stdout(Box::new(stdout_tx))
            .stderr(Box::new(stderr_tx))
            .import(
                "urchin",
                "say_hello",
                Function::new_typed_with_env(&mut self.store, &env, say_hello),
            )
            .run_with_store(module, &mut self.store)?;
        let mut buf = String::new();
        stdout_rx.read_to_string(&mut buf).unwrap();
        eprintln!("Output: {buf}");
        Ok(())
    }
}
