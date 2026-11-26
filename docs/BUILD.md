# Сборка

## Локально (не в этом контейнере)

Нужно: JDK 17, Android SDK (platform 35, build-tools 35), NDK 27, ~30 ГБ диска.

```bash
git clone --recurse-submodules <base-fork> grapeGram
cd grapeGram
cp local.properties.example local.properties   # sdk.dir=...
./gradlew :TMessagesProj:assembleRelease
```

## GitHub Actions

`.github/workflows/build.yml`:

1. checkout (+submodules)
2. JDK 17 (temurin)
3. Android SDK + NDK + CMake
4. Python 3.11 (для Chaquopy)
5. Gradle cache
6. `assembleDebug`, `assembleRelease`, `bundleRelease`
7. тесты данных (`python3 -m unittest`)
8. артефакты: debug APK, release APK, AAB

Release-подпись — только из секретов (`KEYSTORE_BASE64`, `KEYSTORE_PASSWORD`,
`KEY_ALIAS`, `KEY_PASSWORD`). Без них собирается unsigned release.

## Проверка данных бейджей

`.github/workflows/data-verify.yml` — на каждый PR/push, затрагивающий `data/`:
схемы, два владельца, подпись.

## Подпись бейджей

`.github/workflows/badges-sign.yml` — `workflow_dispatch` с ручным
подтверждением (environment `badge-admin`), ключ из `GRP_SIGNING_KEY_PEM`.
Никогда не запускается автоматически по пользовательскому запросу.
