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

Place Lua 5.4 scripts in `/data/adb/magic_mount/lua/` and reboot. Only direct regular `.lua` files are loaded, in filename order; symlinks and subdirectories are excluded. A missing directory means no scripts. Each script uses an independent Lua VM.

Start each file with `API = 1` or `API = 2`; API 2 must then declare `MODULE = "module ID"`. Each literal declaration occupies its own line; leading whitespace, Lua comments, and both quote styles are supported. API and MODULE are immutable. Migrate `bind` to `bind_mount` and `modules[id].skip` to API 2 `skip()`. Legacy custom/custom.lua files are not read; the WebUI does not edit scripts.

API 1 provides only the mount function `bind_mount(source, target)` inside optional `before()/after()` callbacks:

~~~lua
API = 1

function before()
    bind_mount("/data/local/tmp/before.conf", "/system/etc/before.conf")
end

function after()
    bind_mount("/data/local/tmp/hosts", "/system/etc/hosts")
end
~~~

All before callbacks enqueue binds in filename order. Once rules and the mount tree are prepared, those binds execute before Magic Mount, which may cover their targets. All after callbacks run only after successful Magic Mount; binds execute only if every after succeeds. Targets must be absolute. Empty paths, control characters, and `..` components are forbidden. Binds remain read-only; missing targets use a temporary mirror from the nearest existing non-root ancestor.

API 2 matches MODULE against the ID in module.prop, regardless of directory name. Before scanning module mount content, look up the ID in `LUAS: FxHashMap` and invoke `load()` with no arguments. Matching scripts run in filename order. Missing modules or modules with disable, remove, or skip_mount markers do not invoke load. Only API 2 load provides:

- `skip()`: skip all mount content scanning and mounting for this module. Cancel all its rules, ignore later rules, and stop later scripts for the same module. Takes no arguments and is idempotent; use `return` to leave the current Lua callback. No skip_mount file is created.
- `block(path)`: exclude only this module contribution at a file or directory path; original system content and other modules contributions remain.
- `replace(source, target)`: use an absolute regular file or directory as the source for a module-relative target. Directories replace the original contents. Source symlinks and special devices are rejected. The last declaration for a target wins.
- `remove(path)`: hide a file, symlink, or directory in the final mount view, including other modules contributions, without deleting disk contents. Partition roots cannot be removed.

~~~lua
API = 2
MODULE = "example_module"

function load()
    block("system/etc/unwanted.conf")
    replace("/data/local/tmp/hosts", "system/etc/hosts")
    remove("system/app/Example")
end
~~~

To skip the entire module, use `skip(); return` as the load body. Relative paths start at the module root and include `system/` or a configured extra_mount partition, such as `my_product/etc/example.xml`. Absolute targets, empty paths, control characters, and `..` are rejected. skip overrides all rules; otherwise remove > block > replace, with ancestor block/remove covering descendants. Modules without system/ can still contribute rules. Modules are ordered by ID; ordinary conflicts retain the first contribution. block/replace apply before module merging, remove applies to the final tree.

`moduledir` is the modules root (/data/adb/modules). Lua provides base, table, string, math, and utf8 libraries, without os/io/package/debug, file loaders, rawset, or setmetatable. Loading, before, load, or tree preparation errors stop before mounting. An after error cancels that stage binds without rolling back completed Magic Mount. Trusted local scripts have no time or memory quotas.

The modules command and WebUI restore is_mounted, mode, and rules from the startup snapshot by ID without executing scripts. Skipped modules remain listed as unmounted; skip applies only to this boot. The snapshot describes the startup plan, not a per-file mount success audit.

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
