# 前端样式指南

CCR UI 采用现代化的样式系统，基于 Tailwind CSS 构建一致、美观、响应式的用户界面。

## 🎨 设计系统

### 设计原则

1. **一致性**: 统一的视觉语言和交互模式
2. **可访问性**: 符合 WCAG 2.1 AA 标准
3. **响应式**: 适配各种设备和屏幕尺寸
4. **性能优先**: 优化加载速度和运行性能
5. **可维护性**: 模块化、可复用的样式组件

### 色彩系统

**主色调**:
```css
:root {
  /* Primary Colors */
  --primary: 222.2 84% 4.9%;
  --primary-foreground: 210 40% 98%;
  
  /* Secondary Colors */
  --secondary: 210 40% 96%;
  --secondary-foreground: 222.2 84% 4.9%;
  
  /* Accent Colors */
  --accent: 210 40% 96%;
  --accent-foreground: 222.2 84% 4.9%;
  
  /* Muted Colors */
  --muted: 210 40% 96%;
  --muted-foreground: 215.4 16.3% 46.9%;
  
  /* Destructive Colors */
  --destructive: 0 84.2% 60.2%;
  --destructive-foreground: 210 40% 98%;
  
  /* Border & Input */
  --border: 214.3 31.8% 91.4%;
  --input: 214.3 31.8% 91.4%;
  --ring: 222.2 84% 4.9%;
  
  /* Background */
  --background: 0 0% 100%;
  --foreground: 222.2 84% 4.9%;
  
  /* Card */
  --card: 0 0% 100%;
  --card-foreground: 222.2 84% 4.9%;
  
  /* Popover */
  --popover: 0 0% 100%;
  --popover-foreground: 222.2 84% 4.9%;
}

.dark {
  --primary: 210 40% 98%;
  --primary-foreground: 222.2 84% 4.9%;
  
  --secondary: 222.2 84% 4.9%;
  --secondary-foreground: 210 40% 98%;
  
  --accent: 217.2 32.6% 17.5%;
  --accent-foreground: 210 40% 98%;
  
  --muted: 217.2 32.6% 17.5%;
  --muted-foreground: 215 20.2% 65.1%;
  
  --destructive: 0 62.8% 30.6%;
  --destructive-foreground: 210 40% 98%;
  
  --border: 217.2 32.6% 17.5%;
  --input: 217.2 32.6% 17.5%;
  --ring: 212.7 26.8% 83.9%;
  
  --background: 222.2 84% 4.9%;
  --foreground: 210 40% 98%;
  
  --card: 222.2 84% 4.9%;
  --card-foreground: 210 40% 98%;
  
  --popover: 222.2 84% 4.9%;
  --popover-foreground: 210 40% 98%;
}
```

**语义化颜色**:
```typescript
// 状态颜色
const statusColors = {
  success: 'text-green-600 bg-green-50 border-green-200',
  warning: 'text-yellow-600 bg-yellow-50 border-yellow-200',
  error: 'text-red-600 bg-red-50 border-red-200',
  info: 'text-blue-600 bg-blue-50 border-blue-200',
}

// 优先级颜色
const priorityColors = {
  high: 'text-red-600 bg-red-100',
  medium: 'text-yellow-600 bg-yellow-100',
  low: 'text-green-600 bg-green-100',
}
```

### 字体系统

**字体族**:
```css
.font-sans {
  font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif;
}

.font-mono {
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace;
}
```

**字体大小**:
```css
.text-xs    { font-size: 0.75rem; line-height: 1rem; }     /* 12px */
.text-sm    { font-size: 0.875rem; line-height: 1.25rem; } /* 14px */
.text-base  { font-size: 1rem; line-height: 1.5rem; }      /* 16px */
.text-lg    { font-size: 1.125rem; line-height: 1.75rem; } /* 18px */
.text-xl    { font-size: 1.25rem; line-height: 1.75rem; }  /* 20px */
.text-2xl   { font-size: 1.5rem; line-height: 2rem; }      /* 24px */
.text-3xl   { font-size: 1.875rem; line-height: 2.25rem; } /* 30px */
```

**字体权重**:
```css
.font-light    { font-weight: 300; }
.font-normal   { font-weight: 400; }
.font-medium   { font-weight: 500; }
.font-semibold { font-weight: 600; }
.font-bold     { font-weight: 700; }
```

### 间距系统

**内边距和外边距**:
```css
.p-1  { padding: 0.25rem; }  /* 4px */
.p-2  { padding: 0.5rem; }   /* 8px */
.p-3  { padding: 0.75rem; }  /* 12px */
.p-4  { padding: 1rem; }     /* 16px */
.p-6  { padding: 1.5rem; }   /* 24px */
.p-8  { padding: 2rem; }     /* 32px */
.p-12 { padding: 3rem; }     /* 48px */
```

