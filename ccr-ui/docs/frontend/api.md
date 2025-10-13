# 前端 API 接口

本文档详细介绍 CCR UI 前端与后端 API 的交互接口和使用方法。

## 🌐 API 基础配置

### HTTP 客户端配置

```typescript
// src/lib/api/client.ts
import axios from 'axios'

// Next.js 使用 API 路由代理，无需直接配置后端 URL
const apiClient = axios.create({
  baseURL: '/api', // 通过 Next.js rewrites 代理到后端
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// 请求拦截器
apiClient.interceptors.request.use(
  (config) => {
    if (process.env.NODE_ENV === 'development') {
      console.log('API Request:', config.method?.toUpperCase(), config.url)
    }
    return config
  },
  (error) => Promise.reject(error)
)

// 响应拦截器
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('API Error:', error.response?.data || error.message)
    return Promise.reject(error)
  }
)

export { apiClient }
```

### API 代理配置

```javascript
// next.config.mjs
export default {
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://localhost:8081/api/:path*', // 开发环境
      },
    ]
  },
}
```

### 环境变量配置

```bash
# .env.local (开发环境)
NEXT_PUBLIC_API_URL=http://localhost:8081/api

# .env.production (生产环境)
NEXT_PUBLIC_API_URL=https://api.your-domain.com/api
```

::: tip Next.js 环境变量规则
- `NEXT_PUBLIC_*` 前缀的变量会暴露给浏览器
- 无前缀的变量仅在服务器端可用
- 在 Server Components 中可以直接使用 `process.env`
:::

## 📊 数据类型定义

### 通用类型

```typescript
// src/types/api.ts
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}
```

### 配置相关类型

```typescript
// src/types/config.ts
export interface Config {
  name: string;
  path: string;
  isActive: boolean;
  description?: string;
  lastModified?: string;
}

export interface SwitchConfigRequest {
  configName: string;
}

export interface ValidateConfigsResponse {
  valid: boolean;
  errors?: string[];
  warnings?: string[];
}
```

### 命令相关类型

```typescript
// src/types/command.ts
export interface ExecuteCommandRequest {
  command: string;
  args: string[];
}

export interface CommandOutput {
  success: boolean;
  stdout: string;
  stderr: string;
  exitCode: number | null;
  executionTimeMs: number;
}

export interface CommandInfo {
  name: string;
  description: string;
  usage: string;
  examples: string[];
}
```

### 系统信息类型

```typescript
// src/types/system.ts
export interface SystemInfo {
  os: string;
  arch: string;
  cpuCount: number;
  username: string;
  ccrVersion?: string;
  uptime?: number;
}
```

## 🔧 配置管理 API

### 获取配置列表

```typescript
// src/api/configService.ts
import { apiClient } from './client';
import { Config, ApiResponse } from '../types';

export const getConfigs = async (): Promise<Config[]> => {
  const response = await apiClient.get<ApiResponse<Config[]>>('/configs');
  
  if (response.data.success && response.data.data) {
    return response.data.data;
  }
  
  throw new Error(response.data.error || 'Failed to fetch configs');
};
```

**请求示例：**
```typescript
import { getConfigs } from '../api/configService';

const fetchConfigList = async () => {
  try {
    const configs = await getConfigs();
    console.log('Available configs:', configs);
  } catch (error) {
    console.error('Error fetching configs:', error);
  }
};
```

**响应示例：**
```json
{
  "success": true,
  "data": [
    {
      "name": "default",
      "path": "/home/user/.claude/configs/default.toml",
      "isActive": true,
      "description": "Default configuration"
    },
    {
      "name": "work",
      "path": "/home/user/.claude/configs/work.toml",
      "isActive": false,
      "description": "Work environment configuration"
    }
  ]
}
```

### 切换配置

```typescript
export const switchConfig = async (configName: string): Promise<void> => {
  const response = await apiClient.post<ApiResponse<string>>('/configs/switch', {
    configName
  });
  
  if (!response.data.success) {
    throw new Error(response.data.error || 'Failed to switch config');
  }
};
```

