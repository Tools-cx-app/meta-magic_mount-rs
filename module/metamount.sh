#!/system/bin/sh
# Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
# SPDX-License-Identifier: Apache-2.0

# meta-overlayfs Module Mount Handler
# This script is the entry point for dual-directory module mounting

MODDIR="${0%/*}"

ABI=$(getprop ro.product.cpu.abi)

if [ -z "$ABI" ]; then
  abort "! Failed to detect device architecture"
fi

case "$ABI" in
arm64-v8a)
  ARCH_BINARY="magic_mount_rs.aarch64"
  ;;
armeabi-v7a)
  ARCH_BINARY="magic_mount_rs.armv7"
  ;;
*)
  abort "! Unsupported platform: $ABI"
  ;;
esac

BINARY="$MODDIR/bin/$ARCH_BINARY"

if [ ! -f "$BINARY" ]; then
  log "ERROR: Binary not found: $BINARY"
  exit 1
fi

nohup $BINARY 2>&1

EXIT_CODE=$?

if [ "$EXIT_CODE" = 0 ]; then
  /data/adb/ksud kernel notify-module-mounted
fi

exit 0
