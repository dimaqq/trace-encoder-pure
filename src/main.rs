// Include the generated Rust code (prost outputs .rs files into OUT_DIR).
mod otel {
    pub mod trace {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.trace.v1.rs"));
        }
    }
}

use otel::trace::v1::{ResourceSpans, ScopeSpans, Span};

fn main() {
    // Create a dummy Span with some plausible IDs and a name.
    let dummy_span = Span {
        trace_id: "0000000000000000beefdeaddeadbeef".to_string(),
        span_id: "00000000cafebabecafebabe".to_string(),
        name: "my-dummy-operation".to_string(),
        // Prost-generated structs have more fields; we leave them defaulted.
        ..Default::default()
    };

    // Wrap the Span in a ScopeSpans -> ResourceSpans hierarchy
    let scope_spans = ScopeSpans {
        spans: vec![dummy_span],
        ..Default::default()
    };

    let resource_spans = ResourceSpans {
        scope_spans: vec![scope_spans],
        ..Default::default()
    };

    // Serialize to a byte vector using prost.
    let encoded = prost::Message::encode_to_vec(&resource_spans);

    // Write to file for inspection, debugging, or further testing.
    std::fs::write("dummy_span.bin", &encoded).expect("Failed to write file");

    println!(
        "Serialized ResourceSpans to dummy_span.bin ({} bytes).",
        encoded.len()
    );
}
