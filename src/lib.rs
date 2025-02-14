use prost::Message;
use pyo3::ffi::c_str;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

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

use crate::otlp::{
    collector::trace::v1::ExportTraceServiceRequest,
    common::v1::{any_value::Value, AnyValue, InstrumentationScope, KeyValue},
    resource::v1::Resource,
    trace::v1::{span::SpanKind, ResourceSpans, ScopeSpans, Span},
};

/*
pub fn parse_scope(py_scope: &PyAny) -> ScopeKey {
    let name = py_scope
        .getattr("name")
        .and_then(|x| x.extract::<String>()) // TODO: name may have to be set
        .ok();
    let version = py_scope
        .getattr("version")
        .and_then(|x| x.extract::<String>()) // ok to omit
        .ok();
    let schema_url = py_scope
        .getattr("schema_url")
        .and_then(|x| x.extract::<String>()) // ok to omit
        .ok();

    ScopeKey {
        name,
        version,
        schema_url,
    }
}
*/

/// Convert something that satisfies Python dict[str, str] protocol.
/// Must be done with a hack, because OTEL attributes are a mapping, not a dict.
fn dict_like_to_kv(py_mapping: &Bound<'_, PyAny>) -> PyResult<Vec<KeyValue>> {
    let mut rv = Vec::new();
    let items = py_mapping.call_method0("items")?.try_iter()?;
    for kv in items {
        let pair = kv?;
        let k = pair.get_item(0)?.extract::<String>();
        let v = pair.get_item(1)?.extract::<String>();
        if let (Ok(key), Ok(value)) = (k, v) {
            rv.push(KeyValue {
                key,
                value: Some(AnyValue {
                    value: Some(Value::StringValue(value)),
                }),
            });
        }
    }
    Ok(rv)
}

/// encode_spans(sdk_spans: Sequece[ReadableSpan]) -> bytes
/// --
///
/// Encode `sdk_spans` into an OTLP Protobuf and return `bytes`.
#[pyfunction]
#[pyo3(signature = (sdk_spans))]
fn encode_spans(sdk_spans: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    // Incoming data shape:
    // spans[]:
    //   span{}:
    //     resource{}:
    //       attributes{}
    //     instrumentation_scope{}:
    //       ...
    //     trace_id: int
    //     ...
    //
    // Outgoing data shape:
    // ExportTracesServiceRequest{}:
    //   resource_spans[]:
    //     ResourceSpans{}:
    //       Resource{}
    //         attributes: ...
    //         ...
    //       schema_url: str
    //       scope_spans[]:
    //         ScopeSpans{}:
    //           scope: InstrumentationScope{}:
    //             ...
    //           schema_url: str
    //           spans[]:
    //             Span{}:
    //               trace_id: bytes
    //               ...
    //
    // Comments from OTEL Python SDK
    // # We need to inspect the spans and group + structure them as:
    // #
    // #   Resource
    // #     Instrumentation Library
    // #       Spans
    //
    // We don't have to preserve the original order of spans,
    // so we're going to sort them by resource and inst. scope.
    // That groups spans by their ancestry, and emitting spans
    // can be done in a simple loop.

    Python::with_gil(|py| {
        let builtins = PyModule::import(py, "builtins")?;
        // The spans we're asked to send were created in this process
        // and are in memory. Thus, logically same resource is actually
        // the very same resource object. Same holds for inst. scope.
        let key_func = py.eval(
            c_str!("lambda e: (id(e.resource), id(e.instrumentation_scope))"),
            None,
            None,
        )?;
        let kwargs = [("key", key_func)].into_py_dict(py)?;
        let spans = builtins.call_method("sorted", (sdk_spans.as_ref(),), Some(&kwargs))?;

        let mut last_resource = py.None().into_pyobject(py)?;
        let mut last_scope = py.None().into_pyobject(py)?;
        let mut request = ExportTraceServiceRequest {
            resource_spans: Vec::new(),
            ..Default::default()
        };

        for item in spans.try_iter()? {
            let span = item?;
            // .resource cannot be None
            if !span.getattr("resource")?.is(&last_resource) {
                last_resource = span.getattr("resource")?;
                last_scope = py.None().into_pyobject(py)?;

                request.resource_spans.push(ResourceSpans {
                    resource: Some(Resource {
                        attributes: dict_like_to_kv(&last_resource.getattr("attributes")?)?,
                        // dropped_attribute_count: ...
                        ..Default::default()
                    }),
                    scope_spans: Vec::new(),
                    // schema_url: ...
                    ..Default::default()
                });
            }
            // .instrumentation_scope cannot be None
            if !span.getattr("instrumentation_scope")?.is(&last_scope) {
                last_scope = span.getattr("instrumentation_scope")?;

                request
                    .resource_spans
                    .last_mut()
                    .expect(".resource_spans can't be empty")
                    .scope_spans
                    .push(ScopeSpans {
                        scope: Some(InstrumentationScope {
                            // TODO can name be missing?
                            name: last_scope.getattr("name")?.extract::<String>()?,
                            // TODO what is version is missing?
                            version: last_scope.getattr("version")?.extract::<String>()?,
                            // schema_url: ...
                            ..Default::default()
                        }),
                        spans: Vec::new(),
                        // schema_url: ...
                        ..Default::default()
                    });
            }

            let context = span.getattr("context")?;

            request
                .resource_spans
                .last_mut()
                .expect(".resource_spans can't be empty")
                .scope_spans
                .last_mut()
                .expect(".scope_spans can't be empty")
                .spans
                .push(Span {
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
                        .getattr("start_time")
                        .and_then(|t| t.extract::<u64>())
                        .unwrap_or_default(),
                    end_time_unix_nano: span
                        .getattr("end_time")
                        .and_then(|t| t.extract::<u64>())
                        .unwrap_or_default(),
                    flags: span
                        .getattr("flags")
                        .and_then(|f| f.extract::<u32>())
                        .unwrap_or(256),
                    ..Default::default()
                });
        }

        Ok(request.encode_to_vec())
    })

    // FIXME:
    // - resource
    // - scope
    // - timestamps
    // - status
}

/// 🐍Lightweight OTEL span to binary converter, written in Rust🦀
#[pymodule(gil_used = false)]
fn trace_encoder_lite(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode_spans, m)?)
}
