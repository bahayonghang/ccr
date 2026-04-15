# opencode - OpenCode auth 迁移与入口

`ccr opencode` 是 OpenCode 相关的专项命令组。当前面向用户的主能力有两类：

- 作为 OpenCode Auth 页签的终端入口
- 将 CCR 已保存的兼容 Codex 账号增量导入 OpenCode 已保存账号表

## 用法

```bash
ccr opencode
ccr opencode auth import-codex [--dry-run] [--json]
```

## 当前支持的子命令

### `ccr opencode`

不带子命令时，在默认启用 TUI 的构建下会直接进入 OpenCode Auth 页签，适合做可视化查看、切换和导入预览。

### `ccr opencode auth import-codex`

把 CCR 已保存的兼容 Codex 账号导入 OpenCode 的已保存账号 registry。

支持选项：

| 选项 | 说明 |
|---|---|
| `--dry-run` | 只预览迁移结果，不写入任何 OpenCode 账号快照或 registry |
| `--json` | 输出机器可读的迁移报告，便于脚本消费 |

## 行为保证

- 只读取 CCR 已保存的 Codex 账号，不读取未保存的运行时登录态
- 只导入兼容的 ChatGPT OAuth 账号
- 会跳过仅 API Key、缺少快照、无效快照的账号
- 会检查 OpenCode 中同名账号冲突和 `accountId` 冲突
- 不覆盖、不重命名、不删除现有 OpenCode 账号
- 不会因为导入而切换当前 OpenCode 运行时登录
- CLI 和 TUI 共用同一份结构化迁移报告

## 常见示例

```bash
# 先预览可导入账号
ccr opencode auth import-codex --dry-run

# 再执行实际导入
ccr opencode auth import-codex

# 输出 JSON 报告
ccr opencode auth import-codex --json

# 直接进入 OpenCode Auth 页签
ccr opencode
```

在 OpenCode Auth 页签中，按 `i` 可预览并确认导入兼容的已保存 Codex 账号。

## 何时使用

- 你已经用 `ccr codex auth save` 保存了多组 Codex 账号
- 你希望 OpenCode 复用这些账号，而不是逐个重新登录
- 你需要增量导入，且不想影响当前 OpenCode 运行时登录

## 相关文档

- [`codex`](./codex)
- [`tui`](./tui)
- [CLI 工作流](/guide/cli-workflows)