**使用示例：**
```typescript
const handleConfigSwitch = async (configName: string) => {
  try {
    await switchConfig(configName);
    console.log(`Successfully switched to config: ${configName}`);
    // 刷新配置列表
    await fetchConfigList();
  } catch (error) {
    console.error('Error switching config:', error);
  }
};
```

### 验证配置

```typescript
export const validateConfigs = async (): Promise<ValidateConfigsResponse> => {
  const response = await apiClient.post<ApiResponse<ValidateConfigsResponse>>('/configs/validate');
  
  if (response.data.success && response.data.data) {
    return response.data.data;
  }
  
  throw new Error(response.data.error || 'Failed to validate configs');
};
```

**使用示例：**
```typescript
const handleConfigValidation = async () => {
  try {
    const result = await validateConfigs();
    
    if (result.valid) {
      console.log('All configurations are valid');
    } else {
      console.warn('Configuration validation failed:', result.errors);
    }
  } catch (error) {
    console.error('Error validating configs:', error);
  }
};
```

## ⚡ 命令执行 API

### 执行命令

```typescript
// src/api/commandService.ts
export const executeCommand = async (
  command: string, 
  args: string[] = []
): Promise<CommandOutput> => {
  const response = await apiClient.post<ApiResponse<CommandOutput>>('/commands/execute', {
    command,
    args
  });
  
  if (response.data.success && response.data.data) {
    return response.data.data;
  }
  
  throw new Error(response.data.error || 'Failed to execute command');
};
```

**使用示例：**
```typescript
const handleCommandExecution = async () => {
  try {
    const result = await executeCommand('ccr', ['list']);
    
    console.log('Command output:', result.stdout);
    console.log('Execution time:', result.executionTimeMs, 'ms');
    
    if (!result.success) {
      console.error('Command failed:', result.stderr);
    }
  } catch (error) {
    console.error('Error executing command:', error);
  }
};
```

### 获取可用命令列表

```typescript
export const getAvailableCommands = async (): Promise<CommandInfo[]> => {
  const response = await apiClient.get<ApiResponse<CommandInfo[]>>('/commands/list');
  
  if (response.data.success && response.data.data) {
    return response.data.data;
  }
  
  throw new Error(response.data.error || 'Failed to fetch commands');
};
```

## 🖥️ 系统信息 API

### 获取系统信息

```typescript
// src/api/systemService.ts
export const getSystemInfo = async (): Promise<SystemInfo> => {
  const response = await apiClient.get<ApiResponse<SystemInfo>>('/system/info');
  
  if (response.data.success && response.data.data) {
    return response.data.data;
  }
  
  throw new Error(response.data.error || 'Failed to fetch system info');
};
```

**使用示例：**
```typescript
const fetchSystemInfo = async () => {
  try {
    const systemInfo = await getSystemInfo();
    console.log('System Info:', systemInfo);
  } catch (error) {
    console.error('Error fetching system info:', error);
  }
};
```

## 🎣 自定义 Hooks

### useConfigs Hook

```typescript
// src/hooks/useConfigs.ts
import { useState, useEffect } from 'react';
import { getConfigs, switchConfig, validateConfigs } from '../api/configService';
import { Config, ValidateConfigsResponse } from '../types';

export const useConfigs = () => {
  const [configs, setConfigs] = useState<Config[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchConfigs = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const configList = await getConfigs();
      setConfigs(configList);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleSwitchConfig = async (configName: string) => {
    setLoading(true);
    setError(null);
    
    try {
      await switchConfig(configName);
      await fetchConfigs(); // 刷新列表
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleValidateConfigs = async (): Promise<ValidateConfigsResponse | null> => {
    setLoading(true);
    setError(null);
    
    try {
      const result = await validateConfigs();
      return result;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      return null;
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchConfigs();
  }, []);

  return {
    configs,
    loading,
    error,
    refetch: fetchConfigs,
    switchConfig: handleSwitchConfig,
    validateConfigs: handleValidateConfigs,
  };
};
```

