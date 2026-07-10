#!/system/bin/sh
# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

# shellcheck disable=SC2034
SKIPUNZIP=1

if [ -z "$APATCH" ] && [ -z "$KSU" ]; then
  abort "! unsupported root platform"
fi

if [ -n "$KSU_LATE_LOAD" ] && [ -n "$KSU" ]; then
  abort "! unsupported late load mode"
fi

case "$ARCH" in
arm64)
  ui_print "- Selected architecture: arm64-v8a"
  ARCH_BINARY="arm64-v8a"
  ;;
arm)
  ui_print "- Selected architecture: armeabi-v7a"
  ARCH_BINARY="armeabi-v7a"
  ;;
x64)
  ui_print " - Selected architecture: x86_64"
  ARCH_BINARY="x86_64"
  ;;
*)
  abort "! Unsupported platform: $ABI"
  ;;
esac

VERSION=$(grep_prop version "${TMPDIR}/module.prop")
ui_print "- mmrs version ${VERSION}"

ui_print "- Extracting verify.sh"
unzip -o "$ZIPFILE" 'verify.sh' -d "$TMPDIR" >&2
if [ ! -f "$TMPDIR/verify.sh" ]; then
  ui_print "*********************************************************"
  ui_print "! Unable to extract verify.sh!"
  ui_print "! This zip may be corrupted, please try downloading again"
  abort "*********************************************************"
fi
# shellcheck disable=SC1091
. "$TMPDIR/verify.sh"

extract 'module.prop'
extract 'config.toml'
extract 'config_apatch.toml'
extract 'metainstall.sh'
extract 'metamount.sh'
extract 'metauninstall.sh'
extract 'uninstall.sh'
extract 'launcher.png'
mkdir -p "$MODPATH/bin/$ARCH_BINARY"
extract "bin/$ARCH_BINARY/magic_mount_rs" "bin/$ARCH_BINARY" "magic_mount_rs"

# Ensure the binary is executable
chmod 755 "$MODPATH/bin/$ARCH_BINARY" -R || abort "! Failed to set permissions"
ln -s "./bin/$ARCH_BINARY/magic_mount_rs" "$MODPATH/meta-mm" || abort "! Failed to create symlink"

unzip -o "$ZIPFILE" "webroot/*" -x "*.sha256" -d "$MODPATH"

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
