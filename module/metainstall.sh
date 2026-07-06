#!/system/bin/sh
# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

############################################
# meta-mm metainstall.sh
############################################

export KSU_HAS_METAMODULE="true"
export KSU_METAMODULE="mmrs"

# Main installation flow
ui_print "- Using mmrs metainstall"

# Define the module IDs that are not allowed to install (separated by spaces)
BLOCKLIST="scene_swap_controller AAaTempSpoof"

# Fallback mechanism to fetch the current module ID from available environment variables
CURRENT_MODULE="${KSU_MODULE:-$AP_MODULE}"

# Check if the current module is inside the blocklist
if [ -n "$CURRENT_MODULE" ]; then
  for blocked_id in $BLOCKLIST; do
    if [ "$CURRENT_MODULE" = "$blocked_id" ]; then
      ui_print "**********************************************"
      ui_print "! Module '$CURRENT_MODULE' already has self-mounting logic!"
      ui_print "! Marking skip mount"
      ui_print "**********************************************"
      touch "$MODPATH/skip_mount"
    fi
  done
fi

# If executing externally, change this to the absolute path of the target module.
# Target partitions to check

for part in vendor product system_ext; do
  # Check if the system partition is a symlink and the module uses the old layout
  if [ -L "/system/$part" ] && [ -d "$MODPATH/system/$part" ]; then
    if [ -d "$MODPATH/$part" ]; then
      # Safe merge if root directory already exists
      cp -a "$MODPATH/system/$part/." "$MODPATH/$part/" && rm -rf "$MODPATH/system/$part"
    else
      # Standard move operation
      mv "$MODPATH/system/$part" "$MODPATH/$part"
    fi
  fi
done

# we no-op handle_partition
# this way we can support normal hierarchy that ksu changes
handle_partition() {
  echo 0 >/dev/null
  true
}

mark_replace() {
  replace_target="$1"
  mkdir -p "$replace_target"
  setfattr -n trusted.overlay.opaque -v y "$replace_target"
}

# call install function, this is important!
install_module

mm_handle_partition() {
  partition="$1"

  if [ ! -d "$MODPATH/system/$partition" ]; then
    return
  fi

  if [ -L "/system/$partition" ] && [ -d "/$partition" ]; then
    ui_print "- Handle partition /$partition"
    ln -sf "./system/$partition" "$MODPATH/$partition"
  fi
}

mm_handle_partition system_ext
mm_handle_partition vendor
mm_handle_partition product

ui_print "- Installation complete"

metamodule_hot_install() {
  if [ -z "$MODID" ]; then
    return
  fi

  MODDIR_INTERNAL="/data/adb/modules/$MODID"
  MODPATH_INTERNAL="/data/adb/modules_update/$MODID"

  if [ ! -d "$MODDIR_INTERNAL" ] || [ ! -d "$MODPATH_INTERNAL" ]; then
    return
  fi

  # hot install
  busybox rm -rf "$MODDIR_INTERNAL"
  busybox mv "$MODPATH_INTERNAL" "$MODDIR_INTERNAL"

  # run script requested, blocking, just fork it yourselves if you want it on background
  if [ ! -z "$MODULE_HOT_RUN_SCRIPT" ]; then
    [ -f "$MODDIR_INTERNAL/$MODULE_HOT_RUN_SCRIPT" ] && sh "$MODDIR_INTERNAL/$MODULE_HOT_RUN_SCRIPT"
  fi

  # we do this dance to satisfy kernelsu's ensure_file_exists
  mkdir -p "$MODPATH_INTERNAL"
  cat "$MODDIR_INTERNAL/module.prop" >"$MODPATH_INTERNAL/module.prop"

  (
    sleep 3
    rm -rf "$MODDIR_INTERNAL/update"
    rm -rf "$MODPATH_INTERNAL"
  ) & # fork in background

  ui_print "- Module hot install requested!"
  ui_print "- Refresh module page after installation!"
  ui_print "- No need to reboot!"
}

if [ "$MODULE_HOT_INSTALL_REQUEST" = true ]; then
  metamodule_hot_install
fi
