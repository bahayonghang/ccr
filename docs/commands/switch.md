# switch - 切换配置

`switch` 命令用于切换 Claude Code 的 API 配置，包含完整的验证、备份和历史记录功能。

## 📖 命令格式

```bash
ccr switch <config_name>

# 简写形式
ccr <config_name>
```

## 📝 参数

### 必需参数

- `<config_name>` - 要切换到的配置名称

## 💡 使用示例

### 基本使用

```bash
# 完整命令
ccr switch anthropic

# 简写形式（推荐）
ccr anthropic

# 切换到其他配置
ccr anyrouter
ccr openrouter
```

### 查看可用配置

```bash
# 先列出所有配置
ccr list

# 然后切换到目标配置
ccr switch <config_name>
```

## 🔄 执行流程

### 完整流程（5个步骤）

```
步骤 1/5: 读取配置文件
✓ 目标配置 'anyrouter' 验证通过

步骤 2/5: 备份当前设置
✓ 设置已备份: /home/user/.claude/backups/settings.anyrouter.20250110_143022.json.bak

步骤 3/5: 更新 Claude Code 设置
✓ Claude Code 设置已更新

步骤 4/5: 更新配置文件
✓ 当前配置已设置为: anyrouter

步骤 5/5: 记录操作历史
✓ 操作历史已记录

──────────────────────────────────────────────────────────────

配置切换成功

  配置名称: anyrouter
  描述: AnyRouter 代理服务
  Base URL: https://api.anyrouter.ai/v1
  Auth Token: your...here
  Model: claude-sonnet-4-5-20250929

✓ 配置已生效，Claude Code 可以使用新的 API 配置

ℹ 提示: 重启 Claude Code 以确保配置完全生效
```

### 流程详解

#### 步骤 1: 读取和验证配置

```rust
// 1.1 加载配置文件
let config_manager = ConfigManager::default()?;
let config = config_manager.load()?;

// 1.2 获取目标配置节
let target_section = config.get_section(config_name)?;

// 1.3 验证配置
target_section.validate()?;
```

**验证检查**:
- ✅ 配置节存在
- ✅ `base_url` 必填且格式正确
- ✅ `auth_token` 必填且非空
- ✅ `model` 如果提供则非空

#### 步骤 2: 备份当前设置

```rust
let backup_path = if settings_manager.settings_path().exists() {
    let path = settings_manager.backup(Some(config_name))?;
    Some(path.display().to_string())
} else {
    None  // 首次使用，跳过备份
};
```

**备份文件命名**:
```
settings.{config_name}.{timestamp}.json.bak

示例:
settings.anyrouter.20250110_143022.json.bak
settings.anthropic.20250110_150530.json.bak
```

**备份位置**:
```
~/.claude/backups/
```

#### 步骤 3: 更新 Claude Code 设置

```rust
// 3.1 加载现有设置
let old_settings = settings_manager.load().ok();
let mut new_settings = old_settings.unwrap_or_default();

// 3.2 更新环境变量
new_settings.update_from_config(&target_section);

// 3.3 原子保存
settings_manager.save_atomic(&new_settings)?;
```

**环境变量更新**:
```rust
// 清空旧的 ANTHROPIC_* 变量
self.env.retain(|key, _| !key.starts_with("ANTHROPIC_"));

// 设置新变量
ANTHROPIC_BASE_URL       ← section.base_url
ANTHROPIC_AUTH_TOKEN     ← section.auth_token
ANTHROPIC_MODEL          ← section.model (可选)
ANTHROPIC_SMALL_FAST_MODEL ← section.small_fast_model (可选)
```

#### 步骤 4: 更新配置指针

```rust
// 更新 current_config 字段
config.set_current(config_name)?;

// 保存配置文件
config_manager.save(&config)?;
```

**效果**:
```toml
# ~/.ccs_config.toml

# Before
current_config = "anthropic"

# After
current_config = "anyrouter"
```

#### 步骤 5: 记录操作历史

```rust
let mut history_entry = HistoryEntry::new(
    OperationType::Switch,
    OperationDetails {
        from_config: Some("anthropic".into()),
        to_config: Some("anyrouter".into()),
        backup_path: Some(backup_path),
        extra: None,
    },
    OperationResult::Success,
);

// 记录环境变量变更（敏感信息已掩码）
history_entry.add_env_change(
    "ANTHROPIC_BASE_URL",
    Some("api.anthropic.com"),
    Some("api.anyrouter.ai/v1")
);

history_manager.add(history_entry)?;
```

## 🔒 安全保证

### 1. 文件锁保护

```rust
// 获取文件锁（超时 10 秒）
let _lock = self.lock_manager
    .lock_settings(Duration::from_secs(10))?;

// 执行写入操作
settings_manager.save_atomic(&settings)?;

// 锁自动释放
```

