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

## Media & chat UI overhaul (2026-11-10)

### Завершено в этой итерации

- [x] Типизированно различать text/photo/document/video/sticker/voice/audio/service/unsupported TDLib messages; пустой/неизвестный content больше не становится пустым text bubble.
- [x] Показывать карточки файлов с именем, MIME/типом, размером, состоянием и кнопкой скачивания; TDLib `downloadFile` асинхронный, ошибки очищают retry gate, локальное открытие только по явному клику.
- [x] Добавить ограниченную по видимым GTK row фото-загрузку чатов через `GtkListView`, обновление chat photo path и avatar fallback в header/list.
- [x] Устранить stale status «waiting for authorization» при `connectionStateReady`.
- [x] Сохранить существующие `GtkListView`, bounded history и TDLib page request correlation; добавить unit tests для message parsing/file state/chat photos/download request.
- [x] Повторить синтетические 100/1k/10k и 1k media-card прогоны 5 раз и записать медианы/ограничения.

### Следующие задачи с критериями готовности

1. **Проверить живые аватары и скачивания на отдельном test account.** Критерий: photo update → `downloadFile` → `updateFile` отражается в chat list/header; document/photo сохраняют прогресс, ошибку и retry; локальная кнопка открывает только после клика. Не логировать реальные имена/сообщения.
2. **Визуальный review GTK.** Критерий: synthetic fixture со short/long text, documents/photos, RU/emoji проходит normal/narrow/HiDPI UI, скриншоты разрешённым способом либо GTK widget geometry tree сохранён как свидетельство.
3. **Полноразмерные фото и media cache.** Критерий: separate full-size photo file is fetched only on explicit open, bounded thumbnail/photo cache has measurable eviction behavior; memory profile after repeated chat switching.
4. **Видео/audio/sticker UX.** Критерий: verified metadata and safe download/open action per TDLib type; playback buttons appear only when a player path is implemented.
5. **Investigate remaining history UI jank.** Критерий: record GTK frame timings and model/update traces; benchmark at least 5 repeats after fixed warmup, isolate any persistent CPU load from delayed relayout.
6. **Live history lifecycle.** Критерий: separate test account confirms older/newer paging, anchor preservation, chat switch, edit/delete and offline/reconnect without duplicates.

### Технические ограничения

- Channels are still hidden because sponsored-message handling is not implemented.
- Live TDLib media/avatar behavior, photo viewer/full-size fetch, playback and frame-rate/UI pixel review are not verified.
- Synthetic media benchmark uses metadata-only cards and does not model network, decoding, disk cache or real chat switching. Results and settling CPU intervals are in `BENCHMARKS.md`.
