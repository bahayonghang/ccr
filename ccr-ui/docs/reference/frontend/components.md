# 前端组件文档

CCR UI 前端采用组件化架构，基于 React 19 和 Next.js 16 构建可复用、类型安全的 UI 组件。

## 🏗️ 组件架构

### 组件分类

```
src/components/
├── ui/                 # 基础 UI 组件
│   ├── Button.tsx
│   ├── Input.tsx
│   ├── Modal.tsx
│   └── Card.tsx
├── layout/             # 布局组件
│   ├── Header.tsx
│   ├── Sidebar.tsx
│   └── Footer.tsx
├── features/           # 功能组件
│   ├── ConfigList.tsx
│   ├── CommandRunner.tsx
│   └── HistoryViewer.tsx
└── common/             # 通用组件
    ├── Loading.tsx
    ├── ErrorBoundary.tsx
    └── NotFound.tsx
```

## 🎨 基础 UI 组件

### Button 组件

**文件**: `src/components/ui/Button.tsx`

```typescript
import { ButtonHTMLAttributes, forwardRef } from 'react'
import { cn } from '@/lib/utils'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
}

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'primary', size = 'md', loading, children, disabled, ...props }, ref) => {
    const baseStyles = 'inline-flex items-center justify-center rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50'
    
    const variants = {
      primary: 'bg-primary text-primary-foreground hover:bg-primary/90',
      secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
      outline: 'border border-input bg-background hover:bg-accent hover:text-accent-foreground',
      ghost: 'hover:bg-accent hover:text-accent-foreground',
      destructive: 'bg-destructive text-destructive-foreground hover:bg-destructive/90',
    }
    
    const sizes = {
      sm: 'h-9 px-3 text-sm',
      md: 'h-10 px-4 py-2',
      lg: 'h-11 px-8 text-lg',
    }

    return (
      <button
        className={cn(baseStyles, variants[variant], sizes[size], className)}
        ref={ref}
        disabled={disabled || loading}
        {...props}
      >
        {loading && (
          <svg className="mr-2 h-4 w-4 animate-spin" viewBox="0 0 24 24">
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
        )}
        {children}
      </button>
    )
  }
)

Button.displayName = 'Button'

export { Button, type ButtonProps }
```

**使用示例**:
```typescript
import { Button } from '@/components/ui/Button'

function ConfigActions() {
  return (
    <div className="flex space-x-2">
      <Button variant="primary" size="md">
        保存配置
      </Button>
      <Button variant="outline" size="md">
        取消
      </Button>
      <Button variant="destructive" size="sm">
        删除
      </Button>
    </div>
  )
}
```

### Input 组件

**文件**: `src/components/ui/Input.tsx`

```typescript
import { InputHTMLAttributes, forwardRef } from 'react'
import { cn } from '@/lib/utils'

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string
  error?: string
  helperText?: string
}

const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, type = 'text', label, error, helperText, ...props }, ref) => {
    return (
      <div className="space-y-2">
        {label && (
          <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
            {label}
          </label>
        )}
        <input
          type={type}
          className={cn(
            'flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
            error && 'border-destructive focus-visible:ring-destructive',
            className
          )}
          ref={ref}
          {...props}
        />
        {error && (
          <p className="text-sm text-destructive">{error}</p>
        )}
        {helperText && !error && (
          <p className="text-sm text-muted-foreground">{helperText}</p>
        )}
      </div>
    )
  }
)

Input.displayName = 'Input'

export { Input, type InputProps }
```

### Modal 组件

**文件**: `src/components/ui/Modal.tsx`

