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
---

<script setup>
const coreFeatures = [
  {
    icon: '⚡',
    title: '多接口一体',
    details: 'CLI 为主，内置 TUI、轻量 Web API，推荐完整 CCR UI。',
    link: '/guide/quick-start'
  },
  {
    icon: '🛡️',
    title: '并发安全',
    details: '文件锁 + 进程内互斥 + 原子写入，保护配置文件。',
    link: '/reference/architecture'
  },
  {
    icon: '🔀',
    title: '多平台注册表',
    details: '支持 Claude、Codex、Gemini、Qwen、iFlow 等平台。',
    link: '/reference/platforms/'
  },
  {
    icon: '🧭',
    title: '配置直写',
    details: '直接写入 settings.json，自动备份与审计。',
    link: '/reference/commands/switch'
  },
  {
    icon: '☁️',
    title: 'WebDAV 同步',
    details: '多目录注册、批量 push/pull，智能过滤。',
    link: '/reference/commands/sync'
  },
  {
    icon: '📚',
    title: 'Session 管理',
    details: '解析索引 Claude/Codex/Gemini 会话历史，支持搜索恢复。',
    link: '/reference/commands/sessions'
  },
  {
    icon: '💚',
    title: 'Provider 健康检查',
    details: '检测 API 端点连通性、验证 Key、测量延迟。',
    link: '/reference/commands/provider'
  },
  {
    icon: '📊',
    title: '成本统计',
    details: '提供调用统计与成本分析，支持 JSON 输出。',
    link: '/reference/commands/stats'
  }
]

const quickLinks = [
  {
    icon: '📖',
    title: '快速开始',
    details: '5 分钟上手 CCR 配置管理。',
    link: '/guide/quick-start'
  },
  {
    icon: '🖥️',
    title: 'CCR UI',
    details: 'Vue3 + Axum + Tauri 全栈界面。',
    link: '/reference/commands/ui'
  },
  {
    icon: '⌨️',
    title: '命令参考',
    details: '全部 CLI 命令详细文档。',
    link: '/reference/commands/'
  },
  {
    icon: '🏗️',
    title: '架构设计',
    details: '了解 CCR 的分层架构。',
    link: '/reference/architecture'
  }
]
</script>

<HomeFeatures badge="核心功能" title="为什么选择 CCR？" :features="coreFeatures" />

<HomeFeatures badge="快速导航" badge-type="info" title="开始使用" :features="quickLinks" />

## 版本与安装
- 当前版本：3.12.5（Rust 2024）
- 需求：Rust 1.85+；可选 Node.js 18+ + Bun 1.0+（CCR UI 开发），`just`（便捷脚本）

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
ccr temp-token set sk-xxx       # 临时覆盖 token，不改 TOML
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
# 交互式选择同步内容
ccr sync push -i
```

### 界面与服务
```bash
ccr ui -p 3000 --backend-port 38081  # 完整 CCR UI（自动检测或下载）
ccr tui                              # 需启用 tui 特性
ccr web -p 8080 --no-browser         # 轻量 API/兼容用途
```

### Sessions 与 Provider
```bash
ccr sessions list                    # 列出会话历史
ccr sessions search "keyword"        # 搜索会话
ccr sessions resume <id>             # 恢复会话
ccr provider test --all              # 测试所有 Provider 连通性
ccr provider verify <name>           # 验证 API Key
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
