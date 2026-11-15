---
layout: home

hero:
  name: "CCR"
  text: "统一管理 AI CLI 配置与运行时状态"
  tagline: "CLI 主线，显式 Claude / Codex / Grok 运行时与 SQLite 配置库，配套 TUI 与 CCR UI"
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
    details: '围绕 ccr current 与 ccr <claude|codex|grok> profile 组织日常运行时管理。',
    link: '/guide/cli-workflows'
  },
  {
    icon: '🖥️',
    title: 'CCR UI 为图形入口',
    details: 'Tauri 图形界面，共享同一套注册表、profiles.db 配置库、历史记录与备份。',
    link: '/guide/ui-overview'
  }
]

const capabilityCards = [
  {
    icon: '🔀',
    title: '显式多 Runtime 状态',
    details: '通过 ccr current 直观查看运行时路由；不再依赖隐式的全局当前平台开关。',
    link: '/reference/commands/current'
  },
  {
    icon: '🔐',
    title: '平台级 Profile 与 Auth 命令',
    details: '使用 ccr <claude|codex|grok> profile ... 与 auth ... 管理配置库与官方登录态。',
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
    title: 'Doctor / Sessions / Skills',
    details: '统一只读体检、会话索引、Provider 健康检查、技能与提示词共用一套 CLI 入口。',
    link: '/reference/commands/doctor'
  },
  {
    icon: '🛡️',
    title: 'SQLite 配置库与安全写入',
    details: '以 profiles.db 为权威活目录，配合 ExchangeV1 导入导出、文件锁、原子写入与备份。',
    link: '/guide/configuration'
  },
  {
    icon: '🏗️',
    title: '架构与迁移参考',
    details: 'Workspace 分层、运行时流程与 7.x 命令/存储迁移映射集中在参考文档。',
    link: '/reference/migration'
  }
]
</script>

<HomeFeatures badge="入口选择" title="如何使用 CCR" :features="choosePaths" />
<HomeFeatures badge="能力概览" badge-type="info" title="当前项目覆盖范围" :features="capabilityCards" />

## 5 分钟起步

```bash
ccr claude profile init
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin
ccr claude profile list
ccr claude profile switch work
ccr current
ccr validate
```

## 支持矩阵

| 平台 | 状态 | 说明 |
|------|------|------|
| Claude Code (`claude`) | ✅ 已实现 | `auth` + `profile` 双路径，权威存储 `profiles.db` |
| Codex (`codex`) | ✅ 已实现 | `auth`、`profile`、`fix`、`sync-history` 均已实现，权威存储 `profiles.db` |
| Grok (`grok`) | ✅ 已实现 | `auth` + `profile` 完整 CLI/TUI/UI 支持，权威存储 `profiles.db` |
| Antigravity CLI (`gemini`) | 🔄 兼容适配器 | `legacy_adapter`（无 CLI `profile`/`auth` 子命令）；支持 `doctor`、周边工具与 `/antigravity` UI |
| Factory Droid (`droid`) | 🔄 兼容适配器 | `legacy_adapter`（无 CLI `profile`/`auth` 子命令）；支持 `doctor` 与周边工具 |
| Qwen CLI (`qwen`) | 🚧 预留 / Stub | 保留平台键与部分数据域支持 |

## 常用入口

```bash
ccr current --verbose
ccr codex auth current
ccr grok profile list
ccr doctor
ccr ui -p 15173 --backend-port 38081
```
