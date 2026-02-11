<div align="center">

# Magic Mount Metamodule Fork

[![Telegram][telegram-badge]][telegram-url]

</div>

[telegram-badge]: https://img.shields.io/badge/Group-blue?style=for-the-badge&logo=telegram&label=Telegram
[telegram-url]: https://t.me/EdLSPesodITed

Provide systemless mount capabilities for KernelSU.

---

## Configuration

Configuration file path:

`/data/adb/magic_mount/config.toml`

Example:

```toml
mountsource = "KSU"
umount = false
partitions = []
```

```toml
mountsource = "APatch"
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

* output/magic_mount_rs-`VersionName`-`VersionCode`.zip

## Acknowledgements

* [5ec1cff/KernelSU](https://github.com/5ec1cff/KernelSU/blob/52f1f575ce2bd0ca46ebf644fd00a838af9f344e/userspace/ksud/src/magic_mount.rs): original implementation
*  [Tools-cx-app/meta-magic_mount-rs](https://github.com/Tools-cx-app/meta-magic_mount-rs/): Fork source
* [YuzakiKokuban](https://github.com/YuzakiKokuban) Webui modifications

## License

* [GPL-3.0 license](https://www.gnu.org/licenses/gpl-3.0.html)
