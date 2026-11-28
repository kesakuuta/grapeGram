# grape-badge-tools

Rust-реализация инструментов для подписи и проверки данных бейджей grapeGram.
Соответствует Python-скриптам в `scripts/` по формату detached Ed25519-подписей,
именам файлов и правилам валидации. Один бинарник `grape-badge-tools`,
внешних рантайм-зависимостей (openssl, Python) нет.

## Сборка и тесты

```bash
cd rust/badge-tools
cargo build --release
cargo test
```

Бинарник: `target/release/grape-badge-tools`. В CI перед сборкой нужен
`dtolnay/rust-toolchain@stable` (или любой Rust 1.70+). `Cargo.lock`
генерируется при первой сборке и должен быть закоммичен для `--locked` в CI.

## Формат подписи

Подпись проверяется над **точными байтами файла на диске**. Это одинаково
работает для Android-клиента, Python badge-service и Rust badge-service.
`apply-change` записывает JSON в компактном каноническом виде, а затем
подписывает ровно записанные байты. Команда `sign` подписывает исходный файл
без его переформатирования.

**SHA-256 используется только для отпечатка** и не заменяет Ed25519:
подлинность подтверждается только detached Ed25519-подписью.

## Команды

### Генерация ключей

```bash
grape-badge-tools gen-keys --out /secure/grapegram_keys
```