### useCommand Hook

```typescript
// src/hooks/useCommand.ts
import { useState } from 'react';
import { executeCommand } from '../api/commandService';
import { CommandOutput } from '../types';

export const useCommand = () => {
  const [output, setOutput] = useState<CommandOutput | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = async (command: string, args: string[] = []) => {
    setLoading(true);
    setError(null);
    setOutput(null);
    
    try {
      const result = await executeCommand(command, args);
      setOutput(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const clear = () => {
    setOutput(null);
    setError(null);
  };

  return {
    output,
    loading,
    error,
    execute,
    clear,
  };
};
```

## 🔄 实际使用示例

### 配置管理组件

```typescript
// src/components/ConfigManagement.tsx
import React from 'react';
import { useConfigs } from '../hooks/useConfigs';

const ConfigManagement: React.FC = () => {
  const { configs, loading, error, switchConfig, validateConfigs } = useConfigs();

  const handleSwitchClick = async (configName: string) => {
    await switchConfig(configName);
  };

  const handleValidateClick = async () => {
    const result = await validateConfigs();
    if (result) {
      alert(result.valid ? 'All configs are valid' : `Validation failed: ${result.errors?.join(', ')}`);
    }
  };

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <div>
      <h2>Configuration Management</h2>
      <button onClick={handleValidateClick}>Validate Configs</button>
      
      <div>
        {configs.map((config) => (
          <div key={config.name} className={config.isActive ? 'active' : ''}>
            <h3>{config.name}</h3>
            <p>{config.path}</p>
            {!config.isActive && (
              <button onClick={() => handleSwitchClick(config.name)}>
                Switch to this config
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

export default ConfigManagement;
```

### 命令执行组件

```typescript
// src/components/CommandExecutor.tsx
import React, { useState } from 'react';
import { useCommand } from '../hooks/useCommand';

const CommandExecutor: React.FC = () => {
  const [command, setCommand] = useState('');
  const [args, setArgs] = useState('');
  const { output, loading, error, execute, clear } = useCommand();

  const handleExecute = async () => {
    const argArray = args.split(' ').filter(arg => arg.trim() !== '');
    await execute(command, argArray);
  };

  return (
    <div>
      <h2>Command Executor</h2>
      
      <div>
        <input
          type="text"
          placeholder="Command"
          value={command}
          onChange={(e) => setCommand(e.target.value)}
        />
        <input
          type="text"
          placeholder="Arguments (space separated)"
          value={args}
          onChange={(e) => setArgs(e.target.value)}
        />
        <button onClick={handleExecute} disabled={loading || !command}>
          {loading ? 'Executing...' : 'Execute'}
        </button>
        <button onClick={clear}>Clear</button>
      </div>

      {error && (
        <div style={{ color: 'red' }}>
          Error: {error}
        </div>
      )}

      {output && (
        <div>
          <h3>Output:</h3>
          <pre style={{ backgroundColor: '#f5f5f5', padding: '10px' }}>
            {output.stdout}
          </pre>
          
          {output.stderr && (
            <div>
              <h4>Errors:</h4>
              <pre style={{ backgroundColor: '#ffe6e6', padding: '10px' }}>
                {output.stderr}
              </pre>
            </div>
          )}
          
          <div>
            <small>
              Execution time: {output.executionTimeMs}ms | 
              Exit code: {output.exitCode} | 
              Success: {output.success ? 'Yes' : 'No'}
            </small>
          </div>
        </div>
      )}
    </div>
  );
};

export default CommandExecutor;
```

## 🚨 错误处理

### 统一错误处理

