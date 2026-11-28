# grapeGram

Производный Android-клиент на базе exteraless с компонентами Cherrygram и
exteraGram. Исходная основа фиксируется по commit SHA в
`.github/workflows/build.yml`; клиент материализуется из overlay в CI.

## Состав проекта

- `client/overlay` — настройки grapeGram, UI бейджей, launcher icons и credits.
- `rust/materializer` — воспроизводимое применение overlay к чистой основе.
- `backend/server.py` — owner-only сервис выдачи/отзыва бейджей.
- `data/` — подписанный Ed25519 список бейджей и allowlist владельцев.
- `assets/icons/` — 9 собственных иконок grapeGram.
- Cherrygram: 18 вариантов; exteraGram: 31 вариант. Их ресурсы подтягиваются
  в CI из закреплённых upstream commit SHA, а не копируются вручную в APK.
- `docs/` — архитектура, иконки, лицензии и настройка badge-service.

## Бейджи

Владельцы, которым разрешено управлять бейджами:

- `7293365262`
- `960792706`

Клиент принимает только JSON с корректной Ed25519-подписью. Бейджи имеют
индивидуальные настройки светлого/тёмного HEX-цвета, alpha, частиц и анимации.
Отображение подключено к профилю, заголовку чата и списку диалогов.

Административная панель использует Telegram challenge binding. Сервер повторно
проверяет Telegram ID и owner-сессию при каждой операции; проверка ID в UI
не является механизмом авторизации. История операций сохраняется в
`data/audit.jsonl`.

### Развёртывание badge-service

Для Render подготовлен Blueprint `render.yaml` и Dockerfile
`backend/Dockerfile`. Инструкция: [docs/BADGE_SERVICE.md](docs/BADGE_SERVICE.md).

Для рабочего сервиса владелец должен задать секреты на хостинге:

- `GRP_SIGNING_KEY_PEM` — приватный Ed25519 PEM, соответствующий
  `data/keys/grapegram_ed25519.pub.pem`;
- `GRP_BOT_SECRET` — секрет Telegram webhook;
- `GRP_GITHUB_TOKEN` — fine-grained token с Contents read/write для
  `kesakuuta/grapeGram`.

Telegram bot token используется только при настройке webhook через Bot API;
его не нужно помещать в APK или репозиторий. После развёртывания URL сервиса
сохраняется в grapeGram через экран управления бейджами. Без GitHub token
сервис явно сообщает, что изменение сохранено локально, но не опубликовано.

## Сборка и проверки

GitHub Actions workflow `build`:

1. проверяет Python/data/crypto тесты и Rust crates;
2. получает закреплённые исходники и upstream icon resources;
3. применяет Rust overlay и проверяет package identity;
4. собирает подписанные release APK и AAB;
5. публикует артефакты и полный Gradle log.

Сборка запускается для pull request и изменений клиентского кода в `main`.
Изменения только в данных бейджей проходят отдельный `data-verify` workflow,
не запуская длительную Android-сборку.

Открой [GitHub Actions](https://github.com/kesakuuta/grapeGram/actions), выбери
успешный запуск `build` и скачай артефакт `grapeGram-signed` с APK и AAB.
Для текущей работы используется отдельная ветка
`checkpoint/pre-completion-2026-11-28`; изменения не попадают в `main`
до завершения проверок.

## Лицензии

Производный клиент должен соблюдать лицензии исходных проектов. См.
`CREDITS.md` и [docs/LICENSES.md](docs/LICENSES.md).
