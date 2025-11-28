# **Meta-Hybrid Mount**

A Hybrid Mount metamodule for KernelSU/Magisk, implementing both OverlayFS and Magic Mount logic via a native Rust binary.

这是一个用于 KernelSU/Magisk 的混合挂载 (Hybrid Mount) 元模块，通过原生 Rust 二进制文件实现了 OverlayFS 和 Magic Mount 逻辑。
基于俺寻思和Vibe Coding。

## **English**

### **Core Architecture**

* **True Hybrid Engine**:  
  * **Logic**: Written in Rust using rustix for direct syscalls.  
  * **Mechanism**: Supports mixing **OverlayFS** and **Magic Mount** on the same partition. OverlayFS is applied first for performance, followed by Magic Mount stacking for specific modules that require legacy binding.
  * **Fallback**: Automatically falls back to Magic Mount for specific modules if OverlayFS fails.
* **Smart Storage**:  
  * **Priority**: Prioritizes **Tmpfs** (memory-backed filesystem) for maximum speed and stealth.
  * **Compatibility**: Automatically detects if Tmpfs supports XATTR (required for SELinux). If not, it falls back to mounting a 2GB ext4 loop image (`modules.img`) to ensure compatibility.
* **Stealth**:  
  * **Dynamic Mount Point**: Automatically searches for and uses empty decoy directories (e.g., `/oem`, `/mnt/vendor/persist`) as the mount base instead of a static path.
  * **Nuke LKM**: Integrates a kernel module (`nuke.ko`) to unregister `ext4` sysfs nodes when using image mode, erasing traces of the loop mount.
  * **Namespace Detach**: Implements `try_umount` logic to detach mount points in the global namespace.

### **Features**

* **Per-Module Configuration**: Toggle specific modules between "Auto" (OverlayFS) and "Magic" (Bind Mount) modes via WebUI.
* **WebUI**: A Svelte 5 + Vite frontend running in WebView. Now includes settings for **Stealth Mode** (Force Ext4, Enable Nuke LKM).
* **Logging**: Detailed daemon logs at `/data/adb/meta-hybrid/daemon.log`.

### **Build**

**Requirements**:

* Rust (Nightly toolchain recommended for Android targets)  
* Node.js & npm  
* Android NDK (r26+)

## **中文 (Chinese)**

### **核心架构**

* **真·混合引擎**:  
  * **逻辑**: 使用 Rust 编写，利用 rustix 进行直接系统调用。  
  * **机制**: 支持在同一分区上混合使用 **OverlayFS** 和 **Magic Mount**。优先应用 OverlayFS 以保证性能，随后可叠加 Magic Mount 以支持需要传统挂载方式的模块。
  * **回退**: 如果 OverlayFS 挂载失败，特定模块可自动回退至 Magic Mount。
* **智能存储**:  
  * **优先级**: 优先使用 **Tmpfs**（内存文件系统），实现最快的速度和最好的隐蔽性。
  * **兼容性**: 自动检测 Tmpfs 是否支持 XATTR（SELinux 必须）。如果不支持，则自动回退到挂载 2GB 的 ext4 loop 镜像 (`modules.img`)，确保存储环境标准。
* **隐藏机制**:  
  * **动态挂载点**: 启动时自动搜索系统中的空闲诱饵目录（如 `/oem`, `/mnt/vendor/persist`）作为挂载基点，避免使用固定的 `/data/adb` 路径。
  * **Nuke LKM**: 集成内核模块 (`nuke.ko`)，在使用 Ext4 模式时自动注销 sysfs 中的 `ext4` 节点，消除挂载痕迹。
  * **命名空间分离**: 实现了 `try_umount` 逻辑，在全局命名空间中分离挂载点。

### **特性**

* **逐模块配置**: 可通过 WebUI 将特定模块在 "自动" (OverlayFS) 和 "Magic" (绑定挂载) 模式间切换。
* **WebUI**: 基于 Svelte 5 + Vite 的前端。新增 **隐蔽模式** 设置（强制 Ext4、启用 Nuke LKM）。
* **日志**: 守护进程日志位于 `/data/adb/meta-hybrid/daemon.log`。

### **构建**

**环境要求**:

* Rust (建议使用 Nightly 工具链以支持 Android 目标)  
* Node.js & npm  
* Android NDK (r26+)

## **License**

GPL-3.0