# grape-badge-jni

JNI-библиотека проверки Ed25519-подписей бейджей. Убирает зависимость от
`java.security.Signature("Ed25519")` / BouncyCastle, из-за которой на устройстве
возникал `NoSuchAlgorithmException`.

Сигнатура закрывает **точные байты** `badge_users.json` (канонический JSON без
переводов строк), как и в `scripts/canonical.py` и `backend/server.py`.

## Экспортируемые методы

- `Java_app_grapegram_GrapeBadges_nativeVerify(byte[] payload, String sigB64, String pubPem) -> boolean`
- `Java_app_grapegram_GrapeBadges_nativeAvailable() -> boolean` (всегда `true`, smoke-test загрузки `.so`)

Любая ошибка (битые входные данные, неверная длина, неверная подпись, panic)
возвращает `false` — исключение в Java не вылетает.

## Сборка

Нужны Android NDK и `cargo install cargo-ndk`. Команда запускается из каталога
`rust/badge-jni`; `$BASE_ROOT` — корень чистого чекаута базы из материализатора
(каталог, где лежат `TMessagesProj/` и `gradle.properties`).

```sh
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -t x86 \
  -p 23 \
  -o "$BASE_ROOT/TMessagesProj/jni" \
  --release \
  build
```

Результат — по одному файлу на ABI:

```
TMessagesProj/jni/arm64-v8a/libgrapebadge.so
TMessagesProj/jni/armeabi-v7a/libgrapebadge.so
TMessagesProj/jni/x86_64/libgrapebadge.so
TMessagesProj/jni/x86/libgrapebadge.so
```

`[lib] name = "grapebadge"` даёт имя `libgrapebadge.so`; `strip = true` и
`lto = true` в release уменьшают размер. Для загрузки в Java достаточно
`System.loadLibrary("grapebadge")`.

## Проверка без Android

```sh
cargo test
cargo build --release
```

Тесты покрывают roundtrip подпись/проверка, испорченную подпись, base64 без
padding и некорректный PEM.
