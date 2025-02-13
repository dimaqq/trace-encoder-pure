import pytest
from typing import Sequence
from typing_extensions import reveal_type as reveal_type

from opentelemetry.sdk.trace import TracerProvider, ReadableSpan
from opentelemetry.sdk.trace.export import SimpleSpanProcessor
from opentelemetry.sdk.trace.export.in_memory_span_exporter import InMemorySpanExporter

import trace_encoder_lite


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
    # FIXME this doesn't mirror opentelemetry-api
    class Context:
        span_id = 42
        trace_id = 42

    class Span:
        name = "booya"
        context = Context()

    return Span()


def test_encode_spans(mock_span):
    trace_encoder_lite.encode_spans([mock_span])


def test_real_spans(sample_spans):
    trace_encoder_lite.encode_spans(sample_spans)