```typescript
import { ReactNode, useEffect } from 'react'
import { createPortal } from 'react-dom'
import { X } from 'lucide-react'
import { Button } from './Button'

interface ModalProps {
  isOpen: boolean
  onClose: () => void
  title?: string
  children: ReactNode
  size?: 'sm' | 'md' | 'lg' | 'xl'
}

export function Modal({ isOpen, onClose, title, children, size = 'md' }: ModalProps) {
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden'
    } else {
      document.body.style.overflow = 'unset'
    }

    return () => {
      document.body.style.overflow = 'unset'
    }
  }, [isOpen])

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose()
      }
    }

    if (isOpen) {
      document.addEventListener('keydown', handleEscape)
    }

    return () => {
      document.removeEventListener('keydown', handleEscape)
    }
  }, [isOpen, onClose])

  if (!isOpen) return null

  const sizeClasses = {
    sm: 'max-w-md',
    md: 'max-w-lg',
    lg: 'max-w-2xl',
    xl: 'max-w-4xl',
  }

  return createPortal(
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50"
        onClick={onClose}
      />
      
      {/* Modal */}
      <div className={`relative bg-background rounded-lg shadow-lg w-full mx-4 ${sizeClasses[size]}`}>
        {/* Header */}
        {title && (
          <div className="flex items-center justify-between p-6 border-b">
            <h2 className="text-lg font-semibold">{title}</h2>
            <Button
              variant="ghost"
              size="sm"
              onClick={onClose}
              className="h-6 w-6 p-0"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        )}
        
        {/* Content */}
        <div className="p-6">
          {children}
        </div>
      </div>
    </div>,
    document.body
  )
}
```

## 🏢 布局组件

### Header 组件

**文件**: `src/components/layout/Header.tsx`

```typescript
import { useState } from 'react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { Menu, X, Settings, User } from 'lucide-react'
import { Button } from '@/components/ui/Button'

export function Header() {
  const [isMenuOpen, setIsMenuOpen] = useState(false)
  const pathname = usePathname()

  const navigation = [
    { name: '配置管理', href: '/configs' },
    { name: 'MCP 服务器', href: '/mcp' },
    { name: '代理管理', href: '/agents' },
    { name: '插件管理', href: '/plugins' },
    { name: '斜杠命令', href: '/slash-commands' },
    { name: '历史记录', href: '/history' },
  ]

  return (
    <header className="bg-background border-b">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-16">
          {/* Logo */}
          <Link href="/" className="flex items-center space-x-2">
            <div className="w-8 h-8 bg-primary rounded-md flex items-center justify-center">
              <span className="text-primary-foreground font-bold text-sm">CCR</span>
            </div>
            <span className="font-semibold text-lg">CCR UI</span>
          </Link>

          {/* Desktop Navigation */}
          <nav className="hidden md:flex space-x-8">
            {navigation.map((item) => (
              <Link
                key={item.name}
                href={item.href}
                className={`text-sm font-medium transition-colors hover:text-primary ${
                  pathname === item.href
                    ? 'text-primary'
                    : 'text-muted-foreground'
                }`}
              >
                {item.name}
              </Link>
            ))}
          </nav>

          {/* Actions */}
          <div className="flex items-center space-x-4">
            <Button variant="ghost" size="sm">
              <Settings className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <User className="h-4 w-4" />
            </Button>
            
            {/* Mobile menu button */}
            <Button
              variant="ghost"
              size="sm"
              className="md:hidden"
              onClick={() => setIsMenuOpen(!isMenuOpen)}
            >
              {isMenuOpen ? (
                <X className="h-4 w-4" />
              ) : (
                <Menu className="h-4 w-4" />
              )}
            </Button>
          </div>
        </div>

        {/* Mobile Navigation */}
        {isMenuOpen && (
          <div className="md:hidden">
            <div className="px-2 pt-2 pb-3 space-y-1 sm:px-3">
              {navigation.map((item) => (
                <Link
                  key={item.name}
                  href={item.href}
                  className={`block px-3 py-2 rounded-md text-base font-medium ${
                    pathname === item.href
                      ? 'bg-primary text-primary-foreground'
                      : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
                  }`}
                  onClick={() => setIsMenuOpen(false)}
                >
                  {item.name}
                </Link>
              ))}
            </div>
          </div>
        )}
      </div>
    </header>
  )
}
```

## 🚀 功能组件

### ConfigList 组件

**文件**: `src/components/features/ConfigList.tsx`

