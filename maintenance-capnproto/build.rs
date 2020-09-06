extern crate capnpc;

fn main() {
    ::capnpc::CompilerCommand::new()
        .file("maintenance.capnp")
        .run()
        .expect("comiling schema");
}
