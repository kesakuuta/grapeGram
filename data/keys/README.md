# Публичные ключи

`grapegram_ed25519.pub.pem` — публичный ключ, которым проверяются подписи
данных бейджей. Он вшивается в APK.

Приватная часть **никогда** не хранится здесь. Она живёт только в
`GRP_SIGNING_KEY_PEM` (GitHub Actions secret) или в офлайн-хранилище.

Текущий ключ — демонстрационный. Перед релизом:

```bash
python3 scripts/gen_keys.py --out /secure/grapegram_keys
cp /secure/grapegram_keys/grapegram_ed25519.pub.pem data/keys/
# приватный ключ -> секрет GRP_SIGNING_KEY_PEM, затем:
python3 scripts/sign_data.py --key /secure/grapegram_keys/grapegram_ed25519_private.pem \
    data/badge_users.json data/owner_users.json
```
