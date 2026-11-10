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
- UI имеет фильтр по уже загруженным чатам, header выбранного чата, initials-placeholder аватар и стилизованные incoming/outgoing bubbles. История пока остаётся одностраничной.
- При открытии строки header обновляет название и placeholder аватар; CSS provider использует актуальный GTK API.

## Проверено

- `$XDG_CONFIG_HOME/telerust/config.toml` parse/permissions прошли проверку без вывода содержимого: каталог `0700`, файл `0600`, production (`use_test_dc=false`), credentials структурно валидны.
- Старая первопричина: config не задавал `tdlib_library`, поиск не включал user-local каталог, а GTK затирал backend error после закрытия event channel.
- TDLib из ранее собранного `/tmp/telerust-tdlib/libtdjson.so`: версия `1.8.67`, ELF x86_64; `ldd -r` без неразрешённых зависимостей/символов; ABI probe прошёл. Та же библиотека скопирована и повторно проверена в `$XDG_DATA_HOME/telerust/lib/libtdjson.so`; `/tmp` больше не нужен для запуска.
- Прямое подключение к Telegram endpoint в этой сети не проходило. После использования настроенного GNOME proxy TDLib установила соединение с production DC и получила настоящий `authorizationStateWaitOtherDeviceConfirmation.link`.
- Обновлённая release app запущена через установленный launcher. Доступное GTK UI показало `Scan to sign in` и `Scan this QR code…`; link не выводился и не сохранялся отдельно. Позже окно перешло в `Connected` и `Chats`, что подтверждает authorization-ready state и открытие списка. Названия чатов/сообщения не читались, сообщения не отправлялись.
- `desktop-file-validate` прошёл; desktop entry, launcher, binary и runtime library находятся в user-local каталоге.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`, `cargo check --workspace`, `cargo test --workspace` (13 tests, 0 failed), `cargo build --release -p telerust` прошли.
- Для текущего UI состояния выполнены `cargo fmt --all`, `cargo clippy --workspace --all-targets`, `cargo check --workspace`, `cargo test --workspace` (13 tests, 0 failed), `cargo build --release -p telerust`; все завершились успешно без предупреждений.

## Не проверено

- 2FA/email на аккаунте не встречались. История, получение/отправка сообщений, восстановление сессии после перезапуска и logout не прошли E2E.
- GTK credential form на чистом профиле отдельно не тестировалась; в текущей среде уже был готовый закрытый config.
- QR-код доступен в UI, но скриншот не снимался. Доказательство отрисовки: TDLib выдала ссылку, UI перешёл на QR состояние, соответствующие label доступны через AT-SPI.
- Клавиатурная навигация/доступность, live reconnect и performance после входа не измерялись.
- Новые header/search стили не прошли отдельную визуальную проверку на Wayland/HiDPI; фильтр работает только по загруженным чатам, серверного поиска нет.

## Следующее действие пользователя

Окно Telerust сейчас достигло `Connected`/`Chats`. Проверьте список, затем самостоятельно откройте нужную переписку. Мы не открывали чаты и не отправляли сообщения. Для проверки после перезапуска можно закрыть и запустить Telerust повторно; logout в этой сессии не выполнялся.

Не присылайте API credentials, телефон, коды или содержимое переписки в чат.

## Следующий приоритет разработки

1. Разнести GUI из `apps/telerust/src/main.rs` по модулям, не меняя backend-контракты.
2. Сделать безопасную пагинацию истории и проверить её с синтетическими/тестовыми данными; live E2E требует отдельного тестового аккаунта.
3. Провести визуальную проверку UI и оценить очереди событий под нагрузкой.
