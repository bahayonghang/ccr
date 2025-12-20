# sessions

Session 会话管理命令，用于解析、索引和管理 AI CLI 工具产生的会话历史。

## 概述

`ccr sessions` 命令组提供对 Claude、Codex、Gemini 等 AI CLI 工具会话历史的管理功能：

- **索引**: 扫描并索引本地会话文件
- **列表**: 查看历史会话
- **搜索**: 按关键词搜索会话
- **恢复**: 生成恢复会话的命令
- **统计**: 查看索引统计信息

## 支持的平台

| 平台 | 会话路径 | 格式 |
|------|----------|------|
| Claude | `~/.claude/projects/**/*.jsonl` | JSONL |
| Codex | `~/.codex/sessions/*.jsonl` | JSONL |
| Gemini | `~/.gemini/tmp/*` | 自定义格式 |
| Qwen | `~/.qwen/sessions/*.jsonl` | JSONL |
| iFlow | `~/.iflow/sessions/*.jsonl` | JSONL |

## 子命令

### list

列出会话历史。

```bash
ccr sessions list [OPTIONS]
```

**选项：**

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `-p, --platform <PLATFORM>` | 按平台过滤 (claude/codex/gemini/qwen/iflow) | 全部 |
| `-l, --limit <N>` | 限制显示数量 | 20 |
| `--today` | 仅显示今天的会话 | 否 |

**示例：**

```bash
# 列出最近 20 个会话
ccr sessions list

# 仅显示 Claude 会话
ccr sessions list --platform claude

# 显示今天的会话
ccr sessions list --today

# 显示最近 50 个会话
ccr sessions list --limit 50
```

### search

搜索会话。

```bash
ccr sessions search <QUERY> [OPTIONS]
```

**参数：**

| 参数 | 说明 |
|------|------|
| `<QUERY>` | 搜索关键词 |

**选项：**

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `-p, --platform <PLATFORM>` | 按平台过滤 | 全部 |
| `-l, --limit <N>` | 限制结果数量 | 10 |

**示例：**

```bash
# 搜索包含 "refactoring" 的会话
ccr sessions search "refactoring"

# 在 Claude 会话中搜索
ccr sessions search "bug fix" --platform claude
```

### show

查看会话详情。

```bash
ccr sessions show <SESSION_ID>
```

**输出信息：**

- Session ID
- 平台
- 标题
- 工作目录
- 文件路径
- 创建/更新时间
- 消息统计（用户/助手/工具调用）
- 恢复命令

### resume

生成恢复会话的命令。

```bash
ccr sessions resume <SESSION_ID> [OPTIONS]
```

**选项：**

| 选项 | 说明 |
|------|------|
| `--dry-run` | 仅打印命令，不执行 |

**示例：**

```bash
# 生成恢复命令
ccr sessions resume abc123

# 仅打印恢复命令
ccr sessions resume abc123 --dry-run
```

### reindex

重建会话索引。

```bash
ccr sessions reindex [OPTIONS]
```

**选项：**

| 选项 | 说明 |
|------|------|
| `--force` | 强制重建（清空现有索引） |
| `-p, --platform <PLATFORM>` | 仅索引指定平台 |

**示例：**

```bash
# 增量更新索引
ccr sessions reindex

# 强制完全重建
ccr sessions reindex --force

# 仅重建 Claude 索引
ccr sessions reindex --platform claude
```

### stats

显示索引统计信息。

```bash
ccr sessions stats
```

**输出：**

- 总会话数
- 按平台分类统计

### prune

清理过期会话记录（文件已删除的）。

```bash
ccr sessions prune [OPTIONS]
```

**选项：**

| 选项 | 说明 |
|------|------|
| `--confirm` | 跳过确认提示 |

## 数据存储

会话索引存储在 SQLite 数据库中：

```
~/.ccr/data.db
```

首次运行 `ccr sessions` 命令时会自动创建数据库并执行初始索引。

## 使用场景

### 快速恢复中断的会话

```bash
# 查看今天的会话
ccr sessions list --today

# 找到目标会话并恢复
ccr sessions resume abc123
```

### 搜索历史对话

```bash
# 搜索曾经讨论过的主题
ccr sessions search "authentication"

# 查看详情
ccr sessions show <session_id>
```

### 定期维护

```bash
# 清理无效记录
ccr sessions prune --confirm

# 重建索引（遇到问题时）
ccr sessions reindex --force
```

## 版本信息

- **引入版本**: v3.12.0
- **依赖特性**: 需要 SQLite 支持（默认启用）
