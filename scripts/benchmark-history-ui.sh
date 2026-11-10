#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/release/telerust}"
repeats="${TELERUST_BENCHMARK_REPEATS:-5}"
if [[ ! -x "$binary" ]]; then
    printf 'Executable not found: %s\nRun cargo build --release -p telerust first.\n' "$binary" >&2
    exit 2
fi
if [[ -z "${WAYLAND_DISPLAY:-}${DISPLAY:-}" ]]; then
    printf 'No graphical display detected. Run this from a GNOME Wayland session.\n' >&2
    exit 2
fi

median() {
    printf '%s\n' "$@" | sort -n | awk '{ values[NR]=$1 } END {
        if (NR == 0) { print "n/a"; exit }
        if (NR % 2) print values[(NR+1)/2]
        else printf "%.2f\n", (values[NR/2] + values[NR/2+1]) / 2
    }'
}

measure_scenario() {
    local label="$1" count="$2" with_media="$3" with_switches="$4"
    local -a rss_values=() pss_values=() uss_values=() cpu_values=()
    printf 'scenario=%s repeats=%s\n' "$label" "$repeats"
    for repeat in $(seq 1 "$repeats"); do
        local log_file app_pid rss_kib pss_kib uss_kib cpu_ticks_before cpu_ticks_after ticks_per_second cpu
        log_file="$(mktemp)"
        if [[ "$with_switches" == true ]]; then
            TELERUST_SYNTHETIC_CHAT_SWITCHES=1 TELERUST_SYNTHETIC_HISTORY="$count" "$binary" >"$log_file" 2>&1 &
        elif [[ "$with_media" == true ]]; then
            TELERUST_SYNTHETIC_MEDIA=1 TELERUST_SYNTHETIC_HISTORY="$count" "$binary" >"$log_file" 2>&1 &
        else
            TELERUST_SYNTHETIC_HISTORY="$count" "$binary" >"$log_file" 2>&1 &
        fi
        app_pid=$!
        sleep 2
        if ! kill -0 "$app_pid" 2>/dev/null; then
            cat "$log_file" >&2
            rm -f "$log_file"
            printf 'synthetic process exited before sampling\n' >&2
            exit 1
        fi
        local stress_complete=false
        if [[ "$with_switches" == true ]]; then
            for _ in $(seq 1 30); do
                if grep -q '^synthetic-chat-switches ' "$log_file"; then stress_complete=true; break; fi
                sleep 1
            done
        else
            rss_kib="$(awk '/^VmRSS:/ {print $2}' "/proc/$app_pid/status")"
            pss_kib="$(awk '/^Pss:/ {print $2}' "/proc/$app_pid/smaps_rollup")"
            uss_kib="$(awk '/^Private_(Clean|Dirty|Hugetlb):/ {sum += $2} END {print sum+0}' "/proc/$app_pid/smaps_rollup")"
            for _ in $(seq 1 12); do
                if grep -q '^synthetic-scroll ' "$log_file"; then stress_complete=true; break; fi
                sleep 1
            done
        fi
        if [[ "$with_switches" == true ]]; then
            rss_kib="$(awk '/^VmRSS:/ {print $2}' "/proc/$app_pid/status")"
            pss_kib="$(awk '/^Pss:/ {print $2}' "/proc/$app_pid/smaps_rollup")"
            uss_kib="$(awk '/^Private_(Clean|Dirty|Hugetlb):/ {sum += $2} END {print sum+0}' "/proc/$app_pid/smaps_rollup")"
        fi
        if [[ "$stress_complete" == true ]]; then
            cpu_ticks_before="$(awk '{print $14+$15}' "/proc/$app_pid/stat")"
            sleep 3
            cpu_ticks_after="$(awk '{print $14+$15}' "/proc/$app_pid/stat")"
            ticks_per_second="$(getconf CLK_TCK)"
            cpu="$(awk -v before="$cpu_ticks_before" -v after="$cpu_ticks_after" -v ticks="$ticks_per_second" 'BEGIN {printf "%.2f", (after-before)*100/(ticks*3)}')"
            rss_values+=("$rss_kib"); pss_values+=("$pss_kib"); uss_values+=("$uss_kib"); cpu_values+=("$cpu")
            printf '  repeat=%s rss_kib=%s pss_kib=%s uss_kib=%s post-stress_cpu_percent_3s=%s\n' "$repeat" "$rss_kib" "$pss_kib" "$uss_kib" "$cpu"
            grep -E '^synthetic-(history-model|history |scroll|geometry|chat-switches) ' "$log_file" || true
        else
            printf '  repeat=%s rss_kib=%s pss_kib=%s uss_kib=%s post-stress_cpu_percent_3s=unavailable stress=timeout\n' "$repeat" "$rss_kib" "$pss_kib" "$uss_kib"
            grep -E '^synthetic-' "$log_file" || true
        fi
        kill -TERM "$app_pid" 2>/dev/null || true
        wait "$app_pid" 2>/dev/null || true
        rm -f "$log_file"
    done
    printf '  median rss_kib=%s pss_kib=%s uss_kib=%s post-scroll_cpu_percent_3s=%s\n' \
        "$(median "${rss_values[@]}")" "$(median "${pss_values[@]}")" "$(median "${uss_values[@]}")" "$(median "${cpu_values[@]}")"
}

if [[ -z "${TELERUST_BENCHMARK_ONLY:-}" || "${TELERUST_BENCHMARK_ONLY}" == history_100 ]]; then measure_scenario history_100 100 false false; fi
if [[ -z "${TELERUST_BENCHMARK_ONLY:-}" || "${TELERUST_BENCHMARK_ONLY}" == history_1000 ]]; then measure_scenario history_1000 1000 false false; fi
if [[ -z "${TELERUST_BENCHMARK_ONLY:-}" || "${TELERUST_BENCHMARK_ONLY}" == history_10000 ]]; then measure_scenario history_10000 10000 false false; fi
if [[ -z "${TELERUST_BENCHMARK_ONLY:-}" || "${TELERUST_BENCHMARK_ONLY}" == media_1000 ]]; then measure_scenario media_1000 1000 true false; fi
if [[ -z "${TELERUST_BENCHMARK_ONLY:-}" || "${TELERUST_BENCHMARK_ONLY}" == chat_switch_1000 ]]; then measure_scenario chat_switch_1000 1000 false true; fi
