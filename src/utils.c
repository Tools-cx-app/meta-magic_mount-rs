#include "utils.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <sys/xattr.h>
#include <sys/statfs.h>
#include <dirent.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include <stdlib.h>

FILE *g_log_file = NULL;  /* stderr */
int   g_log_level = 2;    /* INFO */

static void vlog(const char *lv, const char *fmt, va_list ap)
{
    FILE *out = g_log_file ? g_log_file : stderr;
    fprintf(out, "[%s] ", lv);
    vfprintf(out, fmt, ap);
    fputc('\n', out);
    fflush(out);
}

void log_msg(const char *lv, const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    vlog(lv, fmt, ap);
    va_end(ap);
}

int path_join(const char *base, const char *name, char *buf, size_t n)
{
    if (!base) {
        errno = EINVAL;
        return -1;
    }

    if (!name || !name[0]) {
        if (snprintf(buf, n, "%s", base) >= (int)n) {
            errno = ENAMETOOLONG;
            return -1;
        }
        return 0;
    }

    if (!base[0] || (base[0] == '/' && !base[1])) {
        if (snprintf(buf, n, "/%s", name) >= (int)n) {
            errno = ENAMETOOLONG;
            return -1;
        }
    } else {
        size_t len = strlen(base);
        if (base[len - 1] == '/') {
            if (snprintf(buf, n, "%s%s", base, name) >= (int)n) {
                errno = ENAMETOOLONG;
                return -1;
            }
        } else {
            if (snprintf(buf, n, "%s/%s", base, name) >= (int)n) {
                errno = ENAMETOOLONG;
                return -1;
            }
        }
    }
    return 0;
}

bool path_exists(const char *p)
{
    struct stat st;
    return (stat(p, &st) == 0);
}

bool path_is_dir(const char *p)
{
    struct stat st;
    return (stat(p, &st) == 0) && S_ISDIR(st.st_mode);
}

bool path_is_symlink(const char *p)
{
    struct stat st;
    return (lstat(p, &st) == 0) && S_ISLNK(st.st_mode);
}

int mkdir_p(const char *dir)
{
    if (!dir || !dir[0]) {
        errno = EINVAL;
        return -1;
    }

    struct stat st;
    if (stat(dir, &st) == 0) {
        if (S_ISDIR(st.st_mode))
            return 0;
        LOGE("%s is not directory", dir);
        errno = ENOTDIR;
        return -1;
    }

    char tmp[PATH_MAX];
    if (strlen(dir) >= sizeof(tmp)) {
        errno = ENAMETOOLONG;
        return -1;
    }

    strcpy(tmp, dir);
    char *s = strrchr(tmp, '/');
    if (s && s != tmp) {
        *s = 0;
        if (mkdir_p(tmp) != 0)
            return -1;
    }

    if (mkdir(dir, 0755) == 0 || errno == EEXIST)
        return 0;

    LOGE("mkdir %s: %s", dir, strerror(errno));
    return -1;
}

static bool is_rw_tmpfs(const char *path)
{
    if (!path_is_dir(path))
        return false;

    struct statfs s;
    if (statfs(path, &s) < 0)
        return false;
    if ((unsigned long)s.f_type != (unsigned long)TMPFS_MAGIC)
        return false;

    char tmpl[PATH_MAX];
    if (path_join(path, ".magic_mount_testXXXXXX", tmpl, sizeof(tmpl)) != 0)
        return false;

    int fd = mkstemp(tmpl);
    if (fd < 0)
        return false;
    close(fd);
    unlink(tmpl);
    return true;
}

void free_string_array(char ***arr, int *count)
{
    if (!arr || !*arr || !count)
        return;
    for (int i = 0; i < *count; ++i)
        free((*arr)[i]);
    free(*arr);
    *arr = NULL;
    *count = 0;
}

const char *select_auto_tempdir(char buf[PATH_MAX])
{
    const char *tmp_list[] = { "/mnt/vendor", "/mnt", "/debug_ramdisk" };
    size_t n = sizeof(tmp_list) / sizeof(tmp_list[0]);

    for (size_t i = 0; i < n; ++i) {
        const char *base = tmp_list[i];
        if (!is_rw_tmpfs(base))
            continue;
        if (path_join(base, ".magic_mount", buf, PATH_MAX) != 0)
            continue;

        LOGI("auto tempdir selected: %s (from %s)", buf, base);
        return buf;
    }

    LOGW("no rw tmpfs found in candidates, fallback to %s",
         DEFAULT_TEMP_DIR);
    return DEFAULT_TEMP_DIR;
}

int set_selinux(const char *path, const char *con)
{
    if (!path || !con) {
        LOGD("set_selinux called with path=%p, con=%p (skip)",
             (void *)path, (void *)con);
        return 0;
    }

    LOGD("set_selinux(%s, \"%s\")", path, con);

    if (lsetxattr(path, SELINUX_XATTR, con, strlen(con), 0) < 0) {
        LOGW("setcon %s: %s", path, strerror(errno));
        return -1;
    }

    LOGD("set_selinux(%s) ok", path);
    return 0;
}

int get_selinux(const char *path, char **out)
{
    *out = NULL;
    if (!path) {
        LOGD("get_selinux called with NULL path");
        errno = EINVAL;
        return -1;
    }

    LOGD("get_selinux(%s)", path);

    ssize_t sz = lgetxattr(path, SELINUX_XATTR, NULL, 0);
    if (sz < 0) {
        LOGW("getcon %s: %s", path, strerror(errno));
        return -1;
    }

    char *buf = malloc((size_t)sz + 1);
    if (!buf) {
        errno = ENOMEM;
        return -1;
    }

    ssize_t r = lgetxattr(path, SELINUX_XATTR, buf, (size_t)sz);
    if (r < 0) {
        LOGW("getcon %s (read): %s", path, strerror(errno));
        free(buf);
        return -1;
    }
    buf[r] = '\0';
    *out = buf;

    LOGD("get_selinux(%s) -> \"%s\"", path, buf);
    return 0;
}
