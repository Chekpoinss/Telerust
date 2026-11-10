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
- UI разделён на `application.rs`, `ui/` компоненты и `services/telegram.rs`; `main.rs` теперь только wiring модулей и запуск.
- История работает страницами TDLib по 50 сообщений в обоих направлениях. Bridge передаёт request ID через TDLib `@extra`; старые ответы после смены чата отбрасываются.
- `HistoryWindow` сортирует/дедуплицирует сообщения и удерживает максимум 1 000 в модели. При движении по более длинной переписке окно перемещается в обе стороны.
- `GtkListView` использует переиспользуемую `SignalListItemFactory`; rows показывают локальные дату/время, группировку отправителя, delivery state и selectable Unicode text. При capped prepend позиция корректируется оценкой высоты вставленных rows; точный pixel anchor без визуального review не подтверждён.
- События TDLib для новых, отредактированных, удалённых, успешно отправленных и неотправленных сообщений обновляют текущую модель. Ошибка страницы освобождает single-flight gate и показывается в status line.
- В `scripts/benchmark-history-ui.sh` и режиме `TELERUST_SYNTHETIC_HISTORY=100|1000|10000` есть benchmark без настроек/сессии TDLib.

## Проверено

- `$XDG_CONFIG_HOME/telerust/config.toml` parse/permissions прошли проверку без вывода содержимого: каталог `0700`, файл `0600`, production (`use_test_dc=false`), credentials структурно валидны.
- Старая первопричина: config не задавал `tdlib_library`, поиск не включал user-local каталог, а GTK затирал backend error после закрытия event channel.
- TDLib из ранее собранного `/tmp/telerust-tdlib/libtdjson.so`: версия `1.8.67`, ELF x86_64; `ldd -r` без неразрешённых зависимостей/символов; ABI probe прошёл. Та же библиотека скопирована и повторно проверена в `$XDG_DATA_HOME/telerust/lib/libtdjson.so`; `/tmp` больше не нужен для запуска.
- Прямое подключение к Telegram endpoint в этой сети не проходило. После использования настроенного GNOME proxy TDLib установила соединение с production DC и получила настоящий `authorizationStateWaitOtherDeviceConfirmation.link`.
- Обновлённая release app запущена через установленный launcher. Доступное GTK UI показало `Scan to sign in` и `Scan this QR code…`; link не выводился и не сохранялся отдельно. Позже окно перешло в `Connected` и `Chats`, что подтверждает authorization-ready state и открытие списка. Названия чатов/сообщения не читались, сообщения не отправлялись.
- `desktop-file-validate` прошёл; desktop entry, launcher, binary и runtime library находятся в user-local каталоге.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`, `cargo check --workspace`, `cargo test --workspace` (13 tests, 0 failed), `cargo build --release -p telerust` прошли.
- Для текущего UI состояния выполнены `cargo fmt --all`, `cargo clippy --workspace --all-targets`, `cargo check --workspace`, `cargo test --workspace` (13 tests, 0 failed), `cargo build --release -p telerust`; все завершились успешно без предупреждений.
- Для истории проведены workspace проверки: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`, `cargo check --workspace`, `cargo test --workspace` (21 tests, 0 failed), `cargo build --release -p telerust`.
- Fedora 45 Wayland synthetic UI smoke прошёл на 100/1 000/10 000 входных сообщений; у запроса 10 000 модель оставила 1 000 сообщений/1 001 timeline item. На модели 1 000 измерено максимум 206 одновременно привязанных ListView строк; конкретные RSS/PSS/USS/scroll цифры см. `BENCHMARKS.md`.

## Не проверено

- 2FA/email на аккаунте не встречались. История, получение/отправка сообщений, восстановление сессии после перезапуска и logout не прошли E2E.
- GTK credential form на чистом профиле отдельно не тестировалась; в текущей среде уже был готовый закрытый config.
- QR-код доступен в UI, но скриншот не снимался. Доказательство отрисовки: TDLib выдала ссылку, UI перешёл на QR состояние, соответствующие label доступны через AT-SPI.
- Клавиатурная навигация/доступность, live reconnect и performance после входа не измерялись.
- Новые header/search стили не прошли отдельную визуальную проверку на Wayland/HiDPI; фильтр работает только по загруженным чатам, серверного поиска нет.
- Персональный GNOME Shell запретил screen capture (`Screenshot is not allowed`), поэтому скриншот и визуальный pixel review недоступны в этой сессии. При этом синтетические окна запускались реально в Fedora Wayland; тёмный UI theme проверен только по живому запуску и factory counts.
- Пагинация state, TDLib request JSON и stale-chat protection покрыты unit tests, но реальная последовательная навигация в авторизованной переписке и сетевые ошибки не E2E-проверены.
- UI window limit равен 1 000. Пользователь может навигировать старше/новее через курсоры, но при переключении окна текущий участок заменяется; глубокие запросы требуют доступности сети/TDLib.

## Следующее действие пользователя

Окно Telerust сейчас достигло `Connected`/`Chats`. Проверьте список, затем самостоятельно откройте нужную переписку. Мы не открывали чаты и не отправляли сообщения. Для проверки после перезапуска можно закрыть и запустить Telerust повторно; logout в этой сессии не выполнялся.

Не присылайте API credentials, телефон, коды или содержимое переписки в чат.

## Следующий приоритет разработки

1. Проверить history paging и изменения сообщений на отдельном тестовом аккаунте без личных сообщений в логах.
2. Визуально проверить HiDPI, клавиатуру и скролл после того, как GNOME capture policy позволит получить локальные скриншоты.
3. Синтетически нагрузить bounded event queue и остановку backend worker.
