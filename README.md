# mountify-rs

> Globally mounted modules via OverlayFS — rewritten in Rust.

基于 OverlayFS 的 KernelSU MetaModule，将模块文件全局挂载到系统分区。

使用 Rust 重构了原始 mountify 的核心逻辑，替代了原有的 shell 脚本实现。

## 特点

- **Rust 核心** — 1770 行 Rust，直接 syscall 操作 mount/umount/xattr
- **双重模式** — tmpfs 模式 / ext4 sparse 模式
- **诱饵挂载** — decoy mount 检测绕过
- **LKM 支持** — 加载内核模块以清理 ext4 sysfs 节点
- **赛博朋克 HUD** — 内置 WebUI 配置面板（中/英文）
- **多平台** — KernelSU / APatch / Magisk

## 快速开始

1. 从 [Releases](../../releases) 下载最新 `mountify_module.zip`
2. 通过 KernelSU Manager / APatch / Magisk Manager 刷入
3. 重启设备

## 配置

刷入后配置位于 `/data/adb/mountify/config.sh`，可通过 WebUI 或直接编辑配置文件。

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `mountify_mounts` | `2` | 0=禁用, 1=手动, 2=自动 |
| `FAKE_MOUNT_NAME` | `mountify` | 挂载点名称 |
| `use_ext4_sparse` | `0` | 启用 ext4 sparse 模式 |
| `spoof_sparse` | `0` | 伪装为 apex 挂载 |
| `mountify_custom_umount` | `0` | 内核级卸载 (1=susfs, 2=ksud) |

## WebUI

安装后在 KernelSU Manager / APatch 中点击模块的"UI"或"WebUI"按钮进入配置界面。

## 构建

```bash
# 1. 安装 Rust Android 目标
rustup target add aarch64-linux-android

# 2. 安装 Android NDK
# 设置 ANDROID_NDK_HOME

# 3. 构建 Rust 二进制
cd module/mountify-rs
cargo build --release --target aarch64-linux-android

# 4. 构建 WebUI
cd webui
npm install && npm run build

# 5. 打包模块
cd module
zip -r ../mountify_module.zip .
```

## 目录结构

```
mountify-rs/
├── module/                    # 可安装模块包
│   ├── mountify-rs/           # Rust 核心源码
│   │   └── src/               # ~1770 行
│   ├── mountify-arm64         # 预编译 ARM64 二进制
│   ├── lkm/                   # ext4 sysfs nuke LKM
│   ├── webroot/               # WebUI 构建产物
│   └── *.sh                   # Shell 入口封装
├── webui/                     # WebUI 源码
└── .github/workflows/         # CI (Rust + WebUI)
```

## 致谢

- [backslashxx/mountify](https://github.com/backslashxx/mountify) — 原始 mountify 项目

## 许可证

MIT
