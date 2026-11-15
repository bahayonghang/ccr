# `import` - 已弃用的顶层配置导入命令

> [!WARNING]
> **已弃用并已隐藏（7.x 兼容）**：顶层 `ccr import <file>` 在 CCR `7.x` 中**已弃用**且在 `--help` 中**已隐藏**。调用时会向 `stderr` 输出弃用警告（不污染 `stdout`），计划于 `8.0` 移除。

## 规范替代命令

| 场景 | 规范替代命令 |
|---|---|
| 增量合并导入（默认安全行为，不删除本地其他条目） | `ccr <claude\|codex\|grok> profile import <FILE> [--on-conflict skip\|overwrite\|rename] [--dedup] [--dry-run] [--json]` |
| 从标准输入导入 | `ccr <claude\|codex\|grok> profile import --stdin [--dry-run]`（或传入 `-`） |
| 破坏性整体替换导入（移除未在文件中的非激活本地条目） | `ccr <claude\|codex\|grok> profile import <FILE> --replace [-y\|--yes]` |

```bash
ccr claude profile import claude-team.toml --dry-run
ccr claude profile import claude-team.toml --on-conflict overwrite --dedup --json
ccr -y codex profile import codex-full.toml --replace --json
```

## 关键行为差异

- **默认模式反转（增量合并 vs 整体替换）**：旧 `ccr import <file>` 未传 `--merge` 时**默认执行破坏性整体替换**（且未传 `--platform` 时隐式默认 `claude`）；规范命令 `ccr <platform> profile import <file>` **默认执行非破坏性增量合并**（`--on-conflict skip`），整体替换必须显式传 `--replace`（非交互、`--stdin` 或 `--json` 下还必须附加 `-y/--yes`）。
- **三态输入兼容与严格平台防呆**：规范导入同时支持 `ExchangeV1`（严格核验 `platform` 匹配）、`LegacyCatalog` 与 `LegacySimplified` 三种格式，并自动拦截脱敏掩码伪密钥。
- **凭据保护、激活项防删与 6+1 桶回执**：缺内联密钥的新增项自动以 `enabled = false` 入库（记入 `missing_secrets`），兼容覆盖更新自动保留本地既有密钥（`preserved_local_secrets`），当前激活 Profile 受保护不可被移除或禁用，写入前自动创建 `0600` 备份，并支持 `--dry-run`、`--no-apply`、`--expected-revision` 与 6+1 桶 `--json` 报告。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`export`](./export)
