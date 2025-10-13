# 添加新功能

哼，让本小姐教你如何给 CCR Desktop 添加新功能！(￣▽￣)ゞ

## 功能开发流程

添加新功能遵循 **后端优先** 的开发流程：

```
1. 📋 需求分析
   ↓
2. 🦀 后端开发 (Rust)
   ├─ 添加 Tauri Command
   └─ 实现业务逻辑
   ↓
3. 🎨 前端开发 (Vue)
   ├─ 更新类型定义
   ├─ 封装 API 调用
   └─ 实现 UI 组件
   ↓
4. 🧪 测试验证
   ↓
5. 📝 更新文档
```

---

## 示例：添加「配置验证」功能

让我们通过一个完整的示例，学习如何添加一个新功能：**验证单个配置的有效性**。

### 步骤 1：需求分析

**功能需求：**
- 用户可以点击配置卡片上的「验证」按钮
- 系统检查配置的 `base_url` 和 `auth_token` 是否有效
- 显示验证结果（成功/失败）

**技术决策：**
- 后端：添加 `validate_config` Command
- 前端：在配置卡片添加验证按钮和状态显示

---

### 步骤 2：后端开发

#### 2.1 添加数据模型

在 `src/commands/mod.rs` 中添加验证结果类型：

```rust
/// 📊 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub config_name: String,
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

::: tip 位置
添加到文件顶部的数据模型部分（Line 14 附近），与其他类型定义放在一起。
:::

#### 2.2 实现 Tauri Command

在 `src/commands/mod.rs` 的配置管理部分添加新 Command：

```rust
/// ✅ 验证单个配置
#[tauri::command]
pub async fn validate_config(name: String) -> Result<ValidationResult, String> {
    let service = ConfigService::default().map_err(|e| e.to_string())?;

    // 加载配置
    let config = service.load_config().map_err(|e| e.to_string())?;
    let section = config
        .sections
        .get(&name)
        .ok_or(format!("配置 {} 不存在", name))?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 验证 base_url
    if let Some(url) = &section.base_url {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            errors.push("base_url 必须以 http:// 或 https:// 开头".to_string());
        }
    } else {
        errors.push("base_url 不能为空".to_string());
    }

    // 验证 auth_token
    if let Some(token) = &section.auth_token {
        if token.len() < 10 {
            warnings.push("auth_token 长度过短，可能无效".to_string());
        }
    } else {
        errors.push("auth_token 不能为空".to_string());
    }

    // 验证 model
    if section.model.is_none() {
        warnings.push("未设置 model，将使用默认值".to_string());
    }

    Ok(ValidationResult {
        config_name: name,
        is_valid: errors.is_empty(),
        errors,
        warnings,
    })
}
```

::: tip 位置
添加到配置管理 Commands 部分（Line 66-307 之间），在 `validate_all` 之后。
:::

#### 2.3 注册 Command

在 `src/main.rs` 的 `invoke_handler` 中注册新 Command：

```rust {10}
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![
        commands::list_configs,
        commands::switch_config,
        // ... 其他命令
        commands::validate_all,
        commands::validate_config,      // 👈 添加这一行！
        commands::get_history,
        // ...
    ])
    .run(tauri::generate_context!())
```

::: danger 重要
如果忘记注册 Command，前端调用时会报错：`Command not found`！
:::

---

### 步骤 3：前端开发

#### 3.1 更新类型定义

在 `src-ui/src/types/index.ts` 中添加验证结果类型：

```typescript
/**
 * 配置验证结果
 */
export interface ValidationResult {
  config_name: string
  is_valid: boolean
  errors: string[]
  warnings: string[]
}
```

#### 3.2 封装 API 调用

在 `src-ui/src/api/index.ts` 中添加 API 函数：

```typescript
/**
 * 验证单个配置的有效性
 * @param name 配置名称
 * @returns 验证结果
 */
export async function validateConfig(name: string): Promise<ValidationResult> {
  return await invoke('validate_config', { name })
}
```

::: tip 命名规范
- Rust Command: `validate_config` (snake_case)
- TypeScript Function: `validateConfig` (camelCase)
- 参数传递时使用 snake_case: `{ name }`
:::

#### 3.3 实现 UI 功能

在 `src-ui/src/App.vue` 中添加验证功能：

**3.3.1 添加状态变量**

在 `<script setup>` 部分添加：

```typescript
// 验证状态
const validationResults = ref<Map<string, ValidationResult>>(new Map())
const validatingConfigs = ref<Set<string>>(new Set())
```

**3.3.2 实现验证函数**

```typescript
/**
 * 验证配置
 */
