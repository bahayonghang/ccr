# 最佳实践

哼，想写出和本小姐一样优雅的代码？那就好好学习这些最佳实践吧！(￣▽￣)ゞ

## 架构原则

### KISS - Keep It Simple, Stupid

**原则：** 追求简洁，拒绝复杂。

**✅ 推荐做法：**

```typescript
// 简单直接
const filteredConfigs = computed(() => {
  if (filterType.value === 'all') return configs.value
  return configs.value.filter(c => c.provider_type === filterType.value)
})
```

**❌ 避免做法：**

```typescript
// 过度抽象
class ConfigFilterStrategyFactory {
  createStrategy(type: string): ConfigFilterStrategy {
    // 不必要的复杂性...
  }
}
```

::: tip 本小姐的建议
简单的代码才是最优雅的代码！笨蛋不要为了炫技而过度设计～ (￣▽￣)／
:::

---

### YAGNI - You Aren't Gonna Need It

**原则：** 只实现当前需要的功能。

**✅ 推荐做法：**

```rust
// 只实现必需的功能
#[tauri::command]
pub async fn switch_config(name: String) -> Result<String, String> {
    let service = ConfigService::default()?;
    service.switch_config(&name)?;
    Ok(format!("✅ 成功切换到配置: {}", name))
}
```

**❌ 避免做法：**

```rust
// 预留未来可能需要的功能
#[tauri::command]
pub async fn switch_config(
    name: String,
    backup: Option<bool>,           // 未来可能需要？
    validate: Option<bool>,         // 未来可能需要？
    rollback_on_failure: Option<bool>,  // 未来可能需要？
) -> Result<String, String> {
    // ...
}
```

::: warning 注意
预留的功能往往永远不会被使用，反而增加维护负担！
:::

---

### DRY - Don't Repeat Yourself

**原则：** 避免代码重复。

**✅ 推荐做法：**

```typescript
// 抽取公共逻辑
const handleApiCall = async <T>(
  apiCall: () => Promise<T>,
  errorMessage: string
): Promise<T | null> => {
  try {
    return await apiCall()
  } catch (error) {
    console.error(errorMessage, error)
    return null
  }
}

// 复用
const configs = await handleApiCall(listConfigs, '加载配置失败')
const history = await handleApiCall(() => getHistory(50), '加载历史失败')
```

**❌ 避免做法：**

```typescript
// 重复的错误处理
try {
  const configs = await listConfigs()
} catch (error) {
  console.error('加载配置失败:', error)
}

try {
  const history = await getHistory(50)
} catch (error) {
  console.error('加载历史失败:', error)
}
```

---

### SOLID 原则

#### S - Single Responsibility (单一职责)

每个函数/类只做一件事。

**✅ 推荐做法：**

```rust
// 每个函数职责单一
pub fn validate_base_url(url: &str) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("URL 必须以 http:// 或 https:// 开头".to_string());
    }
    Ok(())
}

pub fn validate_auth_token(token: &str) -> Result<(), String> {
    if token.len() < 10 {
        return Err("Token 长度不足".to_string());
    }
    Ok(())
}
```

**❌ 避免做法：**

```rust
// 一个函数做太多事
pub fn validate_and_save_and_backup_config(config: &Config) -> Result<(), String> {
    // 验证
    // 保存
    // 备份
    // 太多职责！
}
```

#### O - Open/Closed (开闭原则)

对扩展开放，对修改封闭。

**✅ 推荐做法：**

```typescript
// 使用策略模式扩展
type FilterStrategy = (config: ConfigInfo) => boolean

const filterStrategies: Record<FilterType, FilterStrategy> = {
  all: () => true,
  official_relay: (c) => c.provider_type === 'official_relay',
  third_party_model: (c) => c.provider_type === 'third_party_model',
  uncategorized: (c) => !c.provider_type
}

const filteredConfigs = computed(() => {
  const strategy = filterStrategies[filterType.value]
  return configs.value.filter(strategy)
})
```

#### L - Liskov Substitution (里氏替换)

子类型必须能够替换父类型。

#### I - Interface Segregation (接口隔离)

客户端不应依赖不需要的接口。

**✅ 推荐做法：**

