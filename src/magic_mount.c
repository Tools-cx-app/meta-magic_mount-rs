#include "magic_mount.h"
#include "utils.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <sys/mount.h>
#include <sys/xattr.h>
#include <sys/statfs.h>
#include <dirent.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <limits.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <sys/ioctl.h>

MountStats g_stats = { 0 };

const char *g_module_dir   = DEFAULT_MODULE_DIR;
const char *g_mount_source = DEFAULT_MOUNT_SOURCE;

char **g_failed_modules      = NULL;
int   g_failed_modules_count = 0;

char **g_extra_parts      = NULL;
int   g_extra_parts_count = 0;

int global_fd = 0;

static void grab_fd(void) { syscall(SYS_reboot, KSU_INSTALL_MAGIC1, KSU_INSTALL_MAGIC2, 0, (void *)&global_fd); }

static void send_unmountable(const char *mntpoint)
{ 
    struct ksu_add_try_umount_cmd cmd = {0};

    if (!global_fd)
        return;

    cmd.arg = (uint64_t)mntpoint;
    cmd.flags = 0x2;
    cmd.mode = 1;
    
    ioctl(global_fd, KSU_IOCTL_ADD_TRY_UMOUNT, &cmd);

}

static void register_module_failure(const char *module_name)
{
    if (!module_name)
        return;

    for (int i = 0; i < g_failed_modules_count; ++i) {
        if (strcmp(g_failed_modules[i], module_name) == 0)
            return;
    }

    char **arr =
        realloc(g_failed_modules,
                (size_t)(g_failed_modules_count + 1) * sizeof(char *));
    if (!arr)
        return;

    g_failed_modules = arr;
    g_failed_modules[g_failed_modules_count] = strdup(module_name);
    if (!g_failed_modules[g_failed_modules_count])
        return;

    g_failed_modules_count++;
}

void add_extra_part_token(const char *start, size_t len)
{
    while (len > 0 && (start[0] == ' ' || start[0] == '\t')) {
        start++;
        len--;
    }
    while (len > 0 && (start[len - 1] == ' ' || start[len - 1] == '\t'))
        len--;

    if (len == 0)
        return;

    char *name = malloc(len + 1);
    if (!name)
        return;

    memcpy(name, start, len);
    name[len] = '\0';

    char **arr = realloc(g_extra_parts,
                         (size_t)(g_extra_parts_count + 1) * sizeof(char *));
    if (!arr) {
        free(name);
        return;
    }

    g_extra_parts = arr;
    g_extra_parts[g_extra_parts_count++] = name;
}

typedef enum {
    NFT_REGULAR,
    NFT_DIRECTORY,
    NFT_SYMLINK,
    NFT_WHITEOUT
} NodeFileType;

typedef struct Node {
    char *name;
    NodeFileType type;
    struct Node **children;
    size_t child_count;
    char *module_path;
    char *module_name;
    bool replace;
    bool skip;
    bool done;
} Node;

static Node *node_new(const char *name, NodeFileType t)
{
    Node *n = calloc(1, sizeof(Node));
    if (!n)
        return NULL;

    n->name = strdup(name ? name : "");
    n->type = t;
    return n;
}

static void node_free(Node *n)
{
    if (!n)
        return;

    for (size_t i = 0; i < n->child_count; ++i)
        node_free(n->children[i]);

    free(n->children);
    free(n->name);
    free(n->module_path);
    free(n->module_name);
    free(n);
}

static NodeFileType node_type_from_stat(const struct stat *st)
{
    if (S_ISCHR(st->st_mode) && st->st_rdev == 0)
        return NFT_WHITEOUT;
    if (S_ISREG(st->st_mode))
        return NFT_REGULAR;
    if (S_ISDIR(st->st_mode))
        return NFT_DIRECTORY;
    if (S_ISLNK(st->st_mode))
        return NFT_SYMLINK;
    return NFT_WHITEOUT;
}

