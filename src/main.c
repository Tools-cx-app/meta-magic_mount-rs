#include "magic_mount.h"
#include "utils.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <ctype.h>

#define DEFAULT_CONFIG_PATH "/data/adb/magic_mount/mm.conf"

static void usage(const char *prog)
{
    fprintf(stderr,
            "Magic Mount Version: %s\n"
            "Usage: %s [options]\n"
            "  -m DIR     module dir (default: %s)\n"
            "  -t DIR     temp dir   (default: auto rw tmpfs + \".magic_mount\")\n"
            "  -s SRC     mount src  (default: %s)\n"
            "  -p LIST    extra partitions, comma-separated (e.g. mi_ext,my_stock)\n"
            "  -l FILE    log file   (default: stderr, '-'=stdout)\n"
            "  -c FILE    config file (default: " DEFAULT_CONFIG_PATH ")\n"
            "  -v         enable debug logging (default: info)\n"
            "  -h         help\n",
            VERSION, prog, DEFAULT_MODULE_DIR,
            DEFAULT_MOUNT_SOURCE);
}

static int enter_pid1_ns(void)
{
    int fd = open("/proc/1/ns/mnt", O_RDONLY);
    if (fd < 0) {
        LOGE("open /proc/1/ns/mnt: %s", strerror(errno));
        return -1;
    }

    if (setns(fd, 0) < 0) {
        LOGE("setns: %s", strerror(errno));
        close(fd);
        return -1;
    }

    close(fd);
    LOGI("entered pid1 mnt ns");
    return 0;
}

static char *str_trim(char *s)
{
    if (!s)
        return s;

    char *start = s;
    while (*start && isspace((unsigned char)*start))
        start++;

    char *end = start + strlen(start);
    while (end > start && isspace((unsigned char)end[-1]))
        *--end = '\0';

    return start;
}

static bool str_is_true(const char *v)
{
    if (!v)
        return false;
    if (!strcasecmp(v, "1") ||
        !strcasecmp(v, "true") ||
        !strcasecmp(v, "yes") ||
        !strcasecmp(v, "on"))
        return true;
    return false;
}

static void load_config(const char *config_path,
                        const char **tmp_dir_out,
                        const char **log_path_out,
                        bool *debug_out)
{
    FILE *fp = fopen(config_path, "r");
    if (!fp) {
        LOGD("config file %s not used: %s",
             config_path, strerror(errno));
        return;
    }

    LOGI("loading config file: %s", config_path);

    char line[1024];
    while (fgets(line, sizeof(line), fp)) {
        char *p = line;

        while (*p && isspace((unsigned char)*p))
            p++;

        if (*p == '\0' || *p == '\n' || *p == '#')
            continue;

        char *eq = strchr(p, '=');
        if (!eq) {
            continue;
        }

        *eq = '\0';
        char *key = str_trim(p);
        char *val = str_trim(eq + 1);

        size_t vlen = strlen(val);
        if (vlen > 0 && (val[vlen - 1] == '\n' || val[vlen - 1] == '\r'))
            val[vlen - 1] = '\0';
        val = str_trim(val);

        if (*key == '\0' || *val == '\0')
            continue;

        if (!strcasecmp(key, "module_dir")) {
            char *dup = strdup(val);
            if (dup)
                g_module_dir = dup;
        } else if (!strcasecmp(key, "temp_dir")) {
            char *dup = strdup(val);
            if (dup && tmp_dir_out)
                *tmp_dir_out = dup;
        } else if (!strcasecmp(key, "mount_source")) {
            char *dup = strdup(val);
            if (dup)
                g_mount_source = dup;
        } else if (!strcasecmp(key, "partitions")) {
            const char *list = val;
            const char *q = list;
            while (*q) {
                const char *start = q;
                while (*q && *q != ',')
                    q++;
                size_t len = (size_t)(q - start);
                add_extra_part_token(start, len);
                if (*q == ',')
                    q++;
            }
        } else if (!strcasecmp(key, "log_file")) {
            char *dup = strdup(val);
            if (dup && log_path_out)
                *log_path_out = dup;
        } else if (!strcasecmp(key, "debug")) {
            if (debug_out && str_is_true(val))
                *debug_out = true;
        } else {
            LOGW("unknown config key: %s", key);
        }
    }

    fclose(fp);
}

