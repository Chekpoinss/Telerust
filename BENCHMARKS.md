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

## 2026-11-10: повторные замеры модульного списка и карточек

Пять отдельных запусков на Fedora 45 GNOME Wayland, release build; каждый synthetic application использует отдельный GTK application ID, не читает настройки и не запускает TDLib. Команда: `TELERUST_BENCHMARK_REPEATS=5 scripts/benchmark-history-ui.sh`. RSS/PSS/USS снимались примерно через 2 секунды после запуска. CPU — первые 3 секунды после окончания 40 endpoint moves, поэтому включает возможную отложенную раскладку GTK. В `media_1000` каждый десятый message model был синтетической карточкой фото или документа; медиафайлы не скачивались.

| Сценарий | Медиана RSS KiB | PSS KiB | USS KiB | Model build ms | CPU сразу после scroll, 3 s | Median endpoint moves | Peak bound rows / created |
|---|---:|---:|---:|---:|---:|---:|---:|
| 100 text | 193632 | 66224 | 53356 | 8 | 1.67% | 997 ms | 101 / 101 |
| 1,000 text | 198960 | 70971 | 58116 | 47 | 6.67% | 7729 ms | 206 / 224 |
| 10,000 input, bounded to 1,000 | 200756 | 72678 | 59804 | 54 | 7.33% | 2853 ms | 206 / 224 |
| 1,000, about 200 synthetic media cards | 199468 | 71608 | 58780 | 51 | 0.67% | 2988 ms | 206 / 224 |

The previous isolated 8.33% CPU result is not evidence of persistent idle load. A follow-up 10,000-message run measured 7.33% in the first 3-second interval after scroll, then 0.00%, 0.33%, 0.00%, and 0.33% over four subsequent 3-second intervals. A 1,000-message run measured 7.67%, then 0.00%, 0.00%, 0.00%, and 0.33%. This points to transient GTK relayout immediately after synthetic scroll; process utilization settles below 0.34% in these runs. This is diagnostic, not a long-duration idle baseline.

`GtkListView` remains capped at 1,001 timeline entries and 206 peak realized rows in the 1k/10k runs; it did not create one row widget per retained message. RSS/PSS/USS include GTK/libadwaita startup and loaded libraries. The media case measures metadata-card construction only, not decoding, downloads, image-cache growth or a real chat switch. Scroll elapsed times show high run-to-run variance on the 1k case and are not frame-rate measurements. No Telegram Desktop comparison is implied.

### Chat switching and post-scroll idle follow-up

The synthetic switch case was repeated five times after allowing the GTK window to lay out. Each repeat replaced the bounded 1,000-message history 12 times, with the window geometry confirmed as 320×574 chat list and 695×551 message view before stress. Median results: 8,490 ms for 12 model replacements; RSS/PSS/USS 200,540/72,635/59,828 KiB; 206 peak realized rows and 11,410 row factory creations across the stress. The last number is cumulative row reuse/recreation across switches, not simultaneously resident widgets. This synthetic scenario does not exercise live TDLib/network switching.

The post-scroll CPU samples in the five-run table above were the first 3-second interval and include delayed layout work. Follow-up intervals on prior 1k and 10k runs settled at 0–0.33% for four further 3-second intervals. The five-repeat synthetic chat-switch run measured 0–0.33% in its 3-second post-stress samples (median 0.00%). This supports a transient-relayout explanation for the earlier 8.33% sample in this workload; it is not a 10-minute idle measurement.

Pixel-level visual inspection remains unavailable: GNOME Wayland rejected the screenshot request and `grim` had no supported capture protocol in this environment. No compositor restriction was bypassed. Synthetic geometry and GTK factory counters are available, but do not replace visual review.