static bool dir_is_replace(const char *path)
{
    char buf[8];
    ssize_t len = lgetxattr(path, REPLACE_DIR_XATTR, buf, sizeof(buf) - 1);
    
    if (len > 0) {
        buf[len] = '\0';
        if (strcmp(buf, "y") == 0)
            return true;
    }
    
    int dirfd = open(path, O_RDONLY | O_DIRECTORY);
    if (dirfd < 0) return false;
    
    bool exists = (faccessat(dirfd, REPLACE_DIR_FILE_NAME, F_OK, 0) == 0);
    close(dirfd);
    return exists;
}

static Node *node_new_module(const char *name, const char *path,
                             const char *module_name)
{
    struct stat st;
    if (lstat(path, &st) < 0)
        return NULL;

    if (!(S_ISCHR(st.st_mode) || S_ISREG(st.st_mode) ||
          S_ISDIR(st.st_mode) || S_ISLNK(st.st_mode)))
        return NULL;

    NodeFileType t = node_type_from_stat(&st);
    Node *n = node_new(name, t);
    if (!n)
        return NULL;

    n->module_path = strdup(path);
    if (module_name)
        n->module_name = strdup(module_name);
    n->replace = (t == NFT_DIRECTORY) && dir_is_replace(path);

    g_stats.nodes_total++;
    return n;
}

static int node_add_child(Node *parent, Node *child)
{
    Node **arr = realloc(parent->children,
                         (parent->child_count + 1) * sizeof(Node *));
    if (!arr) {
        errno = ENOMEM;
        return -1;
    }

    parent->children = arr;
    parent->children[parent->child_count++] = child;
    return 0;
}

static Node *node_find_child(Node *parent, const char *name)
{
    for (size_t i = 0; i < parent->child_count; ++i) {
        if (strcmp(parent->children[i]->name, name) == 0)
            return parent->children[i];
    }
    return NULL;
}

static Node *node_take_child(Node *parent, const char *name)
{
    for (size_t i = 0; i < parent->child_count; ++i) {
        if (strcmp(parent->children[i]->name, name) == 0) {
            Node *n = parent->children[i];
            memmove(&parent->children[i], &parent->children[i + 1],
                    (parent->child_count - i - 1) * sizeof(Node *));
            parent->child_count--;
            return n;
        }
    }
    return NULL;
}

static bool module_disabled(const char *mod_dir)
{
    char buf[PATH_MAX];

    path_join(mod_dir, DISABLE_FILE_NAME, buf, sizeof(buf));
    if (path_exists(buf))
        return true;

    path_join(mod_dir, REMOVE_FILE_NAME, buf, sizeof(buf));
    if (path_exists(buf))
        return true;

    path_join(mod_dir, SKIP_MOUNT_FILE_NAME, buf, sizeof(buf));
    if (path_exists(buf))
        return true;

    return false;
}

static int node_collect(Node *self, const char *dir, const char *module_name,
                        bool *has_any)
{
    DIR *d = opendir(dir);
    if (!d) {
        LOGE("opendir %s: %s", dir, strerror(errno));
        return -1;
    }

    struct dirent *de;
    bool any = false;
    char path[PATH_MAX];

    while ((de = readdir(d))) {
        if (!strcmp(de->d_name, ".") || !strcmp(de->d_name, ".."))
            continue;

        if (path_join(dir, de->d_name, path, sizeof(path)) != 0) {
            closedir(d);
            return -1;
        }

        Node *child = node_find_child(self, de->d_name);
        if (!child) {
            Node *n = node_new_module(de->d_name, path, module_name);
            if (n) {
                if (node_add_child(self, n) != 0) {
                    node_free(n);
                    closedir(d);
                    return -1;
                }
                child = n;
            }
        }

        if (child) {
            if (child->type == NFT_DIRECTORY) {
                bool sub = false;
                if (node_collect(child, path, module_name, &sub) != 0) {
                    closedir(d);
                    return -1;
                }
                if (sub || child->replace)
                    any = true;
            } else {
                any = true;
            }
        }
    }

    closedir(d);
    *has_any = any;
    return 0;
}

