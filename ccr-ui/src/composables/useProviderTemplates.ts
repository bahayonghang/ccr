import { useMemo } from 'react'
import { create } from 'zustand'
import { BUILT_IN_PROVIDER_TEMPLATES } from '@/configs/providerTemplates'
import type { ProviderTemplate } from '@/types/providerTemplates'
import {
  deleteCustomProviderTemplate,
  mergeProviderTemplates,
  readCustomProviderTemplates,
  upsertCustomProviderTemplate,
  writeCustomProviderTemplates,
} from '@/utils/providerTemplates'

// Provider 模板跨页共享 store（08-22-state-logic-port 批次 5c；原 Vue composable
// 的模块级单例 customTemplates 迁移，React hook 薄封装保持导出名不变）。
//
// 持久化偏差记录（沿用批次 4 shellPreferences 先例）：不用 zustand/persist 中间件，
// 继续经 utils/providerTemplates 的 read/write 工具按原存储键逐次写入，
// 键布局字节不变。初值在模块加载时读取（等价原模块级 ref 初始化）。

interface ProviderTemplatesState {
  customTemplates: ProviderTemplate[]
  saveCustomTemplate: (template: ProviderTemplate) => void
  removeCustomTemplate: (id: string) => void
  reloadCustomTemplates: () => void
}

const persist = (templates: ProviderTemplate[]) => {
  writeCustomProviderTemplates(templates)
}

export const useProviderTemplatesStore = create<ProviderTemplatesState>()((set, get) => ({
  customTemplates: readCustomProviderTemplates(),

  saveCustomTemplate: (template) => {
    const next = upsertCustomProviderTemplate(get().customTemplates, template)
    set({ customTemplates: next })
    persist(next)
  },

  removeCustomTemplate: (id) => {
    const next = deleteCustomProviderTemplate(get().customTemplates, id)
    set({ customTemplates: next })
    persist(next)
  },

  reloadCustomTemplates: () => {
    set({ customTemplates: readCustomProviderTemplates() })
  },
}))

/**
 * Built-in + custom provider templates with persistence.
 * Signature change（消费方为待迁移 .vue 组件）：返回字段由 Ref/computed 改为普通值。
 */
export function useProviderTemplates() {
  const customTemplates = useProviderTemplatesStore((s) => s.customTemplates)
  const storeSaveCustomTemplate = useProviderTemplatesStore((s) => s.saveCustomTemplate)
  const storeRemoveCustomTemplate = useProviderTemplatesStore((s) => s.removeCustomTemplate)
  const reloadCustomTemplates = useProviderTemplatesStore((s) => s.reloadCustomTemplates)

  // 原 computed(:19)：来源 BUILT_IN_PROVIDER_TEMPLATES（静态常量）、customTemplates
  const templates = useMemo(
    () => mergeProviderTemplates(BUILT_IN_PROVIDER_TEMPLATES, customTemplates),
    [customTemplates],
  )

  return {
    builtInTemplates: BUILT_IN_PROVIDER_TEMPLATES,
    customTemplates,
    templates,
    saveCustomTemplate: storeSaveCustomTemplate,
    removeCustomTemplate: storeRemoveCustomTemplate,
    reloadCustomTemplates,
  }
}
