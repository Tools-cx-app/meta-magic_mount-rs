#!/system/bin/sh
# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

############################################
# mm-mm uninstall.sh
# Cleanup script for metamodule removal
############################################

MODDIR="${0%/*}"
STATE_DIR="/data/adb/magic_mount"
PID_FILE="$STATE_DIR/daemon.lock"
DAEMON="$MODDIR/bin/$(getprop ro.product.cpu.abi)/daemon"

if [ -r "$PID_FILE" ]; then
  DAEMON_PID=$(cat "$PID_FILE")
  case "$DAEMON_PID" in
  '' | *[!0-9]*) ;;
  *)
    RUNNING_BINARY=$(readlink "/proc/$DAEMON_PID/exe" 2>/dev/null)
    case "$RUNNING_BINARY" in
    "$DAEMON" | "$DAEMON (deleted)") kill "$DAEMON_PID" 2>/dev/null ;;
    esac
    ;;
  esac
fi

rm -rf "$STATE_DIR"

exit 0
