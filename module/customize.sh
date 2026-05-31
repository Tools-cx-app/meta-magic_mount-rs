#!/system/bin/sh
# Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
# SPDX-License-Identifier: Apache-2.0

if [ -z "$APATCH" ] && [ -z "$KSU" ]; then
  abort "! unsupported root platform"
fi

if [ -n "$KSU_LATE_LOAD" ] && [ -n "$KSU" ]; then
  abort "! unsupported late load mode"
fi

VERSION=$(grep_prop version "${MODPATH}/module.prop")
ui_print "- mmrs version ${VERSION}"

# Ensure the binary is executable
chmod 755 "$MODPATH/bin/magic_mount_rs.aarch64" || abort "! Failed to set permissions"
chmod 755 "$MODPATH/bin/magic_mount_rs.armv7" || abort "! Failed to set permissions"

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
