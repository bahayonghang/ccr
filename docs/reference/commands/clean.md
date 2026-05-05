# clean - 交互式清理入口

裸 `ccr clean` 会进入交互式清理菜单；脚本和自动化场景应显式使用 `ccr clean planfiles` 或 `ccr clean backups`。

::: tip 重要更新
从 CCR 1.1.5 开始，所有备份操作（`switch`、`init --force`、`import`）都会**自动保留最近10个备份**，无需手动清理。`clean backups` 主要用于清理更早期的备份或手动管理备份策略。
:::

## 用法

```bash
ccr clean
ccr clean planfiles [OPTIONS]
ccr clean backups [OPTIONS]
```

兼容旧脚本的备份入口仍可用：

```bash
ccr clean --days 30 --dry-run
ccr clean --force
```

## 选项

### 交互式菜单

- 裸 `ccr clean`：显示已注册清理目标的编号菜单，当前包含 `planfiles` 和 `backups`
- 输入目标编号执行对应清理；直接回车执行默认编号 `1.planfiles`；输入 `q` 或 `0` 取消
- `ccr -y clean`：执行默认编号并跳过目标命令的确认提示

### 规划文件清理

- `planfiles`: 递归清理当前目录及子目录里的规划文件
- `--dry-run`: 预览清理操作但不实际删除
- `--force`: 跳过确认提示，直接删除命中的规划文件

### 备份清理

- `backups`: 显式清理旧备份文件
- `--days <N>`: 保留最近 N 天的备份(默认：7)
- `--dry-run`: 预览清理操作但不实际删除
- `--force`: 跳过确认提示，直接删除命中的旧备份

## 功能特性

- 裸命令提供交互式清理菜单
- 递归清理 `planning-with-files` 生成的规划文件
- 自动清理旧备份文件
- 可配置备份保留期限(默认 7 天)
- 预览模式可先查看将删除的文件
- 显示释放的磁盘空间
- `planfiles` 仅删除 `task_plan.md`、`findings.md`、`progress.md`
- `planfiles` 默认不跟随符号链接目录
- `backups` 仅删除 `~/.claude/backups/` 中的 `.bak` 文件
- **智能备份管理**：自动保留最近10个备份

## 示例

```bash
# 打开交互式清理菜单
ccr clean

# 自动执行默认编号并跳过确认
ccr -y clean

# 预览当前目录下的规划文件清理
ccr clean planfiles --dry-run

# 直接清理当前目录下的规划文件
ccr clean planfiles --force

# 预览旧备份清理
ccr clean backups --dry-run

# 清理 30 天前的备份
ccr clean backups --days 30
```

## 交互式菜单

裸 `ccr clean` 会显示当前注册的清理目标。每个目标占用一个编号，后续新增目标只需要注册到菜单即可出现为 `2`、`3` 等编号。

```text
清理内容（输入编号执行，回车 = 1，输入 q 取消）
1.planfiles - 清理 task_plan.md / findings.md / progress.md
2.backups - 清理 7 天前旧备份
请选择清理内容 [默认 1]:
```

## 规划文件清理

`ccr clean planfiles` 用于清理 `planning-with-files` skill 生成的三类固定文件：

- `task_plan.md`
- `findings.md`
- `progress.md`

命令会从当前工作目录开始递归扫描子目录，输出命中路径、命中数量和空间统计。它不会跟随符号链接目录。

### 规划文件清理输出

```bash
$ ccr clean planfiles --dry-run
清理规划文件
============

[INFO] 扫描目录: /path/to/project
[INFO] 目标文件: task_plan.md, findings.md, progress.md
[WARN] ⚠ 模拟运行模式(不会实际删除文件)

[STEP] 命中文件
[INFO] 命中: task_plan.md
[INFO] 命中: docs/findings.md
[INFO] 命中: work/progress.md

[INFO] 命中数量: 3 个
[INFO] 预计释放空间: 0.02 MB
```

## 备份清理

`ccr clean backups` 会扫描 `~/.claude/backups/` 下的 `.bak` 文件，并按修改时间删除超过保留期的旧备份。旧脚本入口 `ccr clean --days ...`、`ccr clean --dry-run` 和 `ccr clean --force` 仍保留兼容，但新文档和新脚本建议使用显式 `backups` 目标。

## 备份文件命名

CCR 的备份文件遵循以下命名规则：

### Settings 备份
```
settings_<timestamp>_<config_name>.json.bak
```

示例：
- `settings_20250110_120530_anthropic.json.bak`
- `settings_20250109_083022_anyrouter.json.bak`

### Config 备份
```
.ccs_config.toml.<tag>_<timestamp>.bak
```

示例：
- `.ccs_config.toml.init_20250110_120530.bak`
- `.ccs_config.toml.import_backup_20250109_083022.bak`

## 使用场景

::: tip 自动备份管理
CCR 现在会自动管理备份，大多数情况下你不需要手动运行 `clean` 命令。以下场景可能需要手动清理：
:::

### 1. 长期备份管理

设置定期任务清理更早期的备份：

```bash
# 每周日清理 30 天前的备份
0 0 * * 0 ccr clean backups --days 30

# 每月清理 60 天前的备份
0 0 1 * * ccr clean backups --days 60
```

### 2. 释放空间

磁盘空间不足时清理旧备份：

```bash
# 查看备份占用空间
du -sh ~/.claude/backups/

# 清理 7 天前的备份
ccr clean backups --days 7

# 清理更多备份
ccr clean backups --days 3
```

### 3. 清理前预览

不确定要删除哪些文件时,先预览：

```bash
# 预览将要删除的文件
ccr clean backups --dry-run

# 确认后执行实际清理
ccr clean backups
```

### 4. 紧急清理

需要立即释放空间：

```bash
# 仅保留最近 1 天的备份
ccr clean backups --days 1

# 或手动删除所有旧备份
rm ~/.claude/backups/*.bak
```

### 5. 维护策略

根据重要性设置不同的保留期：

```bash
# 开发环境：保留 7 天
ccr clean backups --days 7

# 生产环境：保留 30 天
ccr clean backups --days 30

# 重要项目：保留 90 天
ccr clean backups --days 90
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
0 0 * * * ccr clean backups --days 7

# 每周：清理 30 天前的备份
0 0 * * 0 ccr clean backups --days 30

# 每月：清理 90 天前的备份
0 0 1 * * ccr clean backups --days 90
```

### 2. 手动归档重要备份

```bash
# 归档重要备份到其他位置
mkdir -p ~/archives/ccr-backups
cp ~/.claude/backups/settings_20250101_*.bak ~/archives/ccr-backups/

# 清理旧备份
ccr clean backups --days 7
```

### 3. 监控备份空间

```bash
#!/bin/bash
# 检查备份目录大小
BACKUP_SIZE=$(du -sm ~/.claude/backups | cut -f1)

if [ $BACKUP_SIZE -gt 100 ]; then
  echo "Warning: Backup directory exceeds 100MB"
  ccr clean backups --days 7
fi
```

### 4. 定期导出重要配置

```bash
# 每周导出配置到安全位置
ccr export -o ~/backups/ccr-weekly-$(date +%Y%m%d).toml

# 清理临时备份
ccr clean backups --days 7
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
- `clean backups` 仅清理自动备份,不影响手动导出的配置
- 删除的备份文件无法恢复
- 建议在清理前先导出当前配置
:::

## 相关命令

- [export](./export) - 导出配置作为长期备份
- [import](./import) - 从备份恢复配置
- [switch](./switch) - 切换配置(会创建自动备份)
- [add](./add) - 添加新配置
- [delete](./delete) - 删除配置