```typescript
import { useState, useEffect } from 'react'
import { Check, Settings, Trash2, Edit } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { Card } from '@/components/ui/Card'
import { Badge } from '@/components/ui/Badge'
import { useConfigs } from '@/hooks/useConfigs'
import type { ConfigItem } from '@/types/api'

export function ConfigList() {
  const { configs, currentConfig, loading, switchConfig, deleteConfig } = useConfigs()
  const [switchingConfig, setSwitchingConfig] = useState<string | null>(null)

  const handleSwitch = async (configName: string) => {
    setSwitchingConfig(configName)
    try {
      await switchConfig(configName)
    } finally {
      setSwitchingConfig(null)
    }
  }

  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {Array.from({ length: 6 }).map((_, i) => (
          <Card key={i} className="p-6">
            <div className="animate-pulse">
              <div className="h-4 bg-muted rounded w-3/4 mb-2" />
              <div className="h-3 bg-muted rounded w-1/2 mb-4" />
              <div className="h-8 bg-muted rounded" />
            </div>
          </Card>
        ))}
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Current Config */}
      {currentConfig && (
        <Card className="p-6 border-primary">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center space-x-2">
              <Check className="h-5 w-5 text-primary" />
              <h3 className="font-semibold">当前配置</h3>
            </div>
            <Badge variant="default">活跃</Badge>
          </div>
          <ConfigCard config={currentConfig} isCurrent />
        </Card>
      )}

      {/* All Configs */}
      <div>
        <h3 className="text-lg font-semibold mb-4">所有配置</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {configs.map((config) => (
            <Card key={config.name} className="p-6">
              <ConfigCard
                config={config}
                isCurrent={config.name === currentConfig?.name}
                onSwitch={() => handleSwitch(config.name)}
                onDelete={() => deleteConfig(config.name)}
                switching={switchingConfig === config.name}
              />
            </Card>
          ))}
        </div>
      </div>
    </div>
  )
}

interface ConfigCardProps {
  config: ConfigItem
  isCurrent?: boolean
  onSwitch?: () => void
  onDelete?: () => void
  switching?: boolean
}

function ConfigCard({ config, isCurrent, onSwitch, onDelete, switching }: ConfigCardProps) {
  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h4 className="font-medium">{config.name}</h4>
          <p className="text-sm text-muted-foreground">{config.description}</p>
        </div>
        {config.is_default && (
          <Badge variant="secondary">默认</Badge>
        )}
      </div>

      {/* Details */}
      <div className="space-y-2 text-sm">
        <div className="flex justify-between">
          <span className="text-muted-foreground">提供商:</span>
          <span>{config.provider}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">模型:</span>
          <span>{config.model}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">账户:</span>
          <span>{config.account}</span>
        </div>
      </div>

      {/* Tags */}
      {config.tags.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {config.tags.map((tag) => (
            <Badge key={tag} variant="outline" className="text-xs">
              {tag}
            </Badge>
          ))}
        </div>
      )}

      {/* Actions */}
      {!isCurrent && onSwitch && (
        <div className="flex space-x-2">
          <Button
            variant="primary"
            size="sm"
            onClick={onSwitch}
            loading={switching}
            className="flex-1"
          >
            切换到此配置
          </Button>
          <Button variant="ghost" size="sm">
            <Edit className="h-4 w-4" />
          </Button>
          {onDelete && (
            <Button variant="ghost" size="sm" onClick={onDelete}>
              <Trash2 className="h-4 w-4" />
            </Button>
          )}
        </div>
      )}
    </div>
  )
}
```

### CommandRunner 组件

**文件**: `src/components/features/CommandRunner.tsx`

