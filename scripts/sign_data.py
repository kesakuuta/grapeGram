#!/usr/bin/env python3
"""Sign a JSON document with the Ed25519 release key.

Output: data/signatures/<name>.json.sig  (base64, one line)

Usage:
  python3 scripts/sign_data.py --key /secure/grapegram_ed25519_private.pem \
      data/badge_users.json data/owner_users.json
"""
import argparse
import base64
import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))



def sign(key: str, payload: bytes) -> bytes:
    with tempfile.NamedTemporaryFile(delete=False) as f:
        f.write(payload)
        tmp = f.name
    try:
        return subprocess.run(
            ["openssl", "pkeyutl", "-sign", "-inkey", key, "-rawin", "-in", tmp],
            check=True,
            capture_output=True,
        ).stdout
    finally:
        os.unlink(tmp)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--key", required=True)
    ap.add_argument("--out", default="data/signatures")
    ap.add_argument("files", nargs="+")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    for path in a.files:
        with open(path, "rb") as f:
            payload = f.read()
        sig = sign(a.key, payload)
        name = os.path.basename(path) + ".sig"
        dst = os.path.join(a.out, name)
        with open(dst, "w", encoding="ascii") as f:
            f.write(base64.b64encode(sig).decode("ascii") + "\n")
        print(f"{path} -> {dst} ({len(sig)} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
