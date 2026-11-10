#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/release/telerust}"
if [[ ! -x "$binary" ]]; then
    printf 'Executable not found: %s\nRun cargo build --release -p telerust first.\n' "$binary" >&2
    exit 2
fi
if [[ -z "${WAYLAND_DISPLAY:-}${DISPLAY:-}" ]]; then
    printf 'No graphical display detected. Run this from the target GNOME Wayland session.\n' >&2
    exit 2
fi

for count in 100 1000 10000; do
    log_file="$(mktemp)"
    TELERUST_SYNTHETIC_HISTORY="$count" "$binary" >"$log_file" 2>&1 &
    app_pid=$!
    sleep 2
    if ! kill -0 "$app_pid" 2>/dev/null; then
        cat "$log_file" >&2
        rm -f "$log_file"
        exit 1
    fi
    status_file="/proc/$app_pid/status"
    rollup_file="/proc/$app_pid/smaps_rollup"
    rss_kib="$(awk '/^VmRSS:/ {print $2}' "$status_file")"
    pss_kib="$(awk '/^Pss:/ {print $2}' "$rollup_file")"
    uss_kib="$(awk '/^Private_(Clean|Dirty|Hugetlb):/ {sum += $2} END {print sum+0}' "$rollup_file")"
    scroll_complete=false
    for _ in $(seq 1 12); do
        if grep -q '^synthetic-scroll ' "$log_file"; then
            scroll_complete=true
            break
        fi
        sleep 1
    done
    if [[ "$scroll_complete" == true ]]; then
        cpu_ticks_before="$(awk '{print $14+$15}' "/proc/$app_pid/stat")"
        sleep 3
        cpu_ticks_after="$(awk '{print $14+$15}' "/proc/$app_pid/stat")"
        ticks_per_second="$(getconf CLK_TCK)"
        cpu="$(awk -v before="$cpu_ticks_before" -v after="$cpu_ticks_after" -v ticks="$ticks_per_second" 'BEGIN {printf "%.2f", (after-before)*100/(ticks*3)}')"
        printf 'messages=%s rss_kib=%s pss_kib=%s uss_kib=%s idle_cpu_percent_3s=%s\n' "$count" "$rss_kib" "$pss_kib" "$uss_kib" "$cpu"
    else
        printf 'messages=%s rss_kib=%s pss_kib=%s uss_kib=%s idle_cpu_percent_3s=unavailable scroll_stress=timeout\n' "$count" "$rss_kib" "$pss_kib" "$uss_kib"
    fi
    grep -E '^synthetic-(history-model|history |scroll) ' "$log_file" || true
    kill -TERM "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
    rm -f "$log_file"
done