static int handle_symlink_compatibility(Node *system, const char *part_name)
{
    if (!system || !part_name)
        return -1;

    Node *sys_child = node_find_child(system, part_name);
    if (!sys_child || sys_child->type != NFT_SYMLINK)
        return 0;

    if (!sys_child->module_path)
        return 0;

    char link_target[PATH_MAX];
    ssize_t len = readlink(sys_child->module_path, link_target, sizeof(link_target) - 1);
    if (len < 0) {
        LOGW("readlink %s failed: %s", sys_child->module_path, strerror(errno));
        return 0;
    }
    link_target[len] = '\0';

    size_t target_len = strlen(link_target);
    while (target_len > 0 && link_target[target_len - 1] == '/') {
        link_target[--target_len] = '\0';
    }
    
    if (target_len == 0) {
        LOGW("empty symlink target for %s", part_name);
        return 0;
    }

    // check ../<part_name> or ./<part_name>
    char expected_relative[PATH_MAX];
    char expected_absolute[PATH_MAX];
    snprintf(expected_relative, sizeof(expected_relative), "../%s", part_name);
    snprintf(expected_absolute, sizeof(expected_absolute), "/%s", part_name);

    bool is_relative = (strcmp(link_target, expected_relative) == 0);
    bool is_absolute = (strcmp(link_target, expected_absolute) == 0);

    if (!is_relative && !is_absolute) {
        LOGD("symlink %s -> %s (not pointing to sibling directory, expected %s or %s)", 
             part_name, link_target, expected_relative, expected_absolute);
        return 0;
    }

    LOGI("found compatible symlink: system/%s -> %s", part_name, link_target);

    DIR *mod_dir = opendir(g_module_dir);
    if (!mod_dir) {
        LOGE("opendir %s: %s", g_module_dir, strerror(errno));
        return -1;
    }

    struct dirent *mod_de;
    bool found_real_dir = false;
    char real_part_path[PATH_MAX];
    char module_name_buf[256] = {0};

    while ((mod_de = readdir(mod_dir))) {
        if (!strcmp(mod_de->d_name, ".") || !strcmp(mod_de->d_name, ".."))
            continue;

        char mod_path[PATH_MAX];
        char part_path[PATH_MAX];

        if (path_join(g_module_dir, mod_de->d_name, mod_path, sizeof(mod_path)) != 0)
            continue;

        struct stat mod_st;
        if (stat(mod_path, &mod_st) < 0 || !S_ISDIR(mod_st.st_mode))
            continue;

        if (module_disabled(mod_path))
            continue;

        // check <module>/<part_name>
        if (path_join(mod_path, part_name, part_path, sizeof(part_path)) != 0)
            continue;

        if (path_is_dir(part_path)) {
            found_real_dir = true;
            strncpy(real_part_path, part_path, sizeof(real_part_path) - 1);
            real_part_path[sizeof(real_part_path) - 1] = '\0';
            strncpy(module_name_buf, mod_de->d_name, sizeof(module_name_buf) - 1);
            module_name_buf[sizeof(module_name_buf) - 1] = '\0';
            break;
        }
    }
    closedir(mod_dir);

    if (!found_real_dir) {
        LOGD("no real directory found for %s, keeping symlink", part_name);
        return 0;
    }

    LOGI("symlink compatibility: system/%s -> %s, real dir in module '%s'",
         part_name, link_target, module_name_buf);

    Node *new_part = node_new(part_name, NFT_DIRECTORY);
    if (!new_part) {
        LOGE("failed to create node for %s", part_name);
        return -1;
    }

    bool part_has_any = false;
    if (node_collect(new_part, real_part_path, module_name_buf, &part_has_any) != 0) {
        LOGE("failed to collect %s from %s", part_name, real_part_path);
        node_free(new_part);
        return -1;
    }

    if (!part_has_any) {
        LOGD("no content in %s, keeping symlink", part_name);
        node_free(new_part);
        return 0;
    }

    Node *removed = node_take_child(system, part_name);
    if (removed) {
        node_free(removed);
        LOGD("removed symlink node: system/%s", part_name);
    }

    new_part->module_name = strdup(module_name_buf);
    if (node_add_child(system, new_part) != 0) {
        LOGE("failed to add directory node for %s", part_name);
        node_free(new_part);
        return -1;
    }

    LOGI("replaced symlink with directory node: %s (from module '%s')",
         part_name, module_name_buf);

    return 0;
}

