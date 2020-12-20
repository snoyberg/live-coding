fn main() {
    tonic_build::compile_protos("proto/snoytest.proto").unwrap();
}
