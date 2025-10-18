# 页面架构设计

CCR UI 采用三级导航架构，为用户提供清晰的功能组织和流畅的导航体验。

## 🎨 设计理念

### 核心原则

1. **层次分明** - 三级导航结构，从概览到详情
2. **视觉统一** - 每个 CLI 工具独特的渐变配色
3. **交互流畅** - 平滑的动画和悬停效果
4. **功能聚焦** - 相关功能集中在对应的 CLI 主页

### 设计风格

- **玻璃拟态（Glassmorphism）** - 半透明背景 + 模糊效果
- **赛博朋克风格** - 渐变边框和发光效果
- **科技感配色** - 深色主题为主，配合鲜艳的渐变色
- **流畅动画** - 卡片浮起、箭头移动、渐变流动

## 📊 三级导航架构

### 第一级：Dashboard 首页 (`/`)

**功能定位**：系统概览和功能入口

**页面组成**：
```
┌─────────────────────────────────────┐
│  CCR UI                             │
│  Claude Code 配置管理中心            │
│  v1.3.0                             │
├─────────────────────────────────────┤
│  系统状态监控                        │
│  [CPU] [内存] [系统信息]             │
├─────────────────────────────────────┤
│  功能模块卡片网格 (4x2)              │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐       │
│  │CC  │ │Cdx │ │Gem │ │Qwen│       │
│  └────┘ └────┘ └────┘ └────┘       │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐       │
│  │IFLW│ │Cmd │ │Cvt │ │Sync│       │
│  └────┘ └────┘ └────┘ └────┘       │
└─────────────────────────────────────┘
```

**特点**：
- 实时系统监控（CPU、内存、系统信息）
- 8 个功能模块卡片
- 动态渐变背景
- 悬停浮起动画

### 第二级：CLI 工具主页

每个 CLI 工具都有独立的主页，展示其所有子功能。

#### Claude Code 主页 (`/claude-code`)

**配色**：蓝色 → 青色 (`from-blue-500 to-cyan-500`)

**子功能模块**：
```
┌─────────────────────────────────────┐
│  [返回首页]                          │
│  Claude Code 图标                    │
│  Claude Code                         │
│  Claude Code 配置管理中心            │
├─────────────────────────────────────┤
│  子功能卡片网格 (3x2)                │
│  ┌────────┐ ┌────────┐ ┌────────┐  │
│  │配置管理│ │云同步  │ │MCP服务 │  │
│  │[核心]  │ │[新功能]│ │        │  │
│  └────────┘ └────────┘ └────────┘  │
│  ┌────────┐ ┌────────┐ ┌────────┐  │
│  │Slash   │ │Agents  │ │插件管理│  │
│  │Commands│ │        │ │        │  │
│  └────────┘ └────────┘ └────────┘  │
└─────────────────────────────────────┘
```

**统计信息**：
- 6 个功能模块
- 云同步支持
- 完整功能覆盖

#### Codex 主页 (`/codex`)

**配色**：紫色 → 粉色 (`from-purple-500 to-pink-500`)

**子功能模块**：
- MCP 服务器
- Profiles
- 基础配置

#### Gemini CLI 主页 (`/gemini-cli`)

**配色**：橙色 → 红色 (`from-orange-500 to-red-500`)

**子功能模块**：
- MCP 服务器
- Agents
- 插件管理
- Slash Commands

#### Qwen 主页 (`/qwen`)

**配色**：绿色 → 青色 (`from-green-500 to-teal-500`)

**子功能模块**：
- MCP 服务器
- Agents
- 插件管理
- Slash Commands

#### IFLOW 主页 (`/iflow`)

**配色**：靛蓝 → 蓝色 (`from-indigo-500 to-blue-500`)

**子功能模块**：
- MCP 服务器
- Agents
- 插件管理
- Slash Commands

### 第三级：功能页面

具体的功能实现页面，保留原有的所有功能。

**示例页面**：
- `/configs` - 配置管理（列表、切换、验证、历史）
- `/sync` - WebDAV 云同步
- `/mcp` - MCP 服务器管理
- `/slash-commands` - Slash Commands 管理
- `/agents` - Agents 管理
- `/plugins` - 插件管理

## 🎨 视觉设计系统

### 配色方案

每个 CLI 工具都有独特的渐变配色：

| CLI 工具 | 主色调 | 渐变类名 | Hex 值 |
|---------|--------|---------|--------|
| Claude Code | 蓝-青 | `from-blue-500 to-cyan-500` | `#3B82F6 → #06B6D4` |
| Codex | 紫-粉 | `from-purple-500 to-pink-500` | `#A855F7 → #EC4899` |
| Gemini CLI | 橙-红 | `from-orange-500 to-red-500` | `#F97316 → #EF4444` |
| Qwen | 绿-青 | `from-green-500 to-teal-500` | `#22C55E → #14B8A6` |
| IFLOW | 靛-蓝 | `from-indigo-500 to-blue-500` | `#6366F1 → #3B82F6` |
| 命令中心 | 灰-黑 | `from-gray-700 to-gray-900` | `#374151 → #111827` |
| 转换器 | 黄-橙 | `from-yellow-500 to-orange-500` | `#EAB308 → #F97316` |
| 云同步 | 青-蓝 | `from-cyan-500 to-blue-500` | `#06B6D4 → #3B82F6` |

