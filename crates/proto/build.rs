fn main() {
    prost_build::compile_protos(&["proto/test/test.proto"], &["proto"]).unwrap();
}
