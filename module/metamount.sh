#!/system/bin/sh
# Hybrid Mount Startup Script

MODDIR="${0%/*}"
BASE_DIR="/data/adb/meta-hybrid"
LOG_FILE="$BASE_DIR/daemon.log"

# Ensure base directory exists
mkdir -p "$BASE_DIR"

# Clean previous log on boot
if [ -f "$LOG_FILE" ]; then
    rm "$LOG_FILE"
fi

log() {
    echo "[Wrapper] $1" >> "$LOG_FILE"
}

log "Starting Hybrid Mount..."

# Note: Storage mounting is now handled by the Rust binary (Smart Storage).

# 1. Prepare Rust binary
BINARY="$MODDIR/meta-hybrid"
if [ ! -f "$BINARY" ]; then
    log "ERROR: Binary not found at $BINARY"
    exit 1
fi

chmod 755 "$BINARY"

# 2. Execute Rust Binary
"$BINARY" >> "$LOG_FILE" 2>&1
EXIT_CODE=$?

log "Hybrid Mount exited with code $EXIT_CODE"

# 3. Notify KernelSU
if [ "$EXIT_CODE" = "0" ]; then
    /data/adb/ksud kernel notify-module-mounted
fi

exit $EXIT_CODE