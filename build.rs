fn main() -> Result<(), Box<dyn std::error::Error>> {
    // If you want to customize the generated code (e.g. module paths, attributes), you can
    // configure prost_build here:
    let mut config = prost_build::Config::new();

    // The root directory where your vendored "opentelemetry" folder resides
    // If your directory layout is different, adjust accordingly.
    //let proto_root = "opentelemetry";

    // List all the .proto files you want compiled
    // Omit any that you don't actually need to generate code for.
    let proto_files = [
        "opentelemetry/proto/common/v1/common.proto",
        "opentelemetry/proto/resource/v1/resource.proto",
        "opentelemetry/proto/trace/v1/trace.proto",
        "opentelemetry/proto/collector/trace/v1/trace_service.proto",
        "opentelemetry/proto/metrics/v1/metrics.proto",
        "opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
        "opentelemetry/proto/logs/v1/logs.proto",
        "opentelemetry/proto/collector/logs/v1/logs_service.proto",
        "opentelemetry/proto/profiles/v1development/profiles.proto",
        "opentelemetry/proto/collector/profiles/v1development/profiles_service.proto",
    ];

    // Pass them all to prost_build with the directory include path:
    //config.compile_protos(&proto_files, &[proto_root])?;
    config.compile_protos(&proto_files, &["."])?;

    Ok(())
}
