#!/system/bin/sh
# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

MODDIR="${0%/*}"

ABI=$(getprop ro.product.cpu.abi)

if [ -z "$ABI" ]; then
  abort "! Failed to detect device architecture"
fi

BINARY="$MODDIR/meta-mm"

if [ ! -f "$BINARY" ]; then
  log "ERROR: Binary not found: $BINARY"
  exit 1
fi

nohup $BINARY 2>&1

EXIT_CODE=$?

if [ "$EXIT_CODE" = 0 ]; then
  /data/adb/ksud kernel notify-module-mounted
fi

exit $EXIT_CODE
