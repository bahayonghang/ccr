# `export` - 已弃用的顶层配置导出命令

> [!WARNING]
> **已弃用并已隐藏（7.x 兼容）**：顶层 `ccr export` 在 CCR `7.x` 中**已弃用**且在 `--help` 中**已隐藏**。调用时会向 `stderr` 输出弃用警告（不污染 `stdout`），计划于 `8.0` 移除。

## 规范替代命令

| 场景 | 规范替代命令 |
|---|---|
| 导出 Claude Profiles（全部、按名称或按标签） | `ccr claude profile export [NAME]... [-t\|--tag <TAG>] [-o <FILE>] [--stdout] [--no-secrets] [-y\|--yes] [--json]` |
| 导出 Codex Profiles | `ccr codex profile export [NAME]... [-t\|--tag <TAG>] [-o <FILE>] [--stdout] [--no-secrets] [-y\|--yes] [--json]` |
| 导出 Grok Profiles | `ccr grok profile export [NAME]... [-t\|--tag <TAG>] [-o <FILE>] [--stdout] [--no-secrets] [-y\|--yes] [--json]` |

```bash
# 脱敏导出（可安全分享或提交版本库）
ccr claude profile export --no-secrets -o claude-team.toml

# 含密钥私有备份（默认包含内联密钥，通过 -o 以 0600 仅属主可读写权限原子落盘）
ccr codex profile export relay -o codex-relay.toml
ccr grok profile export -t work --json
```

> [!IMPORTANT]
> `profile export` 默认包含内联密钥（`secrets_mode = "included"`），落盘文件自动收紧为 `0600` 权限，仅适用于本机私有备份；切勿将未脱敏的 `--stdout` 直接通过 Shell `>` 重定向到普通文件（会退回默认 `umask` 权限），分享或提交到版本库前必须传入 `--no-secrets`。

## 关键行为差异

- **版本化 `ExchangeV1` 信封头**：旧 `ccr export`（未传 `--platform` 时隐式默认 `claude`）输出不含平台元数据的旧式目录 TOML；规范命令 `ccr <platform> profile export` 输出带 `schema_version = 1`、`platform = "<claude|codex|grok>"`、`secrets_mode = "included" | "redacted"` 与 `[profiles.<name>]` 的 `ExchangeV1` 格式，导入时可严格阻止跨平台误导。
- **彻底剥离密钥而非局部打码**：旧 `--no-secrets` 可能残留掩码字符串；规范 `profile export --no-secrets` 彻底移除内联密钥、敏感扩展字段与 URL userinfo，同时显式保留 `auth_mode`。
- **纯只读、子集筛选与覆盖保护**：规范导出对 `profiles.db` 零锁零写入副作用，支持按 `[NAME]...` 或 `-t/--tag` 子集导出，默认含密钥私有备份通过 `-o` / 默认路径收紧为 `0600` 仅属主权限（外发共享请加 `--no-secrets`），且在非交互终端或 `--json` 下覆盖已有 `-o` 文件必须显式传 `-y/--yes`（否则返回退出码 `64`）。

## 相关页面

- [`claude`](./claude)
- [`codex`](./codex)
- [`grok`](./grok)
- [`import`](./import)