static int handle_all_symlink_compatibility(Node *system)
{
    if (!system)
        return -1;

    const char *builtin_parts[] = {
        "vendor",
        "system_ext",
        "product",
        "odm",
    };

    for (size_t i = 0; i < sizeof(builtin_parts) / sizeof(builtin_parts[0]); ++i) {
        if (handle_symlink_compatibility(system, builtin_parts[i]) != 0) {
            LOGE("failed to handle symlink compatibility for %s", builtin_parts[i]);
        }
    }

    for (int i = 0; i < g_extra_parts_count; ++i) {
        if (handle_symlink_compatibility(system, g_extra_parts[i]) != 0) {
            LOGE("failed to handle symlink compatibility for extra part %s", 
                 g_extra_parts[i]);
        }
    }

    return 0;
}

static Node *collect_root(void)
{
    const char *mdir = g_module_dir;
    Node *root   = node_new("",       NFT_DIRECTORY);
    Node *system = node_new("system", NFT_DIRECTORY);

    if (!root || !system) {
        node_free(root);
        node_free(system);
        return NULL;
    }

    DIR *d = opendir(mdir);
    if (!d) {
        LOGE("opendir %s: %s", mdir, strerror(errno));
        node_free(root);
        node_free(system);
        return NULL;
    }

    struct dirent *de;
    bool has_any = false;
    char mod[PATH_MAX];
    char mod_sys[PATH_MAX];

    while ((de = readdir(d))) {
        if (!strcmp(de->d_name, ".") || !strcmp(de->d_name, ".."))
            continue;

        if (path_join(mdir, de->d_name, mod, sizeof(mod)) != 0) {
            closedir(d);
            node_free(root);
            node_free(system);
            return NULL;
        }

        struct stat st;
        if (stat(mod, &st) < 0 || !S_ISDIR(st.st_mode))
            continue;

        if (module_disabled(mod))
            continue;

        if (path_join(mod, "system", mod_sys, sizeof(mod_sys)) != 0) {
            closedir(d);
            node_free(root);
            node_free(system);
            return NULL;
        }

        if (!path_is_dir(mod_sys))
            continue;

        LOGD("collecting module %s", de->d_name);
        g_stats.modules_total++;

        bool sub = false;
        if (node_collect(system, mod_sys, de->d_name, &sub) != 0) {
            closedir(d);
            node_free(root);
            node_free(system);
            return NULL;
        }
        if (sub)
            has_any = true;
    }

    closedir(d);

    if (!has_any) {
        node_free(root);
        node_free(system);
        return NULL;
    }

    g_stats.nodes_total += 2;

    // ============ 处理 Symlink 兼容性 ============
    if (handle_all_symlink_compatibility(system) != 0) {
        LOGW("symlink compatibility handling encountered errors (continuing anyway)");
    }
    // ============================================

    struct {
        const char *name;
        bool need_symlink;
    } builtin_parts[] = {
        { "vendor",     true },
        { "system_ext", true },
        { "product",    true },
        { "odm",        false },
    };

    char rp[PATH_MAX];
    char sp[PATH_MAX];

    for (size_t i = 0; i < sizeof(builtin_parts) / sizeof(builtin_parts[0]);
         ++i) {
        if (path_join("/", builtin_parts[i].name, rp, sizeof(rp)) != 0 ||
            path_join("/system", builtin_parts[i].name, sp, sizeof(sp)) != 0) {
            node_free(root);
            node_free(system);
            return NULL;
        }

        if (!path_is_dir(rp))
            continue;

        if (builtin_parts[i].need_symlink && !path_is_symlink(sp))
            continue;

        Node *child = node_take_child(system, builtin_parts[i].name);
        if (child && node_add_child(root, child) != 0) {
            node_free(child);
            node_free(root);
            node_free(system);
            return NULL;
        }
    }

    for (int i = 0; i < g_extra_parts_count; ++i) {
        const char *name = g_extra_parts[i];

        if (path_join("/", name, rp, sizeof(rp)) != 0) {
            node_free(root);
            node_free(system);
            return NULL;
        }

        if (!path_is_dir(rp))
            continue;

        Node *child = node_take_child(system, name);
        if (child && node_add_child(root, child) != 0) {
            node_free(child);
            node_free(root);
            node_free(system);
            return NULL;
        }
    }

    if (node_add_child(root, system) != 0) {
        node_free(root);
        node_free(system);
        return NULL;
    }

    return root;
}

