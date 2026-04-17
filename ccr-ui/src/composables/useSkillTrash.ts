/**
 * useSkillTrash — 回收站列表 / 恢复 / 永久删除 reactive composable。
 */

import { computed, ref } from 'vue'
import {
  skillsTrashList,
  skillsTrashPurge,
  skillsTrashRestore,
  skillsTrashSoftDelete,
} from '@/api'
import type { TrashEntry } from '@/types/skillVersioning'

export function useSkillTrash() {
  const entries = ref<TrashEntry[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const count = computed(() => entries.value.length)

  async function refresh() {
    loading.value = true
    error.value = null
    try {
      entries.value = await skillsTrashList<TrashEntry[]>()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      entries.value = []
    } finally {
      loading.value = false
    }
  }

  async function softDelete(installPath: string, skillName: string): Promise<TrashEntry | null> {
    loading.value = true
    error.value = null
    try {
      const entry = await skillsTrashSoftDelete<TrashEntry>(installPath, skillName)
      await refresh()
      return entry
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  async function restore(trashId: string): Promise<string | null> {
    loading.value = true
    error.value = null
    try {
      const path = await skillsTrashRestore<string>(trashId)
      await refresh()
      return path
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  async function purge(trashId: string): Promise<boolean> {
    loading.value = true
    error.value = null
    try {
      const ok = await skillsTrashPurge(trashId)
      if (ok) await refresh()
      return ok
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    } finally {
      loading.value = false
    }
  }

  return {
    entries,
    count,
    loading,
    error,
    refresh,
    softDelete,
    restore,
    purge,
  }
}
