# run - 临时运行命令

使用指定配置的环境变量，在一个隔离的子进程中临时运行 AI CLI 工具（如 Claude Code、Codex）或其他命令，而不修改全局或默认的配置文件。

## 用法

```bash
ccr run <config_name> -- <command_and_args>
```

## 参数

- `<config_name>`: 要使用的配置名称（必需）
- `<command_and_args>`: `[--]` 之后的命令和参数将以子进程执行

## 执行流程

1. 🔍 **查找配置关联的平台**: 在所有平台中查找指定配置。
2. 🧮 **准备隔离环境变量**: 提取该配置对应的环境变量（如 `ANTHROPIC_API_KEY` 等），并自动脱敏打印。
3. 🚀 **执行命令**: 使用提取出的环境变量启动子进程并转发标准输入输出。

## 示例

```bash
# 使用名为 anyrouter 的配置临时运行 Claude Code，并输出 Hello
ccr run anyrouter -- claude -p "Hello"

# 使用 my_codex 配置临时运行 Codex 
ccr run my_codex -- codex
```

## 输出效果

```
临时运行配置: anyrouter

步骤 1/3: 查找配置关联的平台
✅ 找到配置 'anyrouter' 属于平台: claude

步骤 2/3: 准备隔离环境变量
  ANTHROPIC_BASE_URL = https://api.anyrouter.ai/v1
  ANTHROPIC_AUTH_TOKEN = sk-a...cdef
  ANTHROPIC_MODEL = claude-3-5-sonnet-20241022
  ANTHROPIC_SMALL_FAST_MODEL = claude-3-5-haiku-20241022

步骤 3/3: 执行命令 `claude -p Hello' ...
```

## 核心优势

- **无副作用**: 不像 `switch` 会修改全局的 `settings.json`，`run` 命令通过仅仅设置子进程的环境变量来实现环境隔离。
- **并发安全**: 可以在多个终端窗口中同时使用不同的大模型配置运行 Claude Code，且互不干扰。
- **安全脱敏**: 打印准备注入的环境变量时，`TOKEN` 或 `KEY` 类别的敏感信息会被自动遮罩。

## 错误处理

### 配置不存在

如果在所有平台的配置中都找不到指定的 `config_name`，将报错提示：

```bash
$ ccr run nonexistent -- claude
Error: 在所有平台中均未找到配置 'nonexistent'
💡 提示:
  • 运行 'ccr list' 查看可用配置
```

## 相关命令

- [switch](./switch) - 全局切换配置
- [list](./list) - 查看可用配置
- [current](./current) - 查看当前配置
