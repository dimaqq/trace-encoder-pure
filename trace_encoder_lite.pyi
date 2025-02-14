from __future__ import annotations
from typing import TYPE_CHECKING, Sequence

if TYPE_CHECKING:
    import opentelemetry.sdk.trace

def encode_spans(
    sdk_spans: Sequence[opentelemetry.sdk.trace.ReadableSpan],
) -> bytes: ...
