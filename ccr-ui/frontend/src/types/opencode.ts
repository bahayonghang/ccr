/**
 * OpenCode 平台 TypeScript 类型定义
 *
 * 对应后端 opencode_manager.rs 中的数据结构
 */

// ============ Provider 类型 ============

export interface OpenCodeProviderOptions {
    baseURL?: string
    apiKey?: string
    headers?: Record<string, string>
    [key: string]: unknown
}

export interface OpenCodeModelLimit {
    context?: number
    output?: number
}

export interface OpenCodeModel {
    name: string
    limit?: OpenCodeModelLimit
    [key: string]: unknown
}

export interface OpenCodeProvider {
    /** Map key，Provider 唯一标识符 */
    id: string
    /** npm 包名，如 "@ai-sdk/anthropic" */
    npm: string
    name?: string
    options: OpenCodeProviderOptions
    models: Record<string, OpenCodeModel>
}

// ============ MCP 服务器类型（原生 OpenCode 格式）============

export interface OpenCodeMcpServer {
    /** Map key，服务器唯一标识符 */
    id: string
    /** 服务器类型："local" | "remote" */
    type: 'local' | 'remote'
    /** local 类型：命令数组 [cmd, ...args] */
    command?: string[]
    /** local 类型：环境变量 */
    environment?: Record<string, string>
    /** remote 类型：URL */
    url?: string
    /** remote 类型：请求头 */
    headers?: Record<string, string>
}

// ============ Plugin 类型 ============

export interface OpenCodePlugin {
    /** npm 包名，如 "@opencode-ai/omo" */
    npm: string
}

// ============ 完整配置类型 ============

export interface OpenCodeConfig {
    $schema?: string
    provider: Record<string, Omit<OpenCodeProvider, 'id'>>
    mcp: Record<string, Omit<OpenCodeMcpServer, 'id'>>
    plugin: string[]
    [key: string]: unknown
}

// ============ 请求类型 ============

export interface OpenCodeProviderRequest {
    id: string
    npm: string
    name?: string
    options?: OpenCodeProviderOptions
    models?: Record<string, { name: string; limit?: OpenCodeModelLimit }>
}

export interface OpenCodeMcpServerRequest {
    id: string
    type: 'local' | 'remote'
    command?: string[]
    environment?: Record<string, string>
    url?: string
    headers?: Record<string, string>
}

export interface OpenCodePluginRequest {
    npm: string
}

// ============ 预设 Provider 定义 ============

export interface OpenCodeProviderPreset {
    id: string
    label: string
    npm: string
    description: string
}

export const OPENCODE_PROVIDER_PRESETS: OpenCodeProviderPreset[] = [
    {
        id: 'anthropic',
        label: 'Anthropic (Claude)',
        npm: '@ai-sdk/anthropic',
        description: 'Claude 3.5 Sonnet, Opus, Haiku 等模型'
    },
    {
        id: 'openai',
        label: 'OpenAI',
        npm: '@ai-sdk/openai',
        description: 'GPT-4o, o1, o3 等模型'
    },
    {
        id: 'google',
        label: 'Google (Gemini)',
        npm: '@ai-sdk/google',
        description: 'Gemini 2.0, 1.5 Flash/Pro 等模型'
    },
    {
        id: 'openai-compatible',
        label: 'OpenAI Compatible',
        npm: '@ai-sdk/openai-compatible',
        description: '兼容 OpenAI API 的自定义端点'
    }
]
