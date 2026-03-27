---
layout: home

hero:
  name: "CCR"
  text: "统一管理 AI CLI 配置与工作入口"
  tagline: "CLI 主线，配套 TUI 与完整 CCR UI"
  image:
    src: /logo.svg
    alt: CCR
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/quick-start
    - theme: alt
      text: CLI 工作流
      link: /guide/cli-workflows
    - theme: alt
      text: CCR UI
      link: /guide/ui-overview
---

<script setup>
const choosePaths = [
  {
    icon: '⚡',
    title: 'CLI 为主线',
    details: '统一入口覆盖 profile 生命周期、平台切换、同步、预算、历史与会话。',
    link: '/guide/cli-workflows'
  },
  {
    icon: '🖥️',
    title: 'CCR UI 为推荐图形入口',
    details: '完整 Vue 3 + Tauri 体验，适合日常图形化管理和模块浏览。',
    link: '/guide/ui-overview'
  }
]

const capabilityCards = [
  {
    icon: '🔀',
    title: '统一平台注册表',
    details: '围绕 claude、codex、gemini、droid 等平台管理独立 profile、历史与备份。',
    link: '/reference/platforms/'
  },
  {
    icon: '☁️',
    title: 'WebDAV 多目录同步',
    details: '支持目录注册、交互式过滤、单目录与批量 push/pull。',
    link: '/reference/commands/sync'
  },
  {
    icon: '📚',
    title: 'Sessions / Provider / Skills',
    details: '会话索引、Provider 健康检查、技能与提示词管理共用一套 CLI 入口。',
    link: '/reference/commands/'
  },
  {
    icon: '📊',
    title: '成本与预算',
    details: '围绕 stats、budget、pricing 三组命令管理用量、成本和预算阈值。',
    link: '/reference/commands/stats'
  },
  {
    icon: '🛡️',
    title: '安全写入',
    details: '文件锁、进程内互斥、原子写入与备份链路共同保证配置切换安全。',
    link: '/guide/quick-start'
  },
  {
    icon: '🏗️',
    title: '架构与集成参考',
    details: 'workspace 分层、crate 边界、运行时流程和迁移说明集中在参考文档。',
    link: '/reference/architecture'
  }
]
</script>

<HomeFeatures badge="入口选择" title="如何使用 CCR" :features="choosePaths" />
<HomeFeatures badge="能力概览" badge-type="info" title="当前项目覆盖范围" :features="capabilityCards" />

## 快速安装
- Rust 1.90+
- 可选：Node.js 18+ 与 Bun 1.0+（仅在开发 `ccr-ui` 时需要）
- 推荐：`just`

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

源码安装与工作区说明见 [快速开始](/guide/quick-start)。

## 5 分钟起步

```bash
ccr init
ccr platform list
ccr add
ccr list
ccr switch <name>
ccr validate
```

下一步：
- 日常 CLI 路径：[`CLI 工作流`](/guide/cli-workflows)
- 浏览器与桌面路径：[`UI 概览`](/guide/ui-overview)
- 入口选择：[`入口选择`](/guide/entrypoints)
- 所有命令：[`命令参考`](/reference/commands/)

## 支持矩阵

| 平台 | 状态 | 说明 |
|------|------|------|
| Claude Code | ✅ Implemented | 默认主线平台，直接写入 `~/.claude/settings.json` |
| Codex | ✅ Implemented | 支持 profile、auth、MCP 等工作流 |
| Gemini CLI | ✅ Implemented | 独立 profile / history / backup |
| Factory Droid | ✅ Implemented | 平台页与模块页已进入 CCR UI |
| Qwen CLI | 🚧 Reserved / Partial | 代码中保留平台键与 UI 分组，文档按保留能力说明 |
| iFlow CLI | 🚧 Reserved / Partial | 代码中保留平台键与 UI 分组，文档按保留能力说明 |

详细平台说明见 [`平台支持`](/reference/platforms/)。

## 常用入口

```bash
ccr ui -p 15173 --backend-port 38081
ccr sync config
ccr sessions list
ccr provider test --all
ccr stats summary --range week --details
```

## 文档地图
- [`快速开始`](/guide/quick-start)：安装、初始化、首个 profile
- [`CLI 工作流`](/guide/cli-workflows)：按日常任务组织命令
- [`入口选择`](/guide/entrypoints)：CLI / TUI / CCR UI 的角色边界
- [`UI 概览`](/guide/ui-overview)：`ccr ui` 的运行模式与推荐使用方式
- [`UI 模块地图`](/guide/ui-modules)：平台模块与工具模块的能力分组
- [`架构设计`](/reference/architecture)：workspace 与分层设计

## 许可证与贡献
MIT。欢迎通过 Issue/PR 反馈与贡献：https://github.com/bahayonghang/ccr
