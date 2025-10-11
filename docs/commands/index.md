# 核心命令

CCR 提供了丰富的命令来管理 Claude Code 配置。本页面概览所有可用命令。

## 命令概览

| 命令 | 别名 | 说明 |
|------|------|------|
| [init](./init) | - | 初始化配置文件 |
| [list](./list) | `ls` | 列出所有可用配置 |
| [current](./current) | `status`, `show` | 显示当前配置状态 |
| [switch](./switch) | - | 切换到指定配置 |
| [validate](./validate) | `check` | 验证配置完整性 |
| [history](./history) | - | 显示操作历史 |
| [web](./web) | - | 启动 Web 配置界面 |
| [export](./export) | - | 导出配置到文件 |
| [import](./import) | - | 从文件导入配置 |
| [clean](./clean) | - | 清理旧备份文件 |
| [update](./update) | - | 更新到最新版本 |
| [version](./version) | `ver` | 显示版本信息 |

## 命令分类

### 配置管理

- **[init](./init)** - 初始化配置文件,创建默认模板
- **[list](./list)** - 查看所有可用配置
- **[current](./current)** - 显示当前使用的配置
- **[switch](./switch)** - 切换到不同的配置
- **[validate](./validate)** - 验证配置完整性

### 数据管理

- **[export](./export)** - 导出配置用于备份或迁移
- **[import](./import)** - 从文件导入配置
- **[clean](./clean)** - 清理旧备份文件

### 历史与监控

- **[history](./history)** - 查看操作历史记录
- **[web](./web)** - 启动 Web 管理界面

### 系统维护

- **[update](./update)** - 更新 CCR 到最新版本
- **[version](./version)** - 显示版本信息

## 常用命令速查

### 快速开始

```bash
# 初始化配置
ccr init

# 查看所有配置
ccr list

# 切换配置(两种方式)
ccr switch anthropic
ccr anthropic
```

### 日常使用

```bash
# 查看当前配置
ccr current

# 验证配置
ccr validate

# 查看历史
ccr history --limit 10
```

### 数据管理

```bash
# 导出配置
ccr export -o backup.toml

# 导入配置
ccr import config.toml --merge

# 清理备份
ccr clean --days 30
```

### 高级功能

```bash
# 启动 Web 界面
ccr web --port 8080

# 更新 CCR
ccr update

# 查看版本
ccr version
```

## 环境变量

CCR 支持以下环境变量：

### CCR_LOG_LEVEL

设置日志级别,用于调试。

**可选值：**
- `trace` - 最详细的日志
- `debug` - 调试信息
- `info` - 一般信息(默认)
- `warn` - 警告信息
- `error` - 仅错误信息

**示例：**

```bash
export CCR_LOG_LEVEL=debug
ccr switch anthropic
```

## 使用技巧

### 1. 命令别名

许多命令都有简短的别名,可以加快输入速度：

```bash
ccr ls              # 等同于 ccr list
ccr status          # 等同于 ccr current
ccr check           # 等同于 ccr validate
ccr ver             # 等同于 ccr version
```

### 2. 快速切换

直接使用配置名称作为参数：

```bash
ccr anthropic       # 等同于 ccr switch anthropic
```

### 3. 命令组合

使用 `&&` 组合多个命令：

```bash
# 切换后立即查看历史
ccr switch anthropic && ccr history --limit 1

# 导入后验证
ccr import config.toml --merge && ccr validate
```

### 4. 定期维护

设置定期任务：

```bash
# 每周清理旧备份(添加到 crontab)
0 0 * * 0 ccr clean --days 30

# 每天导出配置备份
0 0 * * * ccr export -o ~/backups/ccr-$(date +\%Y\%m\%d).toml
```

## 下一步

- 查看 [快速开始](/quick-start) 了解基本使用流程
- 查看 [配置管理](/configuration) 了解高级配置选项
- 点击上方表格中的命令查看详细文档
