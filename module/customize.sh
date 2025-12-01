#!/system/bin/sh

ui_print "- Detecting device architecture..."

ABI=$(grep_get_prop ro.product.cpu.abi)
ui_print "- Detected ABI: $ABI"

if [ ! -f "$MODPATH/meta-hybrid" ]; then
    abort "! Binary not found: meta-hybrid"
fi

chmod 755 "$MODPATH/meta-hybrid"

BASE_DIR="/data/adb/meta-hybrid"
mkdir -p "$BASE_DIR"

if [ ! -f "$BASE_DIR/config.toml" ]; then
  ui_print "- Installing default config"
  cat "$MODPATH/config.toml" > "$BASE_DIR/config.toml"
fi

IMG_FILE="$BASE_DIR/modules.img"
IMG_SIZE_MB=2048

if [ ! -f "$IMG_FILE" ]; then
    ui_print "- Creating 2GB ext4 image for module storage..."
    truncate -s ${IMG_SIZE_MB}M "$IMG_FILE"
    
    /system/bin/mke2fs -t ext4 -O ^has_journal -F "$IMG_FILE" >/dev/null 2>&1
    
    if [ $? -ne 0 ]; then
        ui_print "! Failed to format ext4 image"
    else
        ui_print "- Image created successfully (No Journal Mode)"
    fi
else
    ui_print "- Reusing existing modules.img"
fi

ui_print "- Installation complete"