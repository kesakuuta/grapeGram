#!/usr/bin/env python3
"""Canonical JSON encoding shared by signer, verifier and the Android client.

Rules (must match the Kotlin implementation in the app):
  * UTF-8, no BOM
  * object keys sorted by Unicode code point
  * no insignificant whitespace (',' and ':' separators)
  * non-ASCII characters are emitted literally (not \\uXXXX)
  * only integers are allowed as numbers in signed documents
"""
import json


def canonical_bytes(obj) -> bytes:
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
