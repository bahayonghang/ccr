# 前端开发指南

本指南将帮助你了解 CCR UI 前端项目（基于 Next.js 16 Beta）的开发流程、编码规范和最佳实践。

## 🚀 开发环境设置

### 1. 安装依赖

```bash
cd frontend
npm install
```

### 2. 启动开发服务器

```bash
# 使用 Turbopack（默认）启动开发服务器
npm run dev

# 开发服务器将在 http://localhost:3000 启动
# Turbopack 提供 2-5x 更快的启动和热更新速度
```

### 3. 代码检查

```bash
# 运行 ESLint
npm run lint

# TypeScript 类型检查
npm run type-check
```

### 4. 构建项目

```bash
# 生产构建
npm run build

# 启动生产服务器
npm run start
```

## 📝 编码规范

### Next.js 特定规范

#### 1. Server vs Client Components

```typescript
// Server Component（默认）- 不需要声明
// 在服务器上渲染，可以直接访问数据库/文件系统
export default function ServerComponent() {
  const data = await fetchData() // 可以使用 async/await
  return <div>{data}</div>
}

// Client Component - 需要 'use client' 指令
// 在客户端渲染，可以使用 hooks 和交互
'use client'

export default function ClientComponent() {
  const [state, setState] = useState() // 可以使用 hooks
  return <button onClick={() => setState(...)}>Click</button>
}
```

#### 2. 组件 Props 类型

为所有组件定义明确的 Props 类型：

```typescript
interface ConfigItemProps {
  config: Config;
  onSwitch: (configName: string) => void;
  isLoading?: boolean;
}

const ConfigItem = ({ 
  config, 
  onSwitch, 
  isLoading = false 
}: ConfigItemProps) => {
  // 组件实现
};
```

#### 3. 类型定义

优先使用 `interface` 而不是 `type`：

```typescript
// ✅ 推荐
interface Config {
  name: string;
  path: string;
  isActive: boolean;
}

// ❌ 避免（除非需要联合类型）
type Config = {
  name: string;
  path: string;
  isActive: boolean;
}
```

#### 3. 事件处理器类型

使用正确的事件类型：

```typescript
const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
  event.preventDefault();
  // 处理逻辑
};

const handleClick = (event: React.MouseEvent<HTMLButtonElement>) => {
  // 处理逻辑
};
```

### Next.js 组件规范

#### 1. 页面组件结构 (App Router)

```typescript
// app/configs/page.tsx - 页面组件
import { ConfigList } from '@/components/ConfigList'

// Server Component（默认）
export default async function ConfigsPage() {
  // 可以在服务器上直接获取数据
  const configs = await getConfigs()
  
  return (
    <div>
      <h1>Configurations</h1>
      <ConfigList configs={configs} />
    </div>
  )
}

// 可选：生成页面元数据
export const metadata = {
  title: 'Configurations | CCR UI',
  description: 'Manage your CCR configurations',
}
```

#### 2. 客户端组件结构

```typescript
// components/ConfigList.tsx
'use client' // 声明为客户端组件

import { useState, useEffect } from 'react'
import type { Config } from '@/lib/types'

interface ConfigListProps {
  configs: Config[]
}

export function ConfigList({ configs }: ConfigListProps) {
  // 1. Hooks
  const [selected, setSelected] = useState<string | null>(null)
  
  // 2. 副作用
  useEffect(() => {
    // 副作用逻辑
  }, [selected])
  
  // 3. 事件处理器
  const handleSelect = (id: string) => {
    setSelected(id)
  }
  
  // 4. 渲染逻辑
  return (
    <div>
      {configs.map((config) => (
        <div key={config.name} onClick={() => handleSelect(config.name)}>
          {config.name}
        </div>
      ))}
    </div>
  )
}
```

#### 3. 布局组件

```typescript
// app/layout.tsx - 根布局
import { ThemeProvider } from '@/components/providers/ThemeProvider'
import './globals.css'

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="zh-CN">
      <body>
        <ThemeProvider>
          {children}
        </ThemeProvider>
      </body>
    </html>
  )
}
```

#### 2. 自定义 Hooks

将复杂的状态逻辑提取到自定义 Hooks：

```typescript
// hooks/useConfigs.ts
import { useState, useEffect } from 'react';
import { apiClient } from '../api/client';
import { Config } from '../types';

export const useConfigs = () => {
  const [configs, setConfigs] = useState<Config[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchConfigs = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await apiClient.get<Config[]>('/configs');
      setConfigs(response.data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchConfigs();
  }, []);

  return { configs, loading, error, refetch: fetchConfigs };
};
```

#### 3. 错误边界

为关键组件添加错误边界：

```typescript
import React, { Component, ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Uncaught error:', error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div className="error-boundary">
          <h2>Something went wrong.</h2>
          <details style={{ whiteSpace: 'pre-wrap' }}>
            {this.state.error && this.state.error.toString()}
          </details>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
```

### CSS 和样式规范

#### 1. Tailwind CSS 使用

优先使用 Tailwind CSS 类：

