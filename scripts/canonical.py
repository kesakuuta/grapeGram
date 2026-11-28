#!/usr/bin/env python3
"""Canonical JSON encoding shared by signer, verifier, backend and the Android Java client.

Rules (must match rust/badge-tools/src/canonical.rs and the Android Java implementation):
  * UTF-8, no BOM
  * object keys sorted by Unicode code point
  * no insignificant whitespace (',' and ':' separators)
  * non-ASCII characters are emitted literally (not \\uXXXX)
  * only integers are allowed as numbers in signed documents
"""
import json


def _validate(value, depth=0):
    if depth > 128:
        raise ValueError("JSON nesting is too deep")
    if value is None or isinstance(value, (str, bool)):
        return
    if isinstance(value, int):
        if value < -(1 << 63) or value > (1 << 64) - 1:
            raise ValueError("signed JSON integers must fit i64/u64")
        return
    if isinstance(value, float):
        raise ValueError("only integer numbers are allowed in signed JSON")
    if isinstance(value, list):
        for item in value:
            _validate(item, depth + 1)
        return
    if isinstance(value, dict):
        for key, item in value.items():
            if not isinstance(key, str):
                raise ValueError("signed JSON object keys must be strings")
            _validate(item, depth + 1)
        return
    raise ValueError("unsupported value in signed JSON")


def canonical_bytes(obj) -> bytes:
    _validate(obj)
    return json.dumps(
        obj,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")


def load(path: str):
    with open(path, "rb") as f:
        return json.load(f)


if __name__ == "__main__":
    import sys

    print(canonical_bytes(load(sys.argv[1])).decode("utf-8"))