const handleValidate = async (configName: string) => {
  validatingConfigs.value.add(configName)

  try {
    const result = await validateConfig(configName)
    validationResults.value.set(configName, result)

    // 显示验证结果
    if (result.is_valid) {
      console.log(`✅ 配置 ${configName} 验证通过`)
      if (result.warnings.length > 0) {
        console.warn('警告:', result.warnings)
      }
    } else {
      console.error(`❌ 配置 ${configName} 验证失败:`, result.errors)
    }
  } catch (error) {
    console.error('验证失败:', error)
  } finally {
    validatingConfigs.value.delete(configName)
  }
}
```

**3.3.3 添加 UI 按钮**

在配置卡片的操作区域添加验证按钮：

```vue
<div class="config-actions">
  <button
    v-if="!config.is_current"
    @click="handleSwitch(config.name)"
    class="btn btn-primary"
  >
    切换
  </button>

  <!-- 👇 新增验证按钮 -->
  <button
    @click="handleValidate(config.name)"
    class="btn btn-secondary"
    :disabled="validatingConfigs.has(config.name)"
  >
    {{ validatingConfigs.has(config.name) ? '验证中...' : '验证' }}
  </button>

  <button @click="openEditModal(config)" class="btn btn-secondary">
    编辑
  </button>
  <button @click="handleDelete(config.name)" class="btn btn-danger">
    删除
  </button>
</div>
```

**3.3.4 显示验证结果**

在配置卡片底部添加验证结果显示：

```vue
<!-- 验证结果显示 -->
<div
  v-if="validationResults.has(config.name)"
  class="validation-result"
  :class="{
    'validation-success': validationResults.get(config.name)?.is_valid,
    'validation-error': !validationResults.get(config.name)?.is_valid
  }"
>
  <div v-if="validationResults.get(config.name)?.is_valid">
    ✅ 配置有效
  </div>
  <div v-else>
    <div class="error-title">❌ 配置无效:</div>
    <ul class="error-list">
      <li v-for="(error, idx) in validationResults.get(config.name)?.errors" :key="idx">
        {{ error }}
      </li>
    </ul>
  </div>

  <!-- 警告信息 -->
  <div
    v-if="validationResults.get(config.name)?.warnings.length"
    class="warnings"
  >
    <div class="warning-title">⚠️ 警告:</div>
    <ul class="warning-list">
      <li v-for="(warn, idx) in validationResults.get(config.name)?.warnings" :key="idx">
        {{ warn }}
      </li>
    </ul>
  </div>
</div>
```

**3.3.5 添加样式**

在 `<style>` 部分添加样式：

```css
/* 验证结果样式 */
.validation-result {
  margin-top: 1rem;
  padding: 0.75rem;
  border-radius: 6px;
  font-size: 0.875rem;
}

.validation-success {
  background-color: #d1fae5;
  color: #065f46;
  border: 1px solid #10b981;
}

.validation-error {
  background-color: #fee2e2;
  color: #991b1b;
  border: 1px solid #ef4444;
}

