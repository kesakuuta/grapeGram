# Архитектура клиента grapeGram

Репозиторий хранит overlay и Rust materializer, а не второй полный checkout
Android-клиента. Базовый исходный код берётся из строго закреплённого commit
exteraless; случайная upstream-ветка \`main\` при сборке не используется.

\`\`\`
client/
├── config.json
└── overlay/
    ├── java/app/grapegram/
    │   ├── GrapeConfig.java
    │   ├── GrapeSettingsActivity.java
    │   ├── GrapeCreditsActivity.java
    │   ├── GrapeBadges.java
    │   ├── GrapeBadgeDrawable.java
    │   ├── GrapeBadgeBridge.java
    │   ├── GrapeBadgeAdminActivity.java
    │   ├── GrapeBadgesActivity.java
    │   ├── GrapeSecretStore.java
    │   └── GrapeApiGate.java
    ├── java/org/telegram/ui/Components/StatusBadgeComponent.java
    └── res/drawable-nodpi/
\`\`\`

Другие изменения в upstream-файлах описаны воспроизводимыми патчами
в \`rust/materializer/src/main.rs\`. Это важно: правка только временной папки
\`$RUNNER_TEMP/base\` исчезнет при следующем запуске.

## Закреплённая база

- Repository: \`exteraless/exteraless\`
- Commit: \`31a35e4683fb420ee782cdd32f36aea812bf756c\`
- Тот же SHA должен совпадать в \`client/config.json\` и \`BASE_COMMIT\`
  в GitHub Actions.

Materializer проверяет точные якоря. Если upstream изменит структуру файла,
сборка должна завершиться ошибкой вместо тихого пропуска патча.

## Что материализуется

1. Overlay Java и Android resources копируются в чистый checkout.
2. Package, label, launcher icon, Google Services client и release keystore
   wiring настраиваются для grapeGram.
3. Главный хаб настроек открывает отдельные страницы General, Appearance,
   Chats, Other, Plugins, Badges и Credits.
4. Launcher icon picker удаляется из ThemeActivity и добавляется в
   \`Настройки → Чаты\`; icon-pack editor оформления остаётся отдельной функцией.
5. Журнал активности плагинов добавляется в верхнюю панель PluginsActivity;
   разрешения остаются внутри управления плагинами.
6. AI provider keys мигрируют из plaintext SharedPreferences в шифрованное
   Android Keystore-хранилище; пустая конфигурация не содержит предустановленного
   пользовательского API-ключа.
7. Badge renderer подключается к DialogCell, chat header, UserCell и профилю.
   Подписанные данные загружаются с общего источника, кэш хранит и проверяет
   detached Ed25519 signature.
8. Патчи повторно применимы только к ожидаемой базе и падают при несовпадении
   контекста.

## Иконки

Каталог содержит 58 launcher-вариантов: 9 grapeGram, 18 Cherrygram и 31
exteraGram. Отображаемые названия и исходные ID перечислены в
\`assets/icons/upstream_catalog.json\`; commit SHA и источники ресурсов
описаны в \`docs/ICONS.md\`.

## Бейджи

Публичный ключ в APK используется только для проверки. Приватный ключ и
GitHub token не должны попадать в overlay или APK. Для выдачи бейджей
нужен развёрнутый HTTPS badge-service с Telegram bot secret, GitHub token
и signing key. \`badge_service\` в \`client/config.json\` по умолчанию пуст:
сервер нельзя считать развёрнутым только потому, что его исходники находятся
в репозитории.

## Локальная сборка

\`\`\`bash
cargo test --release --manifest-path rust/materializer/Cargo.toml
cargo build --release --manifest-path rust/materializer/Cargo.toml
rust/materializer/target/release/grape-materializer --root /path/to/clean/base
cd /path/to/clean/base
./gradlew :TMessagesProj:assembleRelease :TMessagesProj:bundleRelease \\
  --no-daemon --stacktrace --warning-mode=all --build-cache
\`\`\`

Нужны JDK 21, Rust stable, Android SDK/NDK/CMake версии из
\`.github/workflows/build.yml\`. Release signing secrets задаются отдельно;
не коммитьте keystore или пароли.