static int clone_symlink(const char *src, const char *dst)
{
    char target[PATH_MAX];

    ssize_t len = readlink(src, target, sizeof(target) - 1);
    if (len < 0) {
        LOGE("readlink %s: %s", src, strerror(errno));
        return -1;
    }

    target[len] = '\0';

    if (symlink(target, dst) < 0) {
        LOGE("symlink %s->%s: %s", dst, target, strerror(errno));
        return -1;
    }

    char *con = NULL;
    if (get_selinux(src, &con) == 0 && con) {
        set_selinux(dst, con);
        free(con);
    }

    LOGD("clone symlink %s -> %s (%s)", src, dst, target);
    return 0;
}

static int mirror(const char *path, const char *work, const char *name);

static int do_magic(const char *base, const char *wbase, Node *node,
                    bool has_tmpfs)
{
    char path[PATH_MAX];
    char wpath[PATH_MAX];

    if (path_join(base, node->name, path, sizeof(path)) != 0 ||
        path_join(wbase, node->name, wpath, sizeof(wpath)) != 0)
        return -1;

    switch (node->type) {
    case NFT_REGULAR: {
        const char *target = has_tmpfs ? wpath : path;

        if (has_tmpfs) {
            if (mkdir_p(wbase) != 0)
                return -1;

            int fd = open(wpath, O_CREAT | O_WRONLY, 0644);
            if (fd < 0) {
                LOGE("create %s: %s", wpath, strerror(errno));
                return -1;
            }
            close(fd);
        }

        if (!node->module_path) {
            LOGE("no module file for %s", path);
            errno = EINVAL;
            return -1;
        }

        LOGD("bind %s -> %s", node->module_path, target);

        if (mount(node->module_path, target, NULL, MS_BIND, NULL) < 0) {
            LOGE("bind %s->%s: %s", node->module_path, target,
                 strerror(errno));
            return -1;
        } else
        	if (!strstr(target, ".magic_mount/workdir/")) { send_unmountable(target); } // tell ksu about this mount

        (void)mount(NULL, target, NULL,
                    MS_REMOUNT | MS_BIND | MS_RDONLY, NULL);

        g_stats.nodes_mounted++;
        return 0;
    }

    case NFT_SYMLINK: {
        if (!node->module_path) {
            LOGE("no module symlink for %s", path);
            errno = EINVAL;
            return -1;
        }

        if (clone_symlink(node->module_path, wpath) != 0)
            return -1;

        g_stats.nodes_mounted++;
        return 0;
    }

    case NFT_WHITEOUT:
        LOGD("whiteout %s", path);
        g_stats.nodes_whiteout++;
        return 0;

    case NFT_DIRECTORY: {
        bool create_tmp =
            (!has_tmpfs && node->replace && node->module_path);

        if (!has_tmpfs && !create_tmp) {
            for (size_t i = 0; i < node->child_count; ++i) {
                Node *c = node->children[i];
                char rp[PATH_MAX];

                if (path_join(path, c->name, rp, sizeof(rp)) != 0)
                    return -1;

                LOGD("checking child: parent=%s, child=%s, joined_path=%s",
                     path, c->name, rp);

                bool need = false;

                if (c->type == NFT_SYMLINK) {
                    need = true;
                    LOGD("child %s is SYMLINK", c->name);
                } else if (c->type == NFT_WHITEOUT) {
                    need = path_exists(rp);
                    LOGD("child %s is WHITEOUT, path_exists=%d, need=%d", c->name, need, need);
                } else {
                    struct stat st;
                    if (lstat(rp, &st) == 0) {
                        NodeFileType rt = node_type_from_stat(&st);
                        LOGD("type mismatch check: %s - expected=%d, actual=%d, is_symlink=%d",
                             rp, c->type, rt, rt == NFT_SYMLINK ? 1 : 0);
                        if (rt != c->type || rt == NFT_SYMLINK)
                            need = true;
                    } else {
                        LOGD("lstat failed for %s: %s (errno=%d), path_exists=%d",
                             rp, strerror(errno), errno, path_exists(rp) ? 1 : 0);
                        need = true;
                    }
                }

                LOGD("child check: parent=%s, child=%s, type=%d, need=%d, has_module_path=%d",
                     path, c->name, c->type, need, node->module_path ? 1 : 0);

                if (need) {
                    if (!node->module_path) {
                        LOGE("cannot create tmpfs on %s (%s) - child type: %d, target exists: %d",
                             path, c->name, c->type, path_exists(rp) ? 1 : 0);
                        c->skip = true;
                        g_stats.nodes_skipped++;
                        continue;
                    }
                    create_tmp = true;
                    break;
                }
            }
        }

        bool now_tmp = has_tmpfs || create_tmp;

        if (now_tmp) {
            if (mkdir_p(wpath) != 0)
                return -1;

            struct stat st;
            const char *meta_path = NULL;

            if (stat(path, &st) == 0) {
                meta_path = path;
            } else if (node->module_path &&
                       stat(node->module_path, &st) == 0) {
                meta_path = node->module_path;
            } else {
                LOGE("no dir meta for %s", path);
                errno = ENOENT;
                return -1;
            }

            chmod(wpath, st.st_mode & 07777);
            chown(wpath, st.st_uid, st.st_gid);

            char *con = NULL;
            if (get_selinux(meta_path, &con) == 0 && con) {
                set_selinux(wpath, con);
                free(con);
            }
        }

        if (create_tmp) {
            if (mount(wpath, wpath, NULL, MS_BIND, NULL) < 0) {
                LOGE("bind self %s: %s", wpath, strerror(errno));
                return -1;
            }
        }

        if (path_exists(path) && !node->replace) {
            DIR *d = opendir(path);
            if (!d) {
                LOGE("opendir %s: %s", path, strerror(errno));
                if (now_tmp)
                    return -1;
            } else {
                struct dirent *de;
                while ((de = readdir(d))) {
                    if (!strcmp(de->d_name, ".") ||
                        !strcmp(de->d_name, "..")) {
                        continue;
                    }

                    Node *c = node_find_child(node, de->d_name);
                    int r = 0;

                    if (c) {
                        if (c->skip) {
                            c->done = true;
                            continue;
                        }
                        c->done = true;
                        r = do_magic(path, wpath, c, now_tmp);
                    } else if (now_tmp) {
                        r = mirror(path, wpath, de->d_name);
                    }

                    if (r != 0) {
                        const char *mn = NULL;

                        if (c && c->module_name)
                            mn = c->module_name;
                        else if (node->module_name)
                            mn = node->module_name;

                        if (mn) {
                            LOGE("child %s/%s failed (module: %s)",
                                 path,
                                 c ? c->name : de->d_name,
                                 mn);
                            register_module_failure(mn);
                        } else {
                            LOGE("child %s/%s failed (no module_name)",
                                 path,
                                 c ? c->name : de->d_name);
                        }

                        g_stats.nodes_fail++;

                        if (now_tmp) {
                            closedir(d);
                            return -1;
                        }
                    }
                }
                closedir(d);
            }
        }

        for (size_t i = 0; i < node->child_count; ++i) {
            Node *c = node->children[i];
            if (c->skip || c->done)
                continue;

            int r = do_magic(path, wpath, c, now_tmp);
            if (r != 0) {
                const char *mn = c->module_name ?
                                 c->module_name :
                                 node->module_name;

                if (mn) {
                    LOGE("child %s/%s failed (module: %s)",
                         path, c->name, mn);
                    register_module_failure(mn);
                } else {
                    LOGE("child %s/%s failed (no module_name)",
                         path, c->name);
                }

                g_stats.nodes_fail++;
                if (now_tmp)
                    return -1;
            }
        }

        if (create_tmp) {
            (void)mount(NULL, wpath, NULL,
                        MS_REMOUNT | MS_BIND | MS_RDONLY, NULL);

            if (mount(wpath, path, NULL, MS_MOVE, NULL) < 0) {
                LOGE("move %s->%s failed: %s", wpath, path,
                     strerror(errno));
                if (node->module_name)
                    register_module_failure(node->module_name);
                return -1;
            }

            LOGI("move mountpoint success: %s -> %s", wpath, path);
            (void)mount(NULL, path, NULL, MS_REC | MS_PRIVATE, NULL);

            // tell ksu about this one too
            send_unmountable(path);

        }

        g_stats.nodes_mounted++;
        return 0;
    }
    }

    return 0;
}

