# Tauri Commands API

本小姐为你整理的完整 Tauri Command API 参考！(￣▽￣)／

## 配置管理

### list_configs

列出所有配置信息。

**函数签名**
```rust
#[tauri::command]
pub async fn list_configs() -> Result<ConfigList, String>
```

**返回类型**
```typescript
interface ConfigList {
  current_config: string;
  default_config: string;
  configs: ConfigInfo[];
}

interface ConfigInfo {
  name: string;
  description: string;
  base_url: string | null;
  auth_token: string | null;
  model: string | null;
  small_fast_model: string | null;
  is_current: boolean;
  is_default: boolean;
  provider: string | null;
  provider_type: string | null;  // "official_relay" | "third_party_model"
  account: string | null;
  tags: string[] | null;
}
```

**前端调用**
```typescript
import { listConfigs } from '@/api'

const configList = await listConfigs()
console.log(configList.configs)  // 所有配置
console.log(configList.current_config)  // 当前配置名
```

---

### get_current_config

获取当前激活的配置信息。

**函数签名**
```rust
#[tauri::command]
pub async fn get_current_config() -> Result<ConfigInfo, String>
```

**返回类型**
```typescript
ConfigInfo | null
```

**前端调用**
```typescript
import { getCurrentConfig } from '@/api'

const current = await getCurrentConfig()
if (current) {
  console.log(`当前配置: ${current.name}`)
}
```

---

### get_config

获取指定配置的详细信息。

**函数签名**
```rust
#[tauri::command]
pub async fn get_config(name: String) -> Result<ConfigInfo, String>
```

**参数**
- `name`: 配置名称

**返回类型**
```typescript
ConfigInfo
```

**前端调用**
```typescript
import { get_config } from '@/api'

const config = await get_config('my-config')
console.log(`配置描述: ${config.description}`)
```

---

### switch_config

切换到指定配置。

**函数签名**
```rust
#[tauri::command]
pub async fn switch_config(name: String) -> Result<String, String>
```

**参数**
- `name`: 要切换到的配置名称

**返回**
- 成功消息字符串

**前端调用**
```typescript
import { switchConfig } from '@/api'

try {
  const message = await switchConfig('anthropic')
  console.log(message)  // "✅ 成功切换到配置: anthropic"
} catch (error) {
  console.error('切换失败:', error)
}
```

**错误处理**
```typescript
// 配置不存在
Error: "配置 xxx 不存在"

// 权限不足
Error: "无法写入 settings.json"
```

---

### create_config

创建新配置。

**函数签名**
```rust
#[tauri::command]
pub async fn create_config(
    name: String,
    description: Option<String>,
    base_url: Option<String>,
    auth_token: Option<String>,
    model: Option<String>,
    small_fast_model: Option<String>,
    provider: Option<String>,
    provider_type: Option<String>,
    account: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<String, String>
```

**参数**
- `name` **(必需)**: 配置名称
- `description`: 配置描述
- `base_url` **(必需)**: API 基础 URL
- `auth_token` **(必需)**: 认证令牌
- `model`: 默认模型名称
- `small_fast_model`: 快速小模型名称
- `provider`: 提供商名称
- `provider_type`: 提供商类型 (`official_relay` | `third_party_model`)
- `account`: 账号标识
- `tags`: 标签数组

**前端调用**
```typescript
import { createConfig } from '@/api'

await createConfig({
  name: 'my-config',
  description: '我的配置',
  base_url: 'https://api.example.com',
  auth_token: 'sk-xxx',
  model: 'claude-3-opus',
  provider_type: 'official_relay',
  tags: ['production', 'high-speed']
})
```

---

### update_config

更新已有配置。

**函数签名**
```rust
#[tauri::command]
pub async fn update_config(
    old_name: String,
    new_name: String,
    description: Option<String>,
    base_url: Option<String>,
    auth_token: Option<String>,
    model: Option<String>,
    small_fast_model: Option<String>,
    provider: Option<String>,
    provider_type: Option<String>,
    account: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<String, String>
```

**参数**
- `old_name` **(必需)**: 原配置名称
- `new_name` **(必需)**: 新配置名称
- 其他参数同 `create_config`

**前端调用**
```typescript
import { updateConfig } from '@/api'

await updateConfig({
  old_name: 'my-config',
  new_name: 'my-config-v2',
  description: '更新后的配置',
  // ... 其他字段
})
```

---

### delete_config

删除指定配置。

**函数签名**
```rust
#[tauri::command]
pub async fn delete_config(name: String) -> Result<String, String>
```

**参数**
- `name`: 要删除的配置名称

**前端调用**
```typescript
import { deleteConfig } from '@/api'

try {
  const message = await deleteConfig('old-config')
  console.log(message)
} catch (error) {
  console.error('删除失败:', error)
}
```

---

### import_config

导入配置文件内容。

