# Архитектура

## База

Три проекта — форки одного и того же Telegram Android (`TMessagesProj`).
Cherrygram ближе к чистому Telegram + модуль `uz/unnarsx/cherrygram`,
exteraGram добавляет `com/exteragram/messenger/plugins`, exteraless —
`app/exteraless` (разрешения, AI, плеер, оформление) и Ayu-базу
(`com/radolyn/ayugram`).

**Рекомендуемая база: exteraless.** Причины:
1. Самая свежая по объёму кода (29 389 файлов против 15 517 / 16 678).
2. Уже содержит слой разрешений и аудита — фундамент безопасности.
3. Уже содержит AI-службу и M3-плеер.
4. Совместима с plugin SDK exteraGram (тот же `TMessagesProj`).

Cherrygram переносим модулем `uz/unnarsx/cherrygram` + правки в
`org/telegram/ui` (MainTabs, AppIconsSelectorCell). exteraGram — модулем
`com/exteragram/messenger/plugins`.

## Модули grapeGram

```
app/                     # Gradle-модуль клиента (обёртка над TMessagesProj)
  src/main/java/app/grapegram/
    core/                # GrapeConfig, GrapeSettingsRegistry
    ui/                  # GrapeSettingsActivity (единый хаб)
    icons/               # IconManager (activity-alias)
    badges/              # BadgeStore, BadgeSync, BadgeVerifier
    plugins/             # PluginPermissionManager, PluginAudit, PluginScanner
    ai/                  # GrapeAiClient (поверх exteraless ai/)
native/                  # Rust (см. docs/RUST.md)
data/                    # подписанные JSON бейджей
backend/                 # сервер выдачи бейджей
```

## Единый пункт настроек

В `org/telegram/ui/SettingsActivity` (и в `settings/utils/SettingsRegistry.java`
exteraless) добавляется **один** заголовок в позицию 0:

```
Настройки grapeGram  ->  GrapeSettingsActivity
```

Внутри — разделы: Cherrygram, exteraGram, exteraless, AyuMoments, Плагины,
Разрешения, Иконки, Темы, Анимации, Производительность, AI, Бейдж.
Оригинальные строки Telegram не двигаются и не удаляются — вставляется
только один `HeaderCell` сверху.

## Иконки (одна настройка, два входа)

`IconManager` хранит выбор в одном `SharedPreferences`-ключе
`grape_icon`. Оба экрана (grapeGram → Иконка и Telegram → Настройки чатов →
Иконка) читают/пишут этот ключ и вызывают `PackageManager.setComponentEnabledSetting`
для 9 `activity-alias`. См. `docs/ICONS.md`.

## Бейджи

Чтение: подписанный JSON из GitHub → проверка Ed25519 → кэш →
отрисовка по Telegram ID. Запись: только сервер (`backend/server.py`) с
challenge-подтверждением через бота. См. `docs/BADGES.md`.

## Производительность

Список чатов и сообщений не трогаем структурно: только хуки в уже
существующие точки. Тяжёлые операции (разбор плагинов, проверка подписей,
парсинг больших JSON) выносим в Rust/фон. Профилирование — только на
устройстве, результаты без замеров не заявляются.
