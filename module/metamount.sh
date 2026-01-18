#!/system/bin/sh
# meta-overlayfs Module Mount Handler
# This script is the entry point for dual-directory module mounting

MODDIR="${0%/*}"

# Binary path (architecture-specific binary selected during installation)
BINARY="$MODDIR/meta-mm"

if [ ! -f "$BINARY" ]; then
  log "ERROR: Binary not found: $BINARY"
  exit 1
fi

if [ -f "/data/adb/magic_mount/mm.log" ]; then
  mv "/data/adb/magic_mount/mm.log" "/data/adb/magic_mount/mm.log.bak"
fi

nohup $BINARY >"/data/adb/magic_mount/mm.log" 2>&1

EXIT_CODE=$?

if [ "$EXIT_CODE" = 0 ]; then
  /data/adb/ksud kernel notify-module-mounted
  log "Mount completed successfully"
else
  log "Mount failed with exit code $EXIT_CODE"
fi

exit 0
