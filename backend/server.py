#!/usr/bin/env python3
"""grapeGram badge service. Python standard library only.

The service stores short-lived challenges/sessions on disk, verifies Telegram
identity from webhook updates, signs badge data with Ed25519 and publishes the
JSON + detached signature in one Git commit.
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
import urllib.error
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA_DIR = os.path.join(ROOT, "data")
ALLOWED_BADGES = {"grapegram", "founder", "developer", "contributor"}
ID_RE = re.compile(r"^[0-9]{5,20}$")
CHALLENGE_TTL = 300
SESSION_TTL = 3600
MAX_BODY = 16384


def _load(path):
    with open(path, "rb") as f:
        return json.load(f)


def _write_atomic(path, raw):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    tmp = path + ".tmp"
    with open(tmp, "wb") as f:
        f.write(raw)
        f.flush()
        os.fsync(f.fileno())
    os.chmod(tmp, 0o600)
    os.replace(tmp, path)


def _json_bytes(obj):
    return (json.dumps(obj, ensure_ascii=False, indent=2, sort_keys=True,
                        allow_nan=False) + "\n").encode("utf-8")


def _sign(payload, key_pem):
    with tempfile.NamedTemporaryFile(delete=False) as kf:
        kf.write(key_pem)
        key_path = kf.name
    with tempfile.NamedTemporaryFile(delete=False) as df:
        df.write(payload)
        data_path = df.name
    try:
        return subprocess.run(
            ["openssl", "pkeyutl", "-sign", "-inkey", key_path, "-rawin",
             "-in", data_path], check=True, capture_output=True).stdout
    finally:
        os.unlink(key_path)
        os.unlink(data_path)


def signing_key():
    pem = os.environ.get("GRP_SIGNING_KEY_PEM")
    if pem:
        return pem.encode("utf-8")
    path = os.environ.get("GRP_SIGNING_KEY_FILE")
    if path:
        with open(path, "rb") as f:
            return f.read()
    raise RuntimeError("GRP_SIGNING_KEY_FILE is not configured")


def _github_request(method, url, token, payload=None):
    data = None if payload is None else json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(url, data=data, method=method, headers={
        "Authorization": "Bearer " + token,
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
        "Content-Type": "application/json",
        "User-Agent": "grapeGram-badge-service/1",
    })
    try:
        with urllib.request.urlopen(req, timeout=20) as response:
            raw = response.read()
            return json.loads(raw) if raw else {}
    except urllib.error.HTTPError as e:
        detail = e.read(2048).decode("utf-8", "replace")
        raise RuntimeError("GitHub API returned %s: %s" % (e.code, detail))


def publish_badge_files(data, signature):
    """Publish both files atomically so clients never see mismatched versions."""
    token = os.environ.get("GRP_GITHUB_TOKEN", "")
    repo = os.environ.get("GRP_GITHUB_REPO", "")
    if not token or not repo:
        raise RuntimeError("GRP_GITHUB_TOKEN and GRP_GITHUB_REPO are required")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
        raise RuntimeError("invalid GRP_GITHUB_REPO")
    base = "https://api.github.com/repos/" + repo
    branch = os.environ.get("GRP_GITHUB_BRANCH", "main")
    ref = _github_request("GET", base + "/git/ref/heads/" +
                          urllib.parse.quote(branch, safe=""), token)
    parent = ref["object"]["sha"]
    commit = _github_request("GET", base + "/git/commits/" + parent, token)
    base_tree = commit["tree"]["sha"]

    def blob(raw):
        result = _github_request("POST", base + "/git/blobs", token, {
            "content": base64.b64encode(raw).decode("ascii"),
            "encoding": "base64",
        })
        return result["sha"]

    data_sha = blob(data)
    sig_sha = blob(signature)
    tree = _github_request("POST", base + "/git/trees", token, {
        "base_tree": base_tree,
        "tree": [
            {"path": "data/badge_users.json", "mode": "100644",
             "type": "blob", "sha": data_sha},
            {"path": "data/signatures/badge_users.json.sig", "mode": "100644",
             "type": "blob", "sha": sig_sha},
        ],
    })
    new_commit = _github_request("POST", base + "/git/commits", token, {
        "message": "chore(badges): publish badge assignments",
        "tree": tree["sha"],
        "parents": [parent],
    })
    _github_request("PATCH", base + "/git/refs/heads/" +
                    urllib.parse.quote(branch, safe=""), token, {
        "sha": new_commit["sha"], "force": False,
    })


def telegram_send(chat_id, text):
    token = os.environ.get("GRP_BOT_TOKEN", "")
    if not token:
        raise RuntimeError("GRP_BOT_TOKEN is not configured")
    payload = json.dumps({"chat_id": chat_id, "text": text}).encode("utf-8")
    req = urllib.request.Request(
        "https://api.telegram.org/bot%s/sendMessage" % token,
        data=payload, method="POST",
        headers={"Content-Type": "application/json", "User-Agent": "grapeGram/1"})
    with urllib.request.urlopen(req, timeout=15) as response:
        result = json.loads(response.read())
    if not result.get("ok"):
        raise RuntimeError("Telegram sendMessage failed")
    return result


class Store:
    def __init__(self, data_dir):
        self.data_dir = data_dir
        self.lock = threading.RLock()
        self.state_path = os.environ.get(
            "GRP_STATE_FILE", os.path.join(data_dir, "backend_state.json"))
        self.audit_path = os.path.join(data_dir, "audit.jsonl")
        self.challenges = {}
        self.sessions = {}
        self._load_state()

    def _load_state(self):
        try:
            state = _load(self.state_path)
            self.challenges = state.get("challenges", {})
            self.sessions = state.get("sessions", {})
        except FileNotFoundError:
            pass
        except (ValueError, OSError):
            raise RuntimeError("backend state file is unreadable; refusing to start")

    def _save_state(self):
        _write_atomic(self.state_path, _json_bytes({
            "challenges": self.challenges,
            "sessions": self.sessions,
        }))

    def owners(self):
        owners = _load(os.path.join(self.data_dir, "owner_users.json"))["owners"]
        if len(owners) != 2 or len(set(owners)) != 2:
            raise RuntimeError("owner list must contain exactly two unique IDs")
        return owners

    def new_challenge(self):
        code = secrets.token_hex(6).upper()
        with self.lock:
            now = time.time()
            self.challenges = {c: v for c, v in self.challenges.items()
                               if v["expires"] > now and not v.get("used")}
            self.challenges[code] = {"expires": now + CHALLENGE_TTL, "used": False}
            self._save_state()
        return code

    def bind(self, code, telegram_id):
        with self.lock:
            ch = self.challenges.get(code)
            if not ch or ch.get("used") or ch["expires"] < time.time():
                raise PermissionError("challenge is invalid or expired")
            if str(telegram_id) not in self.owners():
                raise PermissionError("not an owner")
            ch["used"] = True
            token = secrets.token_urlsafe(32)
            self.sessions[token] = {"owner": str(telegram_id),
                                    "expires": time.time() + SESSION_TTL}
            self._save_state()
            return token

    def discard_session(self, token):
        with self.lock:
            self.sessions.pop(token, None)
            self._save_state()

    def require_owner(self, token):
        with self.lock:
            session = self.sessions.get(token or "")
            if not session or session["expires"] < time.time():
                raise PermissionError("no valid session")
            if session["owner"] not in self.owners():
                raise PermissionError("not an owner")
            return session["owner"]

    def mutate(self, token, target, badge, enabled, reason=""):
        owner = self.require_owner(token)
        target = str(target or "")
        if not ID_RE.fullmatch(target):
            raise ValueError("bad user id")
        if badge not in ALLOWED_BADGES:
            raise ValueError("unknown badge")
        if not isinstance(reason, str) or len(reason) > 240:
            raise ValueError("reason must be a string of at most 240 characters")
        with self.lock:
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
            data = _json_bytes(doc)
            sig = base64.b64encode(_sign(data, signing_key())) + b"\n"
            # Persist locally, then publish the pair in one Git commit.
            _write_atomic(path, data)
            sig_path = os.path.join(self.data_dir, "signatures",
                                    "badge_users.json.sig")
            _write_atomic(sig_path, sig)
            publish_badge_files(data, sig)
            self._audit({"action": "grant" if enabled else "revoke",
                         "by": owner, "target": target, "badge": badge,
                         "reason": reason, "version": doc["version"], "at": now})
            return doc

    def _audit(self, record):
        line = (json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n").encode()
        fd = os.open(self.audit_path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
        try:
            os.write(fd, line)
        finally:
            os.close(fd)


STORE = Store(DATA_DIR)


class Handler(BaseHTTPRequestHandler):
    server_version = "grapeGram-badges/2"

    def _json(self, code, obj):
        body = json.dumps(obj, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _body(self):
        try:
            length = int(self.headers.get("Content-Length", "0"))
        except ValueError:
            raise ValueError("invalid content length")
        if length < 0 or length > MAX_BODY:
            raise ValueError("request body too large")
        if self.headers.get("Content-Type", "").split(";")[0] != "application/json":
            raise ValueError("application/json required")
        value = json.loads(self.rfile.read(length) or b"{}")
        if not isinstance(value, dict):
            raise ValueError("JSON object required")
        return value

    _challenge_lock = threading.Lock()
    _challenge_requests = {}

    def _allow_challenge(self):
        ip = self.client_address[0]
        now = time.time()
        with self._challenge_lock:
            recent = [t for t in self._challenge_requests.get(ip, [])
                      if now - t < 60]
            if len(recent) >= 10:
                self._challenge_requests[ip] = recent
                return False
            recent.append(now)
            self._challenge_requests[ip] = recent
            return True

    def _auth(self):
        header = self.headers.get("Authorization", "")
        return header[7:] if header.startswith("Bearer ") else ""

    def do_GET(self):
        if self.path == "/healthz":
            return self._json(200, {"ok": True, "service": "grapeGram-badges"})
        if self.path == "/data/badge_users.json":
            return self._file(os.path.join(DATA_DIR, "badge_users.json"))
        if self.path == "/data/signatures/badge_users.json.sig":
            return self._file(os.path.join(DATA_DIR, "signatures",
                                           "badge_users.json.sig"))
        return self._json(404, {"error": "not found"})

    def _file(self, path):
        try:
            with open(path, "rb") as f:
                data = f.read()
        except OSError:
            return self._json(404, {"error": "not found"})
        self.send_response(200)
        self.send_header("Content-Type", "application/octet-stream")
        self.send_header("Cache-Control", "no-store")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_POST(self):
        try:
            body = self._body()
            if self.path == "/challenge":
                if not self._allow_challenge():
                    return self._json(429, {"error": "too many challenges; wait one minute"})
                return self._json(200, {"code": STORE.new_challenge(),
                                        "expires_in": CHALLENGE_TTL,
                                        "instruction": "Send /bind CODE to the grapeGram bot"})
            if self.path == "/telegram/webhook":
                secret = os.environ.get("GRP_BOT_SECRET", "")
                if not secret or not secrets.compare_digest(
                        self.headers.get("X-Telegram-Bot-Api-Secret-Token", ""), secret):
                    return self._json(403, {"error": "bad secret"})
                message = body.get("message") or {}
                command = (message.get("text") or "").strip()
                if not command.startswith("/bind ") or not message.get("chat", {}).get("id"):
                    return self._json(200, {"ok": True})
                code = command.split(None, 1)[1].strip().upper()
                token = STORE.bind(code, (message.get("from") or {}).get("id"))
                try:
                    telegram_send(message["chat"]["id"],
                                  "grapeGram admin session (valid 60 min):\n" + token +
                                  "\nDo not share this token.")
                except Exception:
                    STORE.discard_session(token)
                    raise
                return self._json(200, {"ok": True, "bound": True})
            if self.path in ("/admin/grant", "/admin/revoke"):
                doc = STORE.mutate(self._auth(), body.get("user_id"),
                                   body.get("badge", "grapegram"),
                                   self.path.endswith("/grant"),
                                   body.get("reason", ""))
                return self._json(200, {"ok": True, "version": doc["version"]})
            return self._json(404, {"error": "not found"})
        except PermissionError as e:
            return self._json(403, {"error": str(e)})
        except (ValueError, KeyError, TypeError) as e:
            return self._json(400, {"error": str(e)})
        except Exception as e:
            # Do not leak credentials, paths or stack traces to remote callers.
            return self._json(503, {"error": "operation failed", "detail": str(e)[:160]})

    def log_message(self, fmt, *args):
        pass


if __name__ == "__main__":
    port = int(os.environ.get("GRP_PORT", "8080"))
    os.makedirs(DATA_DIR, exist_ok=True)
    print("grapeGram badge service listening on port %d" % port, flush=True)
    ThreadingHTTPServer(("127.0.0.1", port), Handler).serve_forever()
