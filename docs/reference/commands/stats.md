# stats 命令

查看 API 使用统计和成本分析。支持摘要、导入、导出与清理。

## 子命令

- `summary`：统计摘要（推荐，支持分组/Top/趋势）
- `import`：导入 CSV 成本数据
- `export`：导出统计数据（JSON/CSV）
- `clear`：清理历史数据
- `cost`：已废弃别名（等同 `summary`）

## 用法

```bash
ccr stats summary [选项]
ccr stats import <csv_file> [--format auto|claude-hub|custom] [--skip-validation]
ccr stats export [--format json|csv] [--output <path>] [--range today|week|month|custom] [--start YYYY-MM-DD] [--end YYYY-MM-DD]
ccr stats clear [--before YYYY-MM-DD] [--force] [--dry-run]
```

## summary 选项

- `--range`：`today`(默认) | `week` | `month` | `custom`
- `--start` / `--end`：自定义时间段（需配合 `--range custom`）
- `--by-model` / `--by-project` / `--by-platform`：按模型/项目/平台分组
- `--top <N>`：显示成本最高的 N 个会话
- `--details`：输出趋势与更多分组明细

示例：
```bash
ccr stats summary --range week --by-model --details
ccr stats summary --range custom --start 2025-01-01 --end 2025-01-31 --top 10
```

## import 选项

- `<csv_file>`：待导入的 CSV 路径
- `--format`：`auto`(默认) | `claude-hub` | `custom`
- `--skip-validation`：跳过格式校验

```bash
ccr stats import costs.csv --format auto
ccr stats import claude_hub.csv --format claude-hub --skip-validation
```

## export 选项

- `--format`：`json`(默认) | `csv`
- `--output <path>`：输出路径；不填则打印到终端
- `--range` / `--start` / `--end`：同 `summary`

```bash
ccr stats export --format json --output report.json --range month
ccr stats export --format csv --range custom --start 2025-01-01 --end 2025-01-15
```

## clear 选项

- `--before YYYY-MM-DD`：删除该日期之前的数据（默认 30 天前）
- `--force`：跳过确认
- `--dry-run`：仅预览将删除的文件

```bash
ccr stats clear --before 2025-01-01
ccr stats clear --before 2025-01-01 --force
```

## 输出示例（summary）

```
📊 成本统计 - week

ℹ 💰 总成本: $12.3456
ℹ 📊 记录数: 156

✓ 🎫 Token 使用:
  📥 输入: 1.5M tokens
  📤 输出: 800K tokens
  💾 Cache: 300K tokens
  📊 Cache 效率: 65.23%

🤖 按模型分组:
  • 3-5-sonnet-20241022: $85.2000
  • 3-5-haiku-20241022: $32.1000

📁 按项目分组:
  • project-a: $45.0000
  • project-b: $35.2000

📈 每日趋势:
  2025-10-27 - $12.3456 (156 次)
  2025-10-26 - $8.9012 (123 次)
```

## 数据存储

- 目录：`~/.claude/stats/`
- 文件：按月 `costs_YYYYMM.csv`
- 格式：`timestamp,id,session_id,project,platform,model,input_tokens,output_tokens,cache_read_tokens,cache_write_tokens,input_cost,output_cost,cache_cost,total_cost,duration_ms,description`

## 与预算/定价的协同

- `ccr budget status|set|reset`：查看/配置预算，基于 `stats` 数据计算使用率
- `ccr pricing list|set|remove|reset`：管理模型单价，影响成本计算与统计

## 相关命令

- [`budget`](./budget.md) - 预算管理
- [`pricing`](./pricing.md) - 模型定价
- [`history`](./history.md) - 操作历史
- [`export`](./export.md) / [`import`](./import.md) - 配置数据导入导出
