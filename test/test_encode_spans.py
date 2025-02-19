import subprocess
from unittest.mock import Mock
from typing import Sequence

from opentelemetry.sdk.trace import TracerProvider, ReadableSpan
from opentelemetry.sdk.trace.export import SimpleSpanProcessor
from opentelemetry.sdk.trace.export.in_memory_span_exporter import InMemorySpanExporter
from opentelemetry.exporter.otlp.proto.common._internal import trace_encoder
import pytest
from typing_extensions import reveal_type as reveal_type

import otlp_proto


@pytest.fixture
def sample_spans() -> Sequence[ReadableSpan]:
    """Creates and finishes two spans, then returns them as a list."""
    tracer_provider = TracerProvider()
    exporter = InMemorySpanExporter()
    tracer_provider.add_span_processor(SimpleSpanProcessor(exporter))
    tracer = tracer_provider.get_tracer(__name__)

    with tracer.start_as_current_span("span-one"):
        pass
    with tracer.start_as_current_span("span-two"):
        pass

    spans = exporter.get_finished_spans()
    return spans


@pytest.fixture
def mock_span():
    good = Mock()
    good.status_code.value = 0

    class Context:
        span_id = 42
        trace_id = 42

    class Resource:
        attributes = dict()

    class Scope:
        name = "foo"
        version = "1.2.3"

    class Span:
        name = "booya"
        context = Context()
        resource = Resource()
        instrumentation_scope = Scope()
        status = good

    return Span()


def test_encode_spans(mock_span):
    otlp_proto.encode_spans([mock_span])


def test_function_signature(sample_spans):
    res = otlp_proto.encode_spans(sample_spans)
    kwres: bytes = otlp_proto.encode_spans(sdk_spans=sample_spans)
    assert res == kwres


def test_equivalence(sample_spans):
    ours = otlp_proto.encode_spans(sample_spans)
    data = trace_encoder.encode_spans(sample_spans).SerializePartialToString()
    assert text(ours) == text(data)


# TODO: skip tests is protoc is not in PATH
def text(data: bytes) -> str:
    return subprocess.run(
        [
            "protoc",
            "--decode=opentelemetry.proto.collector.trace.v1.ExportTraceServiceRequest",
            "opentelemetry/proto/collector/trace/v1/trace_service.proto",
        ],
        input=data,
        capture_output=True,
        check=True,
    ).stdout.decode("utf-8")
