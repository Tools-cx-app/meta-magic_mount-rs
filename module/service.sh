#!/system/bin/sh
# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

MODDIR="${0%/*}"
DAEMON="$MODDIR/bin/$(getprop ro.product.cpu.abi)/daemon"
PID_FILE="/data/adb/magic_mount/daemon.lock"

if [ ! -x "$DAEMON" ]; then
  exit 1
fi

daemon_is_running() {
  [ -r "$PID_FILE" ] || return 1
  RUNNING_PID=$(cat "$PID_FILE")
  case "$RUNNING_PID" in
  '' | *[!0-9]*) return 1 ;;
  esac
  RUNNING_BINARY=$(readlink "/proc/$RUNNING_PID/exe" 2>/dev/null)
  case "$RUNNING_BINARY" in
  "$DAEMON" | "$DAEMON (deleted)") return 0 ;;
  *) return 1 ;;
  esac
}

if daemon_is_running; then
  exit 0
fi

DAEMON_PID=''
# shellcheck disable=SC2329 # Called by the signal trap below.
stop_daemon() {
  if [ -n "$DAEMON_PID" ]; then
    kill "$DAEMON_PID" 2>/dev/null
    wait "$DAEMON_PID"
  fi
  exit 0
}
trap stop_daemon INT TERM

while [ -x "$DAEMON" ]; do
  "$DAEMON" &
  DAEMON_PID=$!
  wait "$DAEMON_PID"
  DAEMON_STATUS=$?
  DAEMON_PID=''

  if [ "$DAEMON_STATUS" -eq 0 ]; then
    exit 0
  fi
  if daemon_is_running; then
    exit 0
  fi

  sleep 1
done

exit 1