**函数签名**
```rust
#[tauri::command]
pub async fn import_config(
    content: String,
    merge: bool,
    backup: bool,
) -> Result<String, String>
```

**参数**
- `content`: 配置文件内容 (TOML 格式)
- `merge`: 是否合并模式 (true: 合并, false: 替换)
- `backup`: 是否备份现有配置

**前端调用**
```typescript
import { importConfig } from '@/api'

// 从文件读取内容
const fileContent = await readFile('config.toml')
await importConfig(fileContent, true, true)
```

---

### export_config

导出配置为 TOML 格式。

**函数签名**
```rust
#[tauri::command]
pub async fn export_config(include_secrets: bool) -> Result<String, String>
```

**参数**
- `include_secrets`: 是否包含敏感信息 (auth_token 等)

**前端调用**
```typescript
import { exportConfig } from '@/api'

// 导出完整配置 (包含敏感信息)
const fullConfig = await exportConfig(true)

// 导出安全配置 (不包含敏感信息)
const safeConfig = await exportConfig(false)
```

---

### validate_all

验证所有配置的有效性。

**函数签名**
```rust
#[tauri::command]
pub async fn validate_all() -> Result<String, String>
```

**返回**
- 验证报告字符串

**前端调用**
```typescript
import { validateAll } from '@/api'

const report = await validateAll()
console.log(report)
// 输出示例:
// "验证结果: 5/5 配置有效
//  ✅ anthropic - 官方中转
//  ✅ openai - 第三方模型
//  ..."
```

## 历史记录

### get_history

获取操作历史记录。

**函数签名**
```rust
#[tauri::command]
pub async fn get_history(limit: Option<usize>) -> Result<Vec<HistoryEntry>, String>
```

**参数**
- `limit`: 限制返回条数 (默认 50)

**返回类型**
```typescript
interface HistoryEntry {
  id: string;
  timestamp: string;
  operation: string;
  from_config: string | null;
  to_config: string | null;
  actor: string;
}
```

**前端调用**
```typescript
import { getHistory } from '@/api'

// 获取最近 50 条记录
const history = await getHistory()

// 获取最近 10 条记录
const recentHistory = await getHistory(10)
```

## 备份管理

### list_backups

列出所有备份文件。

**函数签名**
```rust
#[tauri::command]
pub async fn list_backups() -> Result<Vec<BackupInfo>, String>
```

**返回类型**
```typescript
interface BackupInfo {
  filename: string;
  path: string;
  created_at: string;
  size: number;
}
```

**前端调用**
```typescript
import { listBackups } from '@/api'

const backups = await listBackups()
backups.forEach(backup => {
  console.log(`${backup.filename} (${backup.size} bytes)`)
})
```

---

### restore_backup

从备份恢复配置。

**函数签名**
```rust
#[tauri::command]
pub async fn restore_backup(backup_path: String) -> Result<String, String>
```

**参数**
- `backup_path`: 备份文件路径

**前端调用**
```typescript
import { restoreBackup } from '@/api'

try {
  const message = await restoreBackup('/path/to/backup.toml')
  console.log(message)
} catch (error) {
  console.error('恢复失败:', error)
}
```

## 系统信息

### get_system_info

获取系统信息。

**函数签名**
```rust
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String>
```

**返回类型**
```typescript
interface SystemInfo {
  hostname: string;
  username: string;
  os: string;
  config_path: string;
  settings_path: string;
  backups_path: string;
}
```

**前端调用**
```typescript
import { getSystemInfo } from '@/api'

const info = await getSystemInfo()
console.log(`系统: ${info.os}`)
console.log(`配置路径: ${info.config_path}`)
```

## 错误处理

所有命令都可能返回错误，前端应该适当处理：

```typescript
import { switchConfig } from '@/api'

try {
  await switchConfig('non-existent')
} catch (error) {
  // 错误是字符串类型
  console.error('操作失败:', error)
  
  // 可以根据错误内容显示不同的提示
  if (error.includes('不存在')) {
    showError('配置不存在')
  } else if (error.includes('权限')) {
    showError('权限不足')
  } else {
    showError('未知错误')
  }
}
```

## 类型定义

完整的 TypeScript 类型定义位于 `src-ui/src/types/index.ts`:

```typescript
// 配置信息
export interface ConfigInfo {
  name: string
  description: string
  base_url: string | null
  auth_token: string | null
  model: string | null
  small_fast_model: string | null
  is_current: boolean
  is_default: boolean
  provider: string | null
  provider_type: string | null
  account: string | null
  tags: string[] | null
}

// 创建配置请求
export interface CreateConfigRequest {
  name: string
  description?: string
  base_url?: string
  auth_token?: string
  model?: string
  small_fast_model?: string
  provider?: string
  provider_type?: string
  account?: string
  tags?: string[]
}

// 更新配置请求
export interface UpdateConfigRequest {
  old_name: string
  new_name: string
  description?: string
  base_url?: string
  auth_token?: string
  model?: string
  small_fast_model?: string
  provider?: string
  provider_type?: string
  account?: string
  tags?: string[]
}
```

