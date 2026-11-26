#!/usr/bin/env python3
"""grapeGram badge service (reference implementation, stdlib only).

Flow:
  app  -> POST /challenge                       -> one-time code (TTL 5 min)
  user -> bot: /bind <code>                     -> Telegram delivers real from.id
  bot  -> POST /telegram/webhook (secret hdr)   -> server binds session to owner id
  app  -> POST /admin/grant|revoke (Bearer)     -> owner-only, signs, publishes
  app  -> GET  /data/<file>                     -> signed public data

Secrets (environment):
  GRP_SIGNING_KEY_PEM   private Ed25519 key (PEM) or
  GRP_SIGNING_KEY_FILE  path to the PEM
  GRP_BOT_SECRET        X-Telegram-Bot-Api-Secret-Token expected from Telegram
  GRP_GITHUB_TOKEN      token with contents:write on the data repo (optional)
  GRP_GITHUB_REPO       "owner/repo" (optional, enables publishing)

Never put these values into the APK or the public repository.
"""
import base64
import json
import os
import re
import secrets
import subprocess
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA_DIR = os.path.join(ROOT, "data")
SIG_DIR = os.path.join(DATA_DIR, "signatures")
KEY_DIR = os.path.join(DATA_DIR, "keys")

ALLOWED_BADGES = {"grapegram", "founder", "developer", "contributor"}
ID_RE = re.compile(r"^[0-9]{5,20}$")
CHALLENGE_TTL = 300
SESSION_TTL = 3600


def canonical_bytes(obj) -> bytes:
    return json.dumps(obj, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False, allow_nan=False).encode("utf-8")


def _load(path):
    with open(path, "rb") as f:
        return json.load(f)


def _write_atomic(path, obj):
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(obj, f, ensure_ascii=False, indent=2, sort_keys=True)
        f.write("\n")
    os.replace(tmp, path)


def _sign(payload: bytes, key_pem: bytes) -> bytes:
    with tempfile.NamedTemporaryFile(delete=False) as kf:
        kf.write(key_pem)
        key_path = kf.name
    with tempfile.NamedTemporaryFile(delete=False) as df:
        df.write(payload)
        data_path = df.name
    try:
        return subprocess.run(
            ["openssl", "pkeyutl", "-sign", "-inkey", key_path, "-rawin",
             "-in", data_path],
            check=True, capture_output=True).stdout
    finally:
        os.unlink(key_path)
        os.unlink(data_path)


def signing_key() -> bytes:
    pem = os.environ.get("GRP_SIGNING_KEY_PEM")
    if pem:
        return pem.encode()
    path = os.environ.get("GRP_SIGNING_KEY_FILE")
    if path:
        with open(path, "rb") as f:
            return f.read()
    raise RuntimeError("signing key is not configured")


