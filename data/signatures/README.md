# Подписи

Здесь лежат отделённые подписи Ed25519 в формате base64 (одна строка):

- `badge_users.json.sig`
- `owner_users.json.sig`

Пересоздать после изменения данных:

```bash
python3 scripts/sign_data.py --key /secure/grapegram_ed25519_private.pem \
    data/badge_users.json data/owner_users.json
```

Проверить:

```bash
python3 scripts/verify_data.py data/badge_users.json \
    data/signatures/badge_users.json.sig
```

Текущие подписи сделаны демонстрационным ключом. Для продакшена замените
ключ (см. `docs/BADGES.md`, раздел «Ротация ключей»).
