#!/usr/bin/env python3
"""Generate an Ed25519 keypair for grapeGram badge data.

The PRIVATE key must never be committed, never be baked into the APK and never
be pasted into chat. Store it only as a protected GitHub Actions secret
(GRP_SIGNING_KEY_PEM) or in an offline vault.

Usage:
  python3 scripts/gen_keys.py --out /secure/grapegram_keys
"""
import argparse
import os
import subprocess
import sys


def run(*args: str) -> bytes:
    return subprocess.run(args, check=True, capture_output=True).stdout


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True, help="directory for the keypair")
    a = ap.parse_args()
    os.makedirs(a.out, mode=0o700, exist_ok=True)
    priv = os.path.join(a.out, "grapegram_ed25519_private.pem")
    pub = os.path.join(a.out, "grapegram_ed25519.pub.pem")
    if os.path.exists(priv):
        print(f"refusing to overwrite {priv}", file=sys.stderr)
        return 1
    run("openssl", "genpkey", "-algorithm", "ED25519", "-out", priv)
    os.chmod(priv, 0o600)
    run("openssl", "pkey", "-in", priv, "-pubout", "-out", pub)
    print(f"private: {priv}\npublic:  {pub}\n")
    print("Commit only the public key (data/keys/grapegram_ed25519.pub.pem).")
    print("Put the private key into the GRP_SIGNING_KEY_PEM secret.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
