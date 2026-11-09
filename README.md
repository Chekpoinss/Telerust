# Telerust

Первый development-прототип Telegram-клиента для Fedora 45 GNOME/Wayland. Интерфейс — GTK4/libadwaita; Telegram backend — TDLib через современный `td_create_client_id`/`td_send`/`td_receive`/`td_execute` JSON C ABI.

Лицензия проекта — [GPL-3.0-or-later](LICENSE). Лицензии crates.io и атрибуции внешних компонентов собраны в [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). GTK/libadwaita и TDLib используются как внешние system/runtime зависимости, исходники Telegram Desktop и AyuGram не включены.

## Состояние

Окно, локальная конфигурация, QR login через TDLib, телефонный вход, prompts кода/2FA/email, список личных/групповых чатов, последние сообщения, текстовый composer и logout подключены. Запросы/updates идут вне GTK main thread. Каналы скрыты до реализации обязательного отображения sponsored messages. QR rendering и auth state mapping покрыты изолированными unit-тестами; живой тестовый login в этой среде не выполнялся.

## Fedora build dependencies

```sh
sudo dnf install cargo rust gtk4-devel libadwaita-devel dbus-devel \
  cmake gcc-c++ openssl-devel zlib-devel gperf
cargo build --workspace
cargo test --workspace
```

Системная runtime TDLib не входит в Cargo сборку. Установите официальный TDLib (`tdjson`) через [сборочный генератор TDLib](https://tdlib.github.io/td/) и задайте путь к `libtdjson.so` в конфигурации ниже. Не используйте непроверенные бинарные сборки для production.

## Локальная конфигурация и вход

Получите собственные `api_id`/`api_hash` в [Telegram API development tools](https://my.telegram.org/apps). Создайте файл вне репозитория:

```sh
install -d -m 700 "${XDG_CONFIG_HOME:-$HOME/.config}/telerust"
${EDITOR:-vi} "${XDG_CONFIG_HOME:-$HOME/.config}/telerust/config.toml"
chmod 600 "${XDG_CONFIG_HOME:-$HOME/.config}/telerust/config.toml"
```

Формат:

```toml
api_id = 123456
api_hash = "replace-with-your-32-character-api-hash"
use_test_dc = true
tdlib_library = "/usr/local/lib/libtdjson.so"
```

`use_test_dc` по умолчанию `true`; для первого теста используйте отдельный test-DC аккаунт. Условия Telegram предупреждают не хранить важную/личную переписку в тестовой среде. Не выключайте test DC для тестовой учётной записи. При переключении на production используется отдельный локальный каталог TDLib.

На первом входе приложение по очереди попросит номер, код, пароль 2FA или email-код согласно состоянию TDLib. UI не логирует ввод. Пользователю нужно самостоятельно инициировать отправку каждого сообщения кнопкой Send; приложение не отправляет сообщения при запуске или выборе чата.

Сессионный encryption key создаётся через системный CSPRNG и сохраняется в GNOME Secret Service. Если Secret Service отсутствует/заблокирован, backend остановится с ошибкой. TDLib хранит базу в `$XDG_DATA_HOME/telerust/tdlib-test` с правами `0700`.

## Проверка TDLib без аккаунта

Можно проверить shared library и ABI без авторизации/сети:

```sh
cargo run -p telerust-tdlib-bridge --example probe -- /path/to/libtdjson.so
```

## Границы

- Ghost Mode и изменение read/online/typing состояния не реализуются: это запрещено Telegram API Terms.
- Каналы пока не отображаются; это предотвращает доступ к канальному контенту до реализации официальных sponsored messages.
- Показаны последние 50 сообщений; пагинации, виртуализации, медиа, Secret Chats и передачи файлов пока нет.
- Авторизация и обмен с реальным test DC не проверялись в этом окружении: тестовые credentials/account не предоставлены.
- QR payload берётся только из TDLib `authorizationStateWaitOtherDeviceConfirmation.link`; при обновлении TDLib auth state QR перестраивается. Переключение из QR к телефону перезапускает неавторизованный TDLib инстанс через `destroy`.
- Logout проходит через `logOut`; по контракту TDLib он удаляет локальные TDLib данные после сетевого завершения. Bridge дополнительно удаляет encryption key из Secret Service и запускает чистый auth instance.
- Окно и библиотека прошли build/probe в локальной Fedora GNOME Wayland среде; `HANDOFF.md` содержит точные пройденные проверки и ограничения.
