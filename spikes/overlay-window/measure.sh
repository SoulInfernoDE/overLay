#!/usr/bin/env bash
# Usage: ./measure.sh [seconds]   (default 30)
exec "$(dirname "$0")/../../tools/measure.sh" overlay-window-spike "$@"
