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
from urllib.error import HTTPError
from urllib.parse import quote
from urllib.request import Request, urlopen

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA_DIR = os.path.join(ROOT, "data")
SIG_DIR = os.path.join(DATA_DIR, "signatures")
KEY_DIR = os.path.join(DATA_DIR, "keys")

ALLOWED_BADGES = {"grapegram", "founder", "developer", "contributor"}
ID_RE = re.compile(r"^[0-9]{5,20}$")
HEX_COLOR_RE = re.compile(r"^#[0-9A-Fa-f]{6}(?:[0-9A-Fa-f]{2})?$")
BADGE_ANIMATIONS = {"none", "pulse", "sparkle", "glow"}
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

def validate_badge_style(style):
    if style is None:
        return None
    if not isinstance(style, dict):
        raise ValueError("style must be an object")
    allowed = {"color_light", "color_dark", "alpha", "particles", "animation"}
    if set(style) - allowed:
        raise ValueError("unknown style field")
    clean = {}
    for key in ("color_light", "color_dark"):
        if key in style:
            value = style[key]
            if not isinstance(value, str) or not HEX_COLOR_RE.fullmatch(value):
                raise ValueError("invalid " + key + ": expected #RRGGBB or #AARRGGBB")
            clean[key] = value.upper()
    if "alpha" in style:
        value = style["alpha"]
        if isinstance(value, bool) or not isinstance(value, (int, float)) or not 0 <= value <= 1:
            raise ValueError("alpha must be between 0 and 1")
        clean["alpha"] = float(value)
    if "particles" in style:
        if not isinstance(style["particles"], bool):
            raise ValueError("particles must be boolean")
        clean["particles"] = style["particles"]
    if "animation" in style:
        if style["animation"] not in BADGE_ANIMATIONS:
            raise ValueError("unknown badge animation")
        clean["animation"] = style["animation"]
    return clean


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
        self.mutation_lock = threading.Lock()
        self.challenges = {}
        self.sessions = {}
        self.audit_path = os.path.join(data_dir, "audit.jsonl")

    def owners(self):
        return _load(os.path.join(self.data_dir, "owner_users.json"))["owners"]

    def new_challenge(self) -> str:
        code, _ = self.new_challenge_pair()
        return code

    def new_challenge_pair(self):
        code = secrets.token_hex(4).upper()
        poll_token = secrets.token_urlsafe(32)
        with self.lock:
            now = time.time()
            self.challenges = {c: v for c, v in self.challenges.items()
                               if v["expires"] > now}
            self.challenges[code] = {"expires": now + CHALLENGE_TTL,
                                     "poll_token": poll_token,
                                     "session": None, "used": False}
        return code, poll_token

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
            ch["session"] = token
            return token

    def poll_challenge(self, code: str, poll_token: str):
        with self.lock:
            ch = self.challenges.get(code)
            if not ch or ch["expires"] < time.time():
                raise PermissionError("challenge is invalid or expired")
            if not secrets.compare_digest(ch["poll_token"], str(poll_token)):
                raise PermissionError("invalid challenge poll token")
            if not ch["session"]:
                return None
            token = ch["session"]
            del self.challenges[code]
            return token

    def require_owner(self, token: str) -> str:
        s = self.sessions.get(token or "")
        if not s or s["expires"] < time.time():
            raise PermissionError("no valid session")
        if s["owner"] not in self.owners():
            raise PermissionError("not an owner")
        return s["owner"]

    def mutate(self, token: str, target: str, badge: str, enabled: bool,
               reason: str = "", style=None):
        with self.mutation_lock:
            return self._mutate_locked(token, target, badge, enabled, reason, style)

    def _mutate_locked(self, token: str, target: str, badge: str, enabled: bool,
                       reason: str = "", style=None):
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
        if style is not None:
            entry["style"] = validate_badge_style(style)
        if enabled and "granted_at" not in entry:
            entry["granted_at"] = now
        doc["badges"][target] = entry
        doc["version"] = int(doc.get("version", 1)) + 1
        doc["updated_at"] = now
        _write_atomic(path, doc)
        with open(path, "rb") as f:
            sig = _sign(f.read(), signing_key())
        os.makedirs(os.path.join(self.data_dir, "signatures"), exist_ok=True)
        with open(os.path.join(self.data_dir, "signatures",
                               "badge_users.json.sig"), "w", encoding="ascii") as f:
            f.write(base64.b64encode(sig).decode("ascii") + "\n")
        self._audit({"action": "grant" if enabled else "revoke",
                     "by": owner, "target": target, "badge": badge,
                     "reason": reason, "style": entry.get("style", {}),
                     "version": doc["version"], "at": now})
        # The Android client reads public signed files from GitHub raw URLs.
        # Publish signature first and JSON second: clients reject the temporary
        # mismatch, then accept the new pair once both commits have landed.
        doc["_published"] = self._publish_github()
        return doc

    def _publish_github(self):
        token = os.environ.get("GRP_GITHUB_TOKEN", "").strip()
        repo = os.environ.get("GRP_GITHUB_REPO", "").strip()
        if not token or not repo:
            return False
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
            raise ValueError("invalid GRP_GITHUB_REPO")
        branch = os.environ.get("GRP_GITHUB_BRANCH", "main").strip() or "main"
        base = "https://api.github.com/repos/" + repo
        headers = {
            "Accept": "application/vnd.github+json",
            "Authorization": "Bearer " + token,
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "grapeGram-badge-service",
        }

        def api(method, path, payload=None):
            data = None if payload is None else json.dumps(payload).encode("utf-8")
            request = Request(base + path, data=data,
                              headers={**headers, "Content-Type": "application/json"},
                              method=method)
            with urlopen(request, timeout=30) as response:
                raw = response.read()
                return json.loads(raw.decode("utf-8")) if raw else {}

        # Merge the remote audit history first: Render's free container may have
        # restarted and lost its ephemeral filesystem since the previous grant.
        audit_path = os.path.join(self.data_dir, "audit.jsonl")
        local_audit = open(audit_path, "rb").read() if os.path.isfile(audit_path) else b""
        remote_audit = b""
        try:
            audit_doc = api("GET", "/contents/data/audit.jsonl?ref=" + quote(branch, safe=""))
            remote_audit = base64.b64decode(audit_doc.get("content", ""))
        except HTTPError as error:
            if error.code != 404:
                raise
        remote_lines = remote_audit.splitlines()
        known = set(remote_lines)
        merged_lines = remote_lines + [
            line for line in local_audit.splitlines() if line and line not in known
        ]
        merged_audit = b"\n".join(merged_lines) + (b"\n" if merged_lines else b"")
        with open(audit_path, "wb") as f:
            f.write(merged_audit)

        # Create one Git commit containing the signature, JSON and audit log.
        # This avoids transient invalid signature/data pairs and duplicate CI runs.
        ref = api("GET", "/git/ref/heads/" + quote(branch, safe="/"))
        parent_sha = ref["object"]["sha"]
        parent = api("GET", "/git/commits/" + parent_sha)
        tree_entries = []
        for name in ("badge_users.json", "signatures/badge_users.json.sig", "audit.jsonl"):
            with open(os.path.join(self.data_dir, name), "rb") as f:
                encoded = base64.b64encode(f.read()).decode("ascii")
            blob = api("POST", "/git/blobs", {"content": encoded, "encoding": "base64"})
            tree_entries.append({
                "path": "data/" + name,
                "mode": "100644",
                "type": "blob",
                "sha": blob["sha"],
            })
        tree = api("POST", "/git/trees", {
            "base_tree": parent["tree"]["sha"],
            "tree": tree_entries,
        })
        commit = api("POST", "/git/commits", {
            "message": "badge service: publish signed badge assignments",
            "tree": tree["sha"],
            "parents": [parent_sha],
        })
        api("PATCH", "/git/refs/heads/" + quote(branch, safe="/"), {
            "sha": commit["sha"],
            "force": False,
        })
        return True

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
                if os.path.isfile(p) and name.endswith((".json", ".sig", ".jsonl")):
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
                code, poll_token = STORE.new_challenge_pair()
                return self._json(200, {"code": code, "poll_token": poll_token,
                                        "expires_in": CHALLENGE_TTL})
            if self.path == "/challenge/status":
                token = STORE.poll_challenge(str(body.get("code", "")),
                                             str(body.get("poll_token", "")))
                if token is None:
                    return self._json(200, {"status": "pending"})
                return self._json(200, {"status": "bound", "session": token})
            if self.path == "/telegram/webhook":
                secret = os.environ.get("GRP_BOT_SECRET", "")
                if not secret or self.headers.get(
                        "X-Telegram-Bot-Api-Secret-Token") != secret:
                    return self._json(403, {"error": "bad secret"})
                msg = body.get("message") or {}
                text = (msg.get("text") or "").strip()
                if not text.startswith("/bind "):
                    return self._json(200, {"ok": True})
                STORE.bind(text.split(None, 1)[1].upper(),
                            msg.get("from", {}).get("id"))
                # The app obtains the session through its private poll token;
                # never return it in Telegram's webhook response.
                return self._json(200, {"ok": True})
            if self.path in ("/admin/grant", "/admin/revoke"):
                doc = STORE.mutate(self._auth(), str(body.get("user_id")),
                                   body.get("badge", "grapegram"),
                                   self.path.endswith("grant"),
                                   body.get("reason", ""), body.get("style"))
                return self._json(200, {"ok": True, "version": doc["version"],
                                        "published": bool(doc.pop("_published", False))})
            return self._json(404, {"error": "not found"})
        except PermissionError as e:
            return self._json(403, {"error": str(e)})
        except (ValueError, KeyError) as e:
            return self._json(400, {"error": str(e)})
        except Exception as e:
            print("badge service request failed:", repr(e), flush=True)
            return self._json(502, {"error": "badge service operation failed; inspect service logs"})

    def log_message(self, fmt, *args):
        pass


if __name__ == "__main__":
    port = int(os.environ.get("GRP_PORT", "8080"))
    print(f"grapeGram badge service on :{port}")
    ThreadingHTTPServer(("0.0.0.0", port), Handler).serve_forever()
