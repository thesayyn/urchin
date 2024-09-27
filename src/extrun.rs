use urchin::ext::Ext;


fn main() -> anyhow::Result<()> {
    let mut ext = Ext::new();
    ext.run("./examples/plugin_go/main.wasm")?;
    ext.run("./examples/plugin_rs/main.wasm")?;
    Ok(())
}
