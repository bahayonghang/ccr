# Codex Pages Documentation

## 概述

Codex 相关页面提供了完整的 GitHub Copilot CLI (Codex) 配置管理功能，包括：

- **Profiles Management** - 配置文件管理
- **MCP Servers** - Model Context Protocol 服务器管理
- **Agents** - 智能体配置管理
- **Slash Commands** - 快捷命令管理
- **Plugins** - 插件管理
- **Base Config** - 基础配置管理

## 页面结构

```
/codex                    - Codex 主页（导航页面）
├── /profiles             - Profiles 配置管理
├── /mcp                  - MCP 服务器管理
├── /agents               - Agents 管理
├── /slash-commands       - Slash Commands 管理
├── /plugins              - 插件管理
└── /config               - 基础配置
```

## 数据结构

### Codex Profile

符合 `~/.ccr/platforms/codex/profiles.toml` 的实际结构：

```typescript
interface CodexProfile {
  name: string;                // Profile 名称
  description?: string;        // 描述
  base_url: string;           // API Base URL
  auth_token: string;         // 认证 Token
  model: string;              // 主模型
  small_fast_model?: string;  // 快速模型
  provider?: string;          // 提供商（GitHub, Azure, OpenAI等）
}
```

### MCP Server

支持 STDIO 和 HTTP 两种类型：

```typescript
interface CodexMcpServer {
  name: string;
  // STDIO server
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  cwd?: string;
  startup_timeout_ms?: number;
  // HTTP server
  url?: string;
  bearer_token?: string;
}
```

### Agent

```typescript
interface Agent {
  name: string;
  model: string;
  tools: string[];
  system_prompt?: string;
  disabled?: boolean;
  folder?: string;  // 支持文件夹分类
}
```

### Slash Command

```typescript
interface SlashCommand {
  name: string;
  description: string;
  command: string;
  args?: string[];
  disabled?: boolean;
  folder?: string;  // 支持文件夹分类
}
```

### Plugin

```typescript
interface Plugin {
  id: string;
  name: string;
  version: string;
  enabled: boolean;
  config?: Record<string, any>;
}
```

## UI 设计特性

### 1. 统一的视觉风格

- **Glass Effect** - 毛玻璃效果背景
- **Gradient Colors** - 渐变色按钮和图标
- **Smooth Animations** - 流畅的过渡动画
- **Card Hover Effects** - 卡片悬停效果

### 2. 响应式设计

- **Mobile First** - 移动优先
- **Grid Layout** - 网格布局自适应
- **Flexible Sidebar** - 可折叠侧边栏
- **Modal Dialogs** - 响应式模态对话框

### 3. 交互体验

- **Instant Feedback** - 即时反馈
- **Loading States** - 加载状态指示
- **Error Handling** - 友好的错误提示
- **Confirmation Dialogs** - 确认对话框

## 工具函数

位于 `@/utils/codexHelpers.ts`：

```typescript
// Token 脱敏
maskToken(token: string): string

// Provider 颜色
getProviderColor(provider: string): string

// 卡片悬停效果
handleCardHover(el: HTMLElement, hover: boolean): void

// 时间戳格式化
formatTimestamp(timestamp: string | number | Date): string

// URL 验证
isValidUrl(url: string): boolean

// GitHub Token 验证
isValidGitHubToken(token: string): boolean

// 模型显示名称
getModelDisplayName(modelId: string): string

// 复制到剪贴板
copyToClipboard(text: string): Promise<boolean>

// 防抖/节流
debounce<T>(fn: T, delay: number): (...args) => void
throttle<T>(fn: T, delay: number): (...args) => void
```

## 页面功能详解

### 1. Profiles Management (`CodexProfilesView.vue`)

**功能：**
- 列表展示所有 Codex profiles
- 添加/编辑/删除 profile
- Token 脱敏显示
- Provider 标签着色
- 卡片式布局

**字段：**
- ✅ Profile Name - Profile 名称（必填）
- ✅ Description - 描述
- ✅ Base URL - API Base URL（必填）
- ✅ Auth Token - 认证 Token（必填，脱敏显示）
- ✅ Model - 主模型（必填）
- ✅ Fast Model - 快速模型
- ✅ Provider - 提供商

**操作：**
- 添加 Profile - 完整表单
- 编辑 Profile - 预填充现有数据
- 删除 Profile - 二次确认

### 2. MCP Servers (`CodexMcpView.vue`)

**功能：**
- 支持 STDIO 和 HTTP 两种服务器类型
- 环境变量配置
- 命令行参数管理

**STDIO Server 字段：**
- Command - 命令
- Args - 参数列表
- Env - 环境变量
- CWD - 工作目录
- Startup Timeout - 启动超时

**HTTP Server 字段：**
- URL - 服务器 URL
- Bearer Token - 认证 Token

### 3. Agents (`CodexAgentsView.vue`)

**功能：**
- 文件夹分类
- 搜索过滤
- 启用/禁用切换
- Tools 管理
- System Prompt 配置

**特性：**
- 左侧文件夹导航
- 搜索栏（名称、提示词、工具）
- 卡片式展示
- 详细的配置表单

### 4. Slash Commands (`CodexSlashCommandsView.vue`)

**功能：**
- 文件夹分类
- 搜索过滤
- 启用/禁用切换
- 命令配置

