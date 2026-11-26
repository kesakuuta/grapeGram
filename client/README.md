# client — как собирается grapeGram

Репозиторий **не хранит** исходники форка. Он хранит только оверлей и
пин базовой версии.

```
client/
├── config.json        # base_repo, base_commit, data_base
├── apply.py           # клон/скачивание базы + все патчи
└── overlay/
    ├── java/app/grapegram/   # GrapeConfig, GrapeBadges, GrapeSettingsActivity
    └── res/drawable-nodpi/   # 9 иконок
```

## Что делает `apply.py`

1. Скачивает архив базы по `base_commit` (или берёт готовый checkout).
2. Копирует `app/grapegram/**` и иконки.
3. Патчит:
   - `SettingsActivity.java` — единственный пункт «Настройки grapeGram» **первым**,
     старый верхнеуровневый пункт exteraless убирается (он внутри хаба);
   - `LauncherIconController.java` — 9 значений enum + дефолт `GRAPE_ORIGINAL`;
   - `AndroidManifest.xml` — 9 `activity-alias`, базовая иконка выключается;
   - `values/strings.xml` — названия иконок.
4. Проверяет результат (9 алиасов, XML парсится, маркеры на месте).
5. Идемпотентен: повторный запуск ничего не дублирует.

Любой сдвиг якорей в базе → падение с понятной ошибкой, а не тихо
сломанная сборка.

## Локально

```bash
python3 client/apply.py --root /path/to/base --download
cd /path/to/base && ./gradlew :TMessagesProj:assembleDebug
```

Нужно: JDK 21, Android SDK 37 / build-tools 36.0.0 / NDK 27.2.12479018,
Python 3.11.

## Обновление базы

1. Взять новый commit в `config.json`.
2. `apply.py` упадёт, если якоря изменились — поправить патчи.
3. Собрать в Actions.
