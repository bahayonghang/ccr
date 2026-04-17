/**
 * useSkillVersions — 版本历史 / diff / 回滚 reactive composable。
 *
 * 绑定某一 skill 的 installPath 作为版本存储键，所有操作走 Tauri IPC。
 * 响应式：列表 / 选中版本 / diff 结果 / loading / error。
 */

import { computed, ref, watch } from 'vue'
import {
  skillsVersionDiff,
  skillsVersionGet,
  skillsVersionList,
  skillsVersionRollback,
  skillsVersionSnapshot,
} from '@/api'
import type {
  DiffResult,
  SnapshotSource,
  Version,
  VersionMeta,
} from '@/types/skillVersioning'

export function useSkillVersions(installPath: () => string | null | undefined) {
  const history = ref<VersionMeta[]>([])
  const selectedVersion = ref<Version | null>(null)
  const diff = ref<DiffResult | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const hasHistory = computed(() => history.value.length > 0)

  async function refresh() {
    const path = installPath()
    if (!path) {
      history.value = []
      return
    }
    loading.value = true
    error.value = null
    try {
      history.value = await skillsVersionList<VersionMeta[]>(path)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      history.value = []
    } finally {
      loading.value = false
    }
  }

  async function loadVersion(versionId: string): Promise<Version | null> {
    const path = installPath()
    if (!path) return null
    loading.value = true
    error.value = null
    try {
      const result = await skillsVersionGet<Version | null>(path, versionId)
      selectedVersion.value = result
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  async function loadDiff(oldId: string, newId: string): Promise<DiffResult | null> {
    const path = installPath()
    if (!path) return null
    loading.value = true
    error.value = null
    try {
      const result = await skillsVersionDiff<DiffResult | null>(path, oldId, newId)
      diff.value = result
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      diff.value = null
      return null
    } finally {
      loading.value = false
    }
  }

  async function takeSnapshot(
    skillName: string,
    message: string,
    source: SnapshotSource = 'manual',
  ): Promise<VersionMeta | null> {
    const path = installPath()
    if (!path) return null
    loading.value = true
    error.value = null
    try {
      const meta = await skillsVersionSnapshot<VersionMeta>(path, skillName, message, source)
      await refresh()
      return meta
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  async function rollback(versionId: string): Promise<VersionMeta | null> {
    const path = installPath()
    if (!path) return null
    loading.value = true
    error.value = null
    try {
      const meta = await skillsVersionRollback<VersionMeta>(path, versionId)
      await refresh()
      return meta
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    } finally {
      loading.value = false
    }
  }

  // install path 变化时自动 refresh；immediate: true 保证初次挂载也触发
  watch(
    installPath,
    () => {
      selectedVersion.value = null
      diff.value = null
      void refresh()
    },
    { immediate: true },
  )

  return {
    history,
    selectedVersion,
    diff,
    loading,
    error,
    hasHistory,
    refresh,
    loadVersion,
    loadDiff,
    takeSnapshot,
    rollback,
  }
}
