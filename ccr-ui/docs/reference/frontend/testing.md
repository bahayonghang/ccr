# 前端测试指南

CCR UI 采用现代化的测试策略，确保代码质量、功能正确性和用户体验的一致性。

## 🧪 测试架构

### 测试金字塔

```
    /\
   /  \     E2E Tests (Cypress)
  /____\    Integration Tests (Vitest + Testing Library)
 /______\   Unit Tests (Vitest)
/__________\ Static Analysis (TypeScript + ESLint)
```

### 测试工具栈

- **单元测试**: Vitest + @testing-library/react
- **集成测试**: Vitest + @testing-library/react + MSW
- **端到端测试**: Cypress
- **视觉回归测试**: Chromatic (可选)
- **性能测试**: Lighthouse CI
- **静态分析**: TypeScript + ESLint + Prettier

## 🔬 单元测试

### 测试配置

**vitest.config.ts**:
```typescript
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    css: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.d.ts',
        '**/*.config.*',
        '**/coverage/**',
      ],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
})
```

**测试设置文件** (`src/test/setup.ts`):
```typescript
import '@testing-library/jest-dom'
import { cleanup } from '@testing-library/react'
import { afterEach, vi } from 'vitest'

// 每个测试后清理
afterEach(() => {
  cleanup()
})

// Mock Next.js router
vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: vi.fn(),
    replace: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
    refresh: vi.fn(),
    prefetch: vi.fn(),
  }),
  usePathname: () => '/',
  useSearchParams: () => new URLSearchParams(),
}))

// Mock Next.js image
vi.mock('next/image', () => ({
  default: ({ src, alt, ...props }: any) => (
    <img src={src} alt={alt} {...props} />
  ),
}))

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})
```

### 组件测试示例

**Button 组件测试**:
```typescript
// src/components/ui/Button.test.tsx
import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { Button } from './Button'

describe('Button', () => {
  it('renders with correct text', () => {
    render(<Button>Click me</Button>)
    expect(screen.getByRole('button', { name: 'Click me' })).toBeInTheDocument()
  })

  it('applies correct variant styles', () => {
    render(<Button variant="destructive">Delete</Button>)
    const button = screen.getByRole('button')
    expect(button).toHaveClass('bg-destructive')
  })

  it('handles click events', () => {
    const handleClick = vi.fn()
    render(<Button onClick={handleClick}>Click me</Button>)
    
    fireEvent.click(screen.getByRole('button'))
    expect(handleClick).toHaveBeenCalledTimes(1)
  })

  it('shows loading state', () => {
    render(<Button loading>Loading</Button>)
    const button = screen.getByRole('button')
    
    expect(button).toBeDisabled()
    expect(screen.getByRole('button')).toHaveAttribute('disabled')
  })

  it('forwards ref correctly', () => {
    const ref = vi.fn()
    render(<Button ref={ref}>Button</Button>)
    expect(ref).toHaveBeenCalled()
  })
})
```

**ConfigList 组件测试**:
```typescript
// src/components/features/ConfigList.test.tsx
import { render, screen, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ConfigList } from './ConfigList'
import { useConfigs } from '@/hooks/useConfigs'

// Mock hook
vi.mock('@/hooks/useConfigs')

const mockUseConfigs = vi.mocked(useConfigs)

describe('ConfigList', () => {
  beforeEach(() => {
    mockUseConfigs.mockReturnValue({
      configs: [],
      currentConfig: null,
      loading: false,
      error: null,
      switchConfig: vi.fn(),
      deleteConfig: vi.fn(),
      refreshConfigs: vi.fn(),
    })
  })

  it('shows loading state', () => {
    mockUseConfigs.mockReturnValue({
      ...mockUseConfigs(),
      loading: true,
    })

    render(<ConfigList />)
    
    // 检查加载骨架屏
    expect(screen.getAllByTestId('config-skeleton')).toHaveLength(6)
  })

  it('renders config list', async () => {
    const mockConfigs = [
      {
        name: 'test-config',
        description: 'Test configuration',
        provider: 'openai',
        model: 'gpt-4',
        account: 'test@example.com',
        tags: ['test'],
        is_default: false,
      },
    ]

    mockUseConfigs.mockReturnValue({
      ...mockUseConfigs(),
      configs: mockConfigs,
    })

    render(<ConfigList />)
    
    await waitFor(() => {
      expect(screen.getByText('test-config')).toBeInTheDocument()
      expect(screen.getByText('Test configuration')).toBeInTheDocument()
      expect(screen.getByText('openai')).toBeInTheDocument()
    })
  })

  it('handles config switching', async () => {
    const mockSwitchConfig = vi.fn()
    const mockConfigs = [
      {
        name: 'test-config',
        description: 'Test configuration',
        provider: 'openai',
        model: 'gpt-4',
        account: 'test@example.com',
        tags: [],
        is_default: false,
      },
    ]

    mockUseConfigs.mockReturnValue({
      ...mockUseConfigs(),
      configs: mockConfigs,
      switchConfig: mockSwitchConfig,
    })

    render(<ConfigList />)
    
    const switchButton = screen.getByText('切换到此配置')
    fireEvent.click(switchButton)
    
    expect(mockSwitchConfig).toHaveBeenCalledWith('test-config')
  })
})
```

