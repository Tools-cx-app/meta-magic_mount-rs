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

将 Lua 5.4 脚本放入 `/data/adb/magic_mount/lua/`，重启生效。只加载该目录直接子级的普通 `.lua` 文件，按文件名升序，不加载符号链接或子目录。目录不存在表示没有脚本，每个脚本有独立的 Lua VM。

文件头必须声明 `API = 1` 或 `API = 2`；API 2 随后声明 `MODULE = "模块 ID"`。每个字面量声明独占一行，允许前导空行、Lua 注释和单双引号。API 和 MODULE 运行时只读。旧 `bind` 改为 `bind_mount`；旧 `modules[id].skip` 改为 API 2 的 `skip()`。旧 custom/custom.lua 不再读取，WebUI 不编辑脚本。

API 1 仅提供挂载函数 `bind_mount(source, target)`，在可选的 `before()/after()` 中调用：

~~~lua
API = 1

function before()
    bind_mount("/data/local/tmp/before.conf", "/system/etc/before.conf")
end

function after()
    bind_mount("/data/local/tmp/hosts", "/system/etc/hosts")
end
~~~

全部 before 按文件名顺序排队。模块规则和挂载树准备成功后，在 Magic Mount 之前执行这些 bind；目标可能被后续 Magic Mount 覆盖。全部 after 仅在 Magic Mount 成功后运行，所有 after 成功才执行其 bind。目标必须是绝对路径，路径不能为空、包含控制字符或 `..` 分量。bind 保持只读；目标不存在时，从最近存在的非根祖先创建临时镜像。

API 2 根据 module.prop 的 ID 匹配 MODULE，不假设目录名等于 ID。扫描模块挂载内容前，按模块 ID 查找匹配的脚本并调用无参数 `load()`，同一模块的脚本按文件名执行。未安装或带 disable、remove、skip_mount 标记的模块不执行 load。仅在 API 2 的 load 内提供：

- `skip()`：跳过整个模块的挂载内容扫描与挂载，取消该模块全部规则，忽略后续规则，并停止同模块后续脚本。无参数且幂等；用 `return` 提前结束当前 Lua 回调。不创建 skip_mount 文件。
- `block(path)`：仅排除当前模块提供的文件或目录，保留系统原内容和其他模块贡献。
- `replace(source, target)`：源是绝对路径的普通文件或目录，目标是模块相对路径。目录整体替换，源不能是符号链接或特殊设备，同路径最后一次声明生效。
- `remove(path)`：在最终挂载视图中隐藏文件、符号链接或目录，包括其他模块同路径贡献，不删除磁盘原内容。不支持移除分区根目录。

~~~lua
API = 2
MODULE = "example_module"

function load()
    block("system/etc/unwanted.conf")
    replace("/data/local/tmp/hosts", "system/etc/hosts")
    remove("system/app/Example")
end
~~~

跳过整个模块时，将 load 的函数体改为 `skip(); return`。相对路径以模块根目录为基准，包含 `system/` 或已配置的 extra_mount 分区名，例如 `my_product/etc/example.xml`。拒绝绝对目标、空路径、控制字符及 `..`。skip 优先于全部规则，其余优先级为 remove > block > replace，父级 block/remove 支配后代。没有 system/ 的模块也能通过规则贡献挂载。模块按 ID 排序，普通冲突保留先到贡献；block/replace 在本模块合并前生效，remove 在最终树生效。

`moduledir` 是模块根目录（/data/adb/modules）。Lua 提供基础库及 table/string/math/utf8，不提供 os/io/package/debug、文件加载器、rawset 和 setmetatable。加载、before、load 或挂载树准备错误会在挂载前终止；after 错误取消该阶段 bind，不回滚已完成的 Magic Mount。可信本地脚本没有时间或内存配额。

modules 命令和 WebUI 按模块 ID 从启动快照恢复 is_mounted、mode 和 rules；查询不执行脚本。skip 模块保留在列表中并显示未挂载，只影响本次启动。快照描述启动计划，不是逐文件挂载成功审计。

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
