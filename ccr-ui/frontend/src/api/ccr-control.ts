// CCR Control API - 收藏命令和命令历史管理

import { api } from './client'
import type { ApiResponse } from '@/types'

// ===== 类型定义 =====

/** 收藏的命令 */
export interface FavoriteCommand {
  id: string
  command: string
  args: string[]
  display_name?: string
  module: string
  created_at: string
}

/** 命令执行历史 */
export interface CommandHistory {
  id: string
  full_command: string
  command: string
  args: string[]
  success: boolean
  executed_at: string
  duration_ms: number
}

/** 收藏列表响应 */
export interface FavoritesResponse {
  favorites: FavoriteCommand[]
}

/** 历史列表响应 */
export interface HistoryResponse {
  history: CommandHistory[]
  total: number
}

/** 添加收藏请求 */
export interface AddFavoriteRequest {
  command: string
  args: string[]
  display_name?: string
  module: string
}

/** 添加历史请求 */
export interface AddHistoryRequest {
  command: string
  args: string[]
  success: boolean
  duration_ms: number
}

// ===== 收藏命令 API =====

/** 获取所有收藏命令 */
export const getFavorites = async (): Promise<FavoriteCommand[]> => {
  const response = await api.get<ApiResponse<FavoritesResponse>>('/ui-state/favorites')
  if (response.data.success && response.data.data) {
    return response.data.data.favorites
  }
  throw new Error(response.data.message || '获取收藏列表失败')
}

/** 添加收藏命令 */
export const addFavorite = async (req: AddFavoriteRequest): Promise<FavoriteCommand> => {
  const response = await api.post<ApiResponse<FavoriteCommand>>('/ui-state/favorites', req)
  if (response.data.success && response.data.data) {
    return response.data.data
  }
  throw new Error(response.data.message || '添加收藏失败')
}

/** 删除收藏命令 */
export const removeFavorite = async (id: string): Promise<boolean> => {
  const response = await api.delete<ApiResponse<boolean>>(`/ui-state/favorites/${id}`)
  if (response.data.success) {
    return true
  }
  throw new Error(response.data.message || '删除收藏失败')
}

// ===== 命令历史 API =====

/** 获取命令历史 */
export const getHistory = async (limit?: number): Promise<CommandHistory[]> => {
  const params = limit ? `?limit=${limit}` : ''
  const response = await api.get<ApiResponse<HistoryResponse>>(`/ui-state/history${params}`)
  if (response.data.success && response.data.data) {
    return response.data.data.history
  }
  throw new Error(response.data.message || '获取历史记录失败')
}

/** 添加命令历史 */
export const addHistory = async (req: AddHistoryRequest): Promise<CommandHistory> => {
  const response = await api.post<ApiResponse<CommandHistory>>('/ui-state/history', req)
  if (response.data.success && response.data.data) {
    return response.data.data
  }
  throw new Error(response.data.message || '添加历史记录失败')
}

/** 清空命令历史 */
export const clearHistory = async (): Promise<boolean> => {
  const response = await api.delete<ApiResponse<boolean>>('/ui-state/history')
  if (response.data.success) {
    return true
  }
  throw new Error(response.data.message || '清空历史记录失败')
}

// ===== CCR 功能模块定义 =====

/** 功能模块 */
export interface CcrModule {
  id: string
  name: string
  icon: string
  description: string
  commands: CcrCommand[]
}

/** 命令定义 */
export interface CcrCommand {
  name: string
  command: string
  description: string
  args?: CcrCommandArg[]
  flags?: CcrCommandFlag[]
  dangerous?: boolean
}

/** 命令参数 */
export interface CcrCommandArg {
  name: string
  description: string
  required: boolean
  type: 'string' | 'number' | 'file' | 'select'
  placeholder?: string
  options?: string[]  // 用于 select 类型
}

/** 命令选项 */
export interface CcrCommandFlag {
  name: string
  flag: string
  description: string
  type: 'boolean' | 'string' | 'number'
  default?: unknown
}

