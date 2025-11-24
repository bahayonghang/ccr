# 提示词模板管理 (prompts)

管理提示词预设模板，快速应用常用的提示词配置到不同目标文件。

**版本支持**: v3.5.0+

**命令格式**：
```bash
ccr prompts <ACTION> [OPTIONS]
```

## 子命令

### List - 列出所有预设

查看当前平台的所有提示词预设。

**用法**：
```bash
ccr prompts list
```

**示例**：
```bash
# 列出 Claude 平台的预设
ccr prompts list

# 指定 Codex 平台
ccr prompts list --platform codex
```

**输出示例**：
```
┌──────────────┬─────────┬──────────────────┬──────┐
│ Name         │ Target  │ Description      │ Tags │
├──────────────┼─────────┼──────────────────┼──────┤
│ code-review  │ claude  │ Code review task │      │
│ doc-gen      │ agents  │ Documentation    │      │
└──────────────┴─────────┴──────────────────┴──────┘
```

### Add - 添加预设

创建新的提示词预设。

**用法**：
```bash
ccr prompts add <NAME> [OPTIONS]
```

**必需参数**：
- `--target <TARGET>` - 目标文件（claude/agents/gemini）
- `--content <CONTENT>` - 提示词内容，支持 `@file` 语法从文件读取
- `--description <DESC>` - 描述（可选）

**示例**：

```bash
# 1. 直接指定内容
ccr prompts add code-review \
  --target claude \
  --content "Please review this code for security issues and best practices." \
  --description "Code review prompt"

# 2. 从文件读取内容（使用 @ 前缀）
ccr prompts add complex-task \
  --target agents \
  --content @/path/to/prompt.txt \
  --description "Complex task definition"

# 3. 多行内容（使用引号）
ccr prompts add doc-generator \
  --target claude \
  --content "Generate documentation following these rules: 1) Clear 2) Concise 3) With examples" \
  --description "Documentation generation"
```

### Apply - 应用预设

将预设应用到目标文件。

**用法**：
```bash
ccr prompts apply <NAME>
```

**示例**：
```bash
# 应用 code-review 预设
ccr prompts apply code-review

# 应用到指定平台
ccr prompts apply doc-generator --platform claude
```

**注意**：应用预设前会自动创建备份文件（`.backup` 后缀）。

### Show - 显示预设内容

查看预设的详细信息。

**用法**：
```bash
ccr prompts show <NAME>
```

**示例**：
```bash
# 查看预设内容
ccr prompts show code-review
```

**输出示例**：
```
Preset: code-review
Description: Code review prompt
Target: claude

--- Content ---
Please review this code for security issues and best practices.
Focus on:
1. Security vulnerabilities
2. Performance issues
3. Code style consistency
```

### Remove - 删除预设

删除指定的提示词预设。

**用法**：
```bash
ccr prompts remove <NAME>
```

**示例**：
```bash
# 删除预设
ccr prompts remove old-prompt
```

### Current - 查看当前内容

显示目标文件的当前内容。

**用法**：
```bash
ccr prompts current <TARGET>
```

**示例**：
```bash
# 查看 claude 配置文件的当前提示词
ccr prompts current claude

# 查看 agents 配置
ccr prompts current agents

# 查看 gemini 配置
ccr prompts current gemini
```

## 选项

### `--platform <PLATFORM>`

指定平台（claude、codex、gemini、qwen），默认为 claude。

**示例**：
```bash
ccr prompts list --platform codex
ccr prompts apply my-prompt --platform gemini
```

## 使用示例

### 完整工作流程：创建和应用提示词模板

