# CCR

**Rust 编写的 AI CLI 配置与运行时管理入口。**  
以 CLI 为主线，围绕显式 Claude Runtime / Codex Runtime，配套 TUI 与完整 CCR UI。

## ✨ 核心特性

- **显式 Runtime 模型**：`ccr current` 并列展示 Claude Runtime 与 Codex Runtime。
- **平台级 Profile 路由**：用 `ccr claude profile ...` 与 `ccr codex profile ...` 代替已退休的全局 `ccr switch` 路径。
- **企业级安全**：支持原子写入、文件锁 (`fs4`)、审计日志与自动备份。
- **多端接口**：CLI、TUI 与 CCR UI。
- **认证迁移友好**：支持保存、导出（加密）、导入 Codex 账号，并迁移兼容账号到 OpenCode。
- **智能同步**：基于 WebDAV 的多目录同步能力。

## 界面预览

### TUI

终端界面将 Profile 切换、当前路由/认证上下文、所选配置详情和键盘操作集中在同一视图中。

![CCR TUI 展示 Codex Profile 选择和路由详情](docs/assets/readme/ccr-tui-overview.png)

### CCR UI Dashboard

Dashboard 在进入具体模块前集中展示桌面后端就绪状态、下一个建议操作、Runtime 可用性和平台信号。

![CCR UI Dashboard 展示就绪状态和下一步操作](docs/assets/readme/ccr-ui-dashboard.png)

### CCR UI Codex Profiles

Codex Profiles 在一个管理视图中整合快速切换、搜索与状态筛选、Profile 卡片、当前配置上下文和配置分布。

![CCR UI Codex Profiles 展示筛选和配置上下文](docs/assets/readme/ccr-ui-codex-profiles.png)

## 🚀 快速开始

```bash
ccr init
ccr current
ccr add
ccr claude profile list
ccr claude profile switch <name>
ccr validate
```

### 迁移速查表

| 旧命令 | 当前做法 |
|---|---|
| `ccr switch <name>` | `ccr claude profile switch <name>` 或 `ccr codex profile switch <name>` |
| `ccr <name>` | 快捷入口已退休；改用同样的显式命令 |
| `ccr platform switch <platform>` | auth/profile 路由已退休 |
| `ccr platform current` | `ccr current` |
| `ccr platform profile ...` | `ccr claude profile ...` / `ccr codex profile ...` |

## 🔐 Runtime 命令

```bash
ccr claude profile list
ccr claude profile switch work
ccr claude profile off
ccr codex auth current
ccr codex profile list
ccr codex profile switch proxy
ccr codex profile off
```

## 🛠️ 开发指南

```bash
just build
just test
just ci
```
