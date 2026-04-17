/**
 * useSkillToggle — skill 启用/禁用开关 reactive composable。
 *
 * 基于 `~/.claude/settings.json` 的 `permissions.deny[]`。
 * 列表懒加载；toggle 操作乐观更新本地 Set 后再刷新。
 */

import { computed, ref } from 'vue'
import { skillsToggleListDisabled, skillsToggleSet } from '@/api'

export function useSkillToggle() {
  const disabled = ref<Set<string>>(new Set())
  const loading = ref(false)
  const error = ref<string | null>(null)

  const disabledList = computed(() => Array.from(disabled.value).sort())

  function isDisabled(skillName: string): boolean {
    return disabled.value.has(skillName)
  }

  function isEnabled(skillName: string): boolean {
    return !disabled.value.has(skillName)
  }

  async function refresh() {
    loading.value = true
    error.value = null
    try {
      const list = await skillsToggleListDisabled()
      disabled.value = new Set(list)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function setEnabled(skillName: string, enabled: boolean): Promise<boolean> {
    loading.value = true
    error.value = null
    // 乐观更新
    const prevDisabled = new Set(disabled.value)
    if (enabled) disabled.value.delete(skillName)
    else disabled.value.add(skillName)
    try {
      await skillsToggleSet(skillName, enabled)
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      // 回滚乐观更新
      disabled.value = prevDisabled
      return false
    } finally {
      loading.value = false
    }
  }

  return {
    disabled,
    disabledList,
    loading,
    error,
    isDisabled,
    isEnabled,
    refresh,
    setEnabled,
  }
}