```typescript
// src/utils/errorHandler.ts
export class ApiError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
    public response?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export const handleApiError = (error: any): ApiError => {
  if (error.response) {
    // 服务器响应错误
    const message = error.response.data?.error || error.response.statusText || 'Server error';
    return new ApiError(message, error.response.status, error.response.data);
  } else if (error.request) {
    // 网络错误
    return new ApiError('Network error: Unable to connect to server');
  } else {
    // 其他错误
    return new ApiError(error.message || 'Unknown error');
  }
};
```

### 在 API 服务中使用

```typescript
// src/api/configService.ts (更新版本)
import { handleApiError } from '../utils/errorHandler';

export const getConfigs = async (): Promise<Config[]> => {
  try {
    const response = await apiClient.get<ApiResponse<Config[]>>('/configs');
    
    if (response.data.success && response.data.data) {
      return response.data.data;
    }
    
    throw new Error(response.data.error || 'Failed to fetch configs');
  } catch (error) {
    throw handleApiError(error);
  }
};
```

## 📈 性能优化

### 请求缓存

```typescript
// src/utils/cache.ts
interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number;
}

class ApiCache {
  private cache = new Map<string, CacheEntry<any>>();

  set<T>(key: string, data: T, ttl: number = 5 * 60 * 1000): void {
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl,
    });
  }

  get<T>(key: string): T | null {
    const entry = this.cache.get(key);
    
    if (!entry) return null;
    
    if (Date.now() - entry.timestamp > entry.ttl) {
      this.cache.delete(key);
      return null;
    }
    
    return entry.data;
  }

  clear(): void {
    this.cache.clear();
  }
}

export const apiCache = new ApiCache();
```

### 带缓存的 API 调用

```typescript
// src/api/configService.ts (缓存版本)
import { apiCache } from '../utils/cache';

export const getConfigs = async (useCache: boolean = true): Promise<Config[]> => {
  const cacheKey = 'configs';
  
  if (useCache) {
    const cached = apiCache.get<Config[]>(cacheKey);
    if (cached) return cached;
  }
  
  try {
    const response = await apiClient.get<ApiResponse<Config[]>>('/configs');
    
    if (response.data.success && response.data.data) {
      const configs = response.data.data;
      apiCache.set(cacheKey, configs, 2 * 60 * 1000); // 缓存 2 分钟
      return configs;
    }
    
    throw new Error(response.data.error || 'Failed to fetch configs');
  } catch (error) {
    throw handleApiError(error);
  }
};
```

## 🧪 API 测试

### Mock 数据

```typescript
// src/__mocks__/api.ts
export const mockConfigs: Config[] = [
  {
    name: 'default',
    path: '/home/user/.claude/configs/default.toml',
    isActive: true,
    description: 'Default configuration',
  },
  {
    name: 'work',
    path: '/home/user/.claude/configs/work.toml',
    isActive: false,
    description: 'Work environment',
  },
];

export const mockSystemInfo: SystemInfo = {
  os: 'linux',
  arch: 'x86_64',
  cpuCount: 8,
  username: 'testuser',
  ccrVersion: '1.0.0',
};
```

### API 测试

```typescript
// src/__tests__/api/configService.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { getConfigs, switchConfig } from '../api/configService';
import { apiClient } from '../api/client';
import { mockConfigs } from '../__mocks__/api';

vi.mock('../api/client');

describe('configService', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('getConfigs', () => {
    it('should return configs on success', async () => {
      vi.mocked(apiClient.get).mockResolvedValue({
        data: {
          success: true,
          data: mockConfigs,
        },
      });

      const result = await getConfigs();
      expect(result).toEqual(mockConfigs);
      expect(apiClient.get).toHaveBeenCalledWith('/configs');
    });

    it('should throw error on failure', async () => {
      vi.mocked(apiClient.get).mockResolvedValue({
        data: {
          success: false,
          error: 'Server error',
        },
      });

      await expect(getConfigs()).rejects.toThrow('Server error');
    });
  });
});
```

这个 API 接口文档提供了完整的前端与后端交互指南，包括类型定义、错误处理、性能优化和测试策略，帮助开发者高效地使用和维护 API 接口。