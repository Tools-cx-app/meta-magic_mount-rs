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
## 目录标记

在模块对应目录中创建空标记文件，重启后生效：

- `.replace`：用模块目录内容替换目标目录，不保留原有内容。
- `.remove`：隐藏整个目标目录及其内容，例如 `<模块>/system/app/Foo/.remove` 隐藏 `/system/app/Foo`。仅对目录节点生效，不支持对普通文件或符号链接添加此属性；原始文件不会被删除。

`.remove` 优先于同目录的 `.replace`，该目录内的模块内容不会挂载。两种标记文件本身均不挂载，`extra_mount` 下的目录采用相同用法。模块根目录的 `remove`（无点号）仍表示移除模块，与 `.remove` 不同。


## 自定义 Lua 钩子

Lua 5.4 钩子取代旧文本 DSL 和声明式规则。**不兼容变更：**
`/data/adb/magic_mount/custom` 和 `/data/adb/magic_mount/custom.lua` 不再受支持，也不会被读取。
请在重启前手动将旧规则转换为钩子，并将脚本移入 `/data/adb/magic_mount/lua/`。
WebUI 已移除忽略列表和自定义 bind 管理，且绝不会编辑脚本。

按文件名升序加载 `/data/adb/magic_mount/lua/` 中所有直接子级的普通 `.lua` 文件，
例如 `10-skip.lua`、`20-bind.lua`；不扫描子目录，也不加载符号链接文件。
目录不存在表示没有钩子；目录访问、读取或脚本加载错误会终止执行，不会被忽略。
每个脚本使用独立 Lua VM，全局变量不会在脚本间泄漏。两个钩子均可省略。
例如创建 `/data/adb/magic_mount/lua/10-custom.lua`：

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

所有 `before()` 按文件名顺序在模块收集和挂载前运行。`modules` 按模块 ID 索引，每项包含
`id`、`name`、`version`、`author`、`description`、`mode`、`is_mounted`、`enabled`、
`source_path`、`rules`（含 `default_mode` 和 `paths`）以及可修改的布尔值 `skip`。
任一脚本设为 `skip = true` 即跳过该模块，其他脚本不能通过 `false` 重新启用；
`false` 也不能覆盖 `disable`、`remove` 或 `skip_mount` 标记的排除效果。
`moduledir` 是模块根目录（`/data/adb/modules`）。所有 `after()` 仅在 Magic Mount 成功后
按文件名顺序运行，只提供 `bind(source, target)` 挂载 API，没有 `ignore` 或跳过模块的 API。
仅当全部 `after()` 成功后才应用排队的 bind。错误包含脚本路径，并终止当前阶段：
加载、语法或 `before` 错误会在挂载前终止；`after` 错误会取消全部排队的 bind，
但不会回滚已完成的 Magic Mount。路径不能为空或包含控制字符。
脚本是可信的本地配置，没有运行时间或内存配额。

`modules` 命令及 WebUI 按模块 ID 从启动快照还原 `is_mounted`、`mode` 和 `rules`，
因此 Lua 跳过的模块显示为未挂载。启动后新增、尚未进入快照的模块也显示为未挂载；
名称、版本和启用状态仍来自当前模块文件，修改这些信息不会改变本次启动的挂载状态。

Lua 提供安全的基础库及 table、string、math、utf8 库，不提供 `os`、`io`、`package`、
`debug` 或文件加载器。bind 目标必须是绝对路径，不能包含 `..` 路径分量；目标不存在时，
会从最近存在的非根祖先构造临时镜像。bind 保持只读，任意 remount 或登记失败都会 detach 回滚。

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

Android lint 与最低支持的 API 26 对齐：
`cargo ndk --platform 26 -t arm64-v8a -t armeabi-v7a clippy -- -D warnings`。

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
