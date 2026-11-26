#!/usr/bin/env python3
"""Verify a detached signature produced by sign_data.py.

Usage:
  python3 scripts/verify_data.py --pub data/keys/grapegram_ed25519.pub.pem \
      data/badge_users.json data/signatures/badge_users.json.sig

Exit code 0 = valid, 1 = invalid. Never apply unsigned data.
"""
import argparse
import base64
import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from canonical import canonical_bytes, load  # noqa: E402


def verify(pub: str, payload: bytes, sig: bytes) -> bool:
    tmp_in = tmp_sig = None
    try:
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(payload)
            tmp_in = f.name
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(sig)
            tmp_sig = f.name
        r = subprocess.run(
            ["openssl", "pkeyutl", "-verify", "-pubin", "-inkey", pub,
             "-rawin", "-in", tmp_in, "-sigfile", tmp_sig],
            capture_output=True,
        )
        return r.returncode == 0
    finally:
        for p in (tmp_in, tmp_sig):
            if p:
                os.unlink(p)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--pub", default="data/keys/grapegram_ed25519.pub.pem")
    ap.add_argument("json")
    ap.add_argument("sig")
    a = ap.parse_args()
    with open(a.sig, "rb") as f:
        sig = base64.b64decode(f.read().strip())
    ok = verify(a.pub, canonical_bytes(load(a.json)), sig)
    print("VALID" if ok else "INVALID")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
