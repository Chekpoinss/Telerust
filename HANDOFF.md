# HANDOFF

Дата: 2026-11-10

## Реализовано

- GTK4/libadwaita окно и QR/phone auth prompts с code, 2FA и email состояниями.
- Собственные Telegram API credentials хранятся локально с `0600` config и `0700` каталогом; UI и логи не раскрывают значения.
- Production DC по умолчанию, отдельные test/prod TDLib базы и Secret Service keys.
- TDLib JSON ABI динамически загружается; библиотека ищется по config path, `$XDG_DATA_HOME/telerust/lib/libtdjson.so`, затем стандартным system paths.
- GNOME manual proxy settings передаются в TDLib; environment proxy — fallback. Proxy credentials не сохраняются в Telerust config и не пишутся в логи.
- Backend error больше не перетирается сообщением `Backend stopped.`; неверный API ID и основные auth ошибки получают безопасные понятные диагностики.
- User-local release installer/launcher, GNOME desktop entry, icon. Uninstaller сохраняет config и TDLib session databases.

## Проверено

- `$XDG_CONFIG_HOME/telerust/config.toml` parse/permissions прошли проверку без вывода содержимого: каталог `0700`, файл `0600`, production (`use_test_dc=false`), credentials структурно валидны.
- Старая первопричина: config не задавал `tdlib_library`, поиск не включал user-local каталог, а GTK затирал backend error после закрытия event channel.
- TDLib из ранее собранного `/tmp/telerust-tdlib/libtdjson.so`: версия `1.8.67`, ELF x86_64; `ldd -r` без неразрешённых зависимостей/символов; ABI probe прошёл. Та же библиотека скопирована и повторно проверена в `$XDG_DATA_HOME/telerust/lib/libtdjson.so`; `/tmp` больше не нужен для запуска.
- Прямое подключение к Telegram endpoint в этой сети не проходило. После использования настроенного GNOME proxy TDLib установила соединение с production DC и получила настоящий `authorizationStateWaitOtherDeviceConfirmation.link`.
- Обновлённая release app запущена через установленный launcher. Доступное GTK UI показывает `Scan to sign in` и `Scan this QR code…`; link не выводился и не сохранялся отдельно. Окно оставлено открытым для ручного сканирования.
- `desktop-file-validate` прошёл; desktop entry, launcher, binary и runtime library находятся в user-local каталоге.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`, `cargo check --workspace`, `cargo test --workspace` (13 tests, 0 failed), `cargo build --release -p telerust` прошли.

## Не проверено

- Пользователь ещё не сканировал QR. Авторизация, 2FA/email на аккаунте, восстановление сессии, список чатов, история, получение/отправка сообщений и logout не прошли E2E.
- GTK credential form на чистом профиле отдельно не тестировалась; в текущей среде уже был готовый закрытый config.
- QR-код доступен в UI, но скриншот не снимался. Доказательство отрисовки: TDLib выдала ссылку, UI перешёл на QR состояние, соответствующие label доступны через AT-SPI.
- Клавиатурная навигация/доступность, live reconnect и performance после входа не измерялись.

## Следующее действие пользователя

В оставленном открытым окне Telerust отсканируйте QR через Telegram → Settings → Devices → Link Desktop Device. Если Telegram потребует подтверждение или приложение покажет 2FA/email prompt, завершите шаг вручную в Telerust. После входа проверьте список и открытие чатов. Сообщения автоматически не отправляются.

Не присылайте API credentials, телефон, коды или содержимое переписки в чат.
