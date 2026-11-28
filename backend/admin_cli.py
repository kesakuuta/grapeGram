#!/usr/bin/env python3
"""Small SSH-only administration helper for the badge API."""
import argparse
import getpass
import json
import os
import urllib.error
import urllib.request

BASE = os.environ.get("GRP_ADMIN_URL", "http://127.0.0.1:8080").rstrip("/")


def request(path, payload=None, token=None):
    raw = json.dumps(payload or {}).encode("utf-8")
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = "Bearer " + token
    req = urllib.request.Request(BASE + path, data=raw, headers=headers,
                                 method="POST" if payload is not None else "GET")
    try:
        with urllib.request.urlopen(req, timeout=15) as response:
            return json.loads(response.read())
    except urllib.error.HTTPError as exc:
        detail = exc.read(1024).decode("utf-8", "replace")
        raise SystemExit("API error %s: %s" % (exc.code, detail))


def main():
    parser = argparse.ArgumentParser(description="grapeGram badge admin")
    sub = parser.add_subparsers(dest="action", required=True)
    sub.add_parser("challenge", help="create a one-time Telegram binding code")
    for action in ("grant", "revoke"):
        p = sub.add_parser(action)
        p.add_argument("user_id")
        p.add_argument("--badge", choices=("grapegram", "founder", "developer", "contributor"),
                       default="grapegram")
        p.add_argument("--reason", default="")
    args = parser.parse_args()
    if args.action == "challenge":
        result = request("/challenge", {})
        print("Send this to the grapeGram bot in a private chat:")
        print("/bind " + result["code"])
        print("Expires in %s seconds." % result["expires_in"])
        return
    token = getpass.getpass("Paste the 60-minute token sent by the bot: ").strip()
    if not token:
        raise SystemExit("No session token supplied")
    result = request("/admin/" + args.action, {
        "user_id": args.user_id,
        "badge": args.badge,
        "reason": args.reason,
    }, token)
    print("Success. Published badge document version %s." % result["version"])


if __name__ == "__main__":
    main()
