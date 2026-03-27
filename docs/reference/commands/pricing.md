# pricing 命令

管理模型定价。用于查看/设置/移除模型单价，影响 `stats` 成本计算与预算评估。

## 子命令

- `list`：列出所有模型定价（支持 `--verbose` 查看缓存定价）
- `set`：设置模型定价（输入/输出/缓存读写）
- `remove`：移除指定模型定价
- `reset`：重置为默认定价

## 用法

```bash
ccr pricing list [--verbose]
ccr pricing set <model> --input <价> --output <价> [--cache-read <价>] [--cache-write <价>]
ccr pricing remove <model> [--force]
ccr pricing reset [--force]
```

### set 选项

- `<model>`：模型名称
- `--input` / `--output`：输入/输出 Token 单价（美元/百万 Token）
- `--cache-read` / `--cache-write`：缓存读写单价（可选）

示例：
```bash
ccr pricing set claude-3-5-sonnet-20241022 --input 3.0 --output 15.0 --cache-read 0.3 --cache-write 3.75
ccr pricing set my-model --input 2.0 --output 10.0
```

### list 选项

- `--verbose`：显示缓存读写单价

```bash
ccr pricing list
ccr pricing list --verbose
```

### remove/reset 选项

- `--force`：跳过确认

```bash
ccr pricing remove my-model
ccr pricing reset --force
```

## 输出示例（list --verbose）

```
💰 模型定价配置

模型名称                      输入价格   输出价格   缓存读取   缓存写入
claude-3-5-sonnet-20241022    $3.00/M   $15.00/M  $0.30/M   $3.75/M
my-model                      $2.00/M   $10.00/M  -         -

🔧 默认定价（用于未配置的模型）
  输入价格: $3.00/M
  输出价格: $15.00/M

💡 提示: 使用 --verbose 查看缓存定价详情
```

## 注意事项

- 定价为非负数；移除后该模型使用默认定价。
- 默认定价可通过 `reset` 恢复；自定义定价存储于配置文件。
- `stats` 成本计算与 `budget` 预警依赖定价配置。

## 相关命令

- [`stats`](./stats.md) - 成本与用量统计
- [`budget`](./budget.md) - 预算管理
