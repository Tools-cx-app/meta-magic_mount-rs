#ifndef MAGIC_MOUNT_H
#define MAGIC_MOUNT_H

#include <stdio.h>

#define DISABLE_FILE_NAME      "disable"
#define REMOVE_FILE_NAME       "remove"
#define SKIP_MOUNT_FILE_NAME   "skip_mount"

#define REPLACE_DIR_XATTR "trusted.overlay.opaque"

#define DEFAULT_MOUNT_SOURCE  "KSU"
#define DEFAULT_MODULE_DIR    "/data/adb/modules"

typedef struct {
    int modules_total;
    int nodes_total;
    int nodes_mounted;
    int nodes_skipped;
    int nodes_whiteout;
    int nodes_fail;
} MountStats;

extern MountStats g_stats;

extern const char *g_module_dir;
extern const char *g_mount_source;

extern char **g_failed_modules;
extern int   g_failed_modules_count;

extern char **g_extra_parts;
extern int   g_extra_parts_count;

int magic_mount(const char *tmp_root);

void add_extra_part_token(const char *start, size_t len);

#endif /* MAGIC_MOUNT_H */
