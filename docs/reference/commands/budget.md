# budget 命令

管理成本预算。支持查看、设置、重置预算，并基于统计数据给出超额/预警状态。

## 子命令

- `status`：查看预算状态与使用率
- `set`：配置每日/每周/每月预算、预警阈值，启用/禁用预算控制
- `reset`：重置所有预算限制

## 用法

```bash
ccr budget status
ccr budget set [--daily <金额>] [--weekly <金额>] [--monthly <金额>] [--warn-at <0-100>] [--enable|--disable]
ccr budget reset [--force]
```

### set 选项

- `--daily` / `--weekly` / `--monthly`：分别设置日/周/月预算（美元）
- `--warn-at <0-100>`：使用率预警阈值（默认 90%）
- `--enable` / `--disable`：启用或关闭预算控制（不可同时使用）

示例：
```bash
ccr budget set --daily 10 --weekly 50 --monthly 200 --warn-at 90 --enable
ccr budget set --monthly 150 --disable  # 仅关闭预算控制
```

### reset 选项

- `--force`：跳过确认直接重置

```bash
ccr budget reset
ccr budget reset --force
```

## 输出示例（status）

```
💰 预算状态
✅ 预算控制已启用

周期    当前成本  预算限制  使用率   状态
每日    $2.50    $10.00   25.0%   ✅ 正常
每周    $15.00   $50.00   30.0%   ✅ 正常
每月    $62.00   $200.00  31.0%   ✅ 正常
```

当使用率接近/超过阈值时会显示 ⚠️/❌ 提示。

## 注意事项

- 预算计算基于 `ccr stats` 数据；确保已收集统计记录。
- 预警阈值仅影响提示，不会阻止调用。

## 相关命令

- [`stats`](./stats.md) - 成本与用量统计
- [`pricing`](./pricing.md) - 模型定价管理
