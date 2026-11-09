# План разработки

## Этап 1 — исследование и ADR (достаточно для вертикального прототипа)

- [x] Инвентаризация репозитория: он был пустым, без Git metadata.
- [x] Первичный обзор AyuGram Desktop, Telegram Desktop, TDLib и Rust клиентов.
- [x] Выявить обязательное ограничение Telegram API Terms на Ghost Mode/read/online/typing вмешательство.
- [x] Решить GTK4/libadwaita для Fedora GNOME Wayland и TDLib JSON C ABI для первого среза.
- [ ] Завершить подробную инвентаризацию AyuGram функций перед реализацией этих функций.
- [ ] Провести лицензионную инвентаризацию прямых и транзитивных зависимостей.

## Этап 2 — проектирование

- [x] ADR: GTK4/libadwaita выбран для GNOME-first клиента; Slint отложен.
- [x] ADR: TDLib JSON C ABI выбран; библиотека загружается динамически.
- [x] Протокол event/backpressure, хранение сессии, cache budgets, virtualized history задокументированы.
- [x] Threat model и тест-план FFI / storage / parser boundaries задокументированы.

## Этап 3 — прототип вертикального среза

- [x] GTK4/libadwaita окно собрано и запущено на Fedora 45 GNOME Wayland без пользовательской сессии.
- [x] TDLib shared library загружается через C JSON ABI; smoke probe без авторизации и сети.
- [x] Auth prompts для телефона, кода, 2FA, email address/code подключены к TDLib; live account flow ещё не проверен.
- [ ] QR auth через `requestQrCodeAuthentication` и отрисовка токена не реализованы.
- [ ] Переключение auth-методов, resend code и обработка детальных auth ошибок не реализованы.
- [ ] Logout/revoke/reset session flow не реализованы.
- [x] Chat list, одна страница истории и text send подключены к TDLib; live account flow ещё не проверен.
- [ ] Проверить test-DC login, список/историю/обмен сообщениями с отдельным тестовым аккаунтом.
- [ ] Измерить idle CPU/RSS после логина и нескольких чатов; текущие измерения пока только GUI shell.
- [ ] Проверить клавиатуру, масштабирование, Wayland, accessibility и несколько размеров окна.

## Первая рабочая версия (MVP)

Цель: development build для Fedora 45 GNOME Wayland. Прототип содержит окно, телефонные auth prompts TDLib, список приватных/групповых чатов, 50 последних сообщений и явную отправку текста. Он не предоставляет QR login и не проверен end-to-end. До MVP нужны QR/phone сценарии целиком, test-DC E2E, logout, виртуализованная история, полноценная обработка сетевых/auth ошибок, каналы и sponsored messages, полный security/licensing review и нагрузочные замеры.
