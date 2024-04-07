use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure().compile(
        &["src/command_server/command_server.proto"],
        &["src/command_server"],
    )?;
    Ok(())
}
