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

## Custom rules

Rules file: `/data/adb/magic_mount/custom`

```text
# Ignore a source file from a module
ignore /data/adb/modules/example/system/app/Example.apk

# Read-only bind a source to a target
bind "/data/local/tmp/source file" "/system/etc/target file"

# Recursively include other rule files; add and file are equivalent
file /data/adb/magic_mount/extra.rules
add /data/adb/magic_mount/more.rules
```

Rules support single quotes, double quotes, paths containing whitespace, and trailing comments.
Control characters are removed. Recursive includes use a per-parse visited set, so include cycles do
not recurse indefinitely.

A custom bind target must be absolute and cannot contain `..`. When the target is missing, the tool
builds a temporary mirror from the nearest existing non-root ancestor. Any remount or registration
failure detaches the mount and rolls the operation back.

## Configuration

Configuration file path:

`/data/adb/magic_mount/config.toml`

Example:

```toml
mountsource = "KSU"
umount = false
partitions = []
```

| Field | Description |
| ------------- | -------------- |
| `mountsource` | Identifier for the Systemless mount source. Default is `"KSU"` to match KernelSU behavior. |
| `umount` | Whether to attempt unmount (depends on KernelSU's umount). |
| `partitions` | A list of specific partitions to perform Systemless operations on, e.g. `"mi_ext"`, `"my_stock"`. |
| `tmpfsdir` | Temporary directory, default is `/debug_ramdisk`. This option is optional. |

Configuration can also be performed via the Web UI (recommended).

---

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
