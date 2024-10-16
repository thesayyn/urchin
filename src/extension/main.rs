use urchin::extension::ext::*;

fn main() -> anyhow::Result<()> {
    let mut ext = Ext::new()?;
    // ext.run("./examples/plugin_go/main.wasm")?;
    ext.run("./examples/plugin_rs/target/wasm32-wasip1/debug/plugin_rs.wasm")?;
    Ok(())
}