```typescript
// 接口专一
interface ConfigReader {
  read(name: string): Promise<ConfigInfo>
}

interface ConfigWriter {
  write(name: string, config: ConfigInfo): Promise<void>
}

// 按需使用
class ConfigDisplay implements ConfigReader {
  async read(name: string): Promise<ConfigInfo> {
    // 只实现读取
  }
}
```

#### D - Dependency Inversion (依赖倒置)

依赖抽象而非具体实现。

**✅ 推荐做法：**

```rust
// 依赖 trait 抽象
pub trait ConfigManager {
    fn load(&self) -> Result<Config>;
    fn save(&self, config: &Config) -> Result<()>;
}

// 具体实现可以替换
pub struct TomlConfigManager;
pub struct JsonConfigManager;

impl ConfigManager for TomlConfigManager { /* ... */ }
impl ConfigManager for JsonConfigManager { /* ... */ }
```

---

## 代码规范

### Rust 代码规范

#### 命名规范

```rust
// ✅ 推荐
pub struct ConfigInfo { }         // PascalCase - 类型
pub fn switch_config() { }        // snake_case - 函数
const MAX_RETRIES: u32 = 3;       // SCREAMING_SNAKE_CASE - 常量

// ❌ 避免
pub struct configInfo { }         // 错误
pub fn SwitchConfig() { }         // 错误
const maxRetries: u32 = 3;        // 错误
```

#### 错误处理

```rust
// ✅ 使用 Result
pub fn load_config() -> Result<Config, CcrError> {
    let content = fs::read_to_string(path)
        .map_err(|e| CcrError::IoError(e))?;
    Ok(parse_config(&content)?)
}

// ✅ 使用自定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum CcrError {
    #[error("配置文件不存在: {0}")]
    ConfigNotFound(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}
```

#### 文档注释

```rust
/// 切换到指定配置
///
/// # Arguments
///
/// * `name` - 配置名称
///
/// # Returns
///
/// 成功返回确认消息，失败返回错误
///
/// # Examples
///
/// ```
/// let result = switch_config("anthropic".to_string());
/// assert!(result.is_ok());
/// ```
#[tauri::command]
pub async fn switch_config(name: String) -> Result<String, String> {
    // ...
}
```

---

### TypeScript 代码规范

#### 命名规范

```typescript
// ✅ 推荐
interface ConfigInfo { }          // PascalCase - 类型/接口
const currentConfig = ref()       // camelCase - 变量/函数
const MAX_RETRY_COUNT = 3         // SCREAMING_SNAKE_CASE - 常量

// ❌ 避免
interface configInfo { }          // 错误
const CurrentConfig = ref()       // 错误
const max_retry_count = 3         // 错误
```

#### 类型注解

```typescript
// ✅ 明确的类型注解
const configs = ref<ConfigInfo[]>([])
const currentConfig = ref<ConfigInfo | null>(null)

async function loadData(): Promise<void> {
  // ...
}

// ❌ 隐式 any
const configs = ref([])           // 类型不明确
```

#### 空值处理

```typescript
// ✅ 显式检查
if (currentConfig.value) {
  console.log(currentConfig.value.name)
}

// ✅ 可选链
console.log(currentConfig.value?.name)

// ❌ 假设非空
console.log(currentConfig.value.name)  // 可能报错！
```

---

### Vue 代码规范

#### 组件结构

```vue
<!-- ✅ 推荐的组件结构 -->
<script setup lang="ts">
// 1. 导入
import { ref, computed, onMounted } from 'vue'
import type { ConfigInfo } from '@/types'

// 2. Props & Emits
const props = defineProps<{
  config: ConfigInfo
}>()

const emit = defineEmits<{
  update: [config: ConfigInfo]
  delete: [name: string]
}>()

// 3. 响应式状态
const isEditing = ref(false)

// 4. 计算属性
const displayName = computed(() => props.config.name.toUpperCase())

// 5. 方法
const handleEdit = () => {
  isEditing.value = true
}

// 6. 生命周期
onMounted(() => {
  console.log('组件已挂载')
})
</script>

<template>
  <div class="config-card">
    <!-- 模板内容 -->
  </div>
</template>

<style scoped>
/* 作用域样式 */
.config-card {
  padding: 1rem;
}
</style>
```

#### Props 验证

```typescript
// ✅ 使用 TypeScript 类型
const props = defineProps<{
  config: ConfigInfo
  editable?: boolean
}>()

