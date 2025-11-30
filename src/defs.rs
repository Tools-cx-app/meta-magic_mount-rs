// src/defs.rs

// Hybrid Mount Constants

// NOTE: The actual content directory is now determined dynamically at runtime.
// Relocated to 'img_mnt' to avoid conflicts and improve clarity
pub const FALLBACK_CONTENT_DIR: &str = "/data/adb/meta-hybrid/img_mnt/";

// The base directory for our own config and logs
pub const BASE_DIR: &str = "/data/adb/meta-hybrid/";

// Runtime state directory
pub const RUN_DIR: &str = "/data/adb/meta-hybrid/run/";

// [NEW] Centralized structured state file
pub const STATE_FILE: &str = "/data/adb/meta-hybrid/run/daemon_state.json";

// Log file path (Must match WebUI)
pub const DAEMON_LOG_FILE: &str = "/data/adb/meta-hybrid/daemon.log";

// Markers
pub const DISABLE_FILE_NAME: &str = "disable";
pub const REMOVE_FILE_NAME: &str = "remove";
pub const SKIP_MOUNT_FILE_NAME: &str = "skip_mount";

// OverlayFS Source Name
pub const OVERLAY_SOURCE: &str = "KSU";
pub const KSU_OVERLAY_SOURCE: &str = OVERLAY_SOURCE;

// Path for overlayfs workdir/upperdir (if needed in future)
#[allow(dead_code)]
pub const SYSTEM_RW_DIR: &str = "/data/adb/meta-hybrid/rw";

// Module Prop Path (for dynamic description updates)
pub const MODULE_PROP_FILE: &str = "/data/adb/modules/meta-hybrid/module.prop";

// Standard Android partitions to check
pub const BUILTIN_PARTITIONS: &[&str] = &["system", "vendor", "product", "system_ext", "odm", "oem"];