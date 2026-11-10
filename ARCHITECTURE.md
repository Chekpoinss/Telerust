# Архитектура и результаты исследования

Дата исследования: 2026-11-09. Репозиторий на момент начала был пустым; измерений клиента нет. Все гипотезы о производительности остаются гипотезами до воспроизводимого профилирования.

## Область и источники

- [AyuGram Desktop](https://github.com/TelegramDevs/tdesktop-ayugram) — форк Telegram Desktop. Публичный README подтверждает Ghost Mode, Messages History, Anti-Recall, Streamer Mode, локальный Premium и внешний вид; полный список — [документация AyuGram](https://docs.ayugram.one/desktop/).
- [Telegram Desktop](https://github.com/TelegramOrg/Telegram-desktop) — крупный C++ клиент на Telegram API/MTProto; исходник GPLv3 с исключением OpenSSL. Это важный источник поведения/совместимости, но не доказательство преимуществ по производительности.
- [TDLib](https://github.com/tdlib/td) — клиентская библиотека Telegram, BSL-1.0. Предоставляет асинхронный API и C JSON интерфейс; поддерживает авторизацию, обновления, чаты, историю, файлы и Secret Chats при включённом `use_secret_chats`. Использование TDLib переносит протокольную и криптографическую сложность в проверенный upstream компонент, но не избавляет от требований к версии, сборке и аудиту ABI.
- [Paper Plane](https://github.com/paper-plane-developers/paper-plane) — Rust + GTK4/libadwaita + TDLib, полезный референс архитектуры GNOME. Сам проект предупреждает, что он не feature-complete и не считается стабильным; это не оценка наших метрик.
- `tg` — Rust CLI поверх TDLib; `telegram-rs` — Rust GUI/MTProto-клиент на grammers. Они показывают два направления (TDLib adapter и native Rust protocol), но не дают сопоставимых бенчмарков для Fedora/GNOME.

## Границы возможностей

TDLib может обслуживать базовые Telegram-функции через методы и updates: авторизацию, чаты/папки, историю, сообщения, поиск, загрузки/отправки файлов, реакции и состояния, документированные в конкретной версии TDLib. Каждая функция должна сверяться с текущей схемой `td_api.tl` и реализовываться через updates, а не предположения о внутренних состояниях.

| Возможность AyuGram | Предварительная оценка | Ограничение / следующий шаг |
|---|---|---|
| Ghost Mode: подавление read/online/typing | Запрещено текущими Telegram API Terms §1.4 | Не реализовывать; политика прямо называет ghost mode и вмешательство в read/online/typing запрещёнными. |
| Messages History / Anti-Recall | Клиентское хранение требует отдельной реализации | Изучить точное поведение и ToS/privacy/security constraints. Не перехватывать/сохранять удаляемый или самоуничтожающийся контент. |
| Appearance, шрифты, ширина сообщений, меню | В основном UI/local state | Реализуемо в presentation layer; проверять требования доступности и ресурсы. |
| Streamer Mode / сокрытие от screen capture | Не подтверждено для GNOME Wayland | Проверить публичные протоколы/порталы и поведение compositor; не обещать поддержку. |
| Local Premium | Частично локальная визуализация, часть связана с серверными ограничениями | Нельзя обходить лимиты или выдавать серверные права за локальные. Проверить пофункционально. |
| Sticker/GIF/video controls, jump-to-beginning, reactions | Комбинация TDLib объектов и UI | Сверить доступность методов/обновлений и UX. |
| Обход ограничений копирования/скачивания | Не включать без отдельной юридической/политической проверки | Требует проверки protected content и Telegram Terms; безопасное значение — уважать ограничения сервера. |

Это исследовательская матрица, не список уже работающих возможностей. Telegram API Terms также требуют собственного `api_id`, ясного обозначения клиента как части Telegram экосистемы, не использовать официальный логотип, не включать Telegram в название без `Unofficial`, не скрывать sponsored messages в каналах. Требования проверять до распространения.

## ADR для первого вертикального прототипа (2026-11-10)

1. **Современный TDLib JSON C ABI** выбран для Rust adapter: `td_create_client_id`, `td_send`, `td_receive`, `td_execute`. Старый `td_json_client_*` interface официальная документация помечает legacy и планируемым к удалению в TDLib 2.0; он не используется. Shared library загружается динамически. `td_receive` вызывается ровно одним выделенным worker thread, и каждый буфер копируется до следующего вызова. Сборка TDLib остаётся отдельной системной зависимостью, путь можно задать в `config.toml`. Это упрощает ABI, но не является утверждением о скорости или меньшем RAM.
2. **GTK4 + libadwaita** выбран для Fedora GNOME Wayland: это платформа-цель, для которой нужны нативные GNOME widgets и интеграция. Rust bindings используются без ручного GTK FFI. Slint не выбран для первого вертикального среза: его отдельные условия лицензии и standalone styling менее полезны для GNOME-first UI. Это платформенное решение; сравнительных результатов производительности нет.
3. **Асинхронность:** TDLib worker thread обслуживает запросы/updates и передаёт ограниченную очередь событий; GTK main loop ждёт `async-channel` без периодического polling. Сетевые вызовы не выполняются на UI thread.
4. **Сессия:** `api_id/api_hash` можно ввести в UI первого запуска; они сохраняются в `$XDG_CONFIG_HOME/telerust/config.toml` (fallback `~/.config/telerust/config.toml`) с правами `0600`, каталог — `0700`. Database encryption key генерируется через OS CSPRNG и сохраняется в GNOME Secret Service через keyring; недоступность/блокировка keyring останавливает backend. TDLib база находится в каталоге `0700`; test и production data dirs и keyring entries разделены.
5. **Режимы:** production DC — значение по умолчанию. Test DC включается только явно флажком первоначальной настройки или `use_test_dc = true`; требует отдельный test-DC аккаунт. В UI нет автоматической отправки.
6. **Каналы:** пока скрыты из списка и заблокированы backend-командами, пока не появится обязательная поддержка sponsored messages. Документация Telegram требует показывать официальные рекламные сообщения при доступе к содержимому каналов. Приватные чаты и группы доступны.
7. **История и UI:** presentation разбит на `ui/{theme,chat_list,chat_header,message_list,message_row,composer}.rs`; app lifecycle/event routing расположен в `application.rs`, а адаптер запуска backend — в `services/telegram.rs`. UI получает только bridge DTO и не формирует TDLib JSON.
8. **История:** `client-core::history::HistoryWindow` содержит сортируемое окно не более чем из 1 000 сообщений, дедуплицирует по ID и назначает request IDs. Bridge привязывает `@extra` к chat ID, cursor и направлению страницы; устаревшие/неизвестные ответы игнорируются. `getChatHistory` загружает по 50 сообщений по краям. GTK `ListView`/`SignalListItemFactory` создаёт переиспользуемые строки; модель сообщений остаётся ограниченной, старые/новые данные вытесняются с противоположного края и могут быть запрошены снова по cursor.
9. **Scroll anchoring:** при вставке более старой страницы сохраняется прежнее значение вертикальной прокрутки с поправкой на изменение высоты модели. Когда bounded window одновременно вытесняет строки с противоположного края и total height почти не меняется, поправка оценивается по средней высоте строки и позиции старого первого сообщения. Новые сообщения автопрокручивают список только когда пользователь был у нижнего края; изменения сохраняют текущую позицию.
10. **Безопасность:** узкий `unsafe` C ABI слой в `tdlib-bridge`; API ошибки не выводят исходные JSON/тексты. Тесты используют фикстуры; вход в Telegram не автоматизирован.

Виртуализация проверяется через factory counters (созданные и одновременно привязанные ListItem rows), а не по одному факту применения `GtkListView`. Один synthetic Wayland прогон на 1 000 retained messages дал пик 206 привязанных строк; это измерение GTK build в данном окружении, не обещание постоянного лимита для всех размеров/тем. Методика и сырые показатели описаны в `BENCHMARKS.md`.

## GUI и лицензии

| Вариант | Сильные стороны для этой цели | Риски/неизвестное | Статус |
|---|---|---|---|
| GTK4 + libadwaita (`gtk-rs`) | GNOME-first, Wayland backend через GDK, зрелые системные widgets; есть Rust GTK клиент Paper Plane | GTK/LGPL и биндинги/зависимости имеют разные лицензии; реальное потребление и виртуализация нужно измерить | Выбран для MVP |
| Slint | Декларативная UI-разметка, Rust API, Linux desktop | GPLv3 или отдельные royalty-free/commercial условия; интеграцию с GNOME пришлось бы отдельно проверять | Не выбран для GNOME-first среза |
| Iced / egui | Rust-first, удобны для быстрых экспериментов и кастомного UI | Не подтверждены для требований GNOME accessibility, системного поведения и целевого списка сообщений | Не выбраны |

Лицензионная инвентаризация пока предварительная и не заменяет юридическую проверку перед публикацией:

| Компонент | Подтверждённая лицензия/условия | Практическое действие |
|---|---|---|
| AyuGram Desktop / Telegram Desktop | GPLv3 с исключением на линковку OpenSSL в лицензии репозитория | При заимствовании кода соблюдать GPLv3 и сохранять notices/исключение там, где применимо; предпочтительно пока брать только поведение/API как референс, не копировать код/assets. |
| TDLib | Boost Software License 1.0 | Сохранить LICENSE/NOTICE и проверить требования для бинарной поставки. |
| GTK4 | LGPL; `gtk-rs` Rust bindings отдельно MIT | Инвентаризировать GTK и все нативные runtime/build dependencies. |
| Slint | GPLv3, royalty-free или коммерческие варианты с отдельными условиями | Уточнить применимый текст лицензии и обязанности attribution/исходников до выбора. |
| Paper Plane | GPL-3.0 | Использовать как архитектурный референс; не переносить код без соблюдения условий GPL и notices. |

Workspace использует GPL-3.0-or-later. После проверки исходников, Cargo metadata и native runtime компонентов подтверждены текст лицензии в `LICENSE` и inventory/атрибуции в `THIRD_PARTY_NOTICES.md`.

### Нагрузка: что известно и что измерять

Исходники раскрывают вероятные горячие зоны, но не позволяют назвать главные потребители CPU/RAM без профиля: TDLib база/сеть/декодирование медиа, кеши изображений и thumbnails, разметка/рендеринг длинных сообщений, число UI-объектов, polling/таймеры и размер истории. Базовый план измерений — в `BENCHMARKS.md`; результатов пока нет.

## Настройка и установка (2026-11-10)

- Первый запуск предлагает локальную форму для собственных `api_id`/`api_hash` с my.telegram.org/apps. Значения скрыты в поле ввода, не логируются и сохраняются однократно в закрытый файл; после сохранения требуется перезапуск.
- API credentials хранятся в файле с режимом `0600`; Secret Service используется для ключа базы TDLib.
- Установка для текущего пользователя размещает release executable, launcher, `.desktop` и SVG icon под `$XDG_DATA_HOME`; uninstall удаляет только файлы приложения, сохраняя настройки и сессию.
- TDLib discovery сначала проверяет config path и `$XDG_DATA_HOME/telerust/lib/libtdjson.so`, затем стандартные system paths. В manual proxy mode TDLib получает proxy из GNOME GSettings; environment proxy используется как fallback. Proxy credentials применяются только в памяти и не записываются в config/logs.
- Backend startup errors сохраняются в UI после закрытия event channel. Известные invalid API ID ошибки преобразуются в redacted diagnostic; неизвестный текст сервера не отображается и не журналируется.
- TDLib discovery сначала проверяет заданный config путь и `$XDG_DATA_HOME/telerust/lib/libtdjson.so`, затем стандартные системные имена. При manual proxy mode TDLib получает proxy из GSettings; если он выключен/не задан — читается proxy URL из стандартных environment variables. Пароль прокси используется только в памяти при вызове TDLib и не записывается в Telerust config/logs.
- Backend startup errors сохраняются в UI после закрытия event channel. Ошибки invalid API ID имеют отдельное безопасное сообщение; неизвестный текст сервера не отображается и не журналируется.

## Текущий разрыв между ADR и прототипом

Реализация покрывает QR/phone auth prompts, чат-лист, одну страницу истории и явную отправку текста. Logout подключён. Полноценные retry/error transitions, выбор страны и chat search отсутствуют. История рендерится через `GtkListBox` и при обновлении строки пересоздаются; виртуализации нет. Поэтому архитектурные пункты выше описывают выбранные границы и будущие требования, а не полностью реализованные свойства приложения. Сетевой E2E без отдельного тестового аккаунта не проверен.

Очередь событий ограничена 256 элементами, но backend публикует через блокирующий `send_blocking`; при замедлении UI это создаёт backpressure, который следует проверить на синтетическом update потоке до утверждения устойчивости под нагрузкой.

## Auth QR и logout (2026-11-10)

- При `authorizationStateWaitPhoneNumber` backend по умолчанию запрашивает QR через TDLib `requestQrCodeAuthentication`. Только `authorizationStateWaitOtherDeviceConfirmation.link` принимается как QR payload; GUI проверяет `tg://login?token=` и кодирует ссылку библиотекой `qrcode`. TDLib отправляет новые auth-state updates при обновлении ссылки; каждый update перерисовывает матрицу. Токен не пишется в лог.
- Переключение из QR в phone login вызывает TDLib `destroy` для ещё не авторизованного инстанса и перезапускает его с телефонным prompt. Из phone/code/password в QR вызывает `requestQrCodeAuthentication` напрямую; TDLib разрешает вызов в документированных auth states.
- Logout использует TDLib `logOut`, который по схеме TDLib завершает инстанс и удаляет локальные данные. После `authorizationStateClosed` bridge удаляет свой encryption key из Secret Service и создаёт новый TDLib instance для следующего входа. Если app завершается в процессе logout, серверное завершение может не успеть.

Источники по интерфейсам и обязательным ограничениям: [TDLib C JSON ABI](https://core.telegram.org/tdlib/docs/td__json__client_8h.html), [TDLib async/update model](https://core.telegram.org/tdlib/getting-started), [Telegram API Terms](https://core.telegram.org/api/terms), [спонсируемые сообщения и тестовый канал](https://core.telegram.org/api/sponsored-messages), [GTK Rust bindings](https://www.gtk.org/docs/language-bindings/rust/), [Slint licensing](https://slint.dev/faqs).
