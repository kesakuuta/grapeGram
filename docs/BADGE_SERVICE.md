# grapeGram badge service

The Android app verifies the Ed25519 signature over the exact bytes of
`data/badge_users.json`. The service must therefore publish both the JSON
and `data/signatures/badge_users.json.sig` to the configured GitHub branch.
The app reads those public files from the repository's raw URLs.

## Deploy

A Render Blueprint is provided in `render.yaml`. Connect the repository and
create the service. Set the following secret environment variables before
using administration:

- `GRP_SIGNING_KEY_PEM`: the private Ed25519 PEM key matching
  `data/keys/grapegram_ed25519.pub.pem`. Never commit or send this key.
- `GRP_BOT_SECRET`: a long random secret used as Telegram's webhook
  `secret_token`.
- `GRP_GITHUB_TOKEN`: a fine-grained GitHub token with Contents: Read and
  write access to `kesakuuta/grapeGram`.

The service uses `GRP_GITHUB_REPO=kesakuuta/grapeGram` and
`GRP_GITHUB_BRANCH=main` from the Blueprint. Do not expose the private
signing key or GitHub token to the Android application.

## Telegram owner binding

1. Set the Telegram bot webhook to
   `https://YOUR-SERVICE/telegram/webhook`, with the same
   `GRP_BOT_SECRET` as Telegram's `secret_token`.
2. In grapeGram, open Settings → grapeGram → Badges → Management.
3. Save the HTTPS service URL and tap **Connect via Telegram**.
4. Send the displayed `/bind CODE` command to the configured bot from the
   Telegram account whose numeric ID is listed in `data/owner_users.json`.
5. Tap **Check binding**. The app polls the challenge endpoint and stores the
   short-lived owner session locally.

Only IDs `7293365262` and `960792706` are owners in the signed allowlist.
The service checks the Telegram sender ID during binding and checks the owner
session again for every grant/revoke. A client-side owner check is only a UI
convenience; server authorization is authoritative.

## API and badge styles

- `POST /challenge` creates a one-time challenge and private polling token.
- `POST /challenge/status` returns pending or the bound owner session.
- `POST /telegram/webhook` accepts Telegram updates authenticated by the
  webhook secret.
- `POST /admin/grant` and `POST /admin/revoke` require
  `Authorization: Bearer <session>`.
- `GET /healthz` is the health check.

The style object accepts `color_light` and `color_dark` as `#RRGGBB` or
`#AARRGGBB`, `alpha` from 0 to 1, `particles` as a boolean, and
`animation` as `none`, `pulse`, `sparkle`, or `glow`.

A successful mutation is written to the local audit log and signed. If GitHub
publishing is not configured, the API reports `published: false`; the app
will not see the change through its public raw-data source until it is
published. When publishing is configured, the signature is uploaded before
the JSON so clients never accept a mismatched pair.
