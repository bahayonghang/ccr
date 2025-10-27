# 统计与成本分析

CCR UI 提供完整的 API 使用统计和成本追踪功能，帮助您深入了解 AI API 的使用情况和相关成本。

## 功能概览

统计功能包括：

- ✅ **实时成本追踪** - 精确记录每次 API 调用的成本
- ✅ **多维度统计** - 按时间、模型、项目分组分析
- ✅ **可视化仪表板** - 直观的数据展示
- ✅ **趋势分析** - 查看每日成本趋势
- ✅ **数据导出** - 支持 JSON 格式导出
- ✅ **响应式设计** - 支持桌面和移动端
- ✅ **深色模式** - 完整的 dark mode 支持

## 访问统计页面

### 通过导航菜单

1. 启动 CCR UI
2. 点击左侧导航栏的"统计分析"或"Stats"
3. 访问 `http://localhost:5173/stats`

### 通过 Dashboard

1. 在 Dashboard 首页找到"📊 统计分析"卡片
2. 点击"查看详情"按钮

## 界面介绍

### 概览卡片

页面顶部显示 4 个关键指标卡片：

#### 💰 总成本
- 显示选定时间范围内的总成本（美元）
- 精确到小数点后 4 位

#### 📊 API 调用次数
- 显示 API 调用的总次数
- 反映使用频率

#### 📥 输入 Token
- 显示发送给 AI 的 Token 数量
- 使用 K/M 简化显示（如 1.5M）

#### 📤 输出 Token
- 显示 AI 生成的 Token 数量
- 使用 K/M 简化显示

### Token 详细统计

展示更详细的 Token 使用信息：

- **Cache Token** - 使用 Prompt Caching 的 Token 数量
- **Cache 效率** - Cache 读取 Token / 总 Cache Token × 100%
- **总 Token** - 所有 Token 的总和

### 按模型分组

展示不同 Claude 模型的成本分布：

```
🤖 按模型分组
• 3-5-sonnet-20241022 ......... $85.2000
• 3-5-haiku-20241022 ........... $32.1000
• 3-opus-20240229 .............. $10.1500
```

### 按项目分组

展示不同项目的成本分布（显示 Top 10）：

```
📁 按项目分组 (Top 10)
• project-a .................... $45.0000
• project-b .................... $35.2000
• project-c .................... $28.0000
```

### 成本趋势

展示最近 7 天的成本趋势：

```
📈 成本趋势
2025-10-27 - $12.3456 (156 次)
2025-10-26 - $8.9012 (123 次)
2025-10-25 - $15.6789 (189 次)
```

## 功能使用

### 时间范围选择

使用页面右上角的下拉菜单选择时间范围：

- **今日** - 查看今天的统计数据
- **本周** - 查看本周的统计数据
- **本月** - 查看本月的统计数据

选择后会自动刷新数据。

### 刷新数据

点击"🔄 刷新"按钮获取最新数据：

- 按钮在数据加载时会显示旋转动画
- 刷新会重新拉取服务器最新数据
- 适用于长时间打开页面后更新数据

### 空状态

如果还没有统计数据，页面会显示：

```
📊 暂无统计数据
开始使用 AI API 后，这里将显示统计信息
```

### 错误处理

如果数据加载失败，会显示错误提示框：

```
❌ 加载失败
[错误信息]
```

## 数据格式说明

### 成本计算

CCR 使用 Anthropic 官方定价计算成本：

| 模型 | 输入 ($/M) | 输出 ($/M) | Cache 读 ($/M) | Cache 写 ($/M) |
|------|-----------|-----------|---------------|---------------|
| Claude 3.5 Sonnet | $3.00 | $15.00 | $0.30 | $3.75 |
| Claude 3.5 Haiku | $1.00 | $5.00 | $0.10 | $1.25 |
| Claude 3 Opus | $15.00 | $75.00 | $1.50 | $18.75 |
| Claude 4.5 Sonnet | $3.00 | $15.00 | $0.30 | $3.75 |
| Claude 4.1 Opus | $15.00 | $75.00 | $1.50 | $18.75 |