```typescript
// ✅ 推荐
<div className="flex items-center justify-between p-4 bg-white rounded-lg shadow-md">
  <h3 className="text-lg font-semibold text-gray-900">Title</h3>
  <button className="px-4 py-2 text-white bg-blue-600 rounded hover:bg-blue-700">
    Action
  </button>
</div>

// ❌ 避免内联样式
<div style={{ display: 'flex', padding: '16px' }}>
  {/* ... */}
</div>
```

#### 2. 响应式设计

使用 Tailwind 的响应式前缀：

```typescript
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
  {/* 移动端单列，平板双列，桌面三列 */}
</div>
```

#### 3. 深色模式支持

使用 `dark:` 前缀支持深色模式：

```typescript
<div className="bg-white dark:bg-gray-800 text-gray-900 dark:text-white">
  {/* 内容 */}
</div>
```

## 🏗️ 项目架构

### 组件分层

```
components/
├── Layout/           # 布局组件
│   ├── Header.tsx
│   ├── Sidebar.tsx
│   └── Footer.tsx
├── UI/              # 基础 UI 组件
│   ├── Button.tsx
│   ├── Input.tsx
│   ├── Modal.tsx
│   └── Loading.tsx
├── Terminal/        # 终端相关组件
│   ├── Terminal.tsx
│   ├── CommandInput.tsx
│   └── OutputDisplay.tsx
└── Config/          # 配置相关组件
    ├── ConfigList.tsx
    ├── ConfigItem.tsx
    └── ConfigSwitcher.tsx
```

### 状态管理模式

#### 1. 本地状态

使用 `useState` 管理组件内部状态：

```typescript
const [isOpen, setIsOpen] = useState(false);
const [inputValue, setInputValue] = useState('');
```

#### 2. 共享状态

使用 Context API 管理跨组件状态：

```typescript
// contexts/AppContext.tsx
import React, { createContext, useContext, useState } from 'react';

interface AppContextType {
  currentConfig: string | null;
  setCurrentConfig: (config: string) => void;
  theme: 'light' | 'dark';
  toggleTheme: () => void;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export const AppProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [currentConfig, setCurrentConfig] = useState<string | null>(null);
  const [theme, setTheme] = useState<'light' | 'dark'>('light');

  const toggleTheme = () => {
    setTheme(prev => prev === 'light' ? 'dark' : 'light');
  };

  return (
    <AppContext.Provider value={{
      currentConfig,
      setCurrentConfig,
      theme,
      toggleTheme
    }}>
      {children}
    </AppContext.Provider>
  );
};

export const useApp = () => {
  const context = useContext(AppContext);
  if (context === undefined) {
    throw new Error('useApp must be used within an AppProvider');
  }
  return context;
};
```

### API 客户端设计

#### 1. 基础配置（Next.js）

```typescript
// lib/api/client.ts
import axios from 'axios'

// 使用 Next.js 的 API 路由代理
const apiClient = axios.create({
  baseURL: '/api', // 代理到 http://localhost:8081/api
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// 请求拦截器
apiClient.interceptors.request.use(
  (config) => {
    // 添加认证头等
    return config
  },
  (error) => Promise.reject(error)
)

// 响应拦截器
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    // 统一错误处理
    return Promise.reject(error)
  }
)

export { apiClient }
```

#### 2. API 路由代理配置

```javascript
// next.config.mjs
export default {
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://localhost:8081/api/:path*',
      },
    ]
  },
}
```

#### 2. API 服务封装

```typescript
// api/configService.ts
import { apiClient } from './client';
import { Config, SwitchConfigRequest } from '../types';

export const configService = {
  async getConfigs(): Promise<Config[]> {
    const response = await apiClient.get<Config[]>('/configs');
    return response.data;
  },

  async switchConfig(request: SwitchConfigRequest): Promise<void> {
    await apiClient.post('/configs/switch', request);
  },

  async validateConfigs(): Promise<{ valid: boolean; errors?: string[] }> {
    const response = await apiClient.post('/configs/validate');
    return response.data;
  },
};
```

## 🧪 测试策略

### 1. 单元测试

使用 Vitest 和 React Testing Library：

```typescript
// __tests__/ConfigItem.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import ConfigItem from '../components/Config/ConfigItem';

describe('ConfigItem', () => {
  const mockConfig = {
    name: 'test-config',
    path: '/path/to/config',
    isActive: false,
  };

  it('renders config name', () => {
    render(<ConfigItem config={mockConfig} onSwitch={vi.fn()} />);
    expect(screen.getByText('test-config')).toBeInTheDocument();
  });

  it('calls onSwitch when clicked', () => {
    const mockOnSwitch = vi.fn();
    render(<ConfigItem config={mockConfig} onSwitch={mockOnSwitch} />);
    
    fireEvent.click(screen.getByRole('button'));
    expect(mockOnSwitch).toHaveBeenCalledWith('test-config');
  });
});
```

### 2. 集成测试

