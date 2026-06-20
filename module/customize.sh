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
arm64-v8a)
  ui_print "- Selected architecture: arm64-v8a"
  ARCH_BINARY="arm64-v8a/magic_mount_rs"
  ;;
armeabi-v7a)
  ui_print "- Selected architecture: armeabi-v7a"
  ARCH_BINARY="armeabi-v7a/magic_mount_rs"
  ;;
*)
  abort "! Unsupported platform: $ABI"
  ;;
esac

# Ensure the binary is executable
chmod 755 "$MODPATH/bin/$ARCH_BINARY" || abort "! Failed to set permissions"
ln -s "./bin/$ARCH_BINARY" "$MODPATH/meta-mm" || abort "! Failed to create symlink"

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

ui_print "- Installation complete"
ui_print "- Welcome to mmrs!"
