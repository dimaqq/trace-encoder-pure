use prost::Message;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

mod otlp {
    pub mod common {
        pub mod v1 {
            include!(concat!(
                env!("OUT_DIR"),
                "/opentelemetry.proto.common.v1.rs"
            ));
        }
    }
    pub mod resource {
        pub mod v1 {
            include!(concat!(
                env!("OUT_DIR"),
                "/opentelemetry.proto.resource.v1.rs"
            ));
        }
    }
    pub mod trace {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/opentelemetry.proto.trace.v1.rs"));
        }
    }
}

use otlp::trace::v1::{ResourceSpans, ScopeSpans, Span};

#[pyfunction]
fn encode_spans(py: Python, spans: &PyAny) -> PyResult<Py<PyBytes>> {
    let iter = spans.iter()?;

    let mut rust_spans = Vec::new();
    for item in iter {
        let span = item?;

        // Extract required fields from the Python object
        let context = span.getattr("context")?;
        let trace_id = context.getattr("trace_id")?.extract::<u128>()?;
        let span_id = context.getattr("span_id")?.extract::<u64>()?;
        let name = span.getattr("name")?.extract::<String>()?;

        rust_spans.push(Span {
            trace_id: trace_id.to_be_bytes().to_vec(),
            span_id: span_id.to_be_bytes().to_vec(),
            name,
            ..Default::default()
        });
    }

    let scope_spans = ScopeSpans {
        spans: rust_spans,
        ..Default::default()
    };
    let resource_spans = ResourceSpans {
        scope_spans: vec![scope_spans],
        ..Default::default()
    };

    let encoded = resource_spans.encode_to_vec();
    let py_bytes = PyBytes::new(py, &encoded);
    Ok(py_bytes.into())
}

#[pymodule]
fn trace_encoder_lite(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode_spans, m)?)?;
    Ok(())
}