```typescript
import { useState } from 'react'
import { Play, Square, Copy, Download } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { Card } from '@/components/ui/Card'
import { Badge } from '@/components/ui/Badge'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'
import { tomorrow } from 'react-syntax-highlighter/dist/esm/styles/prism'

interface CommandResult {
  id: string
  command: string
  status: 'running' | 'completed' | 'failed'
  output: string
  error?: string
  startTime: Date
  endTime?: Date
}

export function CommandRunner() {
  const [command, setCommand] = useState('')
  const [results, setResults] = useState<CommandResult[]>([])
  const [isRunning, setIsRunning] = useState(false)

  const runCommand = async () => {
    if (!command.trim() || isRunning) return

    const result: CommandResult = {
      id: Date.now().toString(),
      command: command.trim(),
      status: 'running',
      output: '',
      startTime: new Date(),
    }

    setResults(prev => [result, ...prev])
    setIsRunning(true)

    try {
      const response = await fetch('/api/commands/execute', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command: command.trim() }),
      })

      const data = await response.json()

      setResults(prev =>
        prev.map(r =>
          r.id === result.id
            ? {
                ...r,
                status: data.success ? 'completed' : 'failed',
                output: data.stdout || '',
                error: data.stderr || data.message,
                endTime: new Date(),
              }
            : r
        )
      )
    } catch (error) {
      setResults(prev =>
        prev.map(r =>
          r.id === result.id
            ? {
                ...r,
                status: 'failed',
                error: error instanceof Error ? error.message : '未知错误',
                endTime: new Date(),
              }
            : r
        )
      )
    } finally {
      setIsRunning(false)
      setCommand('')
    }
  }

  const copyOutput = (output: string) => {
    navigator.clipboard.writeText(output)
  }

  const downloadOutput = (result: CommandResult) => {
    const content = `Command: ${result.command}\nOutput:\n${result.output}${
      result.error ? `\nError:\n${result.error}` : ''
    }`
    const blob = new Blob([content], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `command-${result.id}.txt`
    a.click()
    URL.revokeObjectURL(url)
  }

  return (
    <div className="space-y-6">
      {/* Command Input */}
      <Card className="p-6">
        <div className="space-y-4">
          <h3 className="text-lg font-semibold">执行命令</h3>
          <div className="flex space-x-2">
            <Input
              placeholder="输入 CCR 命令，例如: list, current, switch config-name"
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && runCommand()}
              className="flex-1"
            />
            <Button
              onClick={runCommand}
              disabled={!command.trim() || isRunning}
              loading={isRunning}
            >
              <Play className="h-4 w-4 mr-2" />
              执行
            </Button>
          </div>
        </div>
      </Card>

      {/* Results */}
      <div className="space-y-4">
        {results.map((result) => (
          <Card key={result.id} className="p-6">
            <div className="space-y-4">
              {/* Header */}
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <code className="text-sm bg-muted px-2 py-1 rounded">
                    {result.command}
                  </code>
                  <Badge
                    variant={
                      result.status === 'completed'
                        ? 'default'
                        : result.status === 'failed'
                        ? 'destructive'
                        : 'secondary'
                    }
                  >
                    {result.status === 'running' && '运行中'}
                    {result.status === 'completed' && '完成'}
                    {result.status === 'failed' && '失败'}
                  </Badge>
                </div>
                <div className="flex items-center space-x-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => copyOutput(result.output)}
                  >
                    <Copy className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => downloadOutput(result)}
                  >
                    <Download className="h-4 w-4" />
                  </Button>
                </div>
              </div>

              {/* Output */}
              {result.output && (
                <div>
                  <h4 className="text-sm font-medium mb-2">输出:</h4>
                  <SyntaxHighlighter
                    language="bash"
                    style={tomorrow}
                    customStyle={{
                      margin: 0,
                      borderRadius: '0.5rem',
                      fontSize: '0.875rem',
                    }}
                  >
                    {result.output}
                  </SyntaxHighlighter>
                </div>
              )}

              {/* Error */}
              {result.error && (
                <div>
                  <h4 className="text-sm font-medium mb-2 text-destructive">错误:</h4>
                  <SyntaxHighlighter
                    language="bash"
                    style={tomorrow}
                    customStyle={{
                      margin: 0,
                      borderRadius: '0.5rem',
                      fontSize: '0.875rem',
                      backgroundColor: '#fef2f2',
                    }}
                  >
                    {result.error}
                  </SyntaxHighlighter>
                </div>
              )}

              {/* Timing */}
              <div className="text-xs text-muted-foreground">
                开始时间: {result.startTime.toLocaleString()}
                {result.endTime && (
                  <span className="ml-4">
                    耗时: {result.endTime.getTime() - result.startTime.getTime()}ms
                  </span>
                )}
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  )
}
```

