# client — как собирается grapeGram

Репозиторий **не хранит** исходники форка. Он хранит оверлей, пин базовой
версии и Rust-материализатор.

```
client/
├── config.json            # base_repo, base_commit, data_base
└── overlay/
    ├── java/app/grapegram/    # GrapeConfig, GrapeBadges, GrapeSettingsActivity
    └── res/drawable-nodpi/    # 9 иконок
rust/materializer/         # Rust CLI, заменил Python apply.py
```

## Что делает `grape-materializer`

1. Принимает чистый checkout базы (`--root`) и копирует оверлей.
2. Патчит **идентичность клиента**:
   - `gradle.properties`: `APP_PACKAGE=app.grapegram` (реально другой package name);
   - `AndroidManifest.xml`: `android:label="@string/AppName"`, иконка и
     roundIcon — `@drawable/grape_icon_original`, базовый alias выключен;
   - `values/strings.xml` (и все локали): `AppName=grapeGram`;
   - `google-services.json`: второй client с package `app.grapegram`;
   - `TMessagesProj/build.gradle`: `storeType 'PKCS12'` для ключа подписи.
3. Патчит **единый хаб настроек**: один пункт «Настройки grapeGram» первым,
   старый верхнеуровневый пункт exteraless убирается.
4. Патчит 9 иконок: enum `LauncherIcon`, alias в манифесте, дефолт
   `GRAPE_ORIGINAL`, русские названия.
5. Проверяет результат и падает при сдвиге якорей.

Идемпотентен: повторный запуск ничего не дублирует.

## Локально

```bash
cargo build --release --manifest-path rust/materializer/Cargo.toml
rust/materializer/target/release/grape-materializer --root /path/to/base
cd /path/to/base && ./gradlew :TMessagesProj:assembleDebug
```

Нужно: JDK 21, Android SDK 37 / build-tools 36.0.0 / NDK 27.2.12479018,
Rust stable. Release-подпись — из GitHub Secrets (`KEYSTORE_BASE64`,
`KEYSTORE_PASS`, `ALIAS_NAME`, `ALIAS_PASS`).

## Обновление базы

1. Новый commit в `client/config.json` и `BASE_COMMIT` в `build.yml`.
2. Материализатор упадёт, если якоря изменились — поправить патчи.
3. Собрать в Actions.
