---
layout: home

hero:
  name: "CCR"
  text: "Claude Code Configuration Switcher"
  tagline: "Rust 多平台配置管理 · CLI/TUI/Web/API/UI 一体化"
  image:
    src: /logo.svg
    alt: CCR
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/quick-start
    - theme: alt
      text: 命令参考
      link: /reference/commands/
    - theme: alt
      text: English
      link: /en/

features:
  - icon: ⚡
    title: 直接写入 settings.json
    details: 原子写入 + 文件锁，修改立刻生效且避免并发损坏。
  - icon: 🛡️
    title: 审计与备份
    details: 全量操作日志、自动备份，Merge/Replace 导入均可回滚。
  - icon: 🔀
    title: 多平台统一
    details: Unified Mode 下管理 Claude、Codex、Gemini、Qwen、iFlow 等，兼容 Legacy `~/.ccs_config.toml`。
  - icon: 🧭
    title: 丰富界面
    details: CLI 为主，可选 TUI、轻量 Web API (`ccr web`)、完整 CCR UI (`ccr ui`，Vue 3 + Axum + Tauri)。
  - icon: ☁️
    title: WebDAV 同步
    details: 目录注册、启用/禁用、单目录与全量 push/pull/status，智能过滤备份与锁。
  - icon: 📊
    title: 成本与统计
    details: ccr stats 提供成本/调用统计（web 特性），可输出 JSON。
---

## 版本与安装
- 当前版本：3.4.1（Rust 2024）。需求：Rust 1.85+，可选 Node 18+ 用于 CCR UI 开发。

```bash
# 推荐：直接安装
cargo install --git https://github.com/bahayonghang/ccr ccr

# 源码安装
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

## 快速使用
```bash
ccr init                        # 初始化 Unified Mode (~/.ccr)
ccr platform list               # 查看平台，支持 claude/codex/gemini/qwen/iflow
ccr add                         # 引导创建配置
ccr list && ccr switch <name>   # 查看与切换（也可直接 ccr <name>）
ccr validate                    # 校验配置与 settings
ccr history -l 50               # 查看历史
ccr export --no-secrets         # 导出配置（可选去除敏感信息）
ccr import configs.toml --merge # 合并导入，自动备份
ccr clean --days 30             # 清理旧备份
```

### 同步与多目录
```bash
# 开启 WebDAV 配置
ccr sync config
# 目录注册与启用
ccr sync folder add claude ~/.claude -r /ccr-sync/claude
ccr sync folder enable claude
# 单目录或全量操作
ccr sync claude push
ccr sync all status
ccr sync all pull --force
```

### 界面与服务
```bash
ccr ui -p 3000 --backend-port 8081   # 完整 CCR UI（自动检测或下载）
ccr tui                              # 需启用 tui 特性
ccr web -p 8080 --no-browser         # 轻量 API/兼容用途
```

## 目录结构（工作区）
```
ccr/
|-- src/                # CLI + 库（平台、服务、同步、web、tui）
|-- ccr-ui/             # 完整 UI（backend: Axum；frontend: Vue 3 + Tauri）
|-- docs/               # 本文档
|-- tests/              # 集成测试
`-- justfile            # 通用开发任务
```

## 对比 CCS
| 能力 | CCS (Shell) | CCR (Rust) |
|------|-------------|------------|
| 配置切换/写入 | ✔️ | ✔️ |
| 直接写 settings.json | ❌ | ✔️ |
| 文件锁/原子写 | ❌ | ✔️ |
| 审计历史 | 基础 | 完整 |
| 自动/手动备份 | ❌ | ✔️ |
| 校验/优化 | 基础 | 完整 |
| Web/TUI/UI | Web 简易 | Web API + TUI + CCR UI |
| 多平台 | 单 Claude | 多平台 |

## 许可证与贡献
MIT。欢迎通过 Issue/PR 反馈与贡献：https://github.com/bahayonghang/ccr