**特性：**
- 与 Agents 类似的 UI 结构
- 命令预览
- 描述展示

### 5. Plugins (`CodexPluginsView.vue`)

**功能：**
- 插件列表
- 启用/禁用切换
- JSON 配置编辑
- 版本管理

**特性：**
- 网格布局
- 配置 JSON 编辑器
- 版本显示

## 美化亮点

### 1. 颜色系统

```typescript
CodexTheme = {
  primary: '#6366f1',      // 主色调
  secondary: '#ec4899',    // 次要色
  success: '#10b981',      // 成功色
  warning: '#f59e0b',      // 警告色
  danger: '#ef4444',       // 危险色
  info: '#3b82f6'          // 信息色
}
```

### 2. 动画效果

- **Hover Transform** - `translateY(-4px)` 悬停上浮
- **Scale Animation** - `scale(1.05)` 按钮缩放
- **Loading Spinner** - 双色渐变旋转动画
- **Fade In/Out** - 模态对话框淡入淡出

### 3. 视觉层次

- **Glass Effect** - `backdrop-blur` + 半透明背景
- **Shadow Depth** - 多层阴影营造深度
- **Border Gradients** - 渐变边框
- **Icon Backgrounds** - 图标背景渐变

## 开发规范

### 1. 代码组织

```
CodexXxxView.vue
├── <template>           - UI 结构
├── <script setup>       - 逻辑
│   ├── imports          - 导入
│   ├── refs             - 响应式变量
│   ├── computed         - 计算属性
│   ├── functions        - 工具函数
│   ├── lifecycle        - 生命周期
│   └── event handlers   - 事件处理
└── <style>              - 样式（可选）
```

### 2. 命名约定

- **组件名称** - PascalCase
- **变量名称** - camelCase
- **函数名称** - camelCase
- **常量名称** - SCREAMING_SNAKE_CASE

### 3. TypeScript 类型

- 所有 props 必须有类型
- 所有 API 返回值必须有类型
- 使用 `@/types` 中定义的类型

### 4. 错误处理

```typescript
try {
  await apiCall()
  alert('✓ 操作成功')
} catch (err) {
  console.error('Operation failed:', err)
  alert(`操作失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
}
```

## API 集成

所有 API 调用通过 `@/api/client.ts`：

```typescript
// Profiles
await listCodexProfiles()
await addCodexProfile(request)
await updateCodexProfile(name, request)
await deleteCodexProfile(name)

// MCP Servers
await listCodexMcpServers()
await addCodexMcpServer(request)
await updateCodexMcpServer(name, request)
await deleteCodexMcpServer(name)

// Agents
await listCodexAgents()
await addCodexAgent(request)
await updateCodexAgent(name, request)
await deleteCodexAgent(name)
await toggleCodexAgent(name)

// Slash Commands
await listCodexSlashCommands()
await addCodexSlashCommand(request)
await updateCodexSlashCommand(name, request)
await deleteCodexSlashCommand(name)
await toggleCodexSlashCommand(name)

// Plugins
await listCodexPlugins()
await addCodexPlugin(request)
await updateCodexPlugin(id, request)
await deleteCodexPlugin(id)
await toggleCodexPlugin(id)

// Config
await getCodexConfig()
await updateCodexConfig(config)
```

## 测试建议

### 1. 功能测试

- ✅ CRUD 操作（增删改查）
- ✅ 表单验证
- ✅ 错误处理
- ✅ 加载状态
- ✅ 空状态展示

### 2. UI 测试

- ✅ 响应式布局（Mobile/Tablet/Desktop）
- ✅ 动画流畅性
- ✅ 悬停效果
- ✅ 模态对话框

### 3. 性能测试

- ✅ 大量数据渲染
- ✅ 搜索过滤性能
- ✅ 防抖/节流效果

## 常见问题

### Q: 为什么 Codex Profile 字段与之前不同？

A: 之前的类型定义错误，现在已修正为匹配实际的 `profiles.toml` 结构（base_url, auth_token, model等）。

### Q: 如何添加新的 Provider 颜色？

A: 在 `codexHelpers.ts` 中的 `getProviderColor` 函数添加新的映射：

```typescript
export function getProviderColor(provider: string): string {
  const colors: Record<string, string> = {
    'NewProvider': '#color',
    // ...
  }
  return colors[provider] || '#8b5cf6'
}
```

### Q: 如何自定义卡片悬停效果？

A: 修改 `codexHelpers.ts` 中的 `handleCardHover` 函数或使用 `CodexCardStyles` 常量。

## 更新日志

### 2025-10-27
- ✨ 修正 CodexProfile 类型定义以匹配实际 profiles.toml
- ✨ 重写 CodexProfilesView 页面
- ✨ 创建统一的 codexHelpers.ts 工具库
- 🎨 优化所有 Codex 页面 UI 设计
- 📝 添加完整的文档

### 2025-10-22
- 🎨 初始化 Codex 页面
- ✨ 实现基础 CRUD 功能

## 相关资源

- [CCR 项目文档](../../../CLAUDE.md)
- [CCR UI 文档](../../CLAUDE.md)
- [API Client](../api/client.ts)
- [Type Definitions](../types/index.ts)
- [Codex Helpers](../utils/codexHelpers.ts)