/** CCR 功能模块列表 - 基于 README.md 命令结构 */
export const CCR_MODULES: CcrModule[] = [
  // 1. 配置管理
  {
    id: 'config',
    name: '配置管理',
    icon: 'Settings',
    description: '创建、查看、切换和管理配置',
    commands: [
      {
        name: '初始化',
        command: 'init',
        description: '创建配置文件模板 (~/.ccr/config.toml)',
        dangerous: true
      },
      {
        name: '添加配置',
        command: 'add',
        description: '交互式创建新配置'
      },
      {
        name: '列出配置',
        command: 'list',
        description: '列出当前平台的所有配置'
      },
      {
        name: '当前配置',
        command: 'current',
        description: '显示当前激活的配置'
      },
      {
        name: '切换配置',
        command: 'switch',
        description: '切换到指定配置',
        args: [{ name: 'name', description: '配置名称', required: true, type: 'string', placeholder: 'anthropic' }]
      },
      {
        name: '启用配置',
        command: 'enable',
        description: '启用指定配置',
        args: [{ name: 'name', description: '配置名称', required: true, type: 'string', placeholder: 'my-config' }]
      },
      {
        name: '禁用配置',
        command: 'disable',
        description: '禁用指定配置',
        args: [{ name: 'name', description: '配置名称', required: true, type: 'string', placeholder: 'my-config' }],
        flags: [{ name: '强制', flag: '--force', description: '强制禁用当前配置', type: 'boolean' }]
      },
      {
        name: '验证配置',
        command: 'validate',
        description: '检查配置文件格式是否正确'
      },
      {
        name: '查看历史',
        command: 'history',
        description: '显示配置操作的审计日志',
        flags: [
          { name: '条数', flag: '-l', description: '显示最近 N 条记录', type: 'number', default: 50 },
          { name: '类型', flag: '-t', description: '按操作类型筛选 (switch/backup)', type: 'string' }
        ]
      },
      {
        name: '优化配置',
        command: 'optimize',
        description: '按字母顺序重新排列配置文件'
      }
    ]
  },
  // 2. 平台管理
  {
    id: 'platform',
    name: '平台管理',
    icon: 'Layers',
    description: '管理和切换 AI CLI 平台 (Claude/Codex/Gemini/Qwen/iFlow)',
    commands: [
      {
        name: '列出平台',
        command: 'platform list',
        description: '显示所有支持的平台及其状态'
      },
      {
        name: '切换平台',
        command: 'platform switch',
        description: '切换到指定平台',
        args: [{
          name: 'platform',
          description: '平台名称',
          required: true,
          type: 'select',
          options: ['claude', 'codex', 'gemini', 'qwen', 'iflow']
        }]
      },
      {
        name: '当前平台',
        command: 'platform current',
        description: '显示当前激活的平台'
      },
      {
        name: '平台详情',
        command: 'platform info',
        description: '查看指定平台的配置和状态',
        args: [{
          name: 'platform',
          description: '平台名称',
          required: true,
          type: 'select',
          options: ['claude', 'codex', 'gemini', 'qwen', 'iflow']
        }]
      },
      {
        name: '初始化平台',
        command: 'platform init',
        description: '为指定平台创建配置目录结构',
        args: [{
          name: 'platform',
          description: '平台名称',
          required: true,
          type: 'select',
          options: ['claude', 'codex', 'gemini', 'qwen', 'iflow']
        }]
      }
    ]
  },
  // 3. 导入导出
  {
    id: 'io',
    name: '导入导出',
    icon: 'FileUp',
    description: '配置的导入、导出和备份清理',
    commands: [
      {
        name: '导出配置',
        command: 'export',
        description: '将当前配置导出为文件',
        flags: [
          { name: '输出路径', flag: '-o', description: '指定导出文件路径', type: 'string' },
          { name: '排除密钥', flag: '--no-secrets', description: '导出时排除敏感信息', type: 'boolean' }
        ]
      },
      {
        name: '导入配置',
        command: 'import',
        description: '从文件导入配置',
        args: [{ name: 'file', description: '配置文件路径', required: true, type: 'file', placeholder: 'configs.toml' }],
        flags: [
          { name: '合并模式', flag: '--merge', description: '保留现有配置，仅添加新配置', type: 'boolean' },
          { name: '自动备份', flag: '--backup', description: '导入前自动备份', type: 'boolean' }
        ],
        dangerous: true
      },
      {
        name: '清理备份',
        command: 'clean',
        description: '清理过期的备份文件',
        flags: [
          { name: '天数', flag: '--days', description: '清理 N 天前的备份', type: 'number', default: 30 },
          { name: '模拟运行', flag: '--dry-run', description: '仅显示将要删除的文件', type: 'boolean' }
        ],
        dangerous: true
      }
    ]
  },
  // 4. 迁移
  {
    id: 'migrate',
    name: '配置迁移',
    icon: 'ArrowRightLeft',
    description: '从 Legacy 模式迁移到 Unified 模式',
    commands: [
      {
        name: '检查迁移',
        command: 'migrate --check',
        description: '检查是否需要从 Legacy 迁移到 Unified 模式'
      },
      {
        name: '执行迁移',
        command: 'migrate',
        description: '执行配置迁移',
        dangerous: true
      },
      {
        name: '迁移指定平台',
        command: 'migrate --platform',
        description: '仅迁移指定平台',
        args: [{
          name: 'platform',
          description: '平台名称',
          required: true,
          type: 'select',
          options: ['claude', 'codex', 'gemini', 'qwen', 'iflow']
        }],
        dangerous: true
      }
    ]
  },
  // 5. 临时凭证
  {
    id: 'temp-token',
    name: '临时凭证',
    icon: 'Key',
    description: '设置临时覆盖凭证（不修改配置文件）',
    commands: [
      {
        name: '设置临时凭证',
        command: 'temp-token set',
        description: '设置临时覆盖凭证',
        args: [{ name: 'token', description: 'API Token', required: true, type: 'string', placeholder: 'sk-ant-api03-xxxx' }],
        flags: [
          { name: 'Base URL', flag: '--base-url', description: '临时 API 地址', type: 'string' },
          { name: 'Model', flag: '--model', description: '临时模型名称', type: 'string' }
        ]
      },
      {
        name: '显示临时凭证',
        command: 'temp-token show',
        description: '查看当前设置的临时凭证'
      },
      {
        name: '清除临时凭证',
        command: 'temp-token clear',
        description: '清除所有临时凭证'
      }
    ]
  },
  // 6. 技能管理
  {
    id: 'skills',
    name: '技能管理',
    icon: 'Puzzle',
    description: '管理 AI 助手的技能',
    commands: [
      {
        name: '列出技能',
        command: 'skills list',
        description: '列出已安装的技能'
      },
      {
        name: '扫描目录',
        command: 'skills scan',
        description: '扫描技能目录',
        args: [{ name: 'path', description: '技能目录路径', required: true, type: 'string', placeholder: '~/skills' }]
      },
      {
        name: '安装技能',
        command: 'skills install',
        description: '安装技能',
        args: [{ name: 'path', description: '技能路径', required: true, type: 'string', placeholder: '~/skills/my-skill' }]
      }
    ]
  },
  // 7. 提示词管理
  {
    id: 'prompts',
    name: '提示词管理',
    icon: 'FileText',
    description: '管理提示词预设',
    commands: [
      {
        name: '列出预设',
        command: 'prompts list',
        description: '列出所有提示词预设'
      },
      {
        name: '添加预设',
        command: 'prompts add',
        description: '添加新的提示词预设'
      },
      {
        name: '应用预设',
        command: 'prompts apply',
        description: '将预设应用到目标文件',
        args: [{ name: 'name', description: '预设名称', required: true, type: 'string', placeholder: 'my-preset' }]
      }
    ]
  },
  // 8. 统计 (web feature)
  {
    id: 'stats',
    name: '使用统计',
    icon: 'BarChart',
    description: '查看使用成本和统计信息',
    commands: [
      {
        name: '今日成本',
        command: 'stats cost --today',
        description: '查看今日的使用成本'
      },
      {
        name: '按模型统计',
        command: 'stats cost --by-model',
        description: '按模型分组显示成本'
      },
      {
        name: '本月成本',
        command: 'stats cost --this-month',
        description: '查看本月的使用成本'
      }
    ]
  },
  // 9. 系统命令
  {
    id: 'system',
    name: '系统命令',
    icon: 'Terminal',
    description: '版本信息、更新和冲突检测',
    commands: [
      {
        name: '版本信息',
        command: 'version',
        description: '显示 CCR 版本信息'
      },
      {
        name: '检查更新',
        command: 'update --check',
        description: '检查是否有新版本可用'
      },
      {
        name: '执行更新',
        command: 'update',
        description: '从 GitHub 更新到最新版本',
        flags: [{ name: '分支', flag: '--branch', description: '指定更新分支 (main/dev)', type: 'string', default: 'main' }],
        dangerous: true
      },
      {
        name: '冲突检测',
        command: 'check conflicts',
        description: '检测不同平台之间的环境变量冲突'
      }
    ]
  }
]
