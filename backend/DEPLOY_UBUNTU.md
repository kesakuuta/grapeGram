# Развёртывание backend бейджей на Ubuntu 22.04/24.04

## Схема без домена и открытых портов

На VPS работают два systemd-сервиса:
- `grapegram-badges`: API слушает только `127.0.0.1:8080`;
- `grapegram-bot`: Telegram long polling, исходящее соединение к Telegram.

Входящие порты открывать не нужно. Telegram webhook и домен не нужны. Для администрирования API используется SSH-туннель. Публичный список бейджей остаётся на GitHub Raw; сервер публикует JSON и подпись одним Git-коммитом.

## Требования

- Ubuntu 22.04 или 24.04;
- SSH с sudo;
- Python 3 и OpenSSL;
- GitHub fine-grained PAT с доступом Contents: Read and write только к `kesakuuta/grapeGram`;
- Telegram bot token;
- действующий приватный Ed25519-ключ, соответствующий `data/keys/grapegram_ed25519.pub.pem`.

**Не генерируйте новую пару ключей без ротации публичного ключа в приложении.** Если приватный ключ, которым подписаны текущие данные, утерян, сначала потребуется ротация ключа и новая сборка APK.

## Установка

Выполнять на VPS под пользователем с sudo:

```bash
sudo apt update
sudo apt install -y git python3 openssl
sudo useradd --system --home /opt/grapegram --create-home --shell /usr/sbin/nologin grapegram
sudo mkdir -p /opt/grapegram /etc/grapegram /var/lib/grapegram
sudo chown -R grapegram:grapegram /opt/grapegram /var/lib/grapegram
sudo -u grapegram git clone https://github.com/kesakuuta/grapeGram.git /opt/grapegram
```

Создать конфигурацию из шаблона:

```bash
sudo install -o root -g grapegram -m 0640 /opt/grapegram/backend/badges.env.example /etc/grapegram/badges.env
sudo nano /etc/grapegram/badges.env
```

Заполнить `GRP_BOT_TOKEN`, `GRP_GITHUB_TOKEN`, `GRP_GITHUB_REPO` и `GRP_GITHUB_BRANCH`. PAT не должен иметь доступ к другим репозиториям. Не вставляйте секреты в чат или GitHub.

Поместить приватный PEM-ключ в файл:

```bash
sudo install -o root -g grapegram -m 0640 /path/to/your/grapegram_ed25519_private.pem /etc/grapegram/ed25519_private.pem
```

Проверить, что публичная часть этого ключа совпадает с опубликованным ключом:

```bash
sudo openssl pkey -in /etc/grapegram/ed25519_private.pem -pubout -out /tmp/grapegram-public.pem
diff -u /opt/grapegram/data/keys/grapegram_ed25519.pub.pem /tmp/grapegram-public.pem
sudo rm /tmp/grapegram-public.pem
```

Если `diff` показывает различия, **не запускайте выдачу бейджей**. Нужна ротация ключа и обновление клиента.

Установить сервисы:

```bash
sudo install -o root -g root -m 0644 /opt/grapegram/backend/systemd/grapegram-badges.service /etc/systemd/system/
sudo install -o root -g root -m 0644 /opt/grapegram/backend/systemd/grapegram-bot.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now grapegram-badges.service grapegram-bot.service
sudo systemctl status grapegram-badges.service grapegram-bot.service
```

Если у бота ранее был установлен webhook, удалить его один раз, подставив токен локально на VPS:

```bash
read -rsp 'Bot token: ' BOT_TOKEN; echo
curl -fsS -X POST "https://api.telegram.org/bot${BOT_TOKEN}/deleteWebhook" -d 'drop_pending_updates=false'
unset BOT_TOKEN
```

## Проверка

```bash
curl -fsS http://127.0.0.1:8080/healthz
```

Проверить журналы:

```bash
sudo journalctl -u grapegram-badges -n 100 --no-pager
sudo journalctl -u grapegram-bot -n 100 --no-pager
```

API не слушает внешний интерфейс. Для временного доступа с компьютера открыть SSH-туннель:

```bash
ssh -L 8080:127.0.0.1:8080 USER@VPS_IP
```

Затем на локальном компьютере обращаться к `http://127.0.0.1:8080`. Не публикуйте порт 8080 напрямую в интернет.

## Выдача через SSH CLI (пока нет Android-панели)

После запуска сервисов подключиться по SSH и создать код:

```bash
cd /opt/grapegram
sudo -u grapegram python3 backend/admin_cli.py challenge
```

Отправить полученную команду `/bind КОД` боту в личном чате. Бот пришлёт Bearer token. Затем выдать или отозвать бейдж:

```bash
sudo -u grapegram python3 backend/admin_cli.py grant 123456789 --badge founder --reason "founding contributor"
sudo -u grapegram python3 backend/admin_cli.py revoke 123456789 --badge founder --reason "revoked by owner"
```

CLI запросит токен скрытым вводом; он не попадёт в историю shell. Утилита обращается только к локальному API.

## Авторизация владельца и выдача

1. Клиент/админ-интерфейс запрашивает `POST /challenge` с JSON body `{}`.
2. Владелец отправляет боту `/bind КОД` в личном чате.
3. Бот проверяет настоящий Telegram `from.id` по двум ID в `data/owner_users.json` и присылает одноразовый Bearer session token. Сессия живёт 60 минут и переживает перезапуск процесса.
4. Запрос `POST /admin/grant` или `POST /admin/revoke` с заголовком `Authorization: Bearer TOKEN` и JSON `{"user_id":"123456789","badge":"founder","reason":"..." }`.
5. Сервер подписывает новый JSON Ed25519 и публикует JSON + .sig одним GitHub commit. Клиент продолжает проверять подпись публичным ключом.

Список бейджей: `grapegram`, `founder`, `developer`, `contributor`.

## Важные замечания

- Сейчас API готов для интеграции с админ-интерфейсом; сам интерфейс управления в Android-клиенте ещё не подключён.
- Публикация в GitHub запускает workflow репозитория. Частые выдачи могут запускать Android CI; позже следует отделить data-only публикации от полной сборки APK.
- Не удаляйте `/var/lib/grapegram/backend_state.json`: там хранятся активные challenges и сессии.
- Делайте резервную копию приватного ключа офлайн. Не храните его в репозитории.
