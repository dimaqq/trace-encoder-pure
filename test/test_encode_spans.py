import pytest
import trace_encoder_lite


@pytest.fixture
def mock_span():
    # FIXME this doesn't mirror opentelemetry-api
    class Span:
        span_id = 42
        trace_id = 42
        name = "booya"

    return Span()


def test_encode_spans(mock_span):
    trace_encoder_lite.encode_spans([mock_span])
