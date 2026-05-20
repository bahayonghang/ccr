---
layout: home

hero:
  name: "CCR"
  text: "统一管理 AI CLI 配置与运行时状态"
  tagline: "CLI 主线，显式 Claude Runtime / Codex Runtime，配套 TUI 与 CCR UI"
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
    details: '围绕 ccr current、ccr claude profile、ccr codex profile 组织日常运行时管理。',
    link: '/guide/cli-workflows'
  },
  {
    icon: '🖥️',
    title: 'CCR UI 为图形入口',
    details: 'Vue 3 + Tauri 图形界面，共享同一套 registry、profiles、history 与 backups。',
    link: '/guide/ui-overview'
  }
]

const capabilityCards = [
  {
    icon: '🔀',
    title: '显式双 Runtime',
    details: 'Claude 与 Codex 并列展示；不再依赖全局 current_platform 心智模型。',
    link: '/reference/commands/current'
  },
  {
    icon: '🔐',
    title: '平台级 Profile 命令',
    details: '使用 ccr claude profile ... 与 ccr codex profile ... 管理运行时路由。',
    link: '/reference/commands/'
  },
  {
    icon: '☁️',
    title: 'WebDAV 多目录同步',
    details: '支持目录注册、单目录与批量 push/pull/status。',
    link: '/reference/commands/sync'
  },
  {
    icon: '📚',
    title: 'Sessions / Provider / Skills',
    details: '会话索引、Provider 健康检查、技能与提示词共用一套 CLI 入口。',
    link: '/reference/commands/'
  },
  {
    icon: '🛡️',
    title: '安全写入',
    details: '文件锁、原子写入、备份与审计链路共同保护配置切换。',
    link: '/guide/configuration'
  },
  {
    icon: '🏗️',
    title: '架构与迁移参考',
    details: 'workspace 分层、运行时流程和迁移映射集中在参考文档。',
    link: '/reference/migration'
  }
]
</script>

<HomeFeatures badge="入口选择" title="如何使用 CCR" :features="choosePaths" />
<HomeFeatures badge="能力概览" badge-type="info" title="当前项目覆盖范围" :features="capabilityCards" />

## 5 分钟起步

```bash
ccr init
ccr current
ccr add
ccr claude profile list
ccr claude profile switch <name>
ccr validate
```

## 支持矩阵

| 平台 | 状态 | 说明 |
|------|------|------|
| Claude Code | ✅ Implemented | official auth + profile runtime 双路径 |
| Codex | ✅ Implemented | auth、profile、sync-history 均已实现 |
| Antigravity CLI | ✅ Implemented | 内部 key 保持 `gemini`；旧 Gemini session import 保留兼容 |
| Factory Droid | ✅ Implemented | 保留在 broader platform domain 中 |
| Qwen CLI | 🚧 Reserved / Partial | 保留平台键与部分数据域支持 |

## 常用入口

```bash
ccr current --verbose
ccr codex auth current
ccr codex profile list
ccr ui -p 15173 --backend-port 38081
```
