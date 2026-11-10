# План разработки

## Этап 1 — исследование и ADR (достаточно для вертикального прототипа)

- [x] Инвентаризация репозитория: он был пустым, без Git metadata.
- [x] Первичный обзор AyuGram Desktop, Telegram Desktop, TDLib и Rust клиентов.
- [x] Выявить обязательное ограничение Telegram API Terms на Ghost Mode/read/online/typing вмешательство.
- [x] Решить GTK4/libadwaita для Fedora GNOME Wayland и TDLib JSON C ABI для первого среза.
- [ ] Завершить подробную инвентаризацию AyuGram функций перед реализацией этих функций.
- [x] Провести лицензионную инвентаризацию прямых и транзитивных зависимостей; решение и атрибуции записаны в `THIRD_PARTY_NOTICES.md`.

## Этап 2 — проектирование

- [x] ADR: GTK4/libadwaita выбран для GNOME-first клиента; Slint отложен.
- [x] ADR: TDLib JSON C ABI выбран; библиотека загружается динамически.
- [x] Протокол event/backpressure, хранение сессии, cache budgets, virtualized history задокументированы.
- [x] Threat model и тест-план FFI / storage / parser boundaries задокументированы.

## Этап 3 — прототип вертикального среза

- [x] GTK4/libadwaita окно собрано и запущено на Fedora 45 GNOME Wayland без пользовательской сессии.
- [x] TDLib shared library загружается через C JSON ABI; smoke probe без авторизации и сети.
- [x] Auth prompts для телефона, кода, 2FA, email address/code подключены к TDLib; live account flow ещё не проверен.
- [x] QR auth через TDLib `requestQrCodeAuthentication`, настоящий link из `authorizationStateWaitOtherDeviceConfirmation` и обновление QR при auth-state update.
- [x] Переключение QR/phone методов и переход к существующим code/2FA prompts.
- [x] Logout через TDLib с перезапуском auth worker и удалением database key.
- [ ] Resend code, выбор страны и детальная локализация auth ошибок.
- [x] Chat list, одна страница истории и text send подключены к TDLib; live account flow ещё не проверен.
- [ ] Проверить test-DC login, список/историю/обмен сообщениями с отдельным тестовым аккаунтом.
- [ ] Измерить idle CPU/RSS после логина и нескольких чатов; текущие измерения пока только GUI shell.
- [ ] Проверить клавиатуру, масштабирование, Wayland, accessibility и несколько размеров окна.

## Обновление пользовательского запуска (2026-11-10)

- [x] Production DC по умолчанию; Test DC включается явно и использует отдельную базу и Secret Service key.
- [x] Первичная GTK форма для собственных Telegram API credentials, локальная запись с правами `0600` и ссылка на my.telegram.org/apps.
- [x] User-local release installer/uninstaller, GNOME desktop entry и SVG иконка.
- [ ] Ввести выбор страны/кода, resend кода и более полную обработку повторных попыток.
- [ ] Проверить живой auth/chat/send/logout E2E на отдельном тестовом аккаунте и разрешённом чате.
- [ ] Получить свежие ресурсные измерения по сценариям из `BENCHMARKS.md`.
- [ ] Проверить установленный пакет из GNOME меню и TDLib runtime на чистой Fedora 45.

## TDLib backend/connectivity repair (2026-11-10)

- [x] Сохранять backend diagnostic в UI после закрытия event channel.
- [x] TDLib discovery в постоянном user-local runtime каталоге и ABI probe при установке.
- [x] Подключать TDLib через manual GNOME proxy configuration / environment fallback без записи proxy credentials в Telerust config/logs.
- [x] Получить настоящий production QR, показать QR auth state, затем достичь `Connected`/`Chats`.
- [ ] Проверить содержимое chats/history, receive/send, восстановление и logout без чтения/логирования приватных сообщений.

## UI iteration (2026-11-10)

- [x] Добавить фильтр списка чатов по загруженным названиям и preview.
- [x] Добавить шапку открытого чата с названием и initials-placeholder аватаром.
- [x] Убрать deprecated GTK CSS API и предупреждение о неиспользуемом поле.
- [x] Разнести composition/lifecycle, chat list, header, composer, theme и message rows по UI/application/service модулям; критерий: `main.rs` только подключает модули и вызывает run.
- [x] Добавить bounded message model, bidirectional TDLib page requests с `@extra` correlation, stale-response filtering, dedupe/sort, update/edit/delete, outgoing delivery state и request error events.
- [x] Заменить message `GtkListBox` на `GtkListView` с `SignalListItemFactory`, date dividers, time/status, grouping, copyable wrapped text и scroll anchoring.
- [x] Добавить unit tests state/pagination/dedup/order/chat switch/window cap и synthetic UI benchmark for 100/1k/10k requests.
- [ ] Проверить визуально на GNOME Wayland: светлая/тёмная тема, узкая ширина, клавиатурный фокус и HiDPI.
- [ ] Проверить live history paging, scroll anchoring, send/edit/delete и network retry через отдельный тестовый аккаунт.
- [ ] Перенести «к окну» предел 1 000 сообщений в настройку только после измерений; текущая граница фиксирована, при перемещении окна history cursor позволяет загрузить отброшенную сторону.
- [ ] Исправить известное ограничение очереди backend: проверить bounded-channel backpressure при интенсивном потоке обновлений и обеспечить отзывчивую остановку.

## Первая рабочая версия (MVP)

Цель: development build для Fedora 45 GNOME Wayland. Клиент загружает 50-сообщенные страницы с навигацией в обе стороны и удерживает окно максимум 1 000 сообщений в модели; live history E2E и визуальный screenshot review ещё не выполнены. До MVP нужны test-DC E2E, полноценные сетевые retry/error states, каналы и sponsored messages и повторные performance profiles.
