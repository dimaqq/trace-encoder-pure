fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile our minimal .proto with prost.
    prost_build::compile_protos(&["trace.proto"], &["."])?;
    Ok(())
}