### Hook 测试

**useConfigs Hook 测试**:
```typescript
// src/hooks/useConfigs.test.ts
import { renderHook, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useConfigs } from './useConfigs'

// Mock fetch
global.fetch = vi.fn()

describe('useConfigs', () => {
  beforeEach(() => {
    vi.resetAllMocks()
  })

  it('fetches configs on mount', async () => {
    const mockConfigs = [
      { name: 'config1', provider: 'openai' },
      { name: 'config2', provider: 'anthropic' },
    ]

    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      json: async () => ({ configs: mockConfigs }),
    } as Response)

    const { result } = renderHook(() => useConfigs())

    expect(result.current.loading).toBe(true)

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
      expect(result.current.configs).toEqual(mockConfigs)
    })
  })

  it('handles fetch error', async () => {
    vi.mocked(fetch).mockRejectedValueOnce(new Error('Network error'))

    const { result } = renderHook(() => useConfigs())

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
      expect(result.current.error).toBe('Network error')
    })
  })

  it('switches config successfully', async () => {
    const mockConfigs = [{ name: 'config1', provider: 'openai' }]
    
    vi.mocked(fetch)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ configs: mockConfigs }),
      } as Response)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true }),
      } as Response)

    const { result } = renderHook(() => useConfigs())

    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    await result.current.switchConfig('config1')

    expect(fetch).toHaveBeenCalledWith('/api/configs/switch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ config_name: 'config1' }),
    })
  })
})
```

## 🔗 集成测试

### API 集成测试

**MSW 设置**:
```typescript
// src/test/mocks/handlers.ts
import { rest } from 'msw'

export const handlers = [
  // 配置相关 API
  rest.get('/api/configs', (req, res, ctx) => {
    return res(
      ctx.json({
        configs: [
          {
            name: 'test-config',
            description: 'Test configuration',
            provider: 'openai',
            model: 'gpt-4',
            account: 'test@example.com',
            tags: ['test'],
            is_default: false,
          },
        ],
      })
    )
  }),

  rest.post('/api/configs/switch', (req, res, ctx) => {
    return res(ctx.json({ success: true }))
  }),

  rest.delete('/api/configs/:name', (req, res, ctx) => {
    return res(ctx.json({ success: true }))
  }),

  // 命令执行 API
  rest.post('/api/commands/execute', (req, res, ctx) => {
    return res(
      ctx.json({
        success: true,
        stdout: 'Command executed successfully',
        stderr: '',
      })
    )
  }),
]
```

**集成测试示例**:
```typescript
// src/components/features/ConfigList.integration.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, beforeAll, afterEach, afterAll } from 'vitest'
import { setupServer } from 'msw/node'
import { handlers } from '@/test/mocks/handlers'
import { ConfigList } from './ConfigList'

const server = setupServer(...handlers)

describe('ConfigList Integration', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  it('loads and displays configs from API', async () => {
    render(<ConfigList />)

    // 等待加载完成
    await waitFor(() => {
      expect(screen.queryByTestId('config-skeleton')).not.toBeInTheDocument()
    })

    // 验证配置显示
    expect(screen.getByText('test-config')).toBeInTheDocument()
    expect(screen.getByText('Test configuration')).toBeInTheDocument()
  })

  it('switches config via API', async () => {
    render(<ConfigList />)

    await waitFor(() => {
      expect(screen.getByText('切换到此配置')).toBeInTheDocument()
    })

    fireEvent.click(screen.getByText('切换到此配置'))

    await waitFor(() => {
      expect(screen.getByText('切换成功')).toBeInTheDocument()
    })
  })
})
```

## 🎭 端到端测试

### Cypress 配置

