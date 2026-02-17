<div align="center">

# Magic Mount Metamodule

[![Telegram][telegram-badge]][telegram-url]

[English](README_en.md) [Русский](README_ru.md)

</div>

[telegram-badge]: https://img.shields.io/badge/Group-blue?style=for-the-badge&logo=telegram&label=Telegram
[telegram-url]: https://t.me/mmrs_ci

为 KernelSU 提供 Systemless 修改功能。

作者由于学业原因，更新会放缓

---

## 配置

配置文件路径：

`/data/adb/magic_mount/config.toml`

```toml
mountsource = "KSU"
umount = false
partitions = []
```

| 字段 | 说明 |
| ------------- | -------------- |
| mountsource | Systemless 挂载来源标识。默认值 "KSU" 与 KernelSU 行为保持一致。 |
| umount | 是否尝试卸载（依赖 KernelSU umount ）。 |
| partitions | 指定需要进行 Systemless 操作的特定分区列表，例如 "mi_ext","my_stock" 等。 |
| tmpfsdir | 临时目录，默认 "/debug_ramdisk"，此选项可选。 |

也可通过 WEBUI 进行配置（推荐）。

---

## 开发

依赖：

* Rust nightly toolchain

* Android NDK

* cargo-ndk

* Node.js / npm

* `pnpm` and `vite` as dependency and frontend for webui

环境变量：
```shell
export ANDROID_NDK_HOME=<path/to/ndk>
export ANDROID_NDK_ROOT=$ANDROID_NDK_HOME

```

构建：
```shell
cargo xtask b
```

构建产物将位于：

* output/magic_mount_rs.zip

## 致谢

*  [5ec1cff/KernelSU](https://github.com/5ec1cff/KernelSU/blob/52f1f575ce2bd0ca46ebf644fd00a838af9f344e/userspace/ksud/src/magic_mount.rs)：原始实现
* [YuzakiKokuban](https://github.com/YuzakiKokuban) Webui修改

## 许可证

* [GPL-3.0 license](https://www.gnu.org/licenses/gpl-3.0.html)

