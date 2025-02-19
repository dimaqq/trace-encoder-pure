# otlp-proto

`otlp-proto` is a lightweight OTLP serialisation library.

It's written in Rust 🦀, and doesn't have any run-time dependencies.

It serialises a bunch of spans into OTLP 1.5 binary (protobuf) format.

### Motivation

Tracing should be on by default.

OTLP is the standard data format and API, and the standard Python package is `opentelemetry-exporter-otlp-proto-http`. It brings in a total of 18 packages and adds 9MB to the project virtual environment.

A typical Python application, that's being instrumented, only generates own tracing data and needs to send it out. It doesn't need that much complexity.


### Usage

```py
from otlp_proto import CONTENT_TYPE, encode_spans


class SomeExporter:
    def export(self, spans: Sequece[ReadableSpan]) -> None:
        requests.post(
            "http://localhost:4318/v1/traces",
            data=encode_spans(spans),
            headers={"Content-Type": CONTENT_TYPE},
        )
```

### Library size

- 170KB whl, containing, depending on the target platform
  - 350KB dylib
  - ???KB so

### TODO(doc)

- link to pure Python library
- link to urllib sender
- link to test vector generator

### TODO(features)

- Events
- Links
- Baggage

### TODO(fixes)

- validate what fields are in fact optional
- ???

### Limitations

This library is meant to marshal tracing data that's collected in the same Python process.

It is not meant to be used for data receive and forwarded.