int main(int argc, char **argv)
{
    const char *tmp_dir    = NULL;
    const char *log_path   = NULL;
    const char *config_path = DEFAULT_CONFIG_PATH;
    char auto_tmp[PATH_MAX] = { 0 };
    bool debug_from_cfg = false;

    for (int i = 1; i < argc; ++i) {
        const char *arg = argv[i];

        if ((strcmp(arg, "-c") == 0 || strcmp(arg, "--config") == 0) &&
            i + 1 < argc) {
            config_path = argv[++i];
        }
    }

    load_config(config_path, &tmp_dir, &log_path, &debug_from_cfg);

    if (debug_from_cfg) {
        g_log_level = LV_DEBUG;
    }

    for (int i = 1; i < argc; ++i) {
        const char *arg = argv[i];

        if ((strcmp(arg, "-c") == 0 || strcmp(arg, "--config") == 0) &&
            i + 1 < argc) {
            ++i;
            continue;
        }

        if ((strcmp(arg, "-m") == 0 ||
             strcmp(arg, "--module-dir") == 0) &&
            i + 1 < argc) {
            g_module_dir = argv[++i];

        } else if ((strcmp(arg, "-t") == 0 ||
                    strcmp(arg, "--temp-dir") == 0) &&
                   i + 1 < argc) {
            tmp_dir = argv[++i];

        } else if ((strcmp(arg, "-s") == 0 ||
                    strcmp(arg, "--mount-source") == 0) &&
                   i + 1 < argc) {
            g_mount_source = argv[++i];

        } else if ((strcmp(arg, "-l") == 0 ||
                    strcmp(arg, "--log-file") == 0) &&
                   i + 1 < argc) {
            log_path = argv[++i];

        } else if (!strcmp(arg, "-v") || !strcmp(arg, "--verbose")) {
            g_log_level = LV_DEBUG;

        } else if ((strcmp(arg, "-p") == 0 ||
                    strcmp(arg, "--partitions") == 0) &&
                   i + 1 < argc) {
            const char *list = argv[++i];
            const char *p = list;
            while (*p) {
                const char *start = p;
                while (*p && *p != ',')
                    p++;
                size_t len = (size_t)(p - start);
                add_extra_part_token(start, len);
                if (*p == ',')
                    p++;
            }

        } else if (!strcmp(arg, "-h") || !strcmp(arg, "--help")) {
            usage(argv[0]);
            return 0;

        } else {
            fprintf(stderr, "Unknown arg: %s\n", arg);
            usage(argv[0]);
            return 1;
        }
    }

    if (log_path) {
        if (!strcmp(log_path, "-")) {
            g_log_file = stdout;
        } else {
            g_log_file = fopen(log_path, "a");
            if (!g_log_file) {
                fprintf(stderr, "open log %s: %s\n", log_path,
                        strerror(errno));
                return 1;
            }
        }
    }

    if (!tmp_dir) {
        tmp_dir = select_auto_tempdir(auto_tmp);
    }

    if (geteuid() != 0) {
        LOGE("must run as root (euid=%d)", (int)geteuid());
        if (g_log_file && g_log_file != stdout && g_log_file != stderr)
            fclose(g_log_file);
        free_string_array(&g_extra_parts, &g_extra_parts_count);
        return 1;
    }

    if (enter_pid1_ns() != 0) {
        if (g_log_file && g_log_file != stdout && g_log_file != stderr)
            fclose(g_log_file);
        free_string_array(&g_extra_parts, &g_extra_parts_count);
        return 1;
    }

    int rc = 0;

    LOGI("starting magic_mount process: module_dir=%s temp_dir=%s "
         "mount_source=%s log_level=%d",
         g_module_dir, tmp_dir, g_mount_source, g_log_level);

    rc = magic_mount(tmp_dir);

    if (rc == 0) {
        LOGI("magic_mount completed successfully");
    } else {
        LOGE("magic_mount failed with rc=%d "
             "(check previous logs for details)",
             rc);
    }

    LOGI("magic_mount summary: modules=%d nodes_total=%d mounted=%d "
         "skipped=%d whiteouts=%d failures=%d",
         g_stats.modules_total, g_stats.nodes_total, g_stats.nodes_mounted,
         g_stats.nodes_skipped, g_stats.nodes_whiteout, g_stats.nodes_fail);

    if (g_failed_modules_count > 0) {
        LOGW("modules with failures (%d):", g_failed_modules_count);
        for (int i = 0; i < g_failed_modules_count; ++i) {
            LOGW("  failed module: %s", g_failed_modules[i]);
        }
    }

    if (g_log_file && g_log_file != stdout && g_log_file != stderr)
        fclose(g_log_file);

    free_string_array(&g_failed_modules, &g_failed_modules_count);
    free_string_array(&g_extra_parts, &g_extra_parts_count);

    return rc == 0 ? 0 : 1;
}
