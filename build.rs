fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    // Safety: build scripts are single-threaded, no concurrent env reads.
    unsafe { std::env::set_var("PROTOC", protoc) };
    prost_build::compile_protos(&["protos/blogs.proto"], &["protos/"]).unwrap();
}
