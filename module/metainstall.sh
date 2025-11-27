#!/system/bin/sh
############################################
# meta-hybrid metainstall.sh
############################################

# This tells KernelSU it's a metamodule
export KSU_HAS_METAMODULE="true"
export KSU_METAMODULE="meta-hybrid"

# Constants
BASE_DIR="/data/adb/meta-hybrid"

# 1. The installation process
ui_print "- Using Hybrid Mount metainstall"

# Standard installation (extracts to /data/adb/modules/meta-hybrid)
install_module

# NOTE: We NO LONGER move files to modules.img at install time.
# The Rust daemon will sync files to Tmpfs/Image at boot.
# Users MUST reinstall their modules if upgrading from v0.2.x.

ui_print "- Installation complete"