// ✅ 使用运行时验证
const props = defineProps({
  config: {
    type: Object as PropType<ConfigInfo>,
    required: true
  },
  editable: {
    type: Boolean,
    default: false
  }
})
```

---

## 安全实践

### 1. 输入验证

**前端验证：**

```typescript
const createConfig = async (formData: CreateConfigRequest) => {
  // ✅ 验证输入
  if (!formData.name.trim()) {
    throw new Error('配置名称不能为空')
  }

  if (formData.base_url && !isValidUrl(formData.base_url)) {
    throw new Error('URL 格式无效')
  }

  await createConfigApi(formData)
}

const isValidUrl = (url: string): boolean => {
  try {
    new URL(url)
    return url.startsWith('http://') || url.startsWith('https://')
  } catch {
    return false
  }
}
```

**后端验证：**

```rust
#[tauri::command]
pub async fn create_config(
    name: String,
    base_url: Option<String>,
) -> Result<String, String> {
    // ✅ 验证输入
    if name.trim().is_empty() {
        return Err("配置名称不能为空".to_string());
    }

    if let Some(url) = &base_url {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err("URL 必须以 http:// 或 https:// 开头".to_string());
        }
    }

    // 继续处理...
}
```

---

### 2. 敏感信息处理

**脱敏显示：**

```typescript
// ✅ 显示时脱敏
const maskToken = (token: string): string => {
  if (token.length <= 8) return '***'
  return `${token.slice(0, 4)}...${token.slice(-4)}`
}

console.log(maskToken('sk-1234567890abcdef'))  // sk-12...cdef
```

**日志脱敏：**

```rust
use log::info;

// ✅ 不记录敏感信息
info!("创建配置: name={}", config_name);

// ❌ 避免记录敏感信息
info!("创建配置: name={}, token={}", config_name, auth_token);  // 危险！
```

---

### 3. 文件权限

**检查文件权限：**

```rust
use std::fs::metadata;

// ✅ 检查文件是否可读
let metadata = metadata(path)
    .map_err(|e| format!("无法读取文件: {}", e))?;

if metadata.permissions().readonly() {
    return Err("文件只读，无法修改".to_string());
}
```

---

## 性能优化

### 1. 前端性能

#### 计算属性缓存

```typescript
// ✅ 使用 computed 缓存
const filteredConfigs = computed(() => {
  return configs.value.filter(c => c.provider_type === filterType.value)
})

// ❌ 方法每次调用都会重新计算
const getFilteredConfigs = () => {
  return configs.value.filter(c => c.provider_type === filterType.value)
}
```

#### 防抖和节流

```typescript
import { debounce } from 'lodash-es'

// ✅ 搜索防抖
const debouncedSearch = debounce((query: string) => {
  performSearch(query)
}, 300)
```

#### 虚拟滚动

对于长列表，使用虚拟滚动：

```typescript
// 使用 vue-virtual-scroller
import { RecycleScroller } from 'vue-virtual-scroller'
```

---

### 2. 后端性能

#### 避免重复读取

```rust
// ✅ 缓存读取结果
pub struct ConfigService {
    cached_config: Option<Config>,
}

impl ConfigService {
    pub fn get_config(&mut self) -> Result<&Config> {
        if self.cached_config.is_none() {
            self.cached_config = Some(self.load_config()?);
        }
        Ok(self.cached_config.as_ref().unwrap())
    }
}

// ❌ 每次都读取
pub fn get_config() -> Result<Config> {
    self.load_config()  // 重复 I/O
}
```

#### 使用异步操作

```rust
// ✅ 异步处理耗时操作
#[tauri::command]
pub async fn heavy_operation() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        // 耗时操作
        perform_heavy_work()
    }).await
        .map_err(|e| e.to_string())?
}
```

---

## Git 工作流

### 分支策略

```
main (production)
  │
  ├── develop (development)
  │     │
  │     ├── feature/add-validation
  │     ├── feature/improve-ui
  │     └── bugfix/fix-crash
  │
  └── hotfix/critical-bug
```

### Commit 规范

使用 **Conventional Commits** 规范：

```bash
# 格式
<type>(<scope>): <subject>

