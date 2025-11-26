# 技能管理 (skills)

管理 CCR 的技能系统，包括技能安装、卸载、仓库管理和扫描。

**版本支持**: v3.5.0+

**命令格式**：
```bash
ccr skills <ACTION> [OPTIONS]
```

## 子命令

### List - 列出已安装技能

查看当前平台的所有已安装技能。

**用法**：
```bash
ccr skills list
```

**示例**：
```bash
# 列出 Claude 平台的技能
ccr skills list

# 指定 Codex 平台
ccr skills list --platform codex
```

### Install - 安装技能

从远程仓库安装指定的技能。

**用法**：
```bash
ccr skills install <NAME> [OPTIONS]
```

**示例**：
```bash
# 安装一个技能（需要先通过 scan 发现）
ccr skills install my-skill

# 指定平台
ccr skills install my-skill --platform claude
```

### Uninstall - 卸载技能

卸载已安装的技能。

**用法**：
```bash
ccr skills uninstall <NAME> [OPTIONS]
```

**示例**：
```bash
# 卸载技能
ccr skills uninstall my-skill

# 指定平台
ccr skills uninstall my-skill --platform claude
```

### Repo - 仓库管理

管理技能仓库，包括添加、列出和删除仓库。

**子命令**：

#### Add - 添加仓库

添加新的技能仓库。

```bash
ccr skills repo add <NAME> <URL>
```

**示例**：
```bash
# 添加官方仓库
ccr skills repo add official https://github.com/ccr-skills/official

# 添加社区仓库
ccr skills repo add community https://github.com/ccr-skills/community
```

#### List - 列出仓库

查看所有配置的技能仓库。

```bash
ccr skills repo list
```

#### Remove - 删除仓库

删除指定的仓库。

```bash
ccr skills repo remove <NAME>
```

### Scan - 扫描仓库中的技能

扫描指定仓库中可用的技能。

**用法**：
```bash
ccr skills scan <REPO_NAME>
```

**示例**：
```bash
# 扫描官方仓库
ccr skills scan official

# 扫描社区仓库
ccr skills scan community
```

**注意**：扫描后可以看到可用的技能名称，然后使用 `install` 命令安装。

## 选项

### `--platform <PLATFORM>`

指定平台（claude、codex、gemini、qwen），默认为 claude。

**示例**：
```bash
ccr skills list --platform codex
ccr skills install my-skill --platform gemini
```

## 使用示例

### 完整工作流程

```bash
# 1. 添加技能仓库
ccr skills repo add official https://github.com/ccr-skills/official
ccr skills repo add community https://github.com/ccr-skills/community

# 2. 查看所有仓库
ccr skills repo list

# 3. 扫描仓库发现可用技能
ccr skills scan official
ccr skills scan community

# 4. 安装需要的技能
ccr skills install code-review --platform claude
ccr skills install doc-generator --platform claude

# 5. 查看已安装技能
ccr skills list

# 6. 卸载不需要的技能
ccr skills uninstall old-skill
```

### 多平台技能管理

```bash
# Claude 平台
ccr skills list --platform claude

# Codex 平台
ccr skills install git-helper --platform codex

# Gemini 平台
ccr skills list --platform gemini
```

## 技能存储位置

技能文件存储在各平台的配置目录中：

- Claude: `~/.claude/skills/`
- Codex: `~/.codex/skills/`
- Gemini: `~/.gemini/skills/`

## 技术实现

技能管理使用以下核心模块：

- `Managers::SkillsManager` - 技能管理器
- `Models::Skill` - 技能数据模型
- `Models::SkillRepository` - 技能仓库模型

## 最佳实践

1. **从可信源添加仓库**：只添加官方或可信社区仓库
2. **定期扫描仓库**：仓库更新后重新扫描以发现新技能
3. **按需安装**：只安装需要的技能，避免配置臃肿
4. **平台区分**：不同平台的技能通常不通用，注意选择正确的平台

## 故障排除

### 安装失败

如果技能安装失败：
1. 确认仓库已正确添加：`ccr skills repo list`
2. 确认技能存在于仓库：`ccr skills scan <repo>`
3. 检查网络连接

### 技能不生效

如果技能安装后不生效：
1. 确认技能已成功安装：`ccr skills list`
2. 检查 Claude Code 是否已重启
3. 检查平台配置是否正确：`ccr current`

## 相关命令

- [version](./version) - 查看版本信息
- [commands reference](./) - 所有命令概览
