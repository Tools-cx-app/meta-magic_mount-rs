#ifndef UTILS_H
#define UTILS_H

#include <stdio.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdarg.h>

#define SELINUX_XATTR "security.selinux"
#define DEFAULT_TEMP_DIR "/dev/.magic_mount"

#ifndef TMPFS_MAGIC
#define TMPFS_MAGIC 0x01021994
#endif

#ifndef PATH_MAX
#define PATH_MAX 4096
#endif

/* from linux/sched.h normally */
int setns(int fd, int nstype);

extern FILE *g_log_file;
extern int   g_log_level;  /* 0=ERROR,1=WARN,2=INFO,3=DEBUG */

void log_msg(const char *lv, const char *fmt, ...);

#define LV_ERROR 0
#define LV_WARN  1
#define LV_INFO  2
#define LV_DEBUG 3

#define LOG(lv, ...)                                                           \
    do {                                                                       \
        if ((lv) <= g_log_level)                                               \
            log_msg((lv) == LV_ERROR ? "ERROR" :                               \
                    (lv) == LV_WARN  ? "WARN"  :                               \
                    (lv) == LV_INFO  ? "INFO"  :                               \
                                       "DEBUG",                                \
                    __VA_ARGS__);                                              \
    } while (0)

#define LOGE(...) LOG(LV_ERROR, __VA_ARGS__)
#define LOGW(...) LOG(LV_WARN,  __VA_ARGS__)
#define LOGI(...) LOG(LV_INFO,  __VA_ARGS__)
#define LOGD(...) LOG(LV_DEBUG, __VA_ARGS__)

int  path_join(const char *base, const char *name, char *buf, size_t n);
bool path_exists(const char *p);
bool path_is_dir(const char *p);
bool path_is_symlink(const char *p);
int  mkdir_p(const char *dir);

const char *select_auto_tempdir(char buf[PATH_MAX]);

void free_string_array(char ***arr, int *count);

int set_selinux(const char *path, const char *con);
int get_selinux(const char *path, char **out);

#endif /* UTILS_H */
