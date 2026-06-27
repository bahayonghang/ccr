// 预设供应商模板类型定义

/** 预设供应商分类 */
export type PresetCategory =
  | 'official'       // 官方直连
  | 'cn_official'    // 国内官方模型服务
  | 'aggregator'     // 聚合/中转服务
  | 'third_party'    // 第三方服务商

/** 单个供应商预设模板 */
export interface ProviderPreset {
  id: string                          // 唯一标识 (如 'deepseek')
  name: string                        // 显示名称 (如 'DeepSeek')
  category: PresetCategory
  websiteUrl?: string                 // 供应商官网
  apiKeyUrl?: string                  // API Key 获取地址
  isPartner?: boolean                 // 是否合作伙伴 (显示星标)
  // ---- 自动填充到表单的字段 ----
  base_url: string
  model?: string
  small_fast_model?: string
  claude_code_auto_compact_window?: string
  api_timeout_ms?: string
  claude_code_disable_nonessential_traffic?: string
  provider?: string                   // 供应商名 (如 'DeepSeek')
  provider_type?: 'official_relay' | 'third_party_model'
  description?: string
}

/** 分类元数据 */
export interface PresetCategoryMeta {
  id: PresetCategory
  label: string                       // 中文显示名
  labelEn: string                     // 英文显示名
}

/** 平台预设注册条目 */
export interface PlatformPresets {
  platform: string
  categories: PresetCategoryMeta[]
  presets: ProviderPreset[]
}