### Token 统计

- **输入 Token**: 用户发送给 AI 的文本
- **输出 Token**: AI 生成的响应文本
- **Cache Token**: 使用 Prompt Caching 功能缓存的内容
- **Cache 效率**: 反映缓存命中率

### 成本趋势

- **日期**: YYYY-MM-DD 格式
- **成本**: 当日总成本（美元）
- **次数**: 当日 API 调用次数

## API 端点

统计功能使用以下 API 端点：

### GET /api/stats/cost

获取成本概览（支持 range 参数）

**参数**:
- `range`: `today` | `week` | `month`（默认：`today`）

**响应**:
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
    "claude-3-5-sonnet-20241022": 85.20
  },
  "by_project": {
    "/path/to/project": 45.00
  },
  "trend": [
    {"date": "2025-10-27", "cost": 12.3456, "count": 156}
  ]
}
```

### GET /api/stats/cost/today

获取今日成本（快捷方式）

### GET /api/stats/cost/week

获取本周成本（快捷方式）

### GET /api/stats/cost/month

获取本月成本（快捷方式）

### GET /api/stats/cost/trend

获取成本趋势（支持 range 参数）

### GET /api/stats/cost/by-model

获取按模型分组的成本（支持 range 参数）

### GET /api/stats/cost/by-project

获取按项目分组的成本（支持 range 参数）

### GET /api/stats/cost/top-sessions

获取成本最高的会话（支持 limit 参数）

### GET /api/stats/summary

获取快速摘要（今日/本周/本月成本）

## 技术实现

### 前端

- **框架**: Vue.js 3 Composition API
- **状态管理**: Ref + Computed
- **样式**: Tailwind CSS
- **图标**: SVG 内联图标
- **请求**: Axios

### 后端

- **框架**: Axum (Rust)
- **数据存储**: CSV 文件（按月分文件）
- **数据处理**: Serde + CSV
- **成本计算**: 自定义 ModelPricing 模块

### 数据流

```
Vue Component → API Client → Axum Handler → CostTracker → CSV Files
                    ↓
               TypeScript Types
```

## 性能优化

### 前端优化

- 使用 `computed` 缓存排序结果
- 数据格式化函数优化（K/M 简化）
- 条件渲染减少 DOM 操作

### 后端优化

- CSV 增量读取
- 时间范围索引
- 内存高效的数据结构

## 常见问题

### 为什么看不到统计数据？

**可能原因**:
1. 尚未进行任何 API 调用
2. 统计功能刚刚启用
3. CSV 文件不存在

**解决方法**:
- 使用 AI API 后数据会自动记录
- 检查 `~/.claude/stats/` 目录

### 成本计算准确吗？

是的！成本计算基于：
- Anthropic 官方定价（2025-10-27）
- API 响应中的实际 Token 计数
- 精确到小数点后 6 位

### 数据可以导出吗？

目前前端暂不支持导出，但您可以：
1. 使用 CLI 命令导出：`ccr stats cost --export report.json`
2. 直接访问 CSV 文件：`~/.claude/stats/costs_YYYYMM.csv`

### 如何重置统计数据？

删除统计目录：
```bash
rm -rf ~/.claude/stats/
```

## 相关资源

- [CLI stats 命令文档](../../docs/commands/stats.md)
- [API 文档](../backend/api.md#statistics-endpoints)
- [技术实现文档](../../docs/TODO_ANALYTICS.md)
- [Anthropic 定价页面](https://www.anthropic.com/pricing)

## 未来规划

### 高级图表（计划中）

- 📈 成本趋势折线图（Chart.js）
- 🥧 模型分布饼图
- 📊 项目成本柱状图
- 🌊 Token 使用面积图

### 更多维度（计划中）

- 按用户分组
- 按会话分组
- 自定义时间范围
- 成本预算告警

### 数据导出（计划中）

- 前端直接导出 JSON/CSV
- 导出为 Excel 格式
- 定期自动导出

---

**提示**: 统计功能会持续优化和改进，欢迎提供反馈和建议！
