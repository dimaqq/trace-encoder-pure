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
    pub mod collector {
        pub mod trace {
            pub mod v1 {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/opentelemetry.proto.collector.trace.v1.rs"
                ));
            }
        }
    }
}

use otlp::collector::trace::v1::ExportTraceServiceRequest;
use otlp::trace::v1::{span::SpanKind, ResourceSpans, ScopeSpans, Span};

#[pyfunction]
fn encode_spans(py: Python, sdk_spans: &PyAny) -> PyResult<Py<PyBytes>> {
    let iter = sdk_spans.iter()?;

    let mut rust_spans = Vec::new();
    for item in iter {
        let span = item?;
        let context = span.getattr("context")?;

        rust_spans.push(Span {
            trace_id: context
                .getattr("trace_id")?
                .extract::<u128>()?
                .to_be_bytes()
                .to_vec(),
            span_id: context
                .getattr("span_id")?
                .extract::<u64>()?
                .to_be_bytes()
                .to_vec(),
            name: span.getattr("name")?.extract::<String>()?,
            kind: span
                .getattr("kind")
                .and_then(|k| k.extract::<i32>())
                .unwrap_or(SpanKind::Internal as i32),
            start_time_unix_nano: span
                .getattr("start_time_unix_nano")
                .and_then(|t| t.extract::<u64>())
                .unwrap_or_default(),
            end_time_unix_nano: span
                .getattr("end_time_unix_nano")
                .and_then(|t| t.extract::<u64>())
                .unwrap_or_default(),
            flags: span
                .getattr("flags")
                .and_then(|f| f.extract::<u32>())
                .unwrap_or(256),
            ..Default::default()
        });
    }

    // FIXME:
    // - resource
    // - scope
    // - timestamps
    // - status

    let scope_spans = ScopeSpans {
        spans: rust_spans,
        ..Default::default()
    };
    let resource_spans = ResourceSpans {
        scope_spans: vec![scope_spans],
        ..Default::default()
    };
    let request = ExportTraceServiceRequest {
        resource_spans: vec![resource_spans],
        ..Default::default()
    };

    // TODO: could be more efficient:
    // - preallocate a Python bytes object of correct length
    // - write to that memory
    // --
    // however, that means going through the struct twice,
    // so who knows, maybe a single _to_vec is faster?
    let encoded = request.encode_to_vec();
    let py_bytes = PyBytes::new(py, &encoded);
    Ok(py_bytes.into())
}

#[pymodule]
fn trace_encoder_lite(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode_spans, m)?)?;
    Ok(())
}