```bash
# 1. 创建代码审查预设（从文件读取）
ccr prompts add code-review \
  --target claude \
  --content @~/prompts/code-review.txt \
  --description "Code review template with security focus"

# 2. 查看所有预设
ccr prompts list

# 3. 显示预设内容确认
ccr prompts show code-review

# 4. 应用到 Claude 配置
ccr prompts apply code-review

# 5. 验证应用结果
ccr prompts current claude

# 6. 创建另一个预设：文档生成
cat > /tmp/doc-prompt.txt << 'EOF'
Generate comprehensive documentation for the provided code:
1. Function purpose and parameters
2. Return values and types
3. Usage examples
4. Edge cases and error handling
EOF

cr prompts add doc-generator \
  --target claude \
  --content @/tmp/doc-prompt.txt \
  --description "Documentation generation template"

# 7. 快速切换到文档生成模式
ccr prompts apply doc-generator
```

### 多平台提示词管理

```bash
# Claude 平台
ccr prompts list --platform claude
ccr prompts add task-breakdown --target claude --content "Break down this task into steps"

# Codex 平台
ccr prompts list --platform codex
ccr prompts add github-review --target claude --platform codex --content "Review this PR"

# Gemini 平台
ccr prompts current gemini --platform gemini
```

### 创建常用预设集合

```bash
# 开发辅助预设
cat > /tmp/dev-prompt.txt << 'EOF'
You are an expert Rust developer. Please:
1. Analyze the code for bugs
2. Suggest performance improvements
3. Check for idiomatic Rust patterns
4. Review error handling
EOF

cr prompts add rust-expert \
  --target claude \
  --content @/tmp/dev-prompt.txt \
  --description "Rust code review expert"

# 测试生成预设
cat > /tmp/test-prompt.txt << 'EOF'
Generate comprehensive unit tests for the following code:
1. Happy path tests
2. Edge case tests
3. Error condition tests
4. Integration test examples
EOF

cr prompts add test-generator \
  --target claude \
  --content @/tmp/test-prompt.txt \
  --description "Unit test generation template"
```

## 目标文件说明

`-t, --target <TARGET>` 支持的值：

- **claude** - Claude Code 的主配置文件（`~/.claude/settings.json`）
- **agents** - Agents 定义文件（`~/.claude/agents/` 目录）
- **gemini** - Gemini CLI 的配置文件（暂未完全支持）

## 预设存储位置

预设文件存储在各平台的配置目录中：

- **Claude**: `~/.claude/prompts/`
- **Codex**: `~/.codex/prompts/`
- **Gemini**: `~/.gemini/prompts/`

每个预设以独立的 JSON 文件存储，格式如下：

```json
{
  "name": "code-review",
  "description": "Code review template",
  "target_file": "claude",
  "content": "Review code for security and performance...",
  "tags": []
}
```

## 技术实现

提示词管理使用以下核心模块：

- `Managers::PromptsManager` - 提示词管理器
- `Models::PromptPreset` - 预设数据模型
- `Models::PromptTarget` - 目标文件枚举

## 最佳实践

1. **模块化设计**：为不同场景创建独立预设（代码审查、文档生成、测试等）
2. **版本管理**：对重要预设使用描述字段记录版本信息
3. **备份习惯**：应用预设前会自动备份，但也建议手动备份重要配置
4. **复用内容**：复杂提示词存储在文件中，使用 `@file` 语法引用
5. **定期整理**：删除不再使用的过期预设

## 故障排除

### 预设应用失败

如果应用预设失败：
1. 确认预设存在：`ccr prompts show <name>`
2. 确认目标文件路径正确
3. 检查文件权限

### 从文件读取内容失败

如果使用 `@file` 语法失败：
1. 确认文件路径正确
2. 确认有读取权限
3. 检查文件编码（应为 UTF-8）

### 平台不匹配

确保预设的目标文件与平台匹配：
- `--target claude` 应配合 `--platform claude`
- `--target agents` 应配合 `--platform claude`

## 注意事项

- 应用预设会**覆盖**目标文件的现有内容
- 重要配置建议先备份：`ccr prompts current <target> > backup.txt`
- 预设名称必须唯一
- 同一预设可以应用到不同平台的不同目标文件

## 相关命令

- [skills](./skills) - 技能管理
- [version](./version) - 查看版本信息
- [commands reference](./) - 所有命令概览