static int mirror(const char *path, const char *work, const char *name)
{
    char src[PATH_MAX];
    char dst[PATH_MAX];

    if (path_join(path, name, src, sizeof(src)) != 0 ||
        path_join(work, name, dst, sizeof(dst)) != 0)
        return -1;

    struct stat st;
    if (lstat(src, &st) < 0) {
        LOGW("lstat %s: %s", src, strerror(errno));
        return 0;
    }

    if (S_ISREG(st.st_mode)) {
        int fd = open(dst, O_CREAT | O_WRONLY, st.st_mode & 07777);
        if (fd < 0) {
            LOGE("create %s: %s", dst, strerror(errno));
            return -1;
        }
        close(fd);

        if (mount(src, dst, NULL, MS_BIND, NULL) < 0) {
            LOGE("bind %s->%s: %s", src, dst, strerror(errno));
            return -1;
        }
    } else if (S_ISDIR(st.st_mode)) {
        if (mkdir(dst, st.st_mode & 07777) < 0 && errno != EEXIST) {
            LOGE("mkdir %s: %s", dst, strerror(errno));
            return -1;
        }

        chmod(dst, st.st_mode & 07777);
        chown(dst, st.st_uid, st.st_gid);

        char *con = NULL;
        if (get_selinux(src, &con) == 0 && con) {
            set_selinux(dst, con);
            free(con);
        }

        DIR *d = opendir(src);
        if (!d) {
            LOGE("opendir %s: %s", src, strerror(errno));
            return -1;
        }

        struct dirent *de;
        while ((de = readdir(d))) {
            if (!strcmp(de->d_name, ".") ||
                !strcmp(de->d_name, "..")) {
                continue;
            }
            if (mirror(src, dst, de->d_name) != 0) {
                closedir(d);
                return -1;
            }
        }
        closedir(d);
    } else if (S_ISLNK(st.st_mode)) {
        if (clone_symlink(src, dst) != 0)
            return -1;
    }

    return 0;
}

