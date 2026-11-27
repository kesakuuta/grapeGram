# Иконки приложения

В grapeGram доступно **58 вариантов** иконки:

- 9 собственных grapeGram;
- 18 из Cherrygram;
- 31 из exteraGram.

Полная машиночитаемая таблица с отображаемым названием, исходным ID, Android resource и автором (для community-иконок) находится в `assets/icons/upstream_catalog.json`. Исходные имена Android-ресурсов сохраняются, а компоненты launcher namespaced, чтобы не пересекаться с другими alias приложения.

## Собственные grapeGram

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

## Cherrygram — 18 вариантов

Вишня, Тёмная, Тёмная Bra, Белая, Белая Bra, Лагуна, Аква, Зелёная, Лаванда, Фиолетовый закат, Фиолетовый закат Bra, Закат, Восход, Пальмы, Пальмы Bra, Turbo, Nox, Новогодняя.

Исходные adaptive icon XML и vector layers берутся из Cherrygram commit `9e1c901e6fbca5a9a346683b91c6d529376f5304`.

## exteraGram — 31 вариант

Winter, Monet, Orbit, Editor, Mole, Echpochmak, Avatar, Candy, Guitar, Sus, Frog, Hamster, Sponge, Invincible, Flappy, Nyan Cat, Ice Age, Fish, Psychosis, XP, NASA, Cyberpunk, Space, Jellyfish, Human, Sunset, Amethyst, Aurora, osu!, Google, Google ’26.

Исходные adaptive icon XML, density assets и monochrome layers берутся из exteraGram commit `b4d852f48f2734a0bee1a74a0939be621612262a`. Community authors указаны в `upstream_catalog.json`.

## Размещение и выбор

- Единственное место выбора: **Настройки → Чаты → Иконка приложения**.
- Все 58 вариантов используют общий `LauncherIconController` и единый механизм activity-alias.
- При выборе включается ровно один alias, остальные отключаются.
- По умолчанию используется `grapegram_original_first`.
- Monet colors имеют fallback для старых Android и системные цвета на Android 12+.
- Исходные ресурсы upstream импортируются из зафиксированных commit SHA во время CI-сборки; main-ветки upstream не используются.

## Проверки

- Все alias names уникальны и существуют в manifest.
- Все enum entries имеют собственные title strings.
- Сохранение выбора после перезапуска и отсутствие дублирующихся launcher icons проверяются на устройстве.