**参数**
- `name`: 要删除的配置名称

**前端调用**
```typescript
await deleteConfig('old-config')
```

::: danger 警告
删除操作不可恢复！建议在删除前先导出配置备份。
不能删除当前激活的配置和默认配置。
:::

---

## 配置导入导出

### import_config

导入配置文件。

**函数签名**
```rust
#[tauri::command]
pub async fn import_config(
    content: String,
    merge: bool,
    backup: bool
) -> Result<String, String>
```

**参数**
- `content`: TOML 格式的配置文件内容
- `merge`: 是否合并模式（`true`: 合并, `false`: 替换）
- `backup`: 是否在导入前备份

**返回**
```
"✅ 导入完成！新增: 3, 更新: 2, 跳过: 1"
```

**前端调用**
```typescript
const tomlContent = `...`
const result = await importConfig(tomlContent, true, true)
console.log(result)
```

---

### export_config

导出配置文件。

**函数签名**
```rust
#[tauri::command]
pub async fn export_config(include_secrets: bool) -> Result<String, String>
```

**参数**
- `include_secrets`: 是否包含敏感信息（API 密钥）

**返回**
- TOML 格式的配置文件内容

**前端调用**
```typescript
const content = await exportConfig(true)
// 创建下载
const blob = new Blob([content], { type: 'text/plain' })
const url = URL.createObjectURL(blob)
// ... 触发下载
```

---

## 历史记录

### get_history

获取操作历史记录。

**函数签名**
```rust
#[tauri::command]
pub async fn get_history(limit: Option<usize>) -> Result<Vec<HistoryEntry>, String>
```

**参数**
- `limit`: 返回条数限制（默认 50）

**返回类型**
```typescript
interface HistoryEntry {
  id: string;
  timestamp: string;
  operation: string;
  from_config: string | null;
  to_config: string | null;
  actor: string;
}
```

**前端调用**
```typescript
const history = await getHistory(100)
history.forEach(entry => {
  console.log(`[${entry.timestamp}] ${entry.operation} by ${entry.actor}`)
})
```

---

## 备份管理

### list_backups

列出所有备份文件。

**函数签名**
```rust
#[tauri::command]
pub async fn list_backups() -> Result<Vec<BackupInfo>, String>
```

**返回类型**
```typescript
interface BackupInfo {
  filename: string;
  path: string;
  created_at: string;
  size: number;
}
```

**前端调用**
```typescript
const backups = await listBackups()
backups.forEach(backup => {
  console.log(`${backup.filename} - ${backup.size} bytes`)
})
```

---

### restore_backup

从备份恢复配置。

**函数签名**
```rust
#[tauri::command]
pub async fn restore_backup(backup_path: String) -> Result<String, String>
```

**参数**
- `backup_path`: 备份文件的完整路径

**前端调用**
```typescript
await restoreBackup('/path/to/backup.json')
```

---

## 系统信息

### get_system_info

获取系统信息。

**函数签名**
```rust
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String>
```

**返回类型**
```typescript
interface SystemInfo {
  hostname: string;
  username: string;
  os: string;
  config_path: string;
  settings_path: string;
  backups_path: string;
}
```

**前端调用**
```typescript
const sysInfo = await getSystemInfo()
console.log(`系统: ${sysInfo.os}`)
console.log(`用户: ${sysInfo.username}@${sysInfo.hostname}`)
```

---

## 错误处理

所有 Command 都返回 `Result<T, String>`，错误会以字符串形式返回。

**通用错误处理模式**
```typescript
try {
  const result = await someCommand()
  // 处理成功结果
} catch (error) {
  // error 是 string 类型
  console.error('操作失败:', error)
  showNotification(error, 'error')
}
```

**常见错误类型**
```typescript
// 配置不存在
"配置 xxx 不存在"

// 文件权限错误
"无法写入 ~/.claude/settings.json"

// 验证失败
"base_url 必须以 http:// 或 https:// 开头"

// 文件锁定
"配置文件正在被其他进程使用"
```

---

## 性能优化

### 并行调用

当需要获取多个独立数据时，使用 `Promise.all` 并行调用：

```typescript
const [configs, history, sysInfo] = await Promise.all([
  listConfigs(),
  getHistory(50),
  getSystemInfo()
])
```

### 节流与防抖

对于频繁触发的操作（如搜索、筛选），使用节流或防抖：

```typescript
import { debounce } from 'lodash-es'

const debouncedSearch = debounce(async (query: string) => {
  const configs = await listConfigs()
  // 筛选逻辑...
}, 300)
```

---

**Made with ❤️ by 哈雷酱**

哼，这份 API 文档可是本小姐精心整理的呢！(￣▽￣)／
所有参数类型、返回值、错误处理都写得清清楚楚，笨蛋你要是还看不懂的话...(,,><,,)
