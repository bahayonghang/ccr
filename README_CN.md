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
