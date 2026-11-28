# grapeGram — статус и границы

> Этот репозиторий — **инфраструктурная часть** grapeGram: данные бейджей,
> схемы, подписи, серверная выдача, CI, документация и иконки.
> Клиент собирается в GitHub Actions; см. `.github/workflows/build.yml`.

## Что реально готово и проверено

| Компонент | Файл | Проверено |
|---|---|---|
| Данные владельцев (2 ID) | `data/owner_users.json` | схема + тест |
| Данные бейджей | `data/badge_users.json` | схема + тест |
| JSON-схемы | `data/schemas/*.schema.json` | тест |
| Ed25519 канонизация + подпись | `scripts/canonical.py`, `sign_data.py` | подпись/проверка прогоном |
| Проверка подписи | `scripts/verify_data.py` | прогон, tamper-тест |
| Генерация ключей | `scripts/gen_keys.py` | прогон |
| 9 иконок, имена, альфа | `assets/icons/` | декодирование PNG |
| Сервер бейджей (сессии, подпись, GitHub publish) | `backend/server.py` | unit-тесты + CI |
| Telegram long polling + SSH admin CLI | `backend/bot_polling.py`, `backend/admin_cli.py` | Python syntax + CI |
| Ubuntu VPS deployment | `backend/DEPLOY_UBUNTU.md`, `backend/systemd/` | шаблоны systemd |
| Badge-only CI (без APK rebuild) | `.github/workflows/badge-data.yml` | success |
| CI: сборка, проверка данных, подпись | `.github/workflows/` | синтаксис YAML |
| Оверлей клиента (настройки, иконки, бейджи) | `client/` + `rust/materializer/` | Rust-клиент, проверка якорей |
| Документация | `docs/` | — |

## Что невозможно в этом окружении

Полная сборка Telegram-форка требует: JDK 17, Android SDK/NDK, ~20–40 ГБ диска
и ~2 ГБ исходников трёх проектов. В контейнере на телефоне: нет JDK/SDK/NDK,
свободно **0.9 ГБ**, каталог `/storage/emulated/0` не смонтирован (это путь
внутри контейнера, не общая память Android). Поэтому клиент собирается
**только в GitHub Actions**, где раннер даёт JDK/SDK. Локально это не
проверить, и я не буду утверждать обратное.

## Что нужно от владельца

1. URL GitHub-репозитория (или разрешение создать `grapeGram`).
2. Исходники-основа: какой форк берём за базу (рекомендация в `docs/ARCHITECTURE.md`).
3. `GRP_SIGNING_KEY_PEM` — приватный Ed25519-ключ в GitHub Secrets (сгенерировать: `scripts/gen_keys.py`).
4. Telegram bot token для сервера выдачи (только в секретах сервера).

## Лицензия

Код форков — GPL-2.0 (Cherrygram) и GPL-3.0 (exteraGram, exteraless).
Производный клиент обязан распространяться под GPL-3.0 с сохранением
авторства. См. `CREDITS.md` и `docs/LICENSES.md`.
