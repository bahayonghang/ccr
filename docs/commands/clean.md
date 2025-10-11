# clean - 清理旧备份

清理旧的备份文件以释放磁盘空间。

## 用法

```bash
ccr clean [OPTIONS]
```

## 选项

- `--days <N>`: 保留最近 N 天的备份(默认：7)
- `--dry-run`: 预览清理操作但不实际删除

## 功能特性

- 自动清理旧备份文件
- 可配置保留期限(默认 7 天)
- 预览模式可先查看将删除的文件
- 显示释放的磁盘空间
- 仅删除 `~/.claude/backups/` 中的 `.bak` 文件

## 示例

```bash
# 清理 7 天前的备份(默认)
ccr clean

# 清理 30 天前的备份
ccr clean --days 30

# 预览清理(不实际删除)
ccr clean --dry-run

# 清理 14 天前的备份
ccr clean --days 14
```

## 示例输出

### 正常清理

```bash
$ ccr clean
Cleaning old backups...
────────────────────────────────────────────────────────────────
Retention period: 7 days
Backup directory: /home/user/.claude/backups

Files to be deleted:
  ✗ settings_20250101_100000_anthropic.json.bak (8 days old, 2.3 KB)
  ✗ settings_20250102_120000_anyrouter.json.bak (7 days old, 2.1 KB)
  ✗ settings_20250103_150000_anthropic.json.bak (6 days old, 2.3 KB)

✓ Deleted 3 backup files
✓ Freed 6.7 KB of disk space
```

### 预览模式

```bash
$ ccr clean --dry-run
Cleaning old backups (DRY RUN)...
────────────────────────────────────────────────────────────────
Retention period: 7 days
Backup directory: /home/user/.claude/backups

Files that would be deleted:
  ✗ settings_20250101_100000_anthropic.json.bak (8 days old, 2.3 KB)
  ✗ settings_20250102_120000_anyrouter.json.bak (7 days old, 2.1 KB)

Would delete: 2 files
Would free: 4.4 KB

✓ Dry run completed (no files deleted)
```

### 无文件需要清理

```bash
$ ccr clean
Cleaning old backups...
────────────────────────────────────────────────────────────────
Retention period: 7 days
Backup directory: /home/user/.claude/backups

✓ No old backup files to clean
All backups are within retention period
```

## 备份文件命名

CCR 的备份文件遵循以下命名规则：

```
settings_<timestamp>_<config_name>.json.bak
```

示例：
- `settings_20250110_120530_anthropic.json.bak`
- `settings_20250109_083022_anyrouter.json.bak`

## 使用场景

### 1. 定期清理

设置定期任务清理旧备份：

```bash
# 每周日清理 30 天前的备份
0 0 * * 0 ccr clean --days 30

# 每月清理 60 天前的备份
0 0 1 * * ccr clean --days 60
```

### 2. 释放空间

磁盘空间不足时清理旧备份：

```bash
# 查看备份占用空间
du -sh ~/.claude/backups/

# 清理 7 天前的备份
ccr clean --days 7

# 清理更多备份
ccr clean --days 3
```

### 3. 清理前预览

不确定要删除哪些文件时,先预览：

```bash
# 预览将要删除的文件
ccr clean --dry-run

# 确认后执行实际清理
ccr clean
```

### 4. 紧急清理

需要立即释放空间：

```bash
# 仅保留最近 1 天的备份
ccr clean --days 1

# 或手动删除所有旧备份
rm ~/.claude/backups/*.bak
```

### 5. 维护策略

根据重要性设置不同的保留期：

```bash
# 开发环境：保留 7 天
ccr clean --days 7

# 生产环境：保留 30 天
ccr clean --days 30

# 重要项目：保留 90 天
ccr clean --days 90
```

## 清理逻辑

### 保留期计算

```
当前时间 - 文件修改时间 > 保留天数
```

示例：
```
当前时间: 2025-01-10
保留期: 7 天
文件时间: 2025-01-02
文件年龄: 8 天
结果: 删除(8 > 7)
```

### 文件筛选

仅处理符合以下条件的文件：
1. 位于 `~/.claude/backups/` 目录
2. 文件名匹配 `*.bak` 模式
3. 文件年龄超过保留期

### 安全保护

- 仅删除备份文件,不影响其他文件
- 不会删除当前配置文件
- 保留最新的备份(即使超过保留期)

## 备份管理最佳实践

### 1. 分层保留策略

```bash
# 每天：清理 7 天前的备份
0 0 * * * ccr clean --days 7

# 每周：清理 30 天前的备份
0 0 * * 0 ccr clean --days 30

# 每月：清理 90 天前的备份
0 0 1 * * ccr clean --days 90
```

### 2. 手动归档重要备份

```bash
# 归档重要备份到其他位置
mkdir -p ~/archives/ccr-backups
cp ~/.claude/backups/settings_20250101_*.bak ~/archives/ccr-backups/

# 清理旧备份
ccr clean --days 7
```

### 3. 监控备份空间

```bash
#!/bin/bash
# 检查备份目录大小
BACKUP_SIZE=$(du -sm ~/.claude/backups | cut -f1)

if [ $BACKUP_SIZE -gt 100 ]; then
  echo "Warning: Backup directory exceeds 100MB"
  ccr clean --days 7
fi
```

### 4. 定期导出重要配置

```bash
# 每周导出配置到安全位置
ccr export -o ~/backups/ccr-weekly-$(date +%Y%m%d).toml

# 清理临时备份
ccr clean --days 7
```

## 手动清理

如果需要手动管理备份文件：

### 查看所有备份

```bash
ls -lht ~/.claude/backups/
```

### 删除特定备份

```bash
# 删除特定日期的备份
rm ~/.claude/backups/settings_20250101_*.bak

# 删除特定配置的备份
rm ~/.claude/backups/*_anthropic.json.bak
```

### 删除所有备份

::: danger 危险操作
删除所有备份后无法恢复,请谨慎操作！
:::

```bash
# 先备份到其他位置
cp -r ~/.claude/backups ~/backups-archive

# 删除所有备份
rm ~/.claude/backups/*.bak
```

## 恢复误删的备份

如果误删了需要的备份：

### 1. 检查回收站

某些系统会将删除的文件移到回收站。

### 2. 使用文件恢复工具

```bash
# Linux
sudo apt-get install testdisk
sudo photorec

# macOS
# 使用 Time Machine 恢复
```

### 3. 从其他来源恢复

- 云备份
- Git 仓库
- 导出的配置文件

## 注意事项

::: tip 建议
- 清理前先使用 `--dry-run` 预览
- 定期导出重要配置到安全位置
- 不要设置过短的保留期(建议至少 7 天)
- 重要环境建议保留更长时间(30-90 天)
:::

::: warning 注意
- `clean` 命令仅清理自动备份,不影响手动导出的配置
- 删除的备份文件无法恢复
- 建议在清理前先导出当前配置
:::

## 相关命令

- [export](./export) - 导出配置作为长期备份
- [import](./import) - 从备份恢复配置
- [switch](./switch) - 切换配置(会创建自动备份)
