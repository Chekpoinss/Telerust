# HANDOFF

Дата: 2026-11-10

## Реализовано

- GTK4/libadwaita окно, login view и split chat view.
- Собственная конфигурация `api_id`/`api_hash` с проверкой mode `0600`; test DC включён по умолчанию.
- Современный TDLib JSON C ABI загружается динамически; receive loop и сеть работают вне GTK main thread; TDLib logs отключены.
- QR login вызывает `requestQrCodeAuthentication`, принимает только ссылку `tg://login?token=` из `authorizationStateWaitOtherDeviceConfirmation`, строит QR матрицу и обновляет её при новых auth-state updates.
- QR → phone переключается через TDLib `destroy` до авторизации и перезапуск backend; phone/code/2FA/email → QR запрашивается методом TDLib.
- Авторизационные prompts телефона, кода, 2FA и email; успешная авторизация переключает UI к чатам.
- Явный logout вызывает TDLib `logOut`; после `authorizationStateClosed` bridge удаляет database key из Secret Service и запускает чистый auth instance. По контракту TDLib `logOut` удаляет локальные данные.
- Список личных/групповых чатов, последние 50 сообщений и явная отправка текста. Каналы заблокированы до поддержки sponsored messages.
- Лицензия проекта установлена как GPL-3.0-or-later. Полная inventory 185 registry crates и native/runtime notices — в `THIRD_PARTY_NOTICES.md`.
- GitHub Actions workflow включает системные GTK/DBus пакеты и fmt, clippy, check, test, release build.
- Публичный репозиторий создан: `https://github.com/Chekpoinss/Telerust`, ветка `main`; GitHub Actions прошёл полностью. Последний CI run до этого handoff: `42712611602`.

## Проверки на Fedora 45 GNOME Wayland (2026-11-10)

- RPM development-зависимости GTK4 4.24.1/libadwaita 1.10.0/DBus 1.16.2 установлены; `pkg-config` находит GTK4, libadwaita, DBus, Pango, GdkPixbuf, Cairo и Graphene.
- `cargo fmt --all -- --check` — прошёл.
- `cargo clippy --workspace --all-targets` — прошёл без предупреждений.
- `cargo check --workspace` — прошёл.
- `cargo test --workspace` — прошёл: 8 unit tests (QR matrix validation, TDLib state mapping, existing core/bridge tests), 0 failed.
- `cargo build --release -p telerust` — прошёл.
- Release GUI запускался в текущей Wayland-сессии 8 секунд с изолированными пустыми XDG config/data каталогами и завершён timeout без ошибки. Это проверяет запуск окна, но без TDLib config/runtime не проверяет отображение настоящего QR.
- TDLib 1.8.67 находится в `/tmp/telerust-tdlib/libtdjson.so`; `cargo run -p telerust-tdlib-bridge --example probe -- /tmp/telerust-tdlib/libtdjson.so` прошёл после изменений. Он проверяет загрузку/начальное состояние без входа.
- Старый UI-only замер 173608 KiB RSS / 1.500% CPU за 10 s относится к прежней сборке и не является новым benchmark.

## Не проверено

- Нет собственных API credentials и отдельного test-DC аккаунта. Живой QR/phone login, обновление QR от сервера, 2FA на реальной сессии, список реальных чатов, отправка/получение сообщений и logout в test DC не проверены.
- В этой среде нет установленной пользовательской конфигурации TDLib; QR матрица проверена синтетическим fixture, не выдуманным успешным входом.
- GUI с пустыми изолированными XDG каталогами был открыт в Wayland-сессии 8 секунд и завершён `timeout` (код 124 ожидаемый). Отрисовку полученной от живой TDLib QR-ссылки в активном окне отдельно визуально не подтвердили.
- Прямой TDLib QR flow/logout на живом test-DC аккаунте не проходил; успешный CI подтверждает сборку и unit-тесты, не авторизацию.
- Logout error handling проверен статически/unit-level mapping; фактическое удаление данных TDLib и database key в Secret Service не проходило на живой сессии.
- Нет выбора страны, повторной отправки кода, поиска чатов, пагинации, виртуализации, медиа, notifications, Secret Chats и полноценного reconnect.
- Версия TDLib runtime не закреплена; smoke probe ранее делался на 1.8.67.

## Продолжение

1. Для E2E после подготовки отдельной test app и test-DC аккаунта создать `0600` config с тестовыми credentials; не вносить их в репозиторий.
2. Проверить QR на телефоне, переключение в phone flow, 2FA, chats, history, send/receive и logout, не используя личный аккаунт.
3. Продолжить с виртуализованной пагинацией истории и воспроизводимыми ресурсными измерениями.

## Ограничения

- Ghost Mode и изменение read/online/typing state запрещены Telegram API Terms §1.4.
- Не использовать настоящие пользовательские сессии/сообщения в тестах и логах.
- Функции AyuGram не заявлять как поддерживаемые без отдельной проверки исходников и Terms.
