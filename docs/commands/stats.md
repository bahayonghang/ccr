# stats 命令

查看 API 使用统计和成本分析。

## 概述

`stats` 命令提供详细的成本追踪和使用统计功能，帮助您了解 AI API 的使用情况和相关成本。

## 语法

```bash
ccr stats cost [选项]
```

## 子命令

### cost

查看成本统计信息。

```bash
ccr stats cost [选项]
```

## 选项

### --range \<范围\>

指定时间范围。

- **可选值**: `today`, `week`, `month`
- **默认值**: `today`

```bash
# 查看今日成本
ccr stats cost --range today

# 查看本周成本
ccr stats cost --range week

# 查看本月成本
ccr stats cost --range month
```

### --by-model

按模型分组显示成本。

```bash
ccr stats cost --range month --by-model
```

示例输出：
```
🤖 按模型分组:
  • 3-5-sonnet-20241022: $85.2000
  • 3-5-haiku-20241022: $32.1000
  • 3-opus-20240229: $10.1500
```

### --by-project

按项目分组显示成本。

```bash
ccr stats cost --range month --by-project
```

示例输出：
```
📁 按项目分组:
  • project-a: $45.0000
  • project-b: $35.2000
  • project-c: $28.0000
```

### --top \<数量\>

显示成本最高的 N 个会话。

```bash
# 查看成本最高的 10 个会话
ccr stats cost --top 10

# 查看成本最高的 20 个会话
ccr stats cost --top 20
```

### --details

显示详细统计信息，包括每日趋势。

```bash
ccr stats cost --range month --details
```

### --export \<文件路径\>

将统计数据导出到 JSON 文件。

```bash
# 导出月度报告
ccr stats cost --range month --export monthly_report.json

# 导出详细统计
ccr stats cost --range month --details --export detailed_stats.json
```

## 使用示例

### 基础使用

查看今日成本：

```bash
ccr stats cost
```

输出示例：
```
📊 成本统计 - today
══════════════

ℹ 💰 总成本: $12.3456
ℹ 📊 记录数: 156

✓ 🎫 Token 使用:
  📥 输入: 1.5M tokens
  📤 输出: 800K tokens
  💾 Cache: 300K tokens
  📊 Cache 效率: 65.23%
```

### 查看本月详细统计

```bash
ccr stats cost --range month --details --by-model --by-project
```

输出示例：
```
📊 成本统计 - month
══════════════

ℹ 💰 总成本: $127.4500
ℹ 📊 记录数: 1,234

✓ 🎫 Token 使用:
  📥 输入: 15.2M tokens
  📤 输出: 8.3M tokens
  💾 Cache: 3.1M tokens
  📊 Cache 效率: 72.45%

🤖 按模型分组:
  • 3-5-sonnet-20241022: $85.2000
  • 3-5-haiku-20241022: $32.1000
  • 3-opus-20240229: $10.1500

📁 按项目分组:
  • project-a: $45.0000
  • project-b: $35.2000
  • project-c: $28.0000
  • project-d: $12.5000
  • project-e: $6.7500

📈 每日趋势:
  2025-10-27 - $12.3456 (156 次)
  2025-10-26 - $8.9012 (123 次)
  2025-10-25 - $15.6789 (189 次)
  2025-10-24 - $10.2345 (145 次)
  2025-10-23 - $9.8765 (134 次)
  2025-10-22 - $11.4567 (167 次)
  2025-10-21 - $13.2109 (178 次)
```

### 导出报告

```bash
ccr stats cost --range month --export report.json
```

JSON 格式示例：
```json
{
  "total_cost": 127.45,
  "record_count": 1234,
  "token_stats": {
    "total_input_tokens": 15200000,
    "total_output_tokens": 8300000,
    "total_cache_tokens": 3100000,
    "cache_efficiency": 72.45
  },
  "by_model": {
    "claude-3-5-sonnet-20241022": 85.20,
    "claude-3-5-haiku-20241022": 32.10,
    "claude-3-opus-20240229": 10.15
  },
  "by_project": {
    "/path/to/project-a": 45.00,
    "/path/to/project-b": 35.20,
    "/path/to/project-c": 28.00
  },
  "trend": [
    {"date": "2025-10-27", "cost": 12.3456, "count": 156},
    {"date": "2025-10-26", "cost": 8.9012, "count": 123}
  ]
}
```

