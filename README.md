# Telerust

Первый development-прототип Telegram-клиента для Fedora 45 GNOME/Wayland. Интерфейс — GTK4/libadwaita; Telegram backend — TDLib через современный `td_create_client_id`/`td_send`/`td_receive`/`td_execute` JSON C ABI.

Лицензия проекта — [GPL-3.0-or-later](LICENSE). Лицензии crates.io и атрибуции внешних компонентов собраны в [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). GTK/libadwaita и TDLib используются как внешние system/runtime зависимости, исходники Telegram Desktop и AyuGram не включены.

## Состояние

Окно, локальная конфигурация credentials, QR login через TDLib, телефонный вход, prompts кода/2FA/email, список личных/групповых чатов, последние сообщения, текстовый composer и logout подключены. Production DC выбран по умолчанию; test DC включается явно и хранит данные в отдельной сессии. Запросы/updates идут вне GTK main thread. Каналы скрыты до реализации обязательного отображения sponsored messages. Настоящий production QR получен от TDLib; затем приложение достигло `Connected` и экрана `Chats`. Содержимое чатов не проверялось, переписки не открывались, сообщения не отправлялись.

## Fedora build dependencies

```sh
sudo dnf install cargo rust gtk4-devel libadwaita-devel dbus-devel \
  cmake gcc-c++ openssl-devel zlib-devel gperf
cargo build --workspace
cargo test --workspace
```

Для GNOME меню можно установить release сборку для текущего пользователя: `./scripts/install-user.sh`. При наличии проверенного runtime можно передать его через `TELERUST_TDLIB_SOURCE=/path/to/libtdjson.so ./scripts/install-user.sh`; скрипт запускает ABI probe и копирует library в `$XDG_DATA_HOME/telerust/lib/`. TDLib также ищется по этому пути без `LD_LIBRARY_PATH`, затем в обычных системных местах. Если GNOME настроен на manual proxy, Telerust передаёт эту настройку в TDLib; переменные `HTTPS_PROXY`/`HTTP_PROXY` используются как запасной источник. Удаление приложения: `./scripts/uninstall-user.sh`. Настройки и Telegram-сессия при удалении сохраняются.

Системная runtime TDLib не входит в Cargo сборку. Установите официальный TDLib (`tdjson`) через [сборочный генератор TDLib](https://tdlib.github.io/td/) и задайте путь к `libtdjson.so` в конфигурации ниже. Не используйте непроверенные бинарные сборки для production.

## Локальная конфигурация и вход

Получите собственные `api_id`/`api_hash` в [Telegram API development tools](https://my.telegram.org/apps): войдите на my.telegram.org, откройте API development tools, создайте приложение и возьмите выданные значения. При первом запуске нажмите **Set up Telegram API credentials**, введите значения и сохраните. Они записываются в `$XDG_CONFIG_HOME/telerust/config.toml` (обычно `~/.config/telerust/config.toml`) с правами `0600`. Перезапустите приложение, чтобы начать подключение. Не отправляйте credentials в чат и не добавляйте файл в Git.

Файл можно создать вручную:

```sh
install -d -m 700 "${XDG_CONFIG_HOME:-$HOME/.config}/telerust"
${EDITOR:-vi} "${XDG_CONFIG_HOME:-$HOME/.config}/telerust/config.toml"
chmod 600 "${XDG_CONFIG_HOME:-$HOME/.config}/telerust/config.toml"
```

Формат:

```toml
api_id = 123456
api_hash = "replace-with-your-32-character-api-hash"
# Необязательно; по умолчанию false (production DC)
use_test_dc = false
# Необязательно, если libtdjson.so находится в системном пути
tdlib_library = "/usr/local/lib/libtdjson.so"
```

Production DC используется по умолчанию. Test DC включается явно и предназначен только для отдельного тестового аккаунта; его нельзя смешивать с обычной сессией. Для тестовой среды создаётся отдельная локальная база. Не используйте там важную/личную переписку.

На первом входе приложение по очереди попросит номер, код, пароль 2FA или email-код согласно состоянию TDLib. UI не логирует ввод. Пользователю нужно самостоятельно инициировать отправку каждого сообщения кнопкой Send; приложение не отправляет сообщения при запуске или выборе чата.

Сессионный encryption key создаётся через системный CSPRNG и сохраняется в GNOME Secret Service. Если Secret Service отсутствует/заблокирован, backend остановится с ошибкой. TDLib хранит базу с правами `0700` в `$XDG_DATA_HOME/telerust/tdlib-prod` или `tdlib-test` соответственно.

## Проверка TDLib без аккаунта

Можно проверить shared library и ABI без авторизации/сети:

```sh
cargo run -p telerust-tdlib-bridge --example probe -- /path/to/libtdjson.so
```

## Границы

- Ghost Mode и изменение read/online/typing состояния не реализуются: это запрещено Telegram API Terms.
- Каналы пока не отображаются; это предотвращает доступ к канальному контенту до реализации официальных sponsored messages.
- Показаны последние 50 сообщений; пагинации, виртуализации, медиа, Secret Chats и передачи файлов пока нет.
- QR link от production TDLib и последующий переход приложения в `Connected`/`Chats` проверены. Список/история сообщений не изучались и обмен сообщениями не выполнялся.
- QR payload берётся только из TDLib `authorizationStateWaitOtherDeviceConfirmation.link`; при обновлении TDLib auth state QR перестраивается. Переключение из QR к телефону перезапускает неавторизованный TDLib инстанс через `destroy`.
- Logout проходит через `logOut`; по контракту TDLib он удаляет локальные TDLib данные после сетевого завершения. Bridge дополнительно удаляет encryption key из Secret Service и запускает чистый auth instance.
- Окно и библиотека прошли build/probe в локальной Fedora GNOME Wayland среде; `HANDOFF.md` содержит точные пройденные проверки и ограничения.