**组件间距规范**:
```typescript
const spacing = {
  // 组件内部间距
  component: {
    xs: 'p-2',    // 8px - 小组件
    sm: 'p-4',    // 16px - 中等组件
    md: 'p-6',    // 24px - 大组件
    lg: 'p-8',    // 32px - 页面级组件
  },
  
  // 组件间距
  between: {
    xs: 'space-y-2',  // 8px
    sm: 'space-y-4',  // 16px
    md: 'space-y-6',  // 24px
    lg: 'space-y-8',  // 32px
  },
  
  // 栅格间距
  grid: {
    xs: 'gap-2',  // 8px
    sm: 'gap-4',  // 16px
    md: 'gap-6',  // 24px
    lg: 'gap-8',  // 32px
  }
}
```

## 🏗️ 组件样式架构

### 基础组件样式

**Button 组件样式**:
```typescript
const buttonVariants = {
  // 基础样式
  base: 'inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50',
  
  // 变体样式
  variants: {
    variant: {
      default: 'bg-primary text-primary-foreground hover:bg-primary/90',
      destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
      outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
      secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
      ghost: 'hover:bg-accent hover:text-accent-foreground',
      link: 'text-primary underline-offset-4 hover:underline',
    },
    size: {
      default: 'h-10 px-4 py-2',
      sm: 'h-9 rounded-md px-3',
      lg: 'h-11 rounded-md px-8',
      icon: 'h-10 w-10',
    },
  },
}
```

**Card 组件样式**:
```typescript
const cardStyles = {
  base: 'rounded-lg border bg-card text-card-foreground shadow-sm',
  header: 'flex flex-col space-y-1.5 p-6',
  title: 'text-2xl font-semibold leading-none tracking-tight',
  description: 'text-sm text-muted-foreground',
  content: 'p-6 pt-0',
  footer: 'flex items-center p-6 pt-0',
}
```

### 布局样式

**容器样式**:
```css
/* 页面容器 */
.page-container {
  @apply max-w-7xl mx-auto px-4 sm:px-6 lg:px-8;
}

/* 内容容器 */
.content-container {
  @apply max-w-4xl mx-auto;
}

/* 卡片容器 */
.card-container {
  @apply grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6;
}
```

**栅格系统**:
```css
/* 响应式栅格 */
.grid-responsive {
  @apply grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6;
}

/* 自适应栅格 */
.grid-auto {
  @apply grid grid-cols-[repeat(auto-fit,minmax(280px,1fr))];
}
```

### 动画和过渡

**过渡效果**:
```css
/* 基础过渡 */
.transition-base {
  @apply transition-all duration-200 ease-in-out;
}

/* 颜色过渡 */
.transition-colors {
  @apply transition-colors duration-150 ease-in-out;
}

/* 变换过渡 */
.transition-transform {
  @apply transition-transform duration-200 ease-in-out;
}
```

**动画效果**:
```css
/* 加载动画 */
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1s linear infinite;
}

/* 淡入动画 */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in {
  animation: fadeIn 0.3s ease-out;
}

/* 脉冲动画 */
@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
```

## 📱 响应式设计

### 断点系统

```css
/* Tailwind CSS 断点 */
sm: '640px',   /* 小屏幕 */
md: '768px',   /* 中等屏幕 */
lg: '1024px',  /* 大屏幕 */
xl: '1280px',  /* 超大屏幕 */
2xl: '1536px', /* 超超大屏幕 */
```

### 响应式组件

**导航栏响应式**:
```typescript
function Header() {
  return (
    <header className="bg-background border-b">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-16">
          {/* Logo - 始终显示 */}
          <div className="flex-shrink-0">
            <Logo />
          </div>
          
          {/* 桌面导航 - 中等屏幕以上显示 */}
          <nav className="hidden md:flex space-x-8">
            <NavLinks />
          </nav>
          
          {/* 移动菜单按钮 - 中等屏幕以下显示 */}
          <div className="md:hidden">
            <MobileMenuButton />
          </div>
        </div>
      </div>
    </header>
  )
}
```

**卡片网格响应式**:
```typescript
function ConfigGrid() {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 sm:gap-6">
      {configs.map(config => (
        <ConfigCard key={config.id} config={config} />
      ))}
    </div>
  )
}
```

### 移动端优化

**触摸友好的交互**:
```css
/* 最小触摸目标 44px */
.touch-target {
  @apply min-h-[44px] min-w-[44px];
}

/* 移动端按钮 */
.mobile-button {
  @apply touch-target px-4 py-3 text-base;
}
```

