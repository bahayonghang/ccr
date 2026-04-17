/**
 * useSkillTaxonomy — 分类 / 合并建议 / 冲突 / 健康诊断 reactive composable。
 *
 * 输入一组 skill（id+name+description+frontmatterCategory+realPath），
 * 调用 Tauri 分析，返回分类结果 + category 聚合 + 合并建议 + 冲突 + 健康汇总。
 */

import { ref, shallowRef } from 'vue'
import { skillsTaxonomyAnalyze } from '@/api'
import type {
  CategorySummary,
  Classification,
  ConflictGroup,
  HealthReport,
  MergeSuggestion,
  TaxonomyInput,
  TaxonomyResponse,
} from '@/types/skillVersioning'

export function useSkillTaxonomy() {
  const classifications = ref<Classification[]>([])
  const categories = ref<CategorySummary[]>([])
  const mergeSuggestions = ref<MergeSuggestion[]>([])
  const conflicts = ref<ConflictGroup[]>([])
  const health = shallowRef<HealthReport | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function analyze(items: TaxonomyInput[]) {
    loading.value = true
    error.value = null
    try {
      const response = await skillsTaxonomyAnalyze<TaxonomyResponse>(items)
      classifications.value = response.classifications
      categories.value = response.categories
      mergeSuggestions.value = response.mergeSuggestions
      conflicts.value = response.conflicts
      health.value = response.health
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  function classificationFor(skillId: string): Classification | undefined {
    return classifications.value.find(c => c.skillId === skillId)
  }

  function isInConflict(skillId: string): boolean {
    return conflicts.value.some(g => g.skillIds.includes(skillId))
  }

  return {
    classifications,
    categories,
    mergeSuggestions,
    conflicts,
    health,
    loading,
    error,
    analyze,
    classificationFor,
    isInConflict,
  }
}
