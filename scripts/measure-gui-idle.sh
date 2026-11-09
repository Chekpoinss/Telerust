#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
if pgrep -x telerust >/dev/null 2>&1; then
  echo "Close the existing Telerust window before running this measurement." >&2
  exit 2
fi
if [[ ! -x target/debug/telerust ]]; then
  echo "Build first with: cargo build -p telerust" >&2
  exit 2
fi

target/debug/telerust >/tmp/telerust-gui-idle.log 2>&1 &
app_pid=$!
cleanup() {
  kill -TERM "$app_pid" 2>/dev/null || true
  wait "$app_pid" 2>/dev/null || true
}
trap cleanup EXIT

for _ in $(seq 1 50); do
  [[ -r "/proc/$app_pid/stat" ]] && break
  sleep 0.1
done
sleep 2
read -r start_user start_system < <(awk '{print $14, $15}' "/proc/$app_pid/stat")
start_time=$(date +%s%N)
duration=${1:-10}
sleep "$duration"
end_time=$(date +%s%N)
read -r end_user end_system < <(awk '{print $14, $15}' "/proc/$app_pid/stat")
rss_kib=$(awk '/^VmRSS:/ {print $2}' "/proc/$app_pid/status")
hwm_kib=$(awk '/^VmHWM:/ {print $2}' "/proc/$app_pid/status")
ticks=$(getconf CLK_TCK)
awk -v u="$start_user" -v s="$start_system" -v ue="$end_user" -v se="$end_system" \
  -v tick="$ticks" -v start="$start_time" -v end="$end_time" \
  -v rss="$rss_kib" -v hwm="$hwm_kib" 'BEGIN {
    elapsed=(end-start)/1000000000
    cpu=((ue-u)+(se-s))/tick/elapsed*100
    printf "Scenario: UI-only, no account config / no TDLib worker\n"
    printf "Window RSS: %s KiB\nWindow HWM: %s KiB\n", rss, hwm
    printf "Mean process CPU over %.1f s: %.3f%%\n", elapsed, cpu
  }'
