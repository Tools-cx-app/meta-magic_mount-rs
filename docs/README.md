<div align="center">

![An implementation of a metamodule](./mmrs.svg)

# Magic Mount Metamodule

[![Telegram][telegram-badge]][telegram-url]

[![Version][version-badge]][version-url]
[![GitHub Downloads][download-badge]][download-url]


[English](README_en.md) [Русский](README_ru.md)

</div>

[telegram-badge]: https://img.shields.io/badge/Group-blue?style=for-the-badge&logo=telegram&label=Telegram
[telegram-url]: https://t.me/mmrs_ci
[version-badge]: https://img.shields.io/github/v/release/Tools-cx-app/meta-magic_mount-rs?logo=github
[version-url]: https://github.com/Tools-cx-app/meta-magic_mount-rs/releases/latest
[download-badge]: https://img.shields.io/github/downloads/Tools-cx-app/meta-magic_mount-rs/total?logo=github&logoColor=green
[download-url]: https://github.com/Tools-cx-app/meta-magic_mount-rs/releases/

为 KernelSU/APatch 提供 Systemless 修改功能。

作者由于学业原因，更新会放缓

---

## 配置

配置文件路径：

`/data/adb/magic_mount/config.toml`

```toml
mountsource = "KSU"
umount = false
partitions = []
extra_mount = []
```

| 字段 | 说明 |
| ------------- | -------------- |
| mountsource | Systemless 挂载来源标识。默认值 "KSU" 与 KernelSU 行为保持一致。 |
| umount | 是否尝试卸载（依赖 KernelSU umount ）。 |
| partitions | 指定需要进行 Systemless 操作的特定分区列表，例如 "mi_ext","my_stock" 等。 |

`extra_mount` 默认 `[]`。例如 `extra_mount = ["my_product"]` 将 `<模块>/my_product/` 内容合并挂载到 `/my_product/`，不从 `<模块>/system/my_product/` 读取。要求目标目录已存在，且 `/system/my_product` 完全没有目录项（包括悬空符号链接）。只填写单级分区名，不填写路径；无效条目记录日志并跳过。没有 `system/` 的模块也受支持，仍遵循 `disable`、`remove` 和 `skip_mount` 标记。编辑配置后重启生效。

也可通过 WEBUI 配置页修改（推荐）；MiuiX 和 MD3 均提供“额外挂载分区（模块顶层）”编辑控件，支持添加和删除 `extra_mount` 分区名。保存后重启生效。

---

## 自定义规则

规则文件：`/data/adb/magic_mount/custom`

```text
# 忽略某个模块来源文件
ignore /data/adb/modules/example/system/app/Example.apk

# 将来源只读 bind 到目标
bind "/data/local/tmp/source file" "/system/etc/target file"

# 递归包含其他规则文件；add 与 file 语义相同
file /data/adb/magic_mount/extra.rules
add /data/adb/magic_mount/more.rules
```

规则支持单引号、双引号、包含空白的路径和行尾注释。控制字符会被过滤；递归 include
使用独立 visited 集合，因此循环引用不会无限递归。

自定义 bind 的目标必须是绝对路径，不能包含 `..`。目标不存在时，程序会从最近存在的
非根祖先构造临时镜像；任意 remount 或登记失败都会 detach 回滚。

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
cargo xtask build
```

构建产物将位于：

* output/magic_mount_rs-*.zip

## 致谢

*  [5ec1cff/KernelSU](https://github.com/5ec1cff/KernelSU/blob/52f1f575ce2bd0ca46ebf644fd00a838af9f344e/userspace/ksud/src/magic_mount.rs)：原始实现
* [YuzakiKokuban](https://github.com/YuzakiKokuban) Webui修改

## 许可证

[GPL-v3 许可证](http://www.gnu.org/licenses/gpl.html)

## Star History

<a href="https://star-history.dera.page/#Tools-cx-app/meta-magic_mount-rs">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://star-history.dera.page/svg?repos=Tools-cx-app/meta-magic_mount-rs&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://star-history.dera.page/svg?repos=Tools-cx-app/meta-magic_mount-rs" />
   <img alt="Star History Chart" src="https://star-history.dera.page/svg?repos=Tools-cx-app/meta-magic_mount-rs" />
 </picture>
</a>
