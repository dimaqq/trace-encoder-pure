from typing import TYPE_CHECKING, Sequence

if TYPE_CHECKING:
    from opentelemetry.sdk.trace import ReadableSpan

def encode_spans(spans: Sequence["ReadableSpan"]) -> bytes: ...
