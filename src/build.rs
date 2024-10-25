fn main() {
    ::capnpc::CompilerCommand::new()
        .file("../urchin.capnp")
        .src_prefix("../")
        .run()
        .expect("compiling schema");
}