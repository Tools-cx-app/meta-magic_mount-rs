<div align="center">

![An implementation of a metamodule](./mmrs.svg)

# Magic Mount Metamodule

[![Telegram][telegram-badge]][telegram-url]

</div>

[telegram-badge]: https://img.shields.io/badge/Group-blue?style=for-the-badge&logo=telegram&label=Telegram
[telegram-url]: https://t.me/mmrs_ci

Provide systemless mount capabilities for KernelSU.

The author will update this project less frequently due to academic commitments.

---

## Custom Lua hooks

Lua 5.4 hooks replace the old text DSL and declarative rules. **Breaking change:**
`/data/adb/magic_mount/custom` and `/data/adb/magic_mount/custom.lua` are no longer supported or read.
Manually convert old rules into hooks and move scripts into `/data/adb/magic_mount/lua/` before rebooting.
The WebUI no longer manages ignore lists or custom binds and never edits scripts.

All direct regular `.lua` files in `/data/adb/magic_mount/lua/` are loaded in ascending filename order
(for example, `10-skip.lua`, then `20-bind.lua`). Subdirectories and symlink files are not scanned or loaded.
A missing directory means no hooks; directory, read, and script-load errors abort instead of being ignored.
Each script has its own Lua VM: globals do not leak between scripts. Either hook may be omitted.
For example, create `/data/adb/magic_mount/lua/10-custom.lua`:

```lua
function before()
    if modules["example"] then
        modules["example"].skip = true
    end
end

function after()
    bind("/data/local/tmp/source file", "/system/etc/hosts")
end
```

All `before()` hooks run in filename order before module collection and mounting. `modules` is keyed by
module ID; each entry exposes `id`, `name`, `version`, `author`, `description`, `mode`, `is_mounted`,
`enabled`, `source_path`, `rules` (`default_mode` and `paths`), and a mutable `skip` boolean.
If any script sets `skip = true`, that module is excluded; another script cannot reenable it with `false`.
Setting `skip = false` also cannot override `disable`, `remove`, or `skip_mount` markers.
`moduledir` is the modules root directory (`/data/adb/modules`).
All `after()` hooks run in filename order only after successful Magic Mount. They expose only
`bind(source, target)` as a mount API, with no `ignore` or module-skip API. Queued binds are applied only
after every `after()` hook succeeds. Errors identify the script path and abort the current stage:
load, syntax, or `before` errors stop before mounting; an `after` error cancels all queued binds but
does not roll back completed Magic Mount operations. Paths must be nonempty and contain no control
characters. Scripts are trusted local configuration, with no execution-time or memory quota.

The `modules` command and WebUI restore `is_mounted`, `mode`, and `rules` from the startup snapshot,
matched by module ID. Lua-skipped modules and modules added after that snapshot show as unmounted.
Names, versions, and enabled flags still reflect current module files; changing them does not change
the mount state recorded for this boot.

Lua has safe base, table, string, math, and UTF-8 libraries; `os`, `io`, `package`, `debug`, and
file loaders are unavailable. Bind targets must be absolute and contain no `..` components. If a
target is missing, a temporary mirror is built from the nearest existing non-root ancestor. Binds
remain read-only; a remount or registration failure detaches the mount and rolls the operation back.

## Configuration

Configuration file path:

`/data/adb/magic_mount/config.toml`

Example:

```toml
mountsource = "KSU"
umount = false
partitions = []
extra_mount = []
```

| Field | Description |
| ------------- | -------------- |
| `mountsource` | Identifier for the Systemless mount source. Default is `"KSU"` to match KernelSU behavior. |
| `umount` | Whether to attempt unmount (depends on KernelSU's umount). |
| `partitions` | A list of specific partitions to perform Systemless operations on, e.g. `"mi_ext"`, `"my_stock"`. |
| `tmpfsdir` | Temporary directory, default is `/debug_ramdisk`. This option is optional. |

`extra_mount` defaults to `[]`. For example, `extra_mount = ["my_product"]` merges `<module>/my_product/` into `/my_product/`, rather than reading `<module>/system/my_product/`. The target directory must exist and `/system/my_product` must have no entry, including a dangling symlink. Use single-level partition names, not paths; invalid entries are logged and skipped. Modules without `system/` are supported; `disable`, `remove`, and `skip_mount` still apply. Edit the configuration file and reboot to apply changes.

Settings can also be changed on the Web UI configuration page (recommended). Both MiuiX and MD3 provide a “Top-level Partition Mounts” editor to add and remove `extra_mount` partition names. Save and reboot to apply changes.

---
## Directory markers

Create an empty marker file in the corresponding module directory and reboot:

- `.replace`: replace the target directory with the module contents, without retaining its original contents.
- `.remove`: hide the entire target directory and its contents. For example, `<module>/system/app/Foo/.remove` hides `/system/app/Foo`. Only directory nodes support this flag; regular files and symlinks do not. Original files are not deleted.

`.remove` takes precedence over `.replace` in the same directory; that directory's module contents are not mounted. Neither marker file is mounted. Directories under `extra_mount` use the same markers. The module-root `remove` marker (without a dot) still requests module removal and is distinct from `.remove`.


## Development

Dependencies:

* Rust nightly toolchain
* Android NDK
* `cargo-ndk`
* Node.js / npm
* `pnpm` and `vite` as dependency and frontend for webui

Environment variables:

```shell
export ANDROID_NDK_HOME=<path/to/ndk>
export ANDROID_NDK_ROOT=$ANDROID_NDK_HOME
```

Build:

```shell
cargo xtask b
```

Build artifacts will be located at:

* `output/magic_mount_rs.zip`

## Acknowledgements

* [5ec1cff/KernelSU](https://github.com/5ec1cff/KernelSU/blob/52f1f575ce2bd0ca46ebf644fd00a838af9f344e/userspace/ksud/src/magic_mount.rs): original implementation
* [YuzakiKokuban](https://github.com/YuzakiKokuban) Webui modifications

## License

[GPL-v3License](http://www.gnu.org/licenses/gpl.html)

## Star History

<a href="https://star-history.dera.page/#Tools-cx-app/meta-magic_mount-rs">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://star-history.dera.page/svg?repos=Tools-cx-app/meta-magic_mount-rs&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://star-history.dera.page/svg?repos=Tools-cx-app/meta-magic_mount-rs" />
   <img alt="Star History Chart" src="https://star-history.dera.page/svg?repos=Tools-cx-app/meta-magic_mount-rs" />
 </picture>
</a>