Создаёт `grapegram_ed25519_private.pem` (PKCS#8, режим `0600`) и
`grapegram_ed25519.pub.pem` (SPKI). Каталог создаётся с режимом `0700`.
Если файлы уже есть — отказ без перезаписи. Случайное зерно берётся из
`/dev/urandom` (Linux/Android).

**Приватный ключ никогда не попадает в git, APK, чат или логи.** Единственное
рабочее место для него — защищённый секрет GitHub Actions
`GRP_SIGNING_KEY_PEM` либо офлайн-хранилище. В репозитории лежит только
публичный ключ `data/keys/grapegram_ed25519.pub.pem`.

### Подпись файлов

```bash
grape-badge-tools sign \
  --key /secure/grapegram_keys/grapegram_ed25519_private.pem \
  --out data/signatures \
  data/badge_users.json data/owner_users.json
```

На выходе `data/signatures/<basename>.sig` — одна строка base64 из 64 байт
подписи. Подписываются точные байты каждого JSON-файла.

### Проверка

```bash
grape-badge-tools verify --pub data/keys/grapegram_ed25519.pub.pem \
  data/badge_users.json data/signatures/badge_users.json.sig
```

Печатает `VALID` или `INVALID`; код выхода `1` при `INVALID`. Проверяется
подпись точных байтов файла. Для старых записей допускается проверка
канонического представления только как режим совместимости.

### Изменение бейджа (только для workflow с одобрением владельца)

```bash
ACTION=grant USER_ID=7293365262 BADGE=developer REASON="..." \
OWNER_ID=960792706 GRP_SIGNING_KEY_PEM="$SECRET" GITHUB_ACTOR="$ACTOR" \
grape-badge-tools apply-change --root .
```

Проверяется: `ACTION` ∈ {grant, revoke}, ID по `^[0-9]{5,20}# grape-badge-tools

Rust-реализация инструментов для подписи и проверки данных бейджей grapeGram.
Соответствует Python-скриптам в `scripts/` по формату detached Ed25519-подписей,
именам файлов и правилам валидации. Один бинарник `grape-badge-tools`,
внешних рантайм-зависимостей (openssl, Python) нет.

## Сборка и тесты

```bash
cd rust/badge-tools
cargo build --release
cargo test
```

Бинарник: `target/release/grape-badge-tools`. В CI перед сборкой нужен
`dtolnay/rust-toolchain@stable` (или любой Rust 1.70+). `Cargo.lock`
генерируется при первой сборке и должен быть закоммичен для `--locked` в CI.

## Формат подписи

Подпись проверяется над **точными байтами файла на диске**. Это одинаково
работает для Android-клиента, Python badge-service и Rust badge-service.
`apply-change` записывает JSON в компактном каноническом виде, а затем
подписывает ровно записанные байты. Команда `sign` подписывает исходный файл
без его переформатирования.

**SHA-256 используется только для отпечатка** и не заменяет Ed25519:
подлинность подтверждается только detached Ed25519-подписью.

## Команды

### Генерация ключей

```bash
grape-badge-tools gen-keys --out /secure/grapegram_keys
```

Создаёт `grapegram_ed25519_private.pem` (PKCS#8, режим `0600`) и
`grapegram_ed25519.pub.pem` (SPKI). Каталог создаётся с режимом `0700`.
Если файлы уже есть — отказ без перезаписи. Случайное зерно берётся из
`/dev/urandom` (Linux/Android).

**Приватный ключ никогда не попадает в git, APK, чат или логи.** Единственное
рабочее место для него — защищённый секрет GitHub Actions
`GRP_SIGNING_KEY_PEM` либо офлайн-хранилище. В репозитории лежит только
публичный ключ `data/keys/grapegram_ed25519.pub.pem`.

### Подпись файлов

```bash
grape-badge-tools sign \
  --key /secure/grapegram_keys/grapegram_ed25519_private.pem \
  --out data/signatures \
  data/badge_users.json data/owner_users.json
```

На выходе `data/signatures/<basename>.sig` — одна строка base64 из 64 байт
подписи. Подписываются точные байты каждого JSON-файла.

### Проверка

```bash
grape-badge-tools verify --pub data/keys/grapegram_ed25519.pub.pem \
  data/badge_users.json data/signatures/badge_users.json.sig
```

Печатает `VALID` или `INVALID`; код выхода `1` при `INVALID`. Проверяется
подпись точных байтов файла. Для старых записей допускается проверка
канонического представления только как режим совместимости.

### Изменение бейджа (только для workflow с одобрением владельца)

```bash
ACTION=grant USER_ID=7293365262 BADGE=developer REASON="..." \
OWNER_ID=960792706 GRP_SIGNING_KEY_PEM="$SECRET" GITHUB_ACTOR="$ACTOR" \
grape-badge-tools apply-change --root .
```

, бейдж из
{grapegram, founder, developer, contributor}, непустой PEM приватного ключа,
обе detached-подписи (`owner_users.json` и `badge_users.json`) и обязательный
`OWNER_ID` в подписанном allowlist. Затем инкрементируются
`version` и `record_version`, проставляется `updated_at` в UTC RFC3339
(без chrono — из `SystemTime`), переподписывается `badge_users.json`,
добавляется запись в `data/audit.jsonl`.

`badge_users.json` записывается **в каноническом виде без завершающего
перевода строки**: в этом случае проверка канонических байт и проверка сырых
байт (Python `verify_data.py`, будущий Kotlin-верификатор) дают один и тот же
результат. Запись файла и подписи идёт через временный файл + `rename`.

## Ротация ключей

1. `gen-keys --out <новый каталог>` (на защищённой машине).
2. Обновить `GRP_SIGNING_KEY_PEM` в настройках окружения `badge-admin`.
3. Заменить публичный ключ в `data/keys/grapegram_ed25519.pub.pem` и в
   ресурсах APK, после чего собрать и выпустить новую сборку.
4. В одном коммите переподписать **оба** файла (`badge_users.json`,
   `owner_users.json`) новым ключом и закоммитить новые `.sig`.
5. Проверить: `verify ...` для каждого файла, затем `cargo test`.
6. Старый приватный ключ уничтожить. Подписи старым ключом станут
   недействительными — поэтому шаги 3 и 4 нельзя разносить по релизам:
   клиент с новым публичным ключом должен получить новые подписи сразу.

## Совместимость и допущения

- Порядок ключей в Rust — `Vec<&str>::sort_unstable` по UTF-8, что для
  корректного UTF-8 совпадает с сортировкой Python по коду символа.
- Ограничение вложенности JSON — 128 уровней (по умолчанию serde_json).
- Целые числа больше `u64` / меньше `i64` не поддерживаются: канонизатор
  отвергает их как не-целые. В схемах бейджей таких значений нет.
- PEM парсится напрямую (base64 + фиксированные ASN.1-префиксы PKCS#8/SPKI),
  поэтому не нужны фичи `pkcs8`/`pem` у `ed25519-dalek`. Шифрованные PEM
  отклоняются.
- Зависимости: `ed25519-dalek 2`, `serde_json 1`, `base64 0.22`, `sha2 0.10`.
- Тесты используют только синтетические сид-значения; ни одного приватного
  ключа в исходниках нет.
