#!/system/bin/sh
# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

if [ -z "$APATCH" ] && [ -z "$KSU" ]; then
  abort "! unsupported root platform"
fi

if [ -n "$KSU_LATE_LOAD" ] && [ -n "$KSU" ]; then
  abort "! unsupported late load mode"
fi

VERSION=$(grep_prop version "${MODPATH}/module.prop")
ui_print "- mmrs version ${VERSION}"

ui_print "- Detecting device architecture..."

ABI=$(getprop ro.product.cpu.abi)

if [ -z "$ABI" ]; then
  abort "! Failed to detect device architecture"
fi

ui_print "- Device platform: $ABI"

case "$ABI" in
arm64-v8a|armeabi-v7a)
  ui_print "- Selected architecture: $ABI"
  ;;
*)
  abort "! Unsupported platform: $ABI"
  ;;
esac

# Binary is named "meta-mm" in the zip — metamount.sh references it directly.
chmod 755 "$MODPATH/meta-mm" || abort "! Failed to set permissions"

ui_print "- mmrs binary installed"

mkdir -p "/data/adb/magic_mount"

if [ ! -f "/data/adb/magic_mount/config.toml" ]; then
  ui_print "- Add default config"
  if [ -n "${APATCH:-}" ]; then
    cat "$MODPATH/config_apatch.toml" >"/data/adb/magic_mount/config.toml"
  fi

  if [ -n "${KSU:-}" ]; then
    cat "$MODPATH/config.toml" >"/data/adb/magic_mount/config.toml"
  fi

fi

cp "$MODPATH/module.prop" "$MODPATH/module.prop.orig"

ui_print "- Installation complete"
ui_print "- Welcome to mmrs!"
