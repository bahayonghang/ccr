# history - 查看操作历史

显示 CCR 的脱敏操作审计历史，覆盖配置切换、备份、校验、导入导出与清理等事件。

## 用法

```bash
ccr history [OPTIONS]
```

## 选项

- `-l, --limit <N>`：显示最近 `N` 条记录（默认：`20`）
- `-t, --type <TYPE>`：按操作类型过滤（如 `switch`、`backup`、`validate`、`export`、`import`、`clean`、`update`）

## 历史记录字段

每条记录包含：

- 操作 ID（UUID）
- 时间戳（UTC）
- 操作者（系统用户名）
- 操作类型（`operation_type`）
- 环境变量与配置变更摘要（敏感字段自动脱敏）
- 执行结果（`success` / `failed`）
- 备注信息

## 示例

```bash
# 显示最近 20 条记录
ccr history

# 显示最近 50 条记录
ccr history --limit 50

# 仅筛选配置切换记录
ccr history -t switch

# 仅筛选配置校验记录
ccr history -t validate
```

### 示例输出

```text
Operation History
════════════════════════════════════════════════════════════════
Showing last 20 operations
────────────────────────────────────────────────────────────────

[2026-01-10 12:05:30] switch
ID: 550e8400-e29b-41d4-a716-446655440000
Actor: username
From: official -> relay
Changes:
  ANTHROPIC_BASE_URL: https://api.anthropic.com -> https://api.example.com/v1
  ANTHROPIC_AUTH_TOKEN: sk-a...key -> sk-r...xyz
Result: ✓ Success
Notes: Configuration switched successfully
```

## 敏感信息保护与存储位置

- **自动脱敏**：写入历史记录前，所有 API Key 与 Auth Token 均会被掩码处理（例如 `sk-a...r678`），避免审计文件意外暴露完整凭据。
- **存储位置**：Unified 模式下历史数据位于 `~/.ccr/history/`（及本地存储层），Legacy 模式兼容路径为 `~/.claude/ccr_history.json`。
- **与运行日志的区别**：`ccr history` 面向用户级操作审计；详细调试与错误堆栈记录在 `~/.ccr/logs/` 下按日轮转的日志文件中。

## 相关命令

- [`current`](./current) - 查看当前运行时状态
- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok) - 平台 Profile 管理与切换
- [`validate`](./validate) - 校验配置与宿主设置
- [`clean`](./clean) - 清理旧备份与缓存
