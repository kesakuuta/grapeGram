#!/usr/bin/env python3
"""Long-polling Telegram bot for domainless/IP-only grapeGram deployments.

No inbound port or Telegram webhook certificate is required. Run as a separate
systemd service next to server.py. Only private chats are accepted for /bind.
"""
import json
import os
import time
import urllib.parse
import urllib.request

import server

BOT_TOKEN = os.environ.get("GRP_BOT_TOKEN", "")
if not BOT_TOKEN:
    raise SystemExit("GRP_BOT_TOKEN is required")
API = "https://api.telegram.org/bot" + BOT_TOKEN
OFFSET_FILE = os.environ.get(
    "GRP_BOT_OFFSET_FILE",
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "bot_offset"))


def api(method, params=None, timeout=40):
    data = urllib.parse.urlencode(params or {}).encode("utf-8")
    req = urllib.request.Request(
        API + "/" + method, data=data, method="POST",
        headers={"Content-Type": "application/x-www-form-urlencoded",
                 "User-Agent": "grapeGram-badge-bot/1"})
    with urllib.request.urlopen(req, timeout=timeout) as response:
        result = json.loads(response.read())
    if not result.get("ok"):
        raise RuntimeError("Telegram API request failed")
    return result.get("result")


def send(chat_id, message):
    api("sendMessage", {"chat_id": str(chat_id), "text": message}, timeout=15)


def load_offset():
    try:
        with open(OFFSET_FILE, encoding="ascii") as f:
            return int(f.read().strip())
    except FileNotFoundError:
        return 0


def save_offset(offset):
    tmp = OFFSET_FILE + ".tmp"
    with open(tmp, "w", encoding="ascii") as f:
        f.write(str(offset))
        f.flush()
        os.fsync(f.fileno())
    os.chmod(tmp, 0o600)
    os.replace(tmp, OFFSET_FILE)


def handle(update):
    message = update.get("message") or {}
    text = (message.get("text") or "").strip()
    chat = message.get("chat") or {}
    if chat.get("type") != "private" or not text:
        return
    command = text.split(None, 1)[0].split("@", 1)[0]
    if command == "/start":
        send(chat.get("id"), "grapeGram badge administration.\n"
             "To authorize this device, request a code in the admin app "
             "and send /bind CODE here.")
        return
    if command != "/bind":
        return
    parts = text.split()
    if len(parts) != 2:
        send(chat.get("id"), "Usage: /bind CODE")
        return
    token = None
    try:
        token = server.STORE.bind(parts[1].upper(), (message.get("from") or {}).get("id"))
        send(chat.get("id"), "grapeGram admin session (valid 60 min):\n" +
             token + "\nDo not share this token.")
    except PermissionError:
        send(chat.get("id"), "Authorization failed: code expired/invalid or account is not an owner.")
    except Exception:
        if token:
            server.STORE.discard_session(token)
        send(chat.get("id"), "Could not complete authorization. Try again later.")


def main():
    offset = load_offset()
    print("grapeGram badge bot: long polling started", flush=True)
    while True:
        try:
            updates = api("getUpdates", {
                "offset": str(offset),
                "timeout": "30",
                "allowed_updates": json.dumps(["message"]),
            }, timeout=40) or []
            for update in updates:
                try:
                    handle(update)
                except Exception:
                    # Do not log message text, challenge codes or session tokens.
                    pass
                offset = max(offset, int(update["update_id"]) + 1)
                save_offset(offset)
        except KeyboardInterrupt:
            return
        except Exception as exc:
            print("Telegram polling temporarily failed: " +
                  type(exc).__name__, flush=True)
            time.sleep(5)


if __name__ == "__main__":
    main()