## 数据存储

### 存储位置

成本数据存储在 `~/.claude/stats/` 目录：

```
~/.claude/stats/
├── costs_202510.csv    # 2025年10月的成本数据
├── costs_202509.csv    # 2025年9月的成本数据
└── ...
```

### CSV 格式

每月一个 CSV 文件，格式如下：

```csv
timestamp,id,session_id,project,platform,model,input_tokens,output_tokens,cache_read_tokens,cache_write_tokens,input_cost,output_cost,cache_cost,total_cost,duration_ms,description
2025-10-27T10:00:00Z,uuid-123,sess_abc,/path/to/proj,claude,claude-3-5-sonnet-20241022,1500,800,200,100,0.004500,0.012000,0.000750,0.017250,3500,Implement auth
```

## 模型定价

CCR 支持以下 Claude 模型的准确定价（截至 2025-10-27）：

| 模型 | 输入 ($/M tokens) | 输出 ($/M tokens) | Cache 读 ($/M) | Cache 写 ($/M) |
|------|------------------|------------------|---------------|---------------|
| Claude 3.5 Sonnet | $3.00 | $15.00 | $0.30 | $3.75 |
| Claude 3.5 Haiku | $1.00 | $5.00 | $0.10 | $1.25 |
| Claude 3 Opus | $15.00 | $75.00 | $1.50 | $18.75 |
| Claude 4.5 Sonnet | $3.00 | $15.00 | $0.30 | $3.75 |
| Claude 4.1 Opus | $15.00 | $75.00 | $1.50 | $18.75 |

## 统计指标说明

### 成本指标

- **总成本**: 所有 API 调用的总成本（美元）
- **记录数**: API 调用的总次数
- **输入成本**: 输入 Token 的成本
- **输出成本**: 输出 Token 的成本
- **Cache 成本**: Cache Token 的成本（包括读取和写入）

### Token 指标

- **输入 Token**: 发送给 AI 的 Token 数量
- **输出 Token**: AI 生成的 Token 数量
- **Cache Token**: 使用 Prompt Caching 的 Token 数量
- **Cache 效率**: Cache 读取 Token / 总 Cache Token × 100%

### 性能指标

- **API 调用次数**: 总的 API 请求数
- **平均单次成本**: 总成本 / 调用次数
- **每日趋势**: 每天的成本和调用次数

## Web UI

除了 CLI 命令，您还可以通过 Web UI 查看统计数据：

1. 启动 CCR UI:
```bash
ccr ui
```

2. 访问统计页面:
```
http://localhost:3000/stats
```

Web UI 提供：
- 📊 可视化仪表板
- 📈 交互式图表
- 🔄 实时刷新
- 📱 响应式设计
- 🌙 深色模式支持

## 注意事项

### 数据收集

⚠️ **重要**: 成本统计需要在实际 API 调用时记录数据。如果您是首次使用，可能需要一些时间才能看到统计数据。

### 数据保留

- CSV 文件按月存储
- 建议定期备份统计数据
- 可以安全删除旧月份的 CSV 文件

### 精确度

- 成本计算基于 Anthropic 官方定价
- Token 计数来自 API 响应
- 成本精确到小数点后 6 位

## 常见问题

### 为什么看不到统计数据？

如果统计显示为空，可能的原因：
1. 尚未进行任何 API 调用
2. 统计功能刚刚启用，还没有历史数据
3. CSV 文件不存在或为空

### 如何重置统计数据？

删除统计目录：
```bash
rm -rf ~/.claude/stats/
```

### 统计数据能导入其他工具吗？

可以！导出的 JSON 和 CSV 格式都是标准格式，可以导入到：
- Excel/Google Sheets
- 数据分析工具（pandas, R）
- BI 工具（Tableau, PowerBI）

## 相关命令

- [`list`](./list.md) - 列出所有配置
- [`current`](./current.md) - 查看当前配置
- [`history`](./history.md) - 查看操作历史

## 参考资源

- [Anthropic 定价页面](https://www.anthropic.com/pricing)
- [统计功能开发文档](../TODO_ANALYTICS.md)
- [CCR UI 统计视图](../../ccr-ui/docs/frontend/components.md#statsview)
