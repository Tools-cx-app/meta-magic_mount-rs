// Hybrid Mount Constants
// Content: Where system/, vendor/ files live (Mounted from modules.img)
// This keeps OverlayFS happy with Upperdir/Lowerdir requirements
pub const MODULE_CONTENT_DIR: &str = "/data/adb/meta-hybrid/mnt/";

// The base directory for our own config and logs
// pub const HYBRID_BASE_DIR: &str = "/data/adb/meta-hybrid/"; // Unused for now

// Log file path (Must match WebUI)
pub const DAEMON_LOG_FILE: &str = "/data/adb/meta-hybrid/daemon.log";

// Markers
pub const DISABLE_FILE_NAME: &str = "disable";
pub const REMOVE_FILE_NAME: &str = "remove";
pub const SKIP_MOUNT_FILE_NAME: &str = "skip_mount";

// OverlayFS Source Name
pub const OVERLAY_SOURCE: &str = "KSU";

// --- Fixes for compilation errors ---
pub const KSU_OVERLAY_SOURCE: &str = OVERLAY_SOURCE;
// Path for overlayfs workdir/upperdir (if needed in future)
#[allow(dead_code)]
pub const SYSTEM_RW_DIR: &str = "/data/adb/meta-hybrid/rw";
// End of Hybrid Mount Constants