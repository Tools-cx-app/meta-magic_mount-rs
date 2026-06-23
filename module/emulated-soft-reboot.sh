#!/system/bin/sh
# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

MODDIR="${0%/*}"
BINARY="$MODDIR/meta-mm"

if [ ! -x "$BINARY" ]; then
  log "ERROR: Binary not found: $BINARY"
  exit 1
fi

exec "$BINARY" emulated-soft-reboot