## 🔧 通用组件

### Loading 组件

**文件**: `src/components/common/Loading.tsx`

```typescript
interface LoadingProps {
  size?: 'sm' | 'md' | 'lg'
  text?: string
}

export function Loading({ size = 'md', text }: LoadingProps) {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-8 w-8',
    lg: 'h-12 w-12',
  }

  return (
    <div className="flex flex-col items-center justify-center space-y-2">
      <svg
        className={`animate-spin ${sizeClasses[size]}`}
        viewBox="0 0 24 24"
      >
        <circle
          className="opacity-25"
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          strokeWidth="4"
        />
        <path
          className="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        />
      </svg>
      {text && <p className="text-sm text-muted-foreground">{text}</p>}
    </div>
  )
}
```

### ErrorBoundary 组件

**文件**: `src/components/common/ErrorBoundary.tsx`

```typescript
import { Component, ErrorInfo, ReactNode } from 'react'
import { AlertTriangle, RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { Card } from '@/components/ui/Card'

interface Props {
  children: ReactNode
  fallback?: ReactNode
}

interface State {
  hasError: boolean
  error?: Error
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback
      }

      return (
        <Card className="p-8 text-center">
          <div className="flex flex-col items-center space-y-4">
            <AlertTriangle className="h-12 w-12 text-destructive" />
            <div>
              <h2 className="text-lg font-semibold">出现了一些问题</h2>
              <p className="text-muted-foreground mt-2">
                {this.state.error?.message || '未知错误'}
              </p>
            </div>
            <Button
              onClick={() => this.setState({ hasError: false, error: undefined })}
              variant="outline"
            >
              <RefreshCw className="h-4 w-4 mr-2" />
              重试
            </Button>
          </div>
        </Card>
      )
    }

    return this.props.children
  }
}
```

## 🎯 组件开发规范

### 1. 命名规范

```typescript
// ✅ 正确 - PascalCase
export function ConfigList() {}
export function UserProfile() {}

// ❌ 错误 - camelCase
export function configList() {}
export function userProfile() {}
```

### 2. Props 接口

```typescript
// ✅ 正确 - 明确的接口定义
interface ButtonProps {
  variant?: 'primary' | 'secondary'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
  children: ReactNode
}

// ❌ 错误 - 使用 any
interface ButtonProps {
  [key: string]: any
}
```

### 3. 默认导出 vs 命名导出

```typescript
// ✅ 推荐 - 命名导出（便于重构）
export function Button() {}
export { Button }

// ✅ 可接受 - 默认导出（页面组件）
export default function HomePage() {}
```

### 4. 组件文档

```typescript
/**
 * 配置列表组件
 * 
 * @example
 * ```tsx
 * <ConfigList 
 *   onSelect={(config) => console.log(config)}
 *   loading={false}
 * />
 * ```
 */
export function ConfigList({ onSelect, loading }: ConfigListProps) {
  // 组件实现
}
```

### StatsView 组件（新增）

统计分析视图组件，提供完整的 API 使用统计和成本追踪界面。

**文件**: `src/views/StatsView.vue`

**功能特性**:
- ✅ 4 个概览卡片（总成本、API 调用、输入/输出 Token）
- ✅ Token 详细统计、按模型/项目/提供商分组（调用次数）、成本趋势
- ✅ 时间范围选择器、提供商统计模态（按钮触发）、实时刷新、响应式设计 + 深色模式

详细使用说明请参考 [统计功能指南](../guide/stats.md)。

---

## 📚 相关文档

- [技术栈详解](/frontend/tech-stack)
- [开发指南](/frontend/development)
- [API 接口](/frontend/api)
- [样式指南](/frontend/styling)
- [统计功能指南](../guide/stats.md)
