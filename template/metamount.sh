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

MM_LOG_FILE=$(busybox awk -F= '
/^[[:space:]]*log_file[[:space:]]*=/ {
    val=$2
    sub(/#.*/, "", val)
    gsub(/^[ \t"]+|[ \t"]+$/, "", val)
    print val
}' /data/adb/magic_mount/mm.conf)

if [ -f "$MM_LOG_FILE" ]; then
    mv "$MM_LOG_FILE" "$MM_LOG_FILE".old
fi

# Set environment variables
export MODULE_METADATA_DIR="/data/adb/modules"

log "Metadata directory: $MODULE_METADATA_DIR"
log "Executing $BINARY"

$BINARY

EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    log "Mount failed with exit code $EXIT_CODE"
    exit $EXIT_CODE
fi

log "Mount completed successfully"
exit 0
