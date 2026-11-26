# Карта функций (по реальным путям исходников)

Пути получены из деревьев репозиториев на 2026-11-26 (gh api git/trees).
Статус: `готово` — есть в нашем репо; `план` — переносится при сборке клиента;
`не проверено` — требует запуска на устройстве.

## Cherrygram (GPL-2.0)

| Функция | Файл / класс | Зависимости | Статус |
|---|---|---|---|
| Настройки-хаб | `uz/unnarsx/cherrygram/preferences/CGPreferencesEntry.java`, `CherrygramPreferencesNavigator.kt` | Telegram settings | план |
| Конфиги по темам | `core/configs/Cherrygram*Config.kt` (Appearance, Camera, Chats, Messages, Privacy, Experimental, Debug, Firebase) | SharedPreferences | план |
| Нижние вкладки | `core/ui/mainTabs/MainTabsManager.kt`, `org/telegram/ui/MainTabsActivity.java`, `MainTabsLayout.java` | MainTabsActivity | план |
| Иконки приложения | `preferences/AppIconsPreferencesEntry.java`, `preferences/cells/LauncherIconDrawable.java`, `org/telegram/ui/Cells/AppIconsSelectorCell.java` | activity-alias | план (наши 9 иконок) |
| Папки и их иконки | `preferences/folders/*`, `folders/helpers/FolderIconHelper.java` | DialogsActivity | план |
| Меню сообщений | `chats/CGMessageMenuInjector.kt`, `preferences/MessageMenuPreferencesEntry.java` | ChatActivity | план |
| Камера | `camera/BaseCameraView.java`, `CameraTypeSelector.java`, `CameraXUtils.java` | CameraX | план |
| Бэкап настроек | `core/configs/backup/BackupHelper.kt`, `BackupFileImportActivity.java`, `CherrygramResetKeys.kt` | SAF | план |
| Приватность | `preferences/PrivacyPreferencesEntry.java` + `core/configs/CherrygramPrivacyConfig.kt` | Telegram privacy | план |
| Экспорт контактов | `helpers/ContactsVcfExporter.java` | ContactsContract | план |

## exteraGram (GPL-3.0)

| Функция | Файл / класс | Зависимости | Статус |
|---|---|---|---|
| Plugin SDK | `src/full/java/com/exteragram/messenger/plugins/Plugin.kt` | Chaquopy | план |
| Runtime Python | `PluginsController.kt`, `PythonSdkUpdateDebug.kt` | Chaquopy, Python 3.11 | план |
| Хуки | `xposed/PyMethodHook.kt`, `PyMethodReplacement.kt` | XposedBridge-подобный слой | план |
| Каталог плагинов | `plugins/catalog/db/PluginCatalogDatabase` (+ schemas 1–4) | Room | план |
| Установка/обновление | `plugins/updates/SignedArtifact.kt`, `TelegramUpdateSource.kt` | подпись артефактов | план |
| Дев-инструменты | `utils/NativeCrashHandler.kt`, `PluginsWatchdog.kt`, `SdkSignature.kt` | — | план |
| UI плагинов | `ui/PluginsActivity.kt`, `ui/PluginsInfoActivity.kt`, `InstallPluginBottomSheet.kt` | RecyclerView | план |

## exteraless (GPL-3.0)

| Функция | Файл / класс | Зависимости | Статус |
|---|---|---|---|
| Разрешения плагинов | `app/exteraless/plugins/PluginPermissions.java` | SharedPreferences | модель перенесена в `docs/PERMISSIONS.md` |
| Статический анализ | `plugins/PluginCapabilityScan.java` (ключи `obfuscation`, `dex`, `DexInfo`) | DEX-парсер | план |
| Журнал аудита | `plugins/PluginAuditJournal.java` (`Entry`) | БД/файл | план |
| AI-служба | `ai/AiController.java`, `ai/network/Client.java`, `ai/data/*`, `ai/ui/AiSettingsActivity.java` | OpenAI-совместимый API | план |
| Плеер Material 3 | `player/*` (CoverImage, ControlsRow, Lyrics, LyricsFormats, OnlineLyrics) | Media3 | план |
| Обложки | `player/AudioFileCover.java`, `CoverImage.java` | MediaStore | план |
| Pill Stack | `pillstack/PillStackController.java`, `PillRegistry.java`, `pills/GhostPill.java` | — | план |
| Ghost Mode | `ghost/OwnLastSeen.java`, `com/radolyn/ayugram/utils/AyuGhostUtils.java` | Ayu DB | план |
| Иконки | `appicons/AppIcons.java`, `AppIconsActivity.java`, `IconPreviewView.java` | activity-alias | план (общая система) |
| Monet | `tw/nekomimi/nekogram/helpers/MonetHelper.java` | Android 12+ | план |
| QR-вход | `components/QRCodeSheet.java`, `org/telegram/ui/QrShare/*` | — | план |
| Оформление M3 | `appearance/*` (Material3SwitchHelper, GlassMenuHelper, FabShapeCell) | — | план |
| Настройки-реестр | `settings/utils/SettingsRegistry.java`, `OpenExtera*Activity.java` | — | план |
| Бэкапы | `backup/EtgBackup.java`, `AyuDatabaseImport.java`, `InvisibleEncryptor.java` | SAF | план |

## Наша инфраструктура (готово)

| Компонент | Файл | Статус |
|---|---|---|
| Бейджи: данные | `data/badge_users.json`, `data/owner_users.json` | готово |
| Схемы | `data/schemas/*.schema.json` | готово |
| Подпись Ed25519 | `scripts/sign_data.py`, `verify_data.py` | проверено |
| Сервер выдачи | `backend/server.py` | логика + тесты |
| Иконки | `assets/icons/` (9 шт., 512×512 RGBA) | проверено |
