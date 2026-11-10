# Методика измерения производительности

Результатов измерений пока нет. Не трактовать архитектурные ожидания как результаты.

## Целевые метрики

- RSS/PSS и private dirty memory после старта, авторизации, открытия чата и длительного простоя.
- CPU utilization и wakeups в простое (окно открыто, сеть подключена, без ввода; отдельно offline).
- Время от получения backend update до отрисовки изменения; p50/p95/p99.
- Время открытия чата и прокрутки истории; dropped frames/jank при длинных сообщениях и медиа.
- Пиковая RAM/диск при скачивании файла и работу ограничения кэша.
- Холодный старт и время первой отрисовки.

## Воспроизводимость

Фиксировать модель CPU/RAM/GPU, Fedora/GNOME/Mutter/Wayland, kernel, Rust, сборку/commit, TDLib commit, графическую тему, разрешение/scale, power profile, сеть и состояние кэша. Запускать минимум 5 повторов после прогрева, публиковать сырые данные и медиану/разброс. Сравнивать варианты на одной машине и одинаковых тестовых данных. Для внешних сравнений — одинаковый account/test fixture, размер чатов/медиа и функциональный сценарий; настоящие пользовательские сессии запрещены.

## Сценарии

1. Холодный запуск и idle 10 минут.
2. Тёплый запуск и idle 10 минут после синхронизации.
3. Список из фиксированного количества чатов и открытие 1000-сообщенной fixture истории.
4. Быстрый scroll вверх/вниз; измерять живые row/widget count и frame times.
5. Изображение thumbnail и большой файл через synthetic/local fixture, отдельно сетевое время.
6. Поток синтетических updates при закрытом/открытом чате для проверки backpressure.

Использовать системные инструменты (`/usr/bin/time -v`, `ps`, `pidstat`, `perf`, GNOME Sysprof) и собственные timestamps/trace spans; инструмент и версия должны быть указаны. `scripts/measure-gui-idle.sh` фиксирует RSS/HWM, PSS из `smaps_rollup`, USS как сумму private clean/dirty/hugetlb страниц и средний CPU процесса за интервал. Измерения не должны содержать содержимое сообщений, credentials или идентификаторы реальных пользователей.

## Текущий замер прототипа

Один базовый замер UI без настроенного аккаунта (это **не** замер полного Telegram клиента):

| Дата / хост | Сценарий | RSS в конце | VmHWM | CPU процесса |
|---|---|---:|---:|---:|
| 2026-11-10, Fedora 45 x86_64, GNOME Wayland, GTK 4.24.1, libadwaita 1.10, debug build | Окно открыто, config отсутствует, TDLib worker не стартовал, 10 секунд после прогрева | 173608 KiB | 173608 KiB | 1.500% среднего за 10 s |

Результат снят локальным `scripts/measure-gui-idle.sh 10`, который читает `/proc/<pid>/status` и process CPU ticks. Это один диагностический замер, не стабильный baseline; он не включает TDLib, сеть, авторизованную базу, чаты, историю и медиа. Повторить после `cargo build -p telerust`, закрыв прочие окна приложения. Для сравнения/релизного вывода нужны минимум 5 прогонов на указанной в начале методики полной конфигурации.

После установки Fedora 45 devel RPM сборка с QR/logout проходит, а release GUI стартует в Wayland при изолированных пустых XDG каталогах. Старый единичный UI-only замер не повторялся; RAM/CPU измерение нового бинарника не проводилось. Значения выше относятся только к прежней сборке.

## 2026-11-10: bounded virtualized synthetic history

One release-build diagnostic run on Fedora 45 x86_64 / GNOME Wayland, GTK 4.24.1, libadwaita 1.10.0, Rust 1.99.1, kernel 7.2.15. Command: `scripts/benchmark-history-ui.sh`. Each window uses deterministic synthetic Russian/Unicode text and never starts TDLib or reads account data. RSS/PSS/USS were sampled about 2 seconds after startup while the synthetic scroll stress could be active. `model_build_ms` is the synchronous time to produce and place the bounded model, not first-frame latency. CPU is sampled for 3 seconds after the scroll stress completes, normalized to one CPU core. The scroll test alternates the adjustment between the top and bottom 40 times; it is a synthetic endpoint-churn stress, not an FPS/real-user scroll trace.

| Input count | Retained messages / model rows | RSS KiB | PSS KiB | USS KiB | Model build ms | Idle CPU % (3 s) | 40 endpoint moves ms | Peak realized rows | Factory rows created |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 100 | 100 / 101 | 193420 | 67517 | 52836 | 7 | 3.67 | 1029 | 101 | 101 |
| 1,000 | 1,000 / 1,001 | 198600 | 72284 | 57608 | 51 | 8.33 | 3066 | 206 | 224 |
| 10,000 | 1,000 / 1,001 | 199792 | 73235 | 58556 | 48 | 1.33 | 8194 | 206 | 224 |

These are single runs and are not a baseline. The model cap is 1,000 messages; the remaining input in the 10,000 case is discarded from the UI window and can be requested later through the cursor. GTK factory counters show the list view did not keep one row widget per retained message at 1,000 messages (206 peak bound vs 1,001 model entries). The 100-message case fit within the factory's realized pool, so all 101 model entries were realized. RSS/PSS/USS remain around the GTK process baseline; no comparison to Telegram Desktop is implied.

The benchmark was run in a real Wayland session. Screenshot capture was blocked by the compositor (`grim`: unsupported capture protocol; GNOME Shell DBus: `Screenshot is not allowed`), so pixel-level visual review, theme/HiDPI/keyboard review and a frame-rate measurement were not completed. Live TDLib paging and network transitions were also not exercised; cursor behavior and request construction are unit-tested only.

## 2026-11-10: текущая итерация

Один UI-only прогон текущей debug-сборки на Fedora 45 x86_64 GNOME Wayland, GTK 4.24.1/libadwaita 1.10, без настроек и TDLib worker. Команда: `env XDG_CONFIG_HOME=/tmp/telerust-benchmark-config XDG_DATA_HOME=/tmp/telerust-benchmark-data ./scripts/measure-gui-idle.sh 10`.

| Дата | Сценарий | RSS / HWM | PSS | USS | Средний CPU |
|---|---|---:|---:|---:|---:|
| 2026-11-10 | Окно auth UI, без config/backend, 10 s после 2 s запуска | 174328 KiB | 86828 KiB | 47816 KiB | 0.100% |

Это один диагностический прогон, не baseline с разбросом. Измерены только окно авторизации и отсутствие backend; нет времён до первой отрисовки, открытого чата, TDLib sync, сети и сессии. На Fedora 45 Wayland release binary и установленный launcher также удерживали окно 8 секунд при timeout smoke check; `desktop-file-validate` прошёл. Скриншотный осмотр GNOME меню/окна не выполнялся.