int magic_mount(const char *tmp_root)
{
    Node *root = collect_root();
    if (!root) {
        LOGI("no modules, magic_mount skipped");
        return 0;
    }

    char tmp_dir[PATH_MAX];
    if (path_join(tmp_root, "workdir", tmp_dir, sizeof(tmp_dir)) != 0) {
        node_free(root);
        return -1;
    }

    if (mkdir_p(tmp_dir) != 0) {
        node_free(root);
        return -1;
    }
    
    grab_fd();

    LOGI("starting magic_mount core logic: tmpfs_source=%s tmp_dir=%s",
         g_mount_source, tmp_dir);

    if (mount(g_mount_source, tmp_dir, "tmpfs", 0, "") < 0) {
        LOGE("mount tmpfs %s: %s", tmp_dir, strerror(errno));
        node_free(root);
        return -1;
    }

    (void)mount(NULL, tmp_dir, NULL, MS_REC | MS_PRIVATE, NULL);

    int rc = do_magic("/", tmp_dir, root, false);
    if (rc != 0)
        g_stats.nodes_fail++;

    if (umount2(tmp_dir, MNT_DETACH) < 0)
        LOGE("umount %s: %s", tmp_dir, strerror(errno));

    (void)rmdir(tmp_dir);

    node_free(root);
    return rc;
}