.error-title,
.warning-title {
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.error-list,
.warning-list {
  margin: 0;
  padding-left: 1.5rem;
}

.warnings {
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid #f59e0b;
  color: #92400e;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

---

### 步骤 4：测试验证

#### 4.1 启动开发服务器

```bash
cd /path/to/ccr-tauri
cargo tauri dev
```

#### 4.2 测试流程

1. **打开应用**，查看配置列表
2. **点击「验证」按钮**，确认按钮变为「验证中...」
3. **检查验证结果**：
   - ✅ 有效配置显示绿色成功提示
   - ❌ 无效配置显示红色错误列表
   - ⚠️ 警告信息显示黄色提示
4. **查看控制台日志**，确认无错误
5. **测试边界情况**：
   - 验证不存在的配置
   - 验证缺少必填字段的配置
   - 验证格式错误的 URL

#### 4.3 验证清单

- [ ] 后端 Command 正常工作
- [ ] 前端 API 调用成功
- [ ] UI 按钮显示正确
- [ ] 加载状态显示正常
- [ ] 验证结果显示准确
- [ ] 错误处理完善
- [ ] 控制台无错误

---

### 步骤 5：更新文档

#### 5.1 更新 API 文档

在 `docs/api/commands.md` 中添加新 Command 文档：

````markdown
### validate_config

验证单个配置的有效性。

**函数签名**
```rust
#[tauri::command]
pub async fn validate_config(name: String) -> Result<ValidationResult, String>
```

**参数**
- `name`: 要验证的配置名称

**返回类型**
```typescript
interface ValidationResult {
  config_name: string;
  is_valid: boolean;
  errors: string[];
  warnings: string[];
}
```

**前端调用**
```typescript
const result = await validateConfig('anthropic')

if (result.is_valid) {
  console.log('配置有效')
} else {
  console.error('配置无效:', result.errors)
}
```
````

#### 5.2 更新开发指南

在 `docs/development/add-feature.md` 中添加功能说明（就是本文档！）

---

## 添加新 UI 组件

如果需要添加新的 UI 组件，遵循以下步骤：

### 1. 创建组件文件

```bash
cd src-ui/src/components
touch ConfigValidator.vue
```

### 2. 实现组件

```vue
<!-- src-ui/src/components/ConfigValidator.vue -->
<script setup lang="ts">
import { ref } from 'vue'
import { validateConfig } from '@/api'
import type { ValidationResult } from '@/types'

const props = defineProps<{
  configName: string
}>()

const result = ref<ValidationResult | null>(null)
const isValidating = ref(false)

const validate = async () => {
  isValidating.value = true
  try {
    result.value = await validateConfig(props.configName)
  } catch (error) {
    console.error('验证失败:', error)
  } finally {
    isValidating.value = false
  }
}
</script>

<template>
  <div class="config-validator">
    <button @click="validate" :disabled="isValidating">
      {{ isValidating ? '验证中...' : '验证配置' }}
    </button>

    <div v-if="result" class="result">
      <div v-if="result.is_valid" class="success">
        ✅ 配置有效
      </div>
      <div v-else class="error">
        ❌ 错误: {{ result.errors.join(', ') }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-validator {
  padding: 1rem;
}

.result {
  margin-top: 0.5rem;
}

.success {
  color: #10b981;
}

.error {
  color: #ef4444;
}
</style>
```

### 3. 使用组件

在 `App.vue` 中导入和使用：

```vue
<script setup lang="ts">
import ConfigValidator from './components/ConfigValidator.vue'
</script>

<template>
  <div class="app">
    <ConfigValidator :config-name="currentConfig.name" />
  </div>
</template>
```

---

## 添加新路由

如果需要添加多页面功能，可以集成 Vue Router：

### 1. 安装 Vue Router

```bash
cd src-ui
npm install vue-router@4
```

### 2. 创建路由配置

```typescript
// src-ui/src/router.ts
import { createRouter, createWebHistory } from 'vue-router'
import ConfigList from './views/ConfigList.vue'
import ConfigDetail from './views/ConfigDetail.vue'

const routes = [
  {
    path: '/',
    name: 'ConfigList',
    component: ConfigList
  },
  {
    path: '/config/:name',
    name: 'ConfigDetail',
    component: ConfigDetail
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
```

### 3. 注册路由

```typescript
// src-ui/src/main.ts
import { createApp } from 'vue'
import router from './router'
import App from './App.vue'

createApp(App)
  .use(router)
  .mount('#app')
```

---

## 开发技巧

### 技巧 1：使用 TypeScript 严格模式

确保 `tsconfig.json` 启用严格模式：

```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true
  }
}
```

### 技巧 2：使用 Vue DevTools

安装 Vue DevTools 浏览器扩展，方便调试组件状态。

### 技巧 3：热重载

开发时，Vite 和 Tauri 都支持热重载：
- **前端**：修改 `.vue` 文件自动刷新
- **后端**：修改 `.rs` 文件自动重新编译

### 技巧 4：并行开发

可以独立开发前后端：
- **前端开发**：`cd src-ui && npm run dev`
- **后端测试**：使用 Postman 或 curl 测试 Commands

### 技巧 5：使用 Mock 数据

前端开发时可以使用 Mock 数据：

```typescript
// src-ui/src/api/mock.ts
export const mockListConfigs = (): ConfigList => ({
  current_config: 'anthropic',
  default_config: 'anthropic',
  configs: [
    {
      name: 'anthropic',
      description: 'Official Anthropic',
      base_url: 'https://api.anthropic.com',
      // ...
    }
  ]
})
```

---

## 常见问题

### Q1: Command 调用报错 "Command not found"

**原因：** 忘记在 `main.rs` 中注册 Command

**解决：**
```rust
.invoke_handler(tauri::generate_handler![
    commands::your_new_command,  // 👈 添加这里！
])
```

### Q2: 类型不匹配错误

**原因：** 前端类型定义与后端不一致

**解决：** 检查 `types/index.ts` 和 `commands/mod.rs` 的类型定义

### Q3: 前端调用返回 null

**原因：** 后端返回 `Option<T>`，前端需要处理 `null` 情况

**解决：**
```typescript
const result = await getConfig(name)
if (result === null) {
  console.log('配置不存在')
  return
}
// 使用 result...
```

### Q4: 样式不生效

**原因：** CSS 选择器优先级问题或作用域问题

**解决：**
- 使用 `<style scoped>` 限制作用域
- 使用更具体的选择器
- 检查 CSS 变量是否定义

---

## 最佳实践

### ✅ 推荐做法

1. **后端优先**：先实现 Tauri Command，再实现前端
2. **类型安全**：使用 TypeScript 和 Rust 类型系统
3. **错误处理**：使用 `try-catch` 捕获所有 API 调用
4. **加载状态**：显示加载指示器提升用户体验
5. **代码复用**：抽象公共逻辑为独立函数或组件
6. **文档更新**：及时更新 API 文档和开发指南

### ❌ 避免做法

1. **直接修改核心库**：不要修改 `ccr` 核心库代码
2. **跳过类型定义**：不要使用 `any` 类型
3. **忽略错误处理**：不要让错误静默失败
4. **过度设计**：遵循 YAGNI 原则，不实现未来功能
5. **缺少测试**：不要跳过功能测试
6. **忘记注册 Command**：不要忘记在 `main.rs` 中注册

---

**Made with ❤️ by 哈雷酱**

哼，这份添加新功能的指南可是本小姐精心编写的呢！(￣▽￣)／
从需求分析到代码实现，从前端到后端，每个步骤都写得清清楚楚～
按照本小姐的教程，笨蛋你也能添加新功能了吧！(^_^)b