### 玻璃卡片效果

```css
.glass-card {
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(16px) saturate(180%);
  -webkit-backdrop-filter: blur(16px) saturate(180%);
  border-radius: 16px;
  box-shadow:
    0 8px 32px 0 rgba(0, 0, 0, 0.37),
    inset 0 1px 0 0 rgba(255, 255, 255, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.glass-card:hover {
  background: rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(20px) saturate(200%);
  box-shadow:
    0 12px 48px 0 rgba(0, 0, 0, 0.45),
    inset 0 1px 0 0 rgba(255, 255, 255, 0.15);
}
```

### 动画效果

1. **卡片悬停**
   - 浮起效果：`hover:scale-105 hover:-translate-y-2`
   - 渐变边框显示：`opacity-0 group-hover:opacity-20`
   - 文字渐变：`group-hover:bg-gradient-to-r`

2. **图标动画**
   - 箭头移动：`group-hover:translate-x-1`
   - 返回按钮：`group-hover:-translate-x-1`

3. **背景效果**
   - 动态模糊球：`animate-pulse`
   - 分层延迟：`animationDelay: '1s'`

## 🔄 导航流程

### 用户导航路径

**场景1：配置管理**
```
Dashboard (/)
  → 点击 "Claude Code" 卡片
  → Claude Code 主页 (/claude-code)
  → 点击 "配置管理" 卡片
  → 配置管理页面 (/configs)
```

**场景2：MCP 服务器配置**
```
Dashboard (/)
  → 点击 "Codex" 卡片
  → Codex 主页 (/codex)
  → 点击 "MCP 服务器" 卡片
  → Codex MCP 页面 (/codex/mcp)
```

**场景3：直接访问功能**
```
Dashboard (/)
  → 点击 "云同步" 卡片
  → 云同步页面 (/sync)
```

### 返回导航

每个二级页面（CLI 主页）都有返回首页按钮：
```typescript
<Link href="/" className="inline-flex items-center gap-2">
  <Home className="w-5 h-5 group-hover:-translate-x-1" />
  <span>返回首页</span>
</Link>
```

## 📐 布局规范

### 页面容器

```typescript
<div className="min-h-screen bg-gradient-to-br from-slate-900 via-{color}-900 to-slate-900">
  {/* 动态背景 */}
  <div className="fixed inset-0 overflow-hidden pointer-events-none">
    {/* 渐变球 */}
  </div>

  {/* 内容容器 */}
  <div className="relative z-10 container mx-auto px-4 py-12">
    {/* 页面内容 */}
  </div>
</div>
```

### 卡片网格

**Dashboard (4列)**：
```typescript
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
  {/* 8个卡片 */}
</div>
```

**CLI 主页 (3列)**：
```typescript
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
  {/* 子功能卡片 */}
</div>
```

## 🎯 响应式设计

### 断点策略

- **移动端** (`< 768px`): 单列布局
- **平板** (`768px - 1024px`): 2列布局
- **桌面** (`> 1024px`): 3-4列布局

### 适配重点

1. **卡片大小**：移动端全宽，桌面端固定宽度
2. **文字大小**：标题在小屏幕上适当缩小
3. **间距**：移动端减少间距，桌面端增加
4. **动画**：移动端可选择性禁用复杂动画

## 🚀 性能优化

### 代码分割

每个页面自动代码分割：
```
page.tsx → 独立的 JavaScript chunk
```

### 图片优化

使用 Next.js Image 组件：
```typescript
<Image
  src="/logo.svg"
  alt="CCR UI"
  width={64}
  height={64}
  priority // 首页图标优先加载
/>
```

### CSS 优化

- 使用 Tailwind CSS 的 JIT 模式
- 自动移除未使用的样式
- CSS 按需加载

## 📝 开发规范

### 新增 CLI 工具主页

1. 创建目录和文件：`app/cli-name/page.tsx`
2. 选择独特的渐变配色
3. 定义子功能模块数组
4. 复用通用布局结构
5. 在 Dashboard 添加入口卡片

### 新增功能页面

1. 创建页面文件：`app/feature/page.tsx`
2. 实现具体功能组件
3. 在对应的 CLI 主页添加导航卡片
4. 更新文档

## 🔗 相关文档

- [前端概述](/frontend/overview)
- [组件文档](/frontend/components)
- [样式指南](/frontend/styling)
- [开发指南](/frontend/development)
