# CC-Switch TUI

All-in-One Assistant for Claude Code, Codex & Gemini CLI - Terminal UI Version

## 概述

CC-Switch TUI 是 CC-Switch 的终端界面版本，专为无 GUI 环境设计，特别适合：
- 嵌入式设备（如 Radxa A7Z）
- 服务器环境
- SSH 远程管理
- ARM64 架构设备

## 特性

- ✅ **零 GUI 依赖**：完全移除 WebKitGTK、GTK 等 GUI 库
- ✅ **终端界面**：基于 Ratatui 的交互式 TUI
- ✅ **键盘导航**：完整的键盘快捷键支持
- ✅ **静态链接**：所有依赖可静态编译，无需额外安装
- ✅ **ARM64 优化**：原生支持 ARM64 架构
- ✅ **Systemd 集成**：完整的 Linux 服务支持
- ✅ **代理服务器**：HTTP 代理，支持 Claude Code、Codex、Gemini CLI
- ✅ **自动故障转移**：多 Provider 自动切换
- ✅ **熔断器**：故障容错机制
- ✅ **MCP 集成**：Model Context Protocol 服务器管理

## 架构

```
cc-switch/
├── cc-switch-core/      # 核心库（业务逻辑）
│   ├── database/        # SQLite 数据库
│   ├── proxy/           # Axum HTTP 代理服务器
│   ├── services/        # 业务服务层
│   └── mcp/             # MCP 集成
└── cc-switch-tui/       # TUI 应用
    ├── app.rs           # TUI 界面
    └── main.rs          # CLI 入口
```

## 安装

### 从 Release 下载

```bash
# 下载 ARM64 .deb 包
wget https://github.com/KrOik/cc-switch/releases/latest/download/cc-switch-tui_3.11.1_arm64.deb

# 安装
sudo dpkg -i cc-switch-tui_3.11.1_arm64.deb
```

### 从源码构建

#### 本地构建（x86_64）
```bash
cargo build --release -p cc-switch-tui
```

#### 交叉编译（ARM64）
```bash
# 安装交叉编译工具
sudo apt-get install gcc-aarch64-linux-gnu
rustup target add aarch64-unknown-linux-gnu

# 构建
cargo build --release --target aarch64-unknown-linux-gnu -p cc-switch-tui
```

#### 构建 Debian 包
```bash
./scripts/build-deb.sh
```

## 使用方法

### 交互式 TUI

```bash
cc-switch-tui
```

**键盘快捷键：**
- `P` - Provider 管理
- `X` - 代理控制
- `M` - MCP 服务器
- `S` - 统计信息
- `C` - 配置
- `L` - 日志
- `Q` - 退出
- `Esc` - 返回主界面

### CLI 命令

```bash
# 代理控制
cc-switch-tui proxy start
cc-switch-tui proxy stop
cc-switch-tui proxy status

# Provider 管理
cc-switch-tui provider list
cc-switch-tui provider switch <id>

# Daemon 模式
cc-switch-tui daemon start
cc-switch-tui daemon stop
cc-switch-tui daemon status
```

### Systemd 服务

```bash
# 启动服务
sudo systemctl start cc-switch-tui

# 开机自启
sudo systemctl enable cc-switch-tui

# 查看状态
sudo systemctl status cc-switch-tui

# 查看日志
journalctl -u cc-switch-tui -f
```

## 配置

配置文件位置：`~/.cc-switch/`

- `cc-switch.db` - SQLite 数据库
- `app_paths.json` - 路径配置

## 依赖

### 运行时依赖
- 无（所有依赖静态链接）

### 构建依赖
- Rust 1.85.0+
- Cargo

### 交叉编译依赖（ARM64）
- gcc-aarch64-linux-gnu

## 兼容性

- ✅ Debian 10 (Buster) ARM64
- ✅ Debian 11 (Bullseye) ARM64
- ✅ Debian 12 (Bookworm) ARM64
- ✅ Ubuntu 20.04+ ARM64
- ✅ Radxa A7Z
- ✅ Raspberry Pi 4/5
- ✅ 其他 ARM64 Linux 发行版

## 从 GUI 版本迁移

如果你之前使用 Tauri GUI 版本：

1. 数据库和配置会自动兼容
2. 所有 Provider 配置保留
3. MCP 服务器配置保留
4. 使用统计数据保留

## 开发

### 项目结构

```
cc-switch/
├── Cargo.toml              # Workspace 配置
├── cc-switch-core/         # 核心库
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── database/
│       ├── proxy/
│       ├── services/
│       └── mcp/
├── cc-switch-tui/          # TUI 应用
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── app.rs
├── debian/                 # Debian 打包
│   ├── control
│   ├── postinst
│   └── prerm
└── scripts/
    └── build-deb.sh        # 构建脚本
```

### 运行测试

```bash
cargo test --workspace
```

### 代码检查

```bash
cargo check --workspace
cargo clippy --workspace
```

## 故障排除

### 代理无法启动

检查端口是否被占用：
```bash
sudo netstat -tlnp | grep 15721
```

### 数据库错误

重置数据库：
```bash
rm ~/.cc-switch/cc-switch.db
cc-switch-tui  # 会自动重建
```

### Systemd 服务无法启动

查看日志：
```bash
journalctl -u cc-switch-tui -n 50
```

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License

## 致谢

- 原始 GUI 版本：[farion1231/cc-switch](https://github.com/farion1231/cc-switch)
- TUI 库：[Ratatui](https://github.com/ratatui-org/ratatui)
- 终端处理：[Crossterm](https://github.com/crossterm-rs/crossterm)

## 相关链接

- [Claude Code](https://www.anthropic.com/claude)
- [Codex CLI](https://github.com/anthropics/codex)
- [Gemini CLI](https://ai.google.dev/)
