# Сборка grapeGram

## Архитектура

В репозитории хранится не полный Android-клиент, а overlay и Rust materializer.
Исходная база фиксируется в `BASE_COMMIT` внутри workflow и в
`client/config.json`. Сборка не должна использовать случайный upstream `main`.

1. GitHub Actions скачивает точный commit `exteraless/exteraless` и его submodules.
2. Отдельно получает закреплённые ресурсы иконок Cherrygram и exteraGram.
3. Собирает `rust/materializer` и запускает `grape-materializer --root <base>`.
4. Materializer копирует `client/overlay/java` и `client/overlay/res` в чистый checkout,
   затем применяет проверяемые текстовые патчи к upstream-файлам.
5. Перед Gradle выполняются проверки package name, label, launcher icon и конфигурации.
6. Gradle собирает подписанные Release APK и AAB одной командой.

Важно: исправления grapeGram должны находиться в `client/overlay/` или в
воспроизводимых патчах `rust/materializer/src/main.rs`. Изменения только в
`$RUNNER_TEMP/base` исчезнут при следующей сборке.

## Требования

- JDK 21 (Temurin)
- Python 3.11 (Chaquopy)
- Rust stable
- Android SDK: платформы и инструменты, указанные в
  `.github/workflows/build.yml`
- NDK 27.2.12479018
- CMake 3.22.1
- около 30 ГБ свободного места для полной нативной Android-сборки

## GitHub Actions

### Release

Файл: `.github/workflows/build.yml`.

Обычный push в `main` и pull request запускают data/crypto tests, Rust tests
и Android Release job (кроме изменений, исключённых `paths-ignore`).
В Android job:

- Gradle User Home кэшируется через `gradle/actions/setup-gradle`;
- Cargo registry, git dependencies и build outputs кэшируются через
  `Swatinem/rust-cache`;
- upstream checkout остаётся закреплённым по `BASE_COMMIT`;
- Debug не собирается в Release workflow;
- Release APK и AAB собираются одной командой:

```bash
./gradlew :TMessagesProj:assembleRelease :TMessagesProj:bundleRelease \
  --no-daemon --stacktrace --warning-mode=all --build-cache
```

Артефакты должны быть переименованы в `grapeGram-release.apk` и
`grapeGram-release.aab`, загружены и проверены на наличие. Подпись использует
`KEYSTORE_BASE64`, `KEYSTORE_PASS`, `ALIAS_NAME` и `ALIAS_PASS` из GitHub
Secrets. Не помещать keystore или пароли в репозиторий.

### Debug APK

Файл: `.github/workflows/debug.yml`. Запускается вручную через
GitHub Actions → workflow `debug` → Run workflow. Создаёт отдельный
`grapeGram-debug.apk`.

### Данные бейджей

`.github/workflows/data-verify.yml` проверяет unit tests, JSON signatures и
данные владельцев. Изменения только в badge data не должны запускать полную
Android-сборку.

### Подпись badge data

`.github/workflows/badges-sign.yml` — ручной workflow с environment
`badge-admin`. Приватный Ed25519-ключ берётся только из
`GRP_SIGNING_KEY_PEM`; его нельзя включать в APK или коммитить.

## Локальная сборка

После скачивания точного pinned upstream commit и submodules:

```bash
cargo test --release --manifest-path rust/materializer/Cargo.toml
cargo build --release --manifest-path rust/materializer/Cargo.toml
rust/materializer/target/release/grape-materializer --root /path/to/base
cd /path/to/base
./gradlew :TMessagesProj:assembleRelease :TMessagesProj:bundleRelease \
  --no-daemon --stacktrace --warning-mode=all --build-cache
```

Для локальной Release-подписи нужны те же значения keystore, что и в CI.
Не публикуйте локальные секреты и не копируйте их в `gradle.properties`.

## Что проверять после изменений

- Rust materializer tests и повторный запуск materializer на чистом checkout;
- проверку overlay и identity patches;
- Python unit tests и Ed25519 signatures;
- Release APK и AAB, их подписи и ненулевой размер;
- warning report, а не только общий статус Gradle;
- установку APK на реальном устройстве и ручную проверку навигации.

Успешные Rust/data jobs не означают, что Android APK/AAB собраны. Аналогично,
успешная компиляция не подтверждает работу экранов, сетевого badge-service или
синхронизации бейджей на устройстве.
