# grape-badge-jni

Экспериментальная JNI-обёртка для Ed25519-проверки badge payload.

## Текущее состояние

**Библиотека пока не подключена к APK.** В Java-классе
`app.grapegram.GrapeBadges` нет native declarations и вызова
`System.loadLibrary("grapebadge")`; workflow выполняет только
`cargo check` для этого crate и не собирает `.so` через cargo-ndk.

Рабочая проверка подписи в приложении сейчас выполняется через Android JCA:
`Signature.getInstance("Ed25519")` с перебором доступных platform providers.
На Android 13+ это поддерживаемый путь. Не следует считать этот JNI crate
активным runtime-компонентом или утверждать, что он устраняет ошибки
`NoSuchAlgorithmException` на устройстве, пока не добавлена загрузка и
интеграционный тест.

## JNI API

Экспортируемые функции при будущей интеграции:

- `Java_app_grapegram_GrapeBadges_nativeVerify(byte[] payload, String sigB64, String pubPem) -> boolean`
- `Java_app_grapegram_GrapeBadges_nativeAvailable() -> boolean`

Ошибки входных данных и неверная подпись должны возвращать `false`, а не
пробрасывать panic в Java.

## Проверка crate

```sh
cargo test --manifest-path rust/badge-jni/Cargo.toml
cargo check --release --manifest-path rust/badge-jni/Cargo.toml
```

## Будущая интеграция

Если JCA provider окажется несовместим на поддерживаемом Android API, интеграция
потребует всех шагов вместе:

1. Добавить native declarations и безопасный fallback в `GrapeBadges.java`.
2. Собрать `libgrapebadge.so` через cargo-ndk для реально поддерживаемых ABI
   (сейчас upstream/Chaquopy конфигурация использует arm64-v8a и x86_64).
3. Убедиться, что Gradle включает библиотеки в APK/AAB.
4. Добавить device/instrumentation test: корректная подпись, неверная подпись,
   испорченный base64 и отсутствие библиотеки.
5. Только после этого считать JNI runtime bridge рабочим.
