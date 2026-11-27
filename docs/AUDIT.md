# Аудит grapeGram (2026-11-27)

## Исправлено

| Проблема | Где | Исправление |
|---|---|---|
| POST-step `setup-gradle@v3` падал из-за `gradle-home-cache-cleanup` + Gradle 9.9.2 | старый `build.yml` | шаг убран в более ранних коммитах |
| `NoSuchAlgorithmException` при открытии «Бейджи» | `GrapeBadges.verify()` | R8 вырезал EdDSA-классы BC; добавлены `-keep` правила в `proguard-rules.pro` через материализатор; проверка через провайдеры платформа → AndroidOpenSSL → BC → Conscrypt. В release DEX теперь 57 ссылок на `.../asymmetric/edec` (было 0) |
| Подпись проверялась над каноническим JSON, а файлы подписаны как сырые байты | `GrapeBadges.fetch()` | проверка над сырыми байтами файла; `.sig` в репозитории валиден именно так |
| Новые бейджи не попадали в движок без перезапуска | `GrapeSettingsActivity.showBadges()` | после обновления вызывается `GrapeBadgeBridge.syncAsync()` |
| В главном меню были «Иконка», «Разрешения плагинов», «Журнал активности» | `GrapeSettingsActivity.java` | меню приведено к трём категориям; иконка — только в «Чаты», журнал — кнопкой в «Плагины», разрешения — из карточки плагина |
| AlertDialog «О grapeGram» вместо страницы | `GrapeSettingsActivity.java` | добавлена полноценная страница `GrapeCreditsActivity` на базе `BaseNekoSettingsActivity` |
| Принудительное подавление предупреждений | `build.yml`, `main.rs`, `CMakeLists` | удалены `GRADLE_OPTS=-Dorg.gradle.warning.mode=none`, `CFLAGS/CXXFLAGS=-w`, `-Xlint:-*`, `suppressSourceTargetDeprecationWarning`, `cmake -w`; добавлен `--warning-mode=all` |
| Устаревшие GitHub Actions (Node 20) | все workflow | `checkout@v7.0.1`, `setup-python@v7.1.0`, `setup-java@v6.3.0`, `upload-artifact@v7.0.2` |
| Ложное срабатывание сканера секретов на константе `PEM_BEGIN` | `build.yml` | сканер ищет реальный PEM-блок с телом, а не имя константы |
| Неиспользуемые импорты Rust | `rust/badge-service/src/store.rs` | убраны из библиотеки, добавлены в тестовый модуль |
| Groovy DSL warning `url value` / `version value` / `aidl value` | `settings.gradle`, `TMessagesProj/build.gradle` | переведено на `prop = value` |

## Оставшиеся предупреждения (не подавлены)

- C/C++ из зафиксированного апстрима `exteraless@31a35e`: `unqualified call to std::move`, `pSpatialLayer set but not used`, VLA, `format specifies type long long`, и др. Исправление затрагивает десятки файлов VoIP/WebRTC и не безопасно без регрессионных тестов звонков.
- Gradle 9.x: multi-string dependency notation в Chaquopy (внутри плагина, не в наших исходниках), `source/target 8` в библиотечных модулях media3, `task.project` access. Требуют обновления upstream/плагинов.
- Kotlin: deprecated `Locale(String)` в upstream `Translator.kt`.
- CMake dev-warnings от NDK-тулчейна и подпроектов.
- R8: «Unexpected reference» — информационные сообщенияshrinker'а.

Все они видны в логе, не фильтруются и не делают сборку красной.

## Проверка

- `data + crypto tests`: 18 тестов OK, подписи VALID.
- `rust crates`: тесты materializer/badge-tools/badge-service, `cargo check` JNI — success.
- `android build (signed)`: debug APK, release APK, AAB — success, артефакт `grapeGram-signed`.
- `owner_users.json`: ровно `7293365262`, `960792706`.
