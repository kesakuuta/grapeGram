# grape-badge-tools

Rust-инструменты для генерации ключей, подписи, проверки и изменения данных бейджей grapeGram.

## Сборка и тесты

```bash
cd rust/badge-tools
cargo build --release
cargo test
```

Бинарник: `target/release/grape-badge-tools`. Для CI используется stable Rust.

## Формат подписи

Android-клиент, Python badge-service и Rust badge-service проверяют Ed25519 над точными байтами опубликованного файла. Команда `sign` подписывает исходные байты без переформатирования. Команда `apply-change` записывает JSON компактно, затем подписывает именно записанные байты.

Проверка `verify` принимает точную подпись файла и сохраняет обратную совместимость со старыми каноническими подписями. SHA-256 — только отпечаток, не замена Ed25519.

## Команды

### Генерация ключей

```bash
grape-badge-tools gen-keys --out /secure/grapegram_keys
```

Создаёт приватный PKCS#8 PEM с правами `0600` и публичный SPKI PEM. Существующие файлы не перезаписываются. В репозитории и APK допустим только публичный ключ; приватный ключ хранится в защищённом GitHub Secret `GRP_SIGNING_KEY_PEM` или офлайн-хранилище.

### Подпись файлов

```bash
grape-badge-tools sign \
  --key /secure/grapegram_keys/grapegram_ed25519_private.pem \
  --out data/signatures \
  data/badge_users.json data/owner_users.json
```

Создаёт `data/signatures/<basename>.sig`: base64 от 64 байт подписи.

### Проверка

```bash
grape-badge-tools verify --pub data/keys/grapegram_ed25519.pub.pem \
  data/badge_users.json data/signatures/badge_users.json.sig
```

Печатает `VALID` или `INVALID`; код выхода `1` при неверной подписи.

### Изменение бейджа

Команда предназначена только для доверенного workflow с защищённым окружением:

```bash
ACTION=grant USER_ID=7293365262 BADGE=developer REASON="..." \
OWNER_ID=960792706 GRP_SIGNING_KEY_PEM="$SECRET" GITHUB_ACTOR="$ACTOR" \
grape-badge-tools apply-change --root .
```

Перед изменением проверяются:
- `ACTION` — только `grant` или `revoke`;
- Telegram ID — формат `^[0-9]{5,20}$`;
- тип бейджа — `grapegram`, `founder`, `developer` или `contributor`;
- подписи `owner_users.json` и `badge_users.json` по опубликованному публичному ключу;
- обязательный `OWNER_ID` — он должен входить в подписанный allowlist;
- приватный Ed25519-ключ для переподписания результата.

Затем увеличиваются `version` и `record_version`, обновляются даты, данные записываются атомарно и подписываются. Команда не доверяет неподписанному owner allowlist или неподписанному исходному badge-файлу.

## Ротация ключей

1. Создать новую пару ключей на защищённой машине.
2. Обновить `GRP_SIGNING_KEY_PEM` в защищённом окружении.
3. Обновить публичный ключ в данных и ресурсах клиента.
4. Переподписать `badge_users.json` и `owner_users.json` новым ключом в согласованном релизе.
5. Проверить обе подписи и выполнить Rust/Python tests.
6. Уничтожить старый приватный ключ после перехода клиентов на новый публичный ключ.

## Допущения

- PEM поддерживает незашифрованные Ed25519 PKCS#8/SPKI ключи; зашифрованные PEM отклоняются.
- JSON парсится через serde_json; вложенность ограничена его стандартными лимитами.
- SHA-256 печатается только как контрольный отпечаток.
- Rust tests используют только синтетические ключи; приватного production-ключа в исходниках нет.