**移动端布局**:
```typescript
function MobileLayout({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-screen bg-background">
      {/* 固定头部 */}
      <header className="sticky top-0 z-50 bg-background border-b">
        <MobileHeader />
      </header>
      
      {/* 可滚动内容 */}
      <main className="pb-safe">
        <div className="px-4 py-6">
          {children}
        </div>
      </main>
      
      {/* 底部导航 */}
      <nav className="fixed bottom-0 left-0 right-0 bg-background border-t pb-safe">
        <MobileNavigation />
      </nav>
    </div>
  )
}
```

## 🎯 主题系统

### 深色模式

**主题切换**:
```typescript
import { useTheme } from 'next-themes'

function ThemeToggle() {
  const { theme, setTheme } = useTheme()
  
  return (
    <button
      onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
      className="p-2 rounded-md hover:bg-accent"
    >
      {theme === 'dark' ? <Sun /> : <Moon />}
    </button>
  )
}
```

**主题感知组件**:
```typescript
function ThemedCard({ children }: { children: ReactNode }) {
  return (
    <div className="bg-card text-card-foreground border rounded-lg p-6 shadow-sm dark:shadow-md">
      {children}
    </div>
  )
}
```

### 自定义主题

**主题配置**:
```typescript
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        // 自定义品牌色
        brand: {
          50: '#f0f9ff',
          100: '#e0f2fe',
          500: '#0ea5e9',
          600: '#0284c7',
          900: '#0c4a6e',
        },
        
        // 功能色
        success: '#10b981',
        warning: '#f59e0b',
        error: '#ef4444',
        info: '#3b82f6',
      },
      
      // 自定义字体
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      
      // 自定义阴影
      boxShadow: {
        'soft': '0 2px 8px 0 rgba(0, 0, 0, 0.08)',
        'medium': '0 4px 16px 0 rgba(0, 0, 0, 0.12)',
        'strong': '0 8px 32px 0 rgba(0, 0, 0, 0.16)',
      },
    },
  },
}
```

## 🔧 样式工具和最佳实践

### CSS-in-JS 工具

**clsx 和 cn 工具函数**:
```typescript
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// 使用示例
function Button({ className, variant, ...props }) {
  return (
    <button
      className={cn(
        'px-4 py-2 rounded-md font-medium',
        variant === 'primary' && 'bg-blue-500 text-white',
        variant === 'secondary' && 'bg-gray-200 text-gray-900',
        className
      )}
      {...props}
    />
  )
}
```

### 样式组合模式

**变体组合**:
```typescript
import { cva, type VariantProps } from 'class-variance-authority'

const buttonVariants = cva(
  'inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
        outline: 'border border-input hover:bg-accent hover:text-accent-foreground',
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-9 rounded-md px-3',
        lg: 'h-11 rounded-md px-8',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  }
)

interface ButtonProps extends VariantProps<typeof buttonVariants> {
  // 其他 props
}
```

### 性能优化

**样式优化策略**:
```typescript
// 1. 使用 CSS 变量减少重复
const theme = {
  colors: {
    primary: 'hsl(var(--primary))',
    secondary: 'hsl(var(--secondary))',
  }
}

// 2. 条件样式优化
const getButtonStyles = useMemo(() => {
  return cn(
    'base-button-styles',
    variant === 'primary' && 'primary-styles',
    size === 'large' && 'large-styles'
  )
}, [variant, size])

// 3. 样式缓存
const styleCache = new Map()

function getCachedStyles(key: string, generator: () => string) {
  if (!styleCache.has(key)) {
    styleCache.set(key, generator())
  }
  return styleCache.get(key)
}
```

## 📋 样式规范检查清单

### 代码质量

- [ ] 使用语义化的类名
- [ ] 遵循 BEM 命名规范（如适用）
- [ ] 避免内联样式
- [ ] 使用 CSS 变量定义主题色彩
- [ ] 确保样式的可复用性

### 响应式设计

- [ ] 移动端优先设计
- [ ] 测试所有断点
- [ ] 确保触摸友好的交互
- [ ] 优化移动端性能

### 可访问性

- [ ] 足够的颜色对比度
- [ ] 支持键盘导航
- [ ] 提供焦点指示器
- [ ] 支持屏幕阅读器

### 性能

- [ ] 最小化 CSS 包大小
- [ ] 避免不必要的重绘
- [ ] 使用 CSS 变量减少重复
- [ ] 优化动画性能

## 📚 相关文档

- [技术栈详解](/frontend/tech-stack)
- [组件文档](/frontend/components)
- [开发指南](/frontend/development)
- [API 接口](/frontend/api)