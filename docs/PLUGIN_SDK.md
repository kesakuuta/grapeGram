# Plugin SDK grapeGram

Совместимый слой над exteraGram SDK (Python + Chaquopy) и моделью разрешений
exteraless. Существующие плагины не ломаются: `BasePlugin`, `MethodHook`,
`MethodReplacement` сохраняют сигнатуры.

## Форматы

| Формат | Что внутри | Установка |
|---|---|---|
| `.plugin` | zip: `plugin.json` + `*.py` (+ ресурсы) | Python-плагин |
| `.elyx` | zip-проект Elyx (Python) | Python-плагин |
| `.eaf` | zip: DEX + манифест | DEX-плагин |
| `.dex` | один DEX | DEX-плагин |

`plugin.json` (минимум):

```json
{
  "id": "com.example.plugin",
  "name": "Example",
  "author": "someone",
  "version": "1.2.0",
  "sdk": 1,
  "permissions": ["ui", "network"],
  "entry": "main.py"
}
```

## Жизненный цикл

`install -> validate(schema+hash) -> show permissions -> user approves ->
load -> on_load -> hooks active -> on_unload -> unload`.

Целостность: SHA-256 каждого файла + общий хеш архива. Хеш ≠ подпись
издателя; подпись (если есть) проверяется отдельно.

## Хуки

```python
from base_plugin import BasePlugin, MethodHook

class Plugin(BasePlugin):
    def on_load(self):
        self.hook_all("org.telegram.messenger.MessageObject", "getId", Hook())

class Hook(MethodHook):
    def before_hooked_method(self, param):
        ...
    def after_hooked_method(self, param):
        ...
```

Каждый хук требует разрешение `hooks`. Сетевой вызов из плагина — `network`.
Файл вне песочницы — `files`. Перехват intent — `intents`. Загрузка `.so` —
`native`. Проверка выполняется в момент вызова.

## Безопасность

- Песочница: своя директория на плагин.
- Отключение при повторных падениях (порог 3).
- Безопасный режим при аварийном старте.
- Журнал: установка, разрешения, отказы, хуки, ошибки.
- AI-анализ — см. `docs/AI_ANALYZER.md`.

## Совместимость

`sdk` в манифесте = версия API. Несовпадение мажорной версии → предупреждение
и отказ от загрузки. Изменения SDK документируются в `docs/PLUGIN_SDK_CHANGELOG.md`.
