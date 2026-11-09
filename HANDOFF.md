# HANDOFF

Дата: 2026-11-10

## Работает в текущем прототипе

- GTK4/libadwaita main window, login view и split chat view.
- Конфигурация `$XDG_CONFIG_HOME/telerust/config.toml` с обязательными mode `0600` и проверкой `api_id/api_hash`.
- TDLib загружается динамически через современный `td_create_client_id`/`td_send`/`td_receive`/`td_execute` API; legacy `td_json_client_*` не используется. Receive loop и сеть изолированы от GTK main thread.
- Auth prompt handling для phone, code, 2FA password и email challenge.
- Список чатов показывает title и preview для личных/групповых чатов; последние 50 сообщений загружаются при открытии.
- Явная отправка обычного текста кнопкой Send/Enter; автоматической отправки при старте/открытии чата нет.
- TDLib database encryption key берётся из GNOME Secret Service; test/prod директории раздельны и закрыты `0700`.
- Test DC включён по умолчанию.
- Каналы скрыты и команды открытия/отправки в них блокируются до поддержки sponsored messages.
- В CI добавлен workflow, устанавливающий GTK4/libadwaita/DBus devel packages и запускающий fmt, clippy, check, test и release build.

## Проверки, записанные ранее (2026-11-10, другая конфигурация окружения)

- `cargo fmt --all -- --check` — прошёл после последнего изменения.
- `cargo check --workspace` — прошёл на Fedora 45; SDK-пакеты были распакованы локально в `/tmp`, потому что GTK/libadwaita devel не установлены системно, а sudo недоступен.
- `cargo test --workspace` — 5 unit-тестов прошло; тесты не авторизуются и не используют сеть.
- TDLib modern C JSON ABI probe на временной библиотеке TDLib 1.8.67 — client ID создан, начальный auth state получен, затем клиент закрыт; входа/сети не было, TDLib logs отключены.
- GTK application запускался и регистрировался в D-Bus в текущей GNOME Wayland-сессии; конфигурации аккаунта не было.
- UI-only замер в `BENCHMARKS.md`: RSS/HWM 173608 KiB, средний CPU 1.500% в течение 10 s; не использовать как цифру полного клиента.

## Проверки в текущем окружении (2026-11-10)

- `cargo fmt --all -- --check` — прошёл.
- `cargo check --workspace`, `cargo clippy --workspace --all-targets` и `cargo build --release -p telerust` — не завершились: отсутствуют системные development библиотеки и pkg-config файлы GTK4/Pango/DBus/Cairo/GdkPixbuf/Graphene. Нужно установить Fedora пакеты из README (включая `dbus-devel`, `gtk4-devel`, `libadwaita-devel`) и повторить последовательно.
- `cargo test --workspace` — не завершился по той же причине: workspace включает GTK app.
- `gh auth status` — авторизация GitHub CLI есть (`Chekpoinss`, scopes repo/workflow). `gh repo view Chekpoinss/Telerust` сообщает, что репозиторий не найден.
- В рабочем каталоге нет `.git`; исходной Git history для проверки секретов не существует.
- Собственные API credentials/test account не предоставлены; живой вход по-прежнему не проверен.

## Не проверено / незавершено

- Нет собственных `api_id/api_hash` и отдельного test-DC аккаунта в этом окружении. Поэтому живой login, настоящие chats, history и send/receive end-to-end остаются **не проверены**.
- TDLib 1.8.67 использовался только для ABI smoke probe из внешнего prebuilt package; для разработки/поставки следует собирать официальный TDLib source, версия в конфигурации пока не enforced.
- История загружает одну страницу из 50 сообщений; pagination и list virtualization не готовы.
- Нет поиска, media, notifications, Secret Chats, спонсируемых сообщений, bot ads и полноценного reconnection.
- Ошибки авторизации отображаются в общем виде; восстановление после flood/network failures требует работы.
- QR login (`requestQrCodeAuthentication`), phone/QR переключение, выбор страны и повторная отправка кода не реализованы.
- Logout и контролируемый reset локальной сессии не реализованы.
- В GUI отсутствует поиск чатов; ListBox строки пересоздаются целиком, виртуализации нет.
- Полный AyuGram feature/source map, SBOM, license inventory и окончательное решение по лицензии проекта не завершены.
- В Cargo manifests указана `GPL-3.0-or-later`, но LICENSE/NOTICE отсутствуют и совместимость прямых/транзитивных лицензий не проверена. Это блокирует публикацию по правилу AGENTS.md до лицензионного решения.
- Публичный GitHub репозиторий не создан; публикация исходников не выполнялась.
- Целевой user среды Fedora, но приложение здесь собрано с локально распакованными SDK headers в `/tmp`; обычному build хосту нужны Fedora devel packages из README.

## Следующие действия

1. Пользователь создаёт локальный `config.toml`, где `use_test_dc=true`, настраивает собственную тестовую Telegram app и тестовый аккаунт.
2. Проверить весь login flow, list/history, сообщения между двумя тестовыми аккаунтами, отсутствие секретов в logs и graceful close.
3. Реализовать pagination/virtualized rows; затем sponsored channel messages и только после этого открыть каналы.
4. Повторить RAM/CPU/latency measurements на авторизованном test DC; текущий замер только UI shell.
5. Завершить license/SBOM/source research перед распространением.
6. Установить GTK/DBus development packages и повторить полный набор CI команд; исправить все реальные ошибки, затем проверить GitHub Actions после создания репозитория.

## Исходные ограничения

- Ghost Mode и модификация read/online/typing state запрещены Telegram API Terms §1.4.
- Не использовать реальные пользовательские сессии в тестах; сообщения идут только по явному действию пользователя.
- Не утверждать, что функции AyuGram поддерживаются, пока отдельно не подтверждены implementation и Terms.
- Репозиторий изначально был пустой и не имел Git metadata.
- Не публиковать код до завершения лицензионного инвентаря и добавления согласованных LICENSE/NOTICE файлов.