**cypress.config.ts**:
```typescript
import { defineConfig } from 'cypress'

export default defineConfig({
  e2e: {
    baseUrl: 'http://localhost:3000',
    viewportWidth: 1280,
    viewportHeight: 720,
    video: false,
    screenshotOnRunFailure: true,
    setupNodeEvents(on, config) {
      // 插件配置
    },
  },
  component: {
    devServer: {
      framework: 'next',
      bundler: 'webpack',
    },
  },
})
```

### E2E 测试示例

**配置管理流程测试**:
```typescript
// cypress/e2e/config-management.cy.ts
describe('Config Management', () => {
  beforeEach(() => {
    cy.visit('/configs')
  })

  it('displays config list', () => {
    cy.get('[data-testid="config-list"]').should('be.visible')
    cy.get('[data-testid="config-card"]').should('have.length.at.least', 1)
  })

  it('switches between configs', () => {
    // 点击切换按钮
    cy.get('[data-testid="switch-config-btn"]').first().click()
    
    // 确认切换
    cy.get('[data-testid="confirm-switch"]').click()
    
    // 验证成功消息
    cy.get('[data-testid="success-message"]')
      .should('be.visible')
      .and('contain', '配置切换成功')
    
    // 验证当前配置更新
    cy.get('[data-testid="current-config"]')
      .should('contain', '当前配置')
  })

  it('creates new config', () => {
    cy.get('[data-testid="create-config-btn"]').click()
    
    // 填写表单
    cy.get('[data-testid="config-name"]').type('new-test-config')
    cy.get('[data-testid="config-provider"]').select('openai')
    cy.get('[data-testid="config-model"]').select('gpt-4')
    cy.get('[data-testid="config-account"]').type('test@example.com')
    
    // 提交表单
    cy.get('[data-testid="submit-config"]').click()
    
    // 验证创建成功
    cy.get('[data-testid="success-message"]')
      .should('contain', '配置创建成功')
    
    // 验证新配置出现在列表中
    cy.get('[data-testid="config-list"]')
      .should('contain', 'new-test-config')
  })

  it('deletes config with confirmation', () => {
    // 点击删除按钮
    cy.get('[data-testid="delete-config-btn"]').first().click()
    
    // 确认删除
    cy.get('[data-testid="confirm-delete"]').click()
    
    // 验证删除成功
    cy.get('[data-testid="success-message"]')
      .should('contain', '配置删除成功')
  })
})
```

**命令执行测试**:
```typescript
// cypress/e2e/command-execution.cy.ts
describe('Command Execution', () => {
  beforeEach(() => {
    cy.visit('/commands')
  })

  it('executes command successfully', () => {
    // 输入命令
    cy.get('[data-testid="command-input"]').type('list')
    
    // 执行命令
    cy.get('[data-testid="execute-btn"]').click()
    
    // 验证执行状态
    cy.get('[data-testid="command-status"]')
      .should('contain', '运行中')
    
    // 等待执行完成
    cy.get('[data-testid="command-status"]', { timeout: 10000 })
      .should('contain', '完成')
    
    // 验证输出
    cy.get('[data-testid="command-output"]')
      .should('be.visible')
      .and('not.be.empty')
  })

  it('handles command errors', () => {
    // 输入无效命令
    cy.get('[data-testid="command-input"]').type('invalid-command')
    cy.get('[data-testid="execute-btn"]').click()
    
    // 验证错误状态
    cy.get('[data-testid="command-status"]', { timeout: 10000 })
      .should('contain', '失败')
    
    // 验证错误信息
    cy.get('[data-testid="command-error"]')
      .should('be.visible')
      .and('contain', '命令不存在')
  })
})
```

## 📊 测试覆盖率和质量

### 覆盖率配置

**package.json 脚本**:
```json
{
  "scripts": {
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest --coverage",
    "test:watch": "vitest --watch",
    "test:e2e": "cypress run",
    "test:e2e:open": "cypress open",
    "test:all": "npm run test:coverage && npm run test:e2e"
  }
}
```

### 质量门禁

**覆盖率要求**:
```typescript
// vitest.config.ts
export default defineConfig({
  test: {
    coverage: {
      thresholds: {
        global: {
          branches: 80,
          functions: 80,
          lines: 80,
          statements: 80,
        },
        // 关键文件更高要求
        'src/hooks/**': {
          branches: 90,
          functions: 90,
          lines: 90,
          statements: 90,
        },
      },
    },
  },
})
```

### CI/CD 集成

