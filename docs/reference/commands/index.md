# 命令参考

本页按任务列出当前顶层 `Commands`。嵌套命令以对应页面和 `ccr help` 输出为准。

## 帮助

```bash
ccr --help
ccr help
ccr help claude profile
ccr <command> --help
```

`ccr help [COMMAND_PATH...]` 是真实顶层命令，但不单独占用页面；它可以查看多级命令路径。

## Runtime 与 Profile

| 命令 | 用途 |
|---|---|
| [`current`](./current) | Claude/Codex runtime 总览 |
| [`claude`](./claude) | Claude official auth 与 profile runtime |
| [`codex`](./codex) | Codex auth、profile、quota 与历史同步 |
| [`opencode`](./opencode) | OpenCode auth 兼容与 Codex 导入 |
| [`platform`](./platform) | 平台注册表；当前稳定操作以 `list` 为主 |
| [`switch`](./switch) | 旧 profile 切换入口及迁移说明 |

## 配置与临时覆盖

| 命令 | 用途 |
|---|---|
| [`init`](./init) | 初始化 CCR 配置 |
| [`list`](./list) | 列出配置 |
| [`add`](./add) / [`delete`](./delete) | 创建或删除配置 |
| [`enable`](./enable) / [`disable`](./disable) | 切换配置可用状态 |
| [`temp`](./temp) | 交互式临时配置 |
| [`temp-token`](./temp-token) | 命令式临时 token 覆盖 |
| [`clear`](./clear) | 清理 CCR 管理的设置 |
| [`optimize`](./optimize) | 整理配置结构 |

## 数据、同步与运营

| 命令 | 用途 |
|---|---|
| [`history`](./history) | 查看脱敏操作历史 |
| [`export`](./export) / [`import`](./import) | 导出或导入配置 |
| [`clean`](./clean) | 清理 backup 或 plan 文件 |
| [`sync`](./sync) | WebDAV 配置资产同步 |
| [`sessions`](./sessions) | session 索引、搜索、恢复与统计 |
| [`provider`](./provider) | provider 连通性测试和验证 |
| [`stats`](./stats) | usage 汇总、导入、导出与清理 |
| [`budget`](./budget) / [`pricing`](./pricing) | 预算与模型定价 |

## 扩展、诊断与界面

| 命令 | 用途 |
|---|---|
| [`skills`](./skills) | skills 来源、扫描、安装与清单 |
| [`prompts`](./prompts) | prompt preset 管理 |
| [`validate`](./validate) | 配置与 runtime 校验 |
| [`doctor`](./doctor) | 环境、profile、auth 与可选在线体检 |
| [`check`](./check) | 跨平台配置冲突检查 |
| [`ui`](./ui) | 启动或更新 CCR UI |
| [`tui`](./tui) | 无子命令和平台交互入口说明 |
| [`update`](./update) | 检查或安装 CCR 更新 |
| [`version`](./version) | 版本与构建信息 |

## 推荐起步

```bash
ccr init
ccr current
ccr claude profile list
ccr codex auth current
ccr validate
ccr doctor
```

## 相关页面

- [CLI 工作流](/guide/cli-workflows)
- [配置模型](/guide/configuration)
- [迁移指南](/reference/migration)
