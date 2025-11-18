# CCR UI Frontend - 国际化 (i18n) 开发指南

## 📖 目录

- [概述](#概述)
- [快速开始](#快速开始)
- [文件结构](#文件结构)
- [添加新翻译](#添加新翻译)
- [在组件中使用翻译](#在组件中使用翻译)
- [高级用法](#高级用法)
- [最佳实践](#最佳实践)
- [测试](#测试)
- [故障排查](#故障排查)
- [参考资料](#参考资料)

---

## 概述

CCR UI Frontend 使用 [Vue I18n](https://vue-i18n.intlify.dev/) 实现国际化，当前支持：

- **中文 (zh-CN)** - 简体中文
- **English (en-US)** - 美式英语

### 核心特性

✅ **Composition API 模式** - 使用 Vue 3 Composition API
✅ **LocalStorage 持久化** - 语言选择自动保存
✅ **热重载支持** - 开发时翻译更改立即生效
✅ **类型安全** - TypeScript 支持
✅ **400+ 翻译键** - 覆盖所有32+页面
✅ **15+ 命名空间** - 模块化组织

---

## 快速开始

### 1. 查看当前语言

打开浏览器开发者控制台：

```javascript
// 检查当前语言
localStorage.getItem('ccr-ui-locale')  // "zh-CN" 或 "en-US"

// 手动切换语言
localStorage.setItem('ccr-ui-locale', 'en-US')
location.reload()
```

### 2. 测试语言切换

1. 访问 http://localhost:5175/
2. 点击右上角的语言切换按钮（🌐）
3. 选择 "🇨🇳 中文" 或 "🇺🇸 English"
4. 页面内容立即更新（无需刷新）

### 3. 运行自动化测试

```bash
# 运行 i18n 测试
npm run test:i18n

# 或直接运行
node tests/i18n.test.cjs
```

---

## 文件结构

```
src/
├── i18n/
│   ├── index.ts           # i18n 配置和初始化
│   └── locales/
│       ├── zh-CN.ts       # 中文翻译（400+ 键）
│       └── en-US.ts       # 英文翻译（400+ 键）
├── components/
│   └── LanguageSwitcher.vue  # 语言切换组件
└── main.ts                # 应用入口（注册 i18n）
```

### 翻译文件结构

```typescript
// src/i18n/locales/zh-CN.ts
export default {
  common: {
    home: '首页',
    back: '返回',
    // ...
  },
  nav: {
    home: '首页',
    claudeCode: 'Claude Code',
    // ...
  },
  configs: {
    title: '配置管理',
    // ...
  },
  // ... 更多命名空间
}
```

---

## 添加新翻译

### 步骤 1: 在翻译文件中添加键

#### 中文翻译 (`src/i18n/locales/zh-CN.ts`)

```typescript
export default {
  // ... 现有命名空间

  // 添加新命名空间
  myFeature: {
    title: '我的新功能',
    description: '这是一个新功能的描述',

    // 嵌套对象
    buttons: {
      save: '保存',
      cancel: '取消',
      delete: '删除',
    },

    // 带变量的翻译
    greeting: '你好，{name}！',
    itemCount: '共 {count} 个项目',

    // 复杂变量
    message: '{user} 在 {date} 执行了 {action}',
  },
}
```

#### 英文翻译 (`src/i18n/locales/en-US.ts`)

```typescript
export default {
  // ... existing namespaces

  // Add new namespace (MUST match zh-CN structure)
  myFeature: {
    title: 'My New Feature',
    description: 'This is a description of the new feature',

    // Nested object
    buttons: {
      save: 'Save',
      cancel: 'Cancel',
      delete: 'Delete',
    },

    // With variables
    greeting: 'Hello, {name}!',
    itemCount: '{count} items in total',

    // Complex variables
    message: '{user} performed {action} on {date}',
  },
}
```

### 步骤 2: 确保键名完全匹配

⚠️ **重要**: 两个语言文件的键结构必须完全相同！

```typescript
// ✅ 正确 - 键名匹配
zh-CN: myFeature.title
en-US: myFeature.title

// ❌ 错误 - 键名不匹配
zh-CN: myFeature.title
en-US: myFeature.heading  // 键名不同！
```

### 步骤 3: 运行测试验证

```bash
# 验证翻译文件完整性
npm run test:i18n

# 应该看到所有测试通过
✓ PASS File existence check
✓ PASS File size comparison
✓ PASS Namespace extraction
✓ PASS Required namespaces
✓ PASS Variable placeholders
✓ PASS Syntax validation
✓ PASS Coverage statistics
```

---

## 在组件中使用翻译

### 方法 1: 模板中使用 `$t()`

适用于静态文本：

```vue
<template>
  <div>
    <!-- 基本用法 -->
    <h1>{{ $t('myFeature.title') }}</h1>
    <p>{{ $t('myFeature.description') }}</p>

    <!-- 嵌套键 -->
    <button>{{ $t('myFeature.buttons.save') }}</button>

    <!-- 属性中使用 -->
    <input :placeholder="$t('myFeature.inputPlaceholder')" />
    <button :aria-label="$t('myFeature.buttons.save')">保存</button>

    <!-- 变量插值 -->
    <p>{{ $t('myFeature.greeting', { name: 'Alice' }) }}</p>
    <p>{{ $t('myFeature.itemCount', { count: 10 }) }}</p>
  </div>
</template>
```

### 方法 2: Script 中使用 `t()`

适用于动态内容、逻辑处理：

```vue
<script setup lang="ts">
import { useI18n } from 'vue-i18n'

// 获取 t 函数
const { t } = useI18n()

// 基本用法
const title = t('myFeature.title')

// 带变量
const greeting = t('myFeature.greeting', { name: 'Bob' })

// 在函数中使用
const showMessage = (username: string, action: string) => {
  const message = t('myFeature.message', {
    user: username,
    action: action,
    date: new Date().toLocaleDateString()
  })
  alert(message)
}

// 在计算属性中使用
import { computed } from 'vue'

const buttonLabel = computed(() => {
  return isEditing.value
    ? t('myFeature.buttons.save')
    : t('myFeature.buttons.edit')
})
</script>
```

### 方法 3: Computed 响应式翻译

用于数组、对象等需要响应式更新的数据：

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// ✅ 正确 - 使用 computed
const options = computed(() => [
  { label: t('myFeature.option1'), value: '1' },
  { label: t('myFeature.option2'), value: '2' },
  { label: t('myFeature.option3'), value: '3' },
])

// ❌ 错误 - 不使用 computed（语言切换后不会更新）
const options = [
  { label: t('myFeature.option1'), value: '1' },  // 只在初始化时执行一次
]

// ✅ 正确 - 动态面包屑导航
const breadcrumbItems = computed(() => [
  { label: t('nav.home'), path: '/', icon: Home },
  { label: t('nav.myFeature'), path: '/my-feature', icon: Settings },
])
</script>

<template>
  <!-- 使用响应式 computed 数据 -->
  <select>
    <option
      v-for="option in options"
      :key="option.value"
      :value="option.value"
    >
      {{ option.label }}
    </option>
  </select>

  <Breadcrumb :items="breadcrumbItems" />
</template>
```

### 方法 4: 面包屑导航专用

⚠️ **特别注意**: 面包屑组件使用 `$t()` 而不是 `{{ $t() }}`

```vue
<template>
  <Breadcrumb
    :items="[
      { label: $t('nav.home'), path: '/', icon: Home },
      { label: $t('nav.myFeature'), path: '/my-feature', icon: Settings }
    ]"
  />
</template>

<script setup lang="ts">
import { Home, Settings } from 'lucide-vue-next'
import Breadcrumb from '@/components/Breadcrumb.vue'
</script>
```

---

## 高级用法

### 1. 多变量插值

```vue
<script setup lang="ts">
const { t } = useI18n()

// 翻译键定义
// zh-CN: '{user} 在 {date} 将 {count} 个文件移动到 {folder}'
// en-US: '{user} moved {count} files to {folder} on {date}'

const message = t('myFeature.complexMessage', {
  user: 'Alice',
  date: '2025-01-18',
  count: 5,
  folder: 'Documents'
})
// 中文: Alice 在 2025-01-18 将 5 个文件移动到 Documents
// English: Alice moved 5 files to Documents on 2025-01-18
</script>
```

### 2. 条件翻译

```vue
<script setup lang="ts">
const { t } = useI18n()

const statusMessage = computed(() => {
  if (status.value === 'success') {
    return t('myFeature.successMessage')
  } else if (status.value === 'error') {
    return t('myFeature.errorMessage')
  } else {
    return t('myFeature.processingMessage')
  }
})
</script>

<template>
  <div :class="statusClass">
    {{ statusMessage }}
  </div>
</template>
```

### 3. 复数处理

```typescript
// zh-CN.ts
export default {
  myFeature: {
    // 中文通常不区分单复数，但可以用条件表达
    items: '{count} 个项目',
    noItems: '暂无项目',
    oneItem: '1 个项目',
  }
}

// en-US.ts
export default {
  myFeature: {
    items: '{count} items',
    noItems: 'No items',
    oneItem: '1 item',
  }
}
```

```vue
<script setup lang="ts">
const { t } = useI18n()

const itemCountText = computed(() => {
  const count = items.value.length
  if (count === 0) {
    return t('myFeature.noItems')
  } else if (count === 1) {
    return t('myFeature.oneItem')
  } else {
    return t('myFeature.items', { count })
  }
})
</script>
```

### 4. HTML 内容（慎用）

```vue
<template>
  <!-- 使用 v-html（需确保内容安全） -->
  <div v-html="$t('myFeature.htmlContent')"></div>
</template>
```

```typescript
// zh-CN.ts
export default {
  myFeature: {
    htmlContent: '这是<strong>粗体</strong>文本',
  }
}
```

---

## 最佳实践

### ✅ DO - 推荐做法

#### 1. 使用有意义的命名空间

```typescript
// ✅ 好 - 清晰的命名空间
configs: {
  title: '配置管理',
  buttons: {
    save: '保存',
  }
}

// ❌ 不好 - 过于扁平
configTitle: '配置管理',
configButtonSave: '保存',
```

#### 2. 保持键名一致性

```typescript
// ✅ 好 - 所有按钮使用相同模式
buttons: {
  save: '保存',
  cancel: '取消',
  delete: '删除',
}

// ❌ 不好 - 不一致的命名
saveBtn: '保存',
cancelButton: '取消',
btnDelete: '删除',
```

#### 3. 使用描述性变量名

```typescript
// ✅ 好 - 清晰的变量名
message: '{userName} 删除了 {fileName}'

// ❌ 不好 - 模糊的变量名
message: '{x} 删除了 {y}'
```

#### 4. 为数组使用 computed

```typescript
// ✅ 好 - 响应式更新
const filters = computed(() => [
  { label: t('filters.all'), value: 'all' },
  { label: t('filters.active'), value: 'active' },
])

// ❌ 不好 - 语言切换后不更新
const filters = [
  { label: t('filters.all'), value: 'all' },
]
```

#### 5. 添加翻译后立即测试

```bash
# 每次添加新翻译后
npm run test:i18n
```

### ❌ DON'T - 避免做法

#### 1. 不要硬编码文本

```vue
<!-- ❌ 不好 - 硬编码 -->
<button>保存</button>

<!-- ✅ 好 - 使用翻译 -->
<button>{{ $t('common.save') }}</button>
```

#### 2. 不要在翻译中包含样式

```typescript
// ❌ 不好 - 样式混在翻译中
title: '<span style="color: red;">错误</span>'

// ✅ 好 - 样式在组件中
title: '错误'
```

```vue
<template>
  <span class="text-red-500">{{ $t('errors.title') }}</span>
</template>
```

#### 3. 不要过度嵌套

```typescript
// ❌ 不好 - 过度嵌套
a: {
  b: {
    c: {
      d: {
        e: '值'
      }
    }
  }
}

// ✅ 好 - 合理嵌套（2-3层）
section: {
  subsection: {
    value: '值'
  }
}
```

#### 4. 不要在代码中拼接翻译

```typescript
// ❌ 不好 - 拼接字符串
const message = t('hello') + ', ' + userName + '!'

// ✅ 好 - 使用变量
const message = t('greeting', { name: userName })
```

---

## 测试

### 自动化测试

```bash
# 运行完整的 i18n 测试套件
npm run test:i18n

# 测试内容：
# ✓ 文件存在性检查
# ✓ 文件大小对比
# ✓ 命名空间提取和验证
# ✓ 必需命名空间检查
# ✓ 变量占位符分析
# ✓ 语法验证
# ✓ 覆盖率统计
```

### 手动测试清单

#### 基础功能测试

- [ ] 语言切换按钮显示正常
- [ ] 点击切换按钮显示语言下拉菜单
- [ ] 选择语言后页面内容立即更新
- [ ] 刷新页面后语言选择保持不变

#### 页面测试（抽样）

- [ ] 首页 (/) - 所有文本正确翻译
- [ ] 配置管理 (/configs) - 表格、按钮、筛选器
- [ ] MCP 服务器 (/mcp) - 表单、列表、操作按钮
- [ ] 面包屑导航 - 所有页面的导航路径

#### 动态内容测试

- [ ] 带变量的消息正确显示（如删除确认）
- [ ] 计算属性中的翻译响应语言切换
- [ ] 下拉列表选项正确翻译
- [ ] 错误消息和成功提示正确显示

#### 边界情况测试

- [ ] 长文本不会破坏布局
- [ ] 特殊字符正确显示
- [ ] 变量为空时的处理
- [ ] 缺失翻译键时的回退行为

---

## 故障排查

### 问题 1: 翻译不显示或显示为键名

**症状**:
```
页面显示: myFeature.title
预期显示: 我的新功能
```

**解决方案**:

1. **检查键是否存在**:
```bash
# 搜索翻译键
grep -r "myFeature.title" src/i18n/locales/
```

2. **检查拼写**:
```vue
<!-- ❌ 错误 -->
{{ $t('myFeature.titel') }}  <!-- 拼写错误 -->

<!-- ✅ 正确 -->
{{ $t('myFeature.title') }}
```

3. **检查命名空间**:
```typescript
// 确保键在正确的命名空间下
export default {
  myFeature: {  // 命名空间
    title: '...'  // 键
  }
}
```

### 问题 2: 语言切换后部分内容不更新

**症状**: 切换语言后，某些下拉菜单或列表内容不更新

**解决方案**:

使用 `computed` 包装翻译数组：

```vue
<script setup lang="ts">
// ❌ 错误 - 不会更新
const options = [
  { label: t('option1'), value: '1' }
]

// ✅ 正确 - 会响应式更新
const options = computed(() => [
  { label: t('option1'), value: '1' }
])
</script>
```

### 问题 3: Vite 缓存导致的警告

**症状**: 看到 "Duplicate key" 警告但代码中没有重复

**解决方案**:

```bash
# 清除 Vite 缓存
rm -rf node_modules/.vite

# 重启开发服务器
npm run dev
```

### 问题 4: 变量插值不工作

**症状**:
```
显示: Hello, {name}!
预期: Hello, Alice!
```

**解决方案**:

确保传递变量对象：

```vue
<!-- ❌ 错误 -->
{{ $t('greeting') }}

<!-- ✅ 正确 -->
{{ $t('greeting', { name: 'Alice' }) }}
```

### 问题 5: 自动化测试失败

**症状**: `npm run test:i18n` 报告键不匹配

**解决方案**:

1. **检查两个文件的键结构**:
```bash
# 比较两个文件的结构
diff <(grep -E '^\s*\w+:' src/i18n/locales/zh-CN.ts | sort) \
     <(grep -E '^\s*\w+:' src/i18n/locales/en-US.ts | sort)
```

2. **找到缺失的键**:
```bash
# 运行测试查看详细输出
node tests/i18n.test.cjs
```

3. **添加缺失的键到对应文件**

---

## 命名空间参考

当前已有的命名空间（可在现有命名空间下添加新键）：

### 核心命名空间
- `common` - 通用文本（按钮、操作等）
- `nav` - 导航栏和菜单
- `language` - 语言切换

### 功能命名空间
- `configs` - 配置管理
- `commands` - 命令执行
- `converter` - 配置转换
- `sync` - 同步管理
- `usage` - 使用统计
- `stats` - 统计分析

### 平台特定命名空间
- `claudeCode` - Claude Code 相关
- `codex` - Codex 相关
- `geminiCli` - Gemini CLI 相关
- `qwen` - Qwen 相关
- `iflow` - iFlow 相关

### 组件命名空间
- `mcp` - MCP 服务器管理
- `agents` - 代理管理
- `slashCommands` - 斜杠命令
- `plugins` - 插件管理

---

## 参考资料

### 官方文档
- [Vue I18n 官方文档](https://vue-i18n.intlify.dev/)
- [Vue 3 Composition API](https://vuejs.org/guide/extras/composition-api-faq.html)

### 项目文档
- [CCR UI Frontend CLAUDE.md](../CLAUDE.md)
- [i18n 自动化测试](../tests/README.md)

### 代码示例
- LanguageSwitcher 组件: `src/components/LanguageSwitcher.vue`
- i18n 配置: `src/i18n/index.ts`
- 翻译文件: `src/i18n/locales/`

### 工具和命令
```bash
# 开发服务器
npm run dev

# 运行 i18n 测试
npm run test:i18n

# 类型检查
npm run type-check

# 代码检查
npm run lint
```

---

## 快速参考卡片

### 添加新翻译的完整流程

```bash
# 1. 编辑翻译文件
vim src/i18n/locales/zh-CN.ts  # 添加中文
vim src/i18n/locales/en-US.ts  # 添加英文

# 2. 运行测试
npm run test:i18n

# 3. 在组件中使用
# 模板: {{ $t('namespace.key') }}
# Script: t('namespace.key')

# 4. 在浏览器中测试
# 访问 http://localhost:5175/
# 切换语言验证
```

### 常用翻译模式

```typescript
// 1. 简单文本
title: '标题'

// 2. 嵌套对象
buttons: {
  save: '保存',
  cancel: '取消'
}

// 3. 带变量
message: '{user} 执行了 {action}'

// 4. 列表（使用 computed）
const items = computed(() => [
  { label: t('item1') }
])
```

---

## 贡献指南

欢迎贡献翻译！请遵循以下步骤：

1. **Fork 项目**
2. **创建分支**: `git checkout -b feature/add-translations`
3. **添加翻译**: 同时更新 `zh-CN.ts` 和 `en-US.ts`
4. **运行测试**: `npm run test:i18n`
5. **提交代码**:
   ```bash
   git add src/i18n/locales/
   git commit -m "feat(i18n): 添加 XXX 功能的翻译"
   ```
6. **推送分支**: `git push origin feature/add-translations`
7. **创建 Pull Request**

---

## 版本历史

- **v1.0.0** (2025-01-18) - 初始版本，完成32+页面国际化
  - 400+ 翻译键
  - 15+ 命名空间
  - 2种语言（中文、英文）
  - 自动化测试
  - 语言切换组件

---

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](../../LICENSE) 文件。

---

## 联系方式

如有问题或建议，请：

- 提交 GitHub Issue
- 参与项目讨论
- 阅读项目文档

**祝开发愉快！Happy Coding! 🎉**