**GitHub Actions 配置**:
```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run unit tests
        run: npm run test:coverage
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          file: ./coverage/lcov.info
      
      - name: Run E2E tests
        run: |
          npm run build
          npm start &
          npx wait-on http://localhost:3000
          npm run test:e2e
```

## 🛠️ 测试工具和辅助函数

### 测试工具函数

```typescript
// src/test/utils.tsx
import { ReactElement } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { ThemeProvider } from 'next-themes'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

// 创建测试用的 QueryClient
function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })
}

// 自定义渲染函数
interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  queryClient?: QueryClient
}

export function renderWithProviders(
  ui: ReactElement,
  {
    queryClient = createTestQueryClient(),
    ...renderOptions
  }: CustomRenderOptions = {}
) {
  function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <ThemeProvider attribute="class" defaultTheme="light">
          {children}
        </ThemeProvider>
      </QueryClientProvider>
    )
  }

  return {
    ...render(ui, { wrapper: Wrapper, ...renderOptions }),
    queryClient,
  }
}

// 等待异步操作
export async function waitForLoadingToFinish() {
  await waitFor(() => {
    expect(screen.queryByTestId('loading')).not.toBeInTheDocument()
  })
}

// 模拟用户交互
export function mockUserInteraction() {
  return {
    clickButton: (name: string) => {
      fireEvent.click(screen.getByRole('button', { name }))
    },
    fillInput: (label: string, value: string) => {
      fireEvent.change(screen.getByLabelText(label), {
        target: { value },
      })
    },
    selectOption: (label: string, value: string) => {
      fireEvent.change(screen.getByLabelText(label), {
        target: { value },
      })
    },
  }
}
```

### Mock 数据工厂

```typescript
// src/test/factories.ts
import { faker } from '@faker-js/faker'
import type { ConfigItem, CommandResult } from '@/types/api'

export function createMockConfig(overrides?: Partial<ConfigItem>): ConfigItem {
  return {
    name: faker.lorem.slug(),
    description: faker.lorem.sentence(),
    provider: faker.helpers.arrayElement(['openai', 'anthropic', 'google']),
    model: faker.helpers.arrayElement(['gpt-4', 'claude-3', 'gemini-pro']),
    account: faker.internet.email(),
    tags: faker.helpers.arrayElements(['test', 'prod', 'dev'], { min: 0, max: 3 }),
    is_default: faker.datatype.boolean(),
    ...overrides,
  }
}

export function createMockCommandResult(
  overrides?: Partial<CommandResult>
): CommandResult {
  return {
    id: faker.string.uuid(),
    command: faker.lorem.words(3),
    status: faker.helpers.arrayElement(['running', 'completed', 'failed']),
    output: faker.lorem.paragraphs(2),
    error: faker.datatype.boolean() ? faker.lorem.sentence() : undefined,
    startTime: faker.date.recent(),
    endTime: faker.date.recent(),
    ...overrides,
  }
}
```

## 📋 测试最佳实践

### 测试原则

1. **AAA 模式**: Arrange（准备）、Act（执行）、Assert（断言）
2. **单一职责**: 每个测试只验证一个功能点
3. **独立性**: 测试之间不应相互依赖
4. **可读性**: 测试名称应清晰描述测试内容
5. **可维护性**: 避免重复代码，使用工具函数

### 测试命名规范

```typescript
// ✅ 好的测试名称
describe('Button component', () => {
  it('renders with correct text', () => {})
  it('applies primary variant styles when variant is primary', () => {})
  it('calls onClick handler when clicked', () => {})
  it('shows loading spinner when loading is true', () => {})
  it('is disabled when loading is true', () => {})
})

// ❌ 不好的测试名称
describe('Button', () => {
  it('works', () => {})
  it('test click', () => {})
  it('loading', () => {})
})
```

### 测试数据管理

```typescript
// ✅ 使用工厂函数
const mockConfig = createMockConfig({
  name: 'test-config',
  provider: 'openai',
})

// ✅ 使用常量
const TEST_CONFIG = {
  name: 'test-config',
  provider: 'openai',
  model: 'gpt-4',
} as const

// ❌ 内联数据
it('renders config', () => {
  render(<ConfigCard config={{
    name: 'test-config',
    provider: 'openai',
    model: 'gpt-4',
    // ... 大量重复数据
  }} />)
})
```

## 📚 相关文档

- [技术栈详解](/frontend/tech-stack)
- [开发指南](/frontend/development)
- [组件文档](/frontend/components)
- [API 接口](/frontend/api)