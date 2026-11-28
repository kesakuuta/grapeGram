# grape-badge-service

Rust reference implementation of the badge service in `backend/server.py`. Drop-in
HTTP API, in-memory one-time challenges and sessions, Ed25519 signing and
optional GitHub publication. No secrets are stored in this crate.

## Run

```sh
cargo build --release
cargo test
GRP_SIGNING_KEY_PEM="$(cat /secure/grapegram_ed25519_private.pem)" \
GRP_BOT_SECRET='<telegram secret token>' \
cargo run --release
```

## Environment

| Variable | Required | Meaning |
|---|---|---|
| `GRP_PORT` | no | Listen port, default `8080`, binds `0.0.0.0`. |
| `GRP_DATA_DIR` | no | Data root, default `../../data` relative to this crate. Must contain `owner_users.json` and `badge_users.json`. |
| `GRP_SIGNING_KEY_PEM` | for grant/revoke | PKCS8 Ed25519 **private** key in PEM form. The parser strips the PEM headers, base64-decodes the DER and uses the last 32 bytes as the `SigningKey` seed. Literal `\n` sequences are accepted. |
| `GRP_BOT_SECRET` | for webhook | Expected value of `X-Telegram-Bot-Api-Secret-Token`. If unset, every webhook is rejected. |
| `GRP_GITHUB_REPO` | no | `owner/repo`. When set (and non-empty), changed data files are published with `gh api`. Never set a token in this service or in the APK; `gh` uses its own authentication (`GH_TOKEN` or `gh auth login`). Tokens are never passed as command-line arguments. |

## Routes

| Route | Auth | Result |
|---|---|---|
| `GET /healthz` | none | `{"ok": true}` |
| `GET /data/<file>` | none | Public signed JSON and signature files; only supported data basenames. |
| `GET /data/audit.jsonl` | owner session | Audit history; requires `Authorization: Bearer <session>`. |
| `POST /challenge` | none | Returns a one-time code and an independent 256-bit poll token, TTL 300 s. |
| `POST /challenge/status` | poll token | Returns `pending` until the owner binds; then returns the session once to the app holding the poll token. |\n| `POST /telegram/webhook` | `X-Telegram-Bot-Api-Secret-Token` | Parses `/bind CODE`, reads `message.from.id`, and binds the session. The app retrieves it separately through its poll token. |
| `POST /admin/grant`, `POST /admin/revoke` | `Authorization: Bearer <session>` | Validates `user_id` and `badge`, updates `badge_users.json`, re-signs it, appends `audit.jsonl`, publishes if configured, returns `{"ok": true, "version": N}`. |

## Security notes

- The Telegram ID used for owner checks **comes only from the Telegram bot
  update**, never from the Android app. The app supplies `user_id` in admin
  requests only as the *target* whose badge is changed; it cannot grant itself
  anything without a valid owner session.
- The webhook secret header is mandatory and compared exactly; the app cannot
  forge a bot update.
- Challenges are single-use with a 300 s TTL; sessions expire after 3600 s and
  are re-checked against `data/owner_users.json` on every use.
- Owners, sessions and challenges are checked inside a `Mutex`; the audit trail
  is appended to `data/audit.jsonl`.
- Signatures use Ed25519. `canonical_bytes` matches `scripts/canonical.py`
  (UTF-8, sorted keys, compact separators, literal non-ASCII, integers only).
  As in `backend/server.py`, the stored JSON is written pretty-printed with
  sorted keys and signed verbatim; clients verify the exact served bytes.
- Publishing is skipped entirely unless `GRP_GITHUB_REPO` is set. No private
  key, bot secret or GitHub token is embedded in source, tests or the APK.

## Tests

`cargo test` covers: single-use challenge, non-owner rejection, challenge and
session expiry, target/badge validation, canonical JSON parity, sign/verify with
tamper detection, and manual PKCS8 PEM seed extraction.

## Data layout

```
data/
  owner_users.json
  badge_users.json
  audit.jsonl            (appended)
  signatures/
    badge_users.json.sig
```
