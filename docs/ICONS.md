# Иконки приложения

9 изображений, 512×512, RGBA, прозрачные углы (проверено декодированием PNG).

| Файл | Название |
|---|---|
| `grapegram_original_first.png` | Оригинальная |
| `grape_cluster_01.png` | Аметист |
| `grape_cluster_03.png` | Мозаика |
| `grape_cluster_06.png` | Оникс |
| `grape_cluster_09.png` | Лиловый шёлк |
| `grape_style_02.png` | Неон |
| `grape_style_04.png` | Орбита |
| `grape_style_07.png` | Графит |
| `grape_style_09.png` | Фиалка |

Машиночитаемый список — `assets/icons/icons.json`.

## Реализация

- 9 grapeGram `<activity-alias>` для launcher-активности; в меню выбора
  отображаются только они. У каждого свой ресурс `@drawable/grape_icon_*`.
- Унаследованные exteraless aliases выключены и не показываются в выборе.
- `IconManager` хранит выбор в одном ключе `grape_icon` (значение — имя иконки
  из `icons.json`, по умолчанию `grapegram_original_first`).
- Применение: `setComponentEnabledSetting` — включить выбранный alias,
  выключить остальные, затем `DEFAULT`. Порядок важен, чтобы launcher не
  показал две иконки.
- Android 13+ : monochrome-слой для тематических иконок (отдельный
  `ic_launcher_monochrome`), splash — системный.
- Оба экрана (grapeGram → Иконка; Telegram → Настройки чатов → Иконка)
  используют один `IconManager` и один ключ — состояние синхронно.
- Возврат к стандартной: выбрать `Оригинальная`.

## Проверки

Выбор в обоих местах, сохранение после перезапуска, отсутствие дублей в
launcher, прозрачность углов, отсутствие чёрной рамки, splash на Android 12+.


## Cherrygram icon pack

The materializer imports 18 Cherrygram launcher variants (including dark, white,
laguna, aqua, green, lavanda, violet sunset, sunset, sunrise, palms, Turbo, Nox,
and Dark NY variants). Their vector foreground/background assets are preserved
from Cherrygram's `res-cherrygram/drawable` resources and assembled into
grapeGram-specific adaptive-icon aliases. They appear alongside the nine
grapeGram originals in the shared icon picker.