class Store:
    def __init__(self, data_dir: str):
        self.data_dir = data_dir
        self.lock = threading.Lock()
        self.challenges = {}
        self.sessions = {}
        self.audit_path = os.path.join(data_dir, "audit.jsonl")

    def owners(self):
        return _load(os.path.join(self.data_dir, "owner_users.json"))["owners"]

    def new_challenge(self) -> str:
        code = secrets.token_hex(4).upper()
        with self.lock:
            now = time.time()
            self.challenges = {c: v for c, v in self.challenges.items()
                               if v["expires"] > now}
            self.challenges[code] = {"expires": now + CHALLENGE_TTL,
                                     "session": None, "used": False}
        return code

    def bind(self, code: str, telegram_id: str):
        with self.lock:
            ch = self.challenges.get(code)
            if not ch or ch["used"] or ch["expires"] < time.time():
                raise PermissionError("challenge is invalid or expired")
            if str(telegram_id) not in self.owners():
                raise PermissionError("not an owner")
            ch["used"] = True
            token = secrets.token_urlsafe(32)
            self.sessions[token] = {"owner": str(telegram_id),
                                    "expires": time.time() + SESSION_TTL}
            return token

    def require_owner(self, token: str) -> str:
        s = self.sessions.get(token or "")
        if not s or s["expires"] < time.time():
            raise PermissionError("no valid session")
        if s["owner"] not in self.owners():
            raise PermissionError("not an owner")
        return s["owner"]

    def mutate(self, token: str, target: str, badge: str, enabled: bool,
               reason: str = ""):
        owner = self.require_owner(token)
        if not ID_RE.match(str(target)):
            raise ValueError("bad user id")
        if badge not in ALLOWED_BADGES:
            raise ValueError("unknown badge")
        path = os.path.join(self.data_dir, "badge_users.json")
        doc = _load(path)
        now = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        entry = doc["badges"].get(target, {"record_version": 0})
        entry.update({"badge": badge, "enabled": bool(enabled),
                      "updated_at": now,
                      "record_version": int(entry.get("record_version", 0)) + 1})
        if enabled and "granted_at" not in entry:
            entry["granted_at"] = now
        doc["badges"][target] = entry
        doc["version"] = int(doc.get("version", 1)) + 1
        doc["updated_at"] = now
        sig = _sign(canonical_bytes(doc), signing_key())
        _write_atomic(path, doc)
        os.makedirs(os.path.join(self.data_dir, "signatures"), exist_ok=True)
        with open(os.path.join(self.data_dir, "signatures",
                               "badge_users.json.sig"), "w", encoding="ascii") as f:
            f.write(base64.b64encode(sig).decode("ascii") + "\n")
        self._audit({"action": "grant" if enabled else "revoke",
                     "by": owner, "target": target, "badge": badge,
                     "reason": reason, "version": doc["version"], "at": now})
        return doc

    def _audit(self, rec):
        with open(self.audit_path, "a", encoding="utf-8") as f:
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")


STORE = Store(DATA_DIR)


class Handler(BaseHTTPRequestHandler):
    server_version = "grapeGram-badges/1"

    def _json(self, code, obj):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _body(self):
        n = int(self.headers.get("Content-Length", "0"))
        return json.loads(self.rfile.read(n) or b"{}")

    def _auth(self):
        h = self.headers.get("Authorization", "")
        return h[7:] if h.startswith("Bearer ") else ""

    def do_GET(self):
        if self.path == "/healthz":
            return self._json(200, {"ok": True})
        if self.path.startswith("/data/"):
            name = os.path.basename(self.path)
            for folder in ("", "signatures"):
                p = os.path.join(DATA_DIR, folder, name)
                if os.path.isfile(p) and name.endswith((".json", ".sig")):
                    with open(p, "rb") as f:
                        data = f.read()
                    self.send_response(200)
                    self.send_header("Content-Type", "application/octet-stream")
                    self.send_header("Content-Length", str(len(data)))
                    self.end_headers()
                    self.wfile.write(data)
                    return
        return self._json(404, {"error": "not found"})

    def do_POST(self):
        try:
            body = self._body()
            if self.path == "/challenge":
                return self._json(200, {"code": STORE.new_challenge(),
                                        "expires_in": CHALLENGE_TTL})
            if self.path == "/telegram/webhook":
                secret = os.environ.get("GRP_BOT_SECRET", "")
                if not secret or self.headers.get(
                        "X-Telegram-Bot-Api-Secret-Token") != secret:
                    return self._json(403, {"error": "bad secret"})
                msg = body.get("message") or {}
                text = (msg.get("text") or "").strip()
                if not text.startswith("/bind "):
                    return self._json(200, {"ok": True})
                token = STORE.bind(text.split(None, 1)[1].upper(),
                                   msg.get("from", {}).get("id"))
                return self._json(200, {"ok": True, "session": token})
            if self.path in ("/admin/grant", "/admin/revoke"):
                doc = STORE.mutate(self._auth(), str(body.get("user_id")),
                                   body.get("badge", "grapegram"),
                                   self.path.endswith("grant"),
                                   body.get("reason", ""))
                return self._json(200, {"ok": True, "version": doc["version"]})
            return self._json(404, {"error": "not found"})
        except PermissionError as e:
            return self._json(403, {"error": str(e)})
        except (ValueError, KeyError) as e:
            return self._json(400, {"error": str(e)})

    def log_message(self, fmt, *args):
        pass


if __name__ == "__main__":
    port = int(os.environ.get("GRP_PORT", "8080"))
    print(f"grapeGram badge service on :{port}")
    ThreadingHTTPServer(("0.0.0.0", port), Handler).serve_forever()
