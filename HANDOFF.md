# HANDOFF

Дата: 2026-11-10

## Реализовано в текущей версии

- GTK4/libadwaita окно, QR/phone auth prompts, включая code, 2FA и email; обновление QR по TDLib auth updates.
- Локальная форма первичной настройки собственных Telegram `api_id`/`api_hash` с my.telegram.org/apps. Файл config имеет права `0600`, каталог `0700`; значения не выводятся в логи.
- Production DC по умолчанию; явный test DC использует раздельные TDLib каталоги и Secret Service database keys.
- TDLib JSON ABI загружается динамически; поиск `libtdjson.so` по стандартным именам/путям. QR/phone переключение, logout, chat list, последние 50 сообщений и явная отправка текста реализованы на уровне кода.
- User-local release installer, launcher, GNOME `.desktop` entry и SVG icon; installer/uninstaller сохраняют config и сессию при удалении.
- Проект GPL-3.0-or-later, dependency/native notices — `THIRD_PARTY_NOTICES.md`.

## Проверено

- До текущих изменений Git `main` был чистым и совпадал с `origin/main` на `c310fd9604f7c29cc8eec5c16cd198163d77192d`.
- GitHub Actions CI run `42712846293` для того commit завершился success.
- В текущей итерации прошли `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`, `cargo check --workspace`, `cargo test --workspace` (8 unit tests, 0 failed), `cargo build --release -p telerust`.
- TDLib smoke probe прошёл: `/tmp/telerust-tdlib/libtdjson.so` загрузилась и ответила на начальное состояние без авторизации/сети.
- Fedora 45 Wayland: release executable и затем установленный launcher открыты 8 секунд с изолированными пустыми XDG config/data путями; оба завершены `timeout` (ожидаемый код 124), окна не закрылись сами и в stderr не было ошибок. Это подтверждает запуск процесса/GUI, не визуальную отрисовку/сканирование QR.
- User-local install выполнен. `desktop-file-validate ~/.local/share/applications/org.telerust.Client.desktop` прошёл; desktop entry, launcher, binary и SVG icon установлены.
- Один UI-only idle прогон (`scripts/measure-gui-idle.sh 10`, изолированные пустые XDG dirs): RSS/HWM 174328 KiB, PSS 86828 KiB, USS 47816 KiB, CPU 0.100%. Подробности и ограничения в `BENCHMARKS.md`.
- `git diff --check` прошёл; быстрый поиск credentials/телефонов/QR tokens в изменённых файлах нашёл только synthetic test fixtures.

## Не проверено

- Нет предоставленных собственных API credentials или отдельного test-DC аккаунта. Вход, реальный QR, телефонный код, 2FA/email, списки реальных чатов, история, обмен сообщениями, восстановление и logout не прошли E2E. Не имитировать успешный вход.
- Настройка через GTK форму и права файла на чистом профиле в отдельном GUI-сеансе не проходили.
- GNOME shell обновление меню/иконки и визуальная отрисовка auth screen не проверены скриншотом; `.desktop` файл валиден, установленный launcher запускается.
- Актуальные RSS/PSS/USS/CPU/startup benchmarks отсутствуют. Сравнения с AyuGram нет.
- Выбор страны, повторная отправка кода, расширенные retry/error transitions, виртуализация истории и каналы/sponsored messages не реализованы.

## Действия пользователя для реального входа

1. Получить собственные `api_id` и `api_hash` на https://my.telegram.org/apps.
2. Установить TDLib `libtdjson.so`, если она ещё не установлена.
3. Установить приложение командой `./scripts/install-user.sh`, запустить Telerust из меню приложений и заполнить форму credentials. Для production оставить Test DC выключенным.
4. Перезапустить приложение. Для E2E теста отправки сообщений необходим отдельный тестовый аккаунт и явно разрешённый тестовый чат; не используйте личную сессию для проверки отправки.

Никогда не отправлять API credentials, телефон, сессионные файлы или сообщения в чат/репозиторий.