```typescript
// __tests__/ConfigManagement.test.tsx
import { render, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import ConfigManagement from '../pages/ConfigManagement';
import { configService } from '../api/configService';

vi.mock('../api/configService');

describe('ConfigManagement', () => {
  it('loads and displays configs', async () => {
    const mockConfigs = [
      { name: 'config1', path: '/path1', isActive: true },
      { name: 'config2', path: '/path2', isActive: false },
    ];

    vi.mocked(configService.getConfigs).mockResolvedValue(mockConfigs);

    render(<ConfigManagement />);

    await waitFor(() => {
      expect(screen.getByText('config1')).toBeInTheDocument();
      expect(screen.getByText('config2')).toBeInTheDocument();
    });
  });
});
```

### 3. E2E 测试

使用 Cypress：

```typescript
// cypress/e2e/config-management.cy.ts
describe('Config Management', () => {
  beforeEach(() => {
    cy.visit('/configs');
  });

  it('should display config list', () => {
    cy.get('[data-testid="config-list"]').should('be.visible');
    cy.get('[data-testid="config-item"]').should('have.length.greaterThan', 0);
  });

  it('should switch config', () => {
    cy.get('[data-testid="config-item"]').first().click();
    cy.get('[data-testid="success-message"]').should('be.visible');
  });
});
```

## 🔧 开发工具

### 1. VS Code 配置

推荐的 VS Code 扩展：

```json
// .vscode/extensions.json
{
  "recommendations": [
    "bradlc.vscode-tailwindcss",
    "esbenp.prettier-vscode",
    "dbaeumer.vscode-eslint",
    "ms-vscode.vscode-typescript-next"
  ]
}
```

工作区设置：

```json
// .vscode/settings.json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "typescript.preferences.importModuleSpecifier": "relative",
  "tailwindCSS.experimental.classRegex": [
    ["clsx\\(([^)]*)\\)", "(?:'|\"|`)([^']*)(?:'|\"|`)"]
  ]
}
```

### 2. 调试配置

```json
// .vscode/launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Launch Chrome",
      "request": "launch",
      "type": "chrome",
      "url": "http://localhost:5173",
      "webRoot": "${workspaceFolder}/src"
    }
  ]
}
```

## 📈 性能优化

### 1. 代码分割

```typescript
// 路由级别的代码分割
import { lazy, Suspense } from 'react';

const ConfigManagement = lazy(() => import('./pages/ConfigManagement'));
const CommandExecutor = lazy(() => import('./pages/CommandExecutor'));

function App() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <Routes>
        <Route path="/configs" element={<ConfigManagement />} />
        <Route path="/commands" element={<CommandExecutor />} />
      </Routes>
    </Suspense>
  );
}
```

### 2. 组件优化

```typescript
import { memo, useMemo, useCallback } from 'react';

const ConfigItem = memo<ConfigItemProps>(({ config, onSwitch }) => {
  const handleClick = useCallback(() => {
    onSwitch(config.name);
  }, [config.name, onSwitch]);

  const statusColor = useMemo(() => {
    return config.isActive ? 'text-green-600' : 'text-gray-400';
  }, [config.isActive]);

  return (
    <div onClick={handleClick}>
      <span className={statusColor}>{config.name}</span>
    </div>
  );
});
```

### 3. 虚拟化长列表

对于大量数据的列表，使用虚拟化：

```typescript
import { FixedSizeList as List } from 'react-window';

const VirtualizedConfigList = ({ configs }: { configs: Config[] }) => {
  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => (
    <div style={style}>
      <ConfigItem config={configs[index]} onSwitch={handleSwitch} />
    </div>
  );

  return (
    <List
      height={400}
      itemCount={configs.length}
      itemSize={60}
      width="100%"
    >
      {Row}
    </List>
  );
};
```

## 🚀 部署准备

### 1. 环境变量

```bash
# .env.production
VITE_API_BASE_URL=https://api.example.com
VITE_APP_VERSION=1.0.0
```

### 2. 构建优化

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          router: ['react-router-dom'],
          ui: ['lucide-react'],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },
});
```

## 📚 最佳实践

### 1. 组件设计原则

- **单一职责**：每个组件只负责一个功能
- **可复用性**：设计通用的组件接口
- **可测试性**：避免复杂的组件逻辑
- **可访问性**：遵循 WCAG 指南

### 2. 状态管理原则

- **最小化状态**：只存储必要的状态
- **状态提升**：将共享状态提升到合适的层级
- **不可变更新**：使用不可变的方式更新状态

### 3. 性能优化原则

- **避免不必要的重渲染**：使用 memo、useMemo、useCallback
- **懒加载**：按需加载组件和资源
- **缓存策略**：合理使用缓存减少网络请求

## 🔍 调试技巧

### 1. React DevTools

使用 React DevTools 调试组件状态和性能。

### 2. 网络调试

使用浏览器开发者工具监控 API 请求。

### 3. 错误追踪

```typescript
// 添加全局错误处理
window.addEventListener('unhandledrejection', (event) => {
  console.error('Unhandled promise rejection:', event.reason);
});
```

## 📖 相关文档

- [技术栈详解](/frontend/tech-stack)
- [组件文档](/frontend/components)
- [API 接口](/frontend/api)
- [样式指南](/frontend/styling)
- [测试指南](/frontend/testing)