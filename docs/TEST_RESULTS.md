# Результаты тестов

Прогон: 2026-11-26, окружение — контейнер на телефоне (Python 3.12, OpenSSL 3).

```
python3 -m unittest discover -s tests -v
Ran 17 tests ... OK
```

| Тест | Результат |
|---|---|
| `owner_users.json` = ровно два ID, строки | pass |
| структура `badge_users.json` | pass |
| канонизация не зависит от порядка ключей | pass |
| Ed25519 подпись/проверка (roundtrip, 64 байта) | pass |
| подделанный документ → INVALID | pass |
| чужая пара ключей → INVALID | pass |
| приватного ключа нет в репозитории | pass |
| challenge одноразовый | pass |
| просроченный challenge | pass |
| не-владелец не может привязать сессию | pass |
| не-владелец не может выдать бейдж | pass |
| выдача: подпись валидна, версия +1 | pass |
| отзыв бейджа | pass |
| неизвестный тип бейджа отклонён | pass |
| некорректный ID отклонён | pass |
| журнал аудита пишется | pass |
| 9 иконок: имена уникальны, 512×512, RGBA, прозрачные углы, непрозрачный центр | pass |

Дополнительно вручную:

```
python3 scripts/sign_data.py ...   -> 64-byte signatures
python3 scripts/verify_data.py ... -> VALID (оба файла)
scripts/apply_change.py grant/revoke -> version 2 -> 3 -> 4, VALID
OWNER_ID вне списка -> exit 1
```

## Не проведено

Сборка APK/AAB, запуск на устройстве, профилирование (FPS/jank/RAM),
UI-тесты, тесты Python/DEX-плагинов, сетевые тесты сервера под HTTPS.
Причина — нет JDK/SDK/NDK и устройства в этом окружении (см.
`docs/KNOWN_ISSUES.md`).
