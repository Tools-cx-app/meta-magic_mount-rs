// utils
pub const SELINUX_XATTR: &str = "security.selinux";
pub const TEMP_DIR_SUFFIX: &str = ".magic_mount";
pub const TMPFS_CANDIDATES: &[&str] = &["/debug_ramdisk", "/patch_hw", "/oem", "/root", "/sbin"];
// magic_mount
pub const DISABLE_FILE_NAME: &str = "disable";
pub const REMOVE_FILE_NAME: &str = "remove";
pub const SKIP_MOUNT_FILE_NAME: &str = "skip_mount";
pub const REPLACE_DIR_FILE_NAME: &str = ".replace";
pub const REPLACE_DIR_XATTR: &str = "trusted.overlay.opaque";

// config
pub const CONFIG_FILE_DEFAULT: &str = "/data/adb/magic_mount/config.toml";
