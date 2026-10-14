#!/usr/bin/env bash
# Measures memory (PSS) and CPU of a running program, including its WebKit helper
# processes. PSS splits shared pages fairly between processes, so the sum is honest.
#
# Usage: tools/measure.sh <binary name | pid> [seconds]   (seconds default 30)
# With a binary name, the first process running that executable is the root.
set -euo pipefail
export LC_ALL=C  # printf expects "." as decimal separator

target=${1:?usage: measure.sh <binary name | pid> [seconds]}
seconds=${2:-30}

root=""
if [[ "$target" =~ ^[0-9]+$ ]]; then
  root=$target
fi
for exe in /proc/[0-9]*/exe; do
  [[ -n "$root" ]] && break
  if [[ "$(readlink "$exe" 2>/dev/null)" == */"$target" ]]; then
    root=${exe#/proc/}
    root=${root%/exe}
    break
  fi
done
if [[ -z "$root" ]]; then
  echo "$target is not running" >&2
  exit 1
fi

descendants() {
  echo "$1"
  local child
  for child in $(cat /proc/"$1"/task/*/children 2>/dev/null); do
    descendants "$child"
  done
}

cpu_ticks() {
  local total=0 pid fields
  for pid in "$@"; do
    # Fields 14 and 15 (utime, stime) after the command name, which may contain spaces.
    fields=$(sed 's/^.*) //' /proc/"$pid"/stat 2>/dev/null) || continue
    total=$((total + $(awk '{print $12 + $13}' <<<"$fields")))
  done
  echo "$total"
}

mapfile -t pids < <(descendants "$root")

printf '%-8s %-28s %10s %10s\n' PID PROCESS RSS_MB PSS_MB
total_pss=0
for pid in "${pids[@]}"; do
  [[ -r /proc/$pid/smaps_rollup ]] || continue
  rss=$(awk '/^Rss:/ {print $2}' /proc/"$pid"/smaps_rollup)
  pss=$(awk '/^Pss:/ {print $2}' /proc/"$pid"/smaps_rollup)
  total_pss=$((total_pss + pss))
  printf '%-8s %-28s %10.1f %10.1f\n' "$pid" "$(cat /proc/"$pid"/comm)" "$(bc -l <<<"$rss/1024")" "$(bc -l <<<"$pss/1024")"
done
printf '%-37s %21.1f MB PSS\n' "TOTAL (${#pids[@]} processes)" "$(bc -l <<<"$total_pss/1024")"

hz=$(getconf CLK_TCK)
start=$(cpu_ticks "${pids[@]}")
sleep "$seconds"
end=$(cpu_ticks "${pids[@]}")
printf 'CPU over %ss: %.2f %% of one core\n' "$seconds" "$(bc -l <<<"($end - $start) * 100 / ($hz * $seconds)")"
