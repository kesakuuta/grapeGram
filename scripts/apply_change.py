#!/usr/bin/env python3
"""Apply an owner-approved badge change and sign the result.

Used by .github/workflows/badges-sign.yml. Authorization comes from the
protected GitHub environment `badge-admin` plus repository write access;
this script validates the *data* (id format, badge type, owner list) and
signs with GRP_SIGNING_KEY_PEM. It never trusts a Telegram ID sent by an app.

Environment: ACTION (grant|revoke), USER_ID, BADGE, REASON,
             GRP_SIGNING_KEY_PEM, GITHUB_ACTOR (optional)
"""
import argparse
import base64
import json
import os
import re
import subprocess
import sys
import tempfile
import time

ALLOWED = {"grapegram", "founder", "developer", "contributor"}
ID_RE = re.compile(r"^[0-9]{5,20}$")


def sign(payload, pem):
    with tempfile.NamedTemporaryFile(delete=False) as k:
        k.write(pem)
        kp = k.name
    with tempfile.NamedTemporaryFile(delete=False) as d:
        d.write(payload)
        dp = d.name
    try:
        return subprocess.run(
            ["openssl", "pkeyutl", "-sign", "-inkey", kp, "-rawin", "-in", dp],
            check=True, capture_output=True).stdout
    finally:
        os.unlink(kp)
        os.unlink(dp)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=os.path.dirname(
        os.path.dirname(os.path.abspath(__file__))))
    a = ap.parse_args()
    action = os.environ["ACTION"]
    user_id = os.environ["USER_ID"].strip()
    badge = os.environ["BADGE"].strip()
    reason = os.environ.get("REASON", "")
    pem = os.environ.get("GRP_SIGNING_KEY_PEM", "").encode()
    if action not in ("grant", "revoke"):
        sys.exit("bad action")
    if not ID_RE.match(user_id):
        sys.exit("bad user id")
    if badge not in ALLOWED:
        sys.exit("bad badge")
    if not pem:
        sys.exit("GRP_SIGNING_KEY_PEM is not set")

    data = os.path.join(a.root, "data")
    owners = json.load(open(os.path.join(data, "owner_users.json")))["owners"]
    owner_id = os.environ.get("OWNER_ID", "").strip()
    if owner_id and owner_id not in owners:
        sys.exit("OWNER_ID is not in owner_users.json")
    path = os.path.join(data, "badge_users.json")
    doc = json.load(open(path))

    now = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    entry = doc["badges"].get(user_id, {"record_version": 0})
    entry["badge"] = badge
    entry["enabled"] = action == "grant"
    entry["updated_at"] = now
    entry["record_version"] = int(entry.get("record_version", 0)) + 1
    if action == "grant" and "granted_at" not in entry:
        entry["granted_at"] = now
    doc["badges"][user_id] = entry
    doc["version"] = int(doc["version"]) + 1
    doc["updated_at"] = now

    with open(path, "w", encoding="utf-8") as f:
        json.dump(doc, f, ensure_ascii=False, indent=2, sort_keys=True)
        f.write("\n")
    with open(path, "rb") as f:
        sig = sign(f.read(), pem)
    os.makedirs(os.path.join(data, "signatures"), exist_ok=True)
    with open(os.path.join(data, "signatures", "badge_users.json.sig"),
              "w", encoding="ascii") as f:
        f.write(base64.b64encode(sig).decode("ascii") + "\n")
    with open(os.path.join(data, "audit.jsonl"), "a", encoding="utf-8") as f:
        f.write(json.dumps({
            "action": action, "target": user_id, "badge": badge,
            "reason": reason, "version": doc["version"], "at": now,
            "by": owner_id or os.environ.get("GITHUB_ACTOR", "unknown"),
            "owners_configured": owners,
        }, ensure_ascii=False) + "\n")
    print(f"{action} {badge} -> {user_id}, version {doc['version']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
