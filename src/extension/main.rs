use urchin::extension::ext::*;
use futures::join;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    let ext = Ext::new();
    join!(
        // ext.run("./examples/plugin_go/output/main.wasm"),
        ext.run("./examples/plugin_rs/target/wasm32-wasip1/debug/plugin_rs.wasm")
    );
    Ok(( ))
}