# 示例
feat(config): 添加配置验证功能
fix(ui): 修复配置卡片样式问题
docs(api): 更新 API 文档
refactor(commands): 重构 switch_config 逻辑
test(service): 添加 ConfigService 单元测试
```

**Type 类型：**

| Type     | 说明           | 示例                              |
| -------- | -------------- | --------------------------------- |
| feat     | 新功能         | feat: 添加配置导出功能             |
| fix      | Bug 修复       | fix: 修复配置切换失败问题          |
| docs     | 文档更新       | docs: 更新快速开始指南             |
| style    | 代码格式       | style: 格式化代码                  |
| refactor | 代码重构       | refactor: 重构配置加载逻辑         |
| perf     | 性能优化       | perf: 优化配置列表渲染性能         |
| test     | 测试           | test: 添加单元测试                 |
| chore    | 构建/工具变动  | chore: 更新依赖版本                |

### Pull Request 流程

1. **创建分支**
   ```bash
   git checkout -b feature/add-validation
   ```

2. **开发并提交**
   ```bash
   git add .
   git commit -m "feat(config): 添加配置验证功能"
   ```

3. **推送分支**
   ```bash
   git push -u origin feature/add-validation
   ```

4. **创建 PR**
   - 填写 PR 描述
   - 关联相关 Issue
   - 请求代码审查

5. **合并 PR**
   - 确保 CI 通过
   - 至少一人审查通过
   - 使用 Squash Merge

---

## 文档规范

### 1. 代码注释

**何时添加注释：**

```rust
// ✅ 解释 "为什么" 而非 "是什么"
// 使用原子写入避免文件损坏
self.write_atomic(&temp_path, &target_path)?;

// ❌ 无用的注释
// 读取配置文件
let config = self.read_config();
```

### 2. API 文档

每个 Tauri Command 都应有完整文档：

```markdown
### command_name

简短描述。

**函数签名**
\`\`\`rust
#[tauri::command]
pub async fn command_name(param: String) -> Result<Type, String>
\`\`\`

**参数**
- `param`: 参数描述

**返回**
- 成功返回的类型和说明
- 失败返回的错误信息

**前端调用**
\`\`\`typescript
const result = await commandName('value')
\`\`\`

**示例**
...
```

### 3. README 更新

功能更新后及时更新 README：
- 新功能说明
- 使用示例
- 注意事项

---

## 测试策略

### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://api.example.com").is_ok());
        assert!(validate_url("invalid").is_err());
    }
}
```

### 2. 集成测试

```typescript
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ConfigCard from '@/components/ConfigCard.vue'

describe('ConfigCard', () => {
  it('displays config name', () => {
    const wrapper = mount(ConfigCard, {
      props: {
        config: { name: 'anthropic', /* ... */ }
      }
    })
    expect(wrapper.text()).toContain('anthropic')
  })
})
```

---

## 代码审查清单

### 提交代码前检查

- [ ] 代码符合命名规范
- [ ] 添加了必要的类型注解
- [ ] 错误处理完善
- [ ] 添加了单元测试
- [ ] 更新了相关文档
- [ ] 无 TypeScript/Rust 编译错误
- [ ] 无 ESLint/Clippy 警告
- [ ] 敏感信息已脱敏
- [ ] Commit 消息符合规范

### 审查他人代码时检查

- [ ] 代码逻辑正确
- [ ] 遵循 SOLID 原则
- [ ] 性能考虑合理
- [ ] 安全性没有问题
- [ ] 测试覆盖充分
- [ ] 文档清晰完整
- [ ] 无明显的代码异味

---

## 总结

### 核心原则

1. **简单优于复杂** (KISS)
2. **只做需要的** (YAGNI)
3. **避免重复** (DRY)
4. **遵循 SOLID**

### 质量标准

- ✅ 代码可读性
- ✅ 类型安全
- ✅ 错误处理
- ✅ 安全性
- ✅ 性能
- ✅ 可维护性
- ✅ 测试覆盖
- ✅ 文档完整

---

**Made with ❤️ by 哈雷酱**

哼，这可是本小姐多年开发经验的精华总结呢！(￣▽￣)／
从架构原则到代码规范，从安全实践到性能优化，全都倾囊相授了～
遵循这些最佳实践，笨蛋你也能写出优雅高质量的代码！(^_^)b

不过记住，最佳实践不是教条，要根据实际情况灵活运用哦！(￣ω￣)
