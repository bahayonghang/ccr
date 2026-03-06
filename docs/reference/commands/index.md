# 命令总览

CCR 当前的 CLI 可以分为 5 组：平台与初始化、profile 与临时覆盖、数据与同步、界面与成本、扩展与维护。

## 命令清单

| 命令 | 说明 | 备注 |
|------|------|------|
| [`init`](./init) | 初始化配置目录 | Unified Mode 默认入口 |
| [`platform`](./platform) | 管理平台注册表 | list / switch / current / info / init |
| [`codex`](./codex) | 管理 Codex 多账号 | `ccr codex auth *` |
| [`migrate`](./migrate) | Legacy → Unified 迁移 | 多平台迁移入口 |
| [`add`](./add) / [`delete`](./delete) | 增删 profile | 面向当前平台 |
| [`list`](./list) / [`current`](./current) / [`switch`](./switch) | 查看与切换 profile | `ccr <name>` 是 `switch` 快捷方式 |
| [`temp`](./temp) / [`temp-token`](./temp-token) | 临时覆盖当前设置 | `temp` 为交互式，`temp-token` 为命令式 |
| [`validate`](./validate) / [`enable`](./enable) / [`disable`](./disable) / [`clear`](./clear) / [`optimize`](./optimize) | 校验与整理配置 | |
| [`history`](./history) / [`export`](./export) / [`import`](./import) / [`clean`](./clean) | 审计、导入、导出、清理 | |
| [`sync`](./sync) | WebDAV 同步 | 目录注册、push/pull/status |
| [`sessions`](./sessions) / [`provider`](./provider) / [`check`](./check) | 会话、健康检查、冲突检测 | 诊断向命令组 |
| [`ui`](./ui) / [`web`](./web) / [`tui`](./tui) | 图形界面、Legacy Web API、终端界面 | `ui` 为推荐入口 |
| [`stats`](./stats) / [`budget`](./budget) / [`pricing`](./pricing) | 成本与预算 | 依赖统计与定价数据 |
| [`skills`](./skills) / [`prompts`](./prompts) | 扩展能力管理 | |
| [`update`](./update) / [`version`](./version) | 版本维护 | |

## 推荐上手顺序

```bash
ccr init
ccr platform list
ccr add
ccr list
ccr switch <name>
ccr validate
```

如果你更偏向浏览器式操作：

```bash
ccr ui -p 15173 --backend-port 38081
```

如果你只需要兼容 HTTP 接口：

```bash
ccr web --host 127.0.0.1 --port 19527 --no-browser
```

## 按任务找命令

### 初始化与平台

- [`init`](./init)
- [`platform`](./platform)
- [`migrate`](./migrate)

### Profile 与临时覆盖

- [`add`](./add)
- [`delete`](./delete)
- [`list`](./list)
- [`current`](./current)
- [`switch`](./switch)
- [`temp`](./temp)
- [`temp-token`](./temp-token)
- [`validate`](./validate)
- [`enable`](./enable)
- [`disable`](./disable)
- [`clear`](./clear)
- [`optimize`](./optimize)

### 数据、同步与诊断

- [`history`](./history)
- [`export`](./export)
- [`import`](./import)
- [`clean`](./clean)
- [`sync`](./sync)
- [`sessions`](./sessions)
- [`provider`](./provider)
- [`check`](./check)

### 界面与成本

- [`ui`](./ui)
- [`web`](./web)
- [`tui`](./tui)
- [`stats`](./stats)
- [`budget`](./budget)
- [`pricing`](./pricing)

### 扩展与维护

- [`codex`](./codex)
- [`skills`](./skills)
- [`prompts`](./prompts)
- [`update`](./update)
- [`version`](./version)

## 相关文档

- [CLI 工作流](/guide/cli-workflows)
- [接口选择：ccr ui vs ccr web](/guide/web-guide)
- [Web API 参考](/reference/api)