**保护内容**:
- 防止多进程同时写入 settings.json
- 防止配置文件损坏
- 超时保护避免死锁

### 2. 原子写入

```rust
// 使用临时文件 + 原子替换
let temp_file = NamedTempFile::new_in(parent_dir)?;
fs::write(temp_file.path(), content)?;
temp_file.persist(&settings_path)?;  // ⭐ 原子操作
```

**保证**:
- 要么完全成功
- 要么完全失败
- 不会出现部分写入

### 3. 自动备份

```rust
// 切换前自动备份
let backup_path = settings_manager.backup(Some(config_name))?;
```

**备份内容**:
- 当前的 `~/.claude/settings.json`
- 带时间戳和配置名称
- 可用于紧急恢复

## ⚠️ 错误处理

### 常见错误

#### 1. 配置不存在

```bash
$ ccr switch nonexistent

✗ 配置节 'nonexistent' 不存在
  建议: 运行 'ccr list' 查看可用配置
```

**退出码**: 12

#### 2. 配置验证失败

```bash
$ ccr switch broken-config

✗ 验证失败: base_url 不能为空
  建议: 运行 'ccr validate' 查看详细验证报告
```

**退出码**: 90

#### 3. 文件锁超时

```bash
$ ccr switch anthropic

✗ 获取文件锁超时: claude_settings
  建议: 可能有其他 ccr 进程正在运行，请稍后重试
```

**退出码**: 31

#### 4. 权限拒绝

```bash
$ ccr switch anthropic

✗ 权限拒绝: /home/user/.claude/settings.json
  建议: 请检查文件权限
```

**退出码**: 70

### 退出码列表

| 退出码 | 错误类型 | 说明 |
|--------|---------|------|
| `0` | 成功 | 配置切换成功 |
| `10` | ConfigError | 配置文件错误 |
| `11` | ConfigMissing | 配置文件不存在 |
| `12` | ConfigSectionNotFound | 配置节不存在 |
| `13` | ConfigFieldMissing | 配置字段缺失 |
| `14` | ConfigFormatInvalid | 配置格式无效 |
| `20` | SettingsError | 设置文件错误 |
| `21` | SettingsMissing | 设置文件不存在 |
| `30` | FileLockError | 文件锁错误 |
| `31` | LockTimeout | 获取锁超时 |
| `90` | ValidationError | 验证失败 |

## 🔍 验证前置条件

switch 命令会在执行前验证以下条件：

### 1. 配置文件存在

```bash
# 检查
ls -la ~/.ccs_config.toml

# 如果不存在，创建配置文件
# 参考: /installation/configuration
```

### 2. 目标配置存在

```bash
# 列出所有配置
ccr list

# 确保目标配置在列表中
```

### 3. 配置格式正确

```bash
# 验证配置
ccr validate

# 修复错误
vim ~/.ccs_config.toml
```

### 4. 目录权限

```bash
# 检查 .claude 目录权限
ls -la ~/.claude/

# 创建目录（如果不存在）
mkdir -p ~/.claude
chmod 700 ~/.claude
```

## 💡 最佳实践

### 1. 切换前验证

```bash
# 推荐流程
ccr validate        # 验证配置
ccr list            # 确认配置存在
ccr switch anyrouter  # 执行切换
```

### 2. 查看切换历史

```bash
# 切换后查看历史
ccr history --limit 5

# 查看所有切换操作
ccr history --filter-type switch
```

### 3. 备份管理

```bash
# 查看备份文件
ls -la ~/.claude/backups/

# 备份文件命名规则
# settings.{config_name}.{timestamp}.json.bak
```

### 4. 故障恢复

```bash
# 如果切换后发现问题
ccr switch previous-config  # 切换回去

# 或从备份恢复（手动）
cp ~/.claude/backups/settings.*.json.bak ~/.claude/settings.json
```

## 🌐 在 Web 界面中使用

### 通过 Web 界面切换

1. 启动 Web 服务器：
```bash
ccr web
```

2. 在浏览器中访问 http://localhost:8080

3. 点击目标配置的"切换"按钮

4. 确认切换操作

### API 调用

```javascript
// 使用 Fetch API
fetch('/api/switch', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ config_name: 'anyrouter' })
})
.then(response => response.json())
.then(data => {
    if (data.success) {
        console.log('切换成功');
    }
});
```

## 🔗 相关命令

- [list](/commands/list) - 列出所有配置
- [current](/commands/current) - 查看当前配置
- [validate](/commands/validate) - 验证配置
- [history](/commands/history) - 查看切换历史

## 📚 相关文档

- [配置文件格式](/installation/configuration)
- [数据流程](/architecture/data-flow)
- [错误处理](/api/errors)

