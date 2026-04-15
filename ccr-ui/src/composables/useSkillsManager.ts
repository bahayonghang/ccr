/**
 * useSkillsManager - Skills Manager 页面 Composable
 *
 * 在 useUnifiedSkills 基础上增加:
 * - master-detail 面板状态管理
 * - Fuse.js 模糊搜索
 * - 按来源分组 (可折叠)
 * - 多选模式 + 批量操作
 */

import Fuse from 'fuse.js'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import type { SkillRecord, Platform } from '@/types/skills'
import type { SkillGroup, SkillPanelMode } from '@/types/skillsManager'
import { isTauriRuntime } from '@/utils/tauriRuntime'

export function useSkillsManager() {
  const skills = useUnifiedSkills()
  let stopSkillsEvent: (() => void) | null = null

  // ============ 面板状态 ============

  const panelMode = ref<SkillPanelMode>({ type: 'empty' })
  const searchQuery = ref('')
  const selectedKeys = ref<Set<string>>(new Set())
  const isMultiSelectMode = ref(false)
  const selectedPlatforms = ref<Platform[]>([])

  // ============ 分组逻辑 ============

  /** 按名称聚合: SkillRecord[] → SkillGroup[] */
  const groupedSkills = computed<SkillGroup[]>(() => {
    const allSkills = skills.skills?.value ?? []
    const map = new Map<string, SkillRecord[]>()
    for (const skill of allSkills) {
      const existing = map.get(skill.name) ?? []
      map.set(skill.name, [...existing, skill])
    }
    return Array.from(map.entries()).map(([name, items]) => {
      const first = items[0]
      const platforms = [...new Set(items.flatMap(s => s.installations.map(i => i.platformId)))]
      return {
        name,
        description: first.description ?? '',
        items,
        platforms,
        origin: first.origin ?? 'unknown',
      }
    })
  })

  // ============ Fuse.js 搜索 ============

  const fuse = computed(() =>
    new Fuse(groupedSkills.value, {
      keys: [
        { name: 'name', weight: 2 },
        { name: 'description', weight: 1 },
        { name: 'origin', weight: 0.5 },
      ],
      threshold: 0.4,
      includeScore: true,
    }),
  )

  const filteredGroups = computed<SkillGroup[]>(() => {
    if (!searchQuery.value.trim()) return groupedSkills.value
    return fuse.value.search(searchQuery.value).map(r => r.item)
  })

  // ============ 统计 ============

  const stats = computed(() => ({
    logicalSkills: groupedSkills.value.length,
    installations: (skills.skills?.value ?? []).reduce((sum, s) => sum + s.installations.length, 0),
    sources: skills.sources?.value?.length ?? 0,
  }))

  // ============ 选中状态 ============

  const activeSkill = computed<SkillRecord | null>(() => {
    if (panelMode.value.type === 'detail' && 'skillId' in panelMode.value) {
      const targetId = (panelMode.value as { skillId: string }).skillId
      return (skills.skills?.value ?? []).find(s => s.id === targetId) ?? null
    }
    // 默认选中第一个
    const allSkills = skills.skills?.value ?? []
    if (panelMode.value.type === 'empty' && allSkills.length > 0) {
      return allSkills[0]
    }
    return null
  })

  const activeGroup = computed<SkillGroup | null>(() => {
    if (!activeSkill.value) return null
    return groupedSkills.value.find(g => g.name === activeSkill.value?.name) ?? null
  })

  const effectiveSelectedKeys = computed<Set<string>>(() => {
    if (selectedKeys.value.size > 0) return selectedKeys.value
    if (activeSkill.value && !isMultiSelectMode.value) {
      return new Set([activeSkill.value.id])
    }
    return new Set()
  })

  const selectedGroups = computed(() => {
    const allSkills = skills.skills?.value ?? []
    return allSkills.filter(s => selectedKeys.value.has(s.id))
  })

  // ============ 操作 ============

  function selectSkill(skillId: string) {
    if (isMultiSelectMode.value) {
      const next = new Set(selectedKeys.value)
      if (next.has(skillId)) next.delete(skillId)
      else next.add(skillId)
      selectedKeys.value = next
    } else {
      selectedKeys.value = new Set([skillId])
      panelMode.value = { type: 'detail', skillId }
      skills.selectSkill(skillId, null)
      void skills.ensureDetail(skillId, true)
    }
  }

  function openCreate() {
    selectedKeys.value = new Set()
    panelMode.value = { type: 'create' }
  }

  function openImport() {
    selectedKeys.value = new Set()
    panelMode.value = { type: 'import' }
  }

  function openImportGithub() {
    selectedKeys.value = new Set()
    panelMode.value = { type: 'import-github' }
  }

  function closePanel() {
    panelMode.value = activeSkill.value
      ? { type: 'detail', skillId: activeSkill.value.id }
      : { type: 'empty' }
  }

  function toggleMultiSelect() {
    isMultiSelectMode.value = !isMultiSelectMode.value
    if (!isMultiSelectMode.value) {
      selectedKeys.value = new Set()
    }
  }

  function selectDetectedPlatforms() {
    const platformList = skills.platforms?.value ?? []
    selectedPlatforms.value = platformList
      .filter((p: { detected: boolean }) => p.detected)
      .map((p: { id: Platform }) => p.id)
  }

  async function removeSkill(skillId: string) {
    await skills.removeSkillRecord(skillId)
  }

  async function bulkDelete() {
    for (const skill of selectedGroups.value) {
      await skills.removeSkillRecord(skill.id)
    }
    selectedKeys.value = new Set()
    isMultiSelectMode.value = false
  }

  async function refresh(includeMarketplace = false) {
    await skills.refresh(includeMarketplace)
  }

  // 首次加载后自动选中
  watch(groupedSkills, (groups) => {
    if (panelMode.value.type === 'empty' && groups.length > 0 && !isMultiSelectMode.value) {
      const first = groups[0].items[0]
      if (first) {
        panelMode.value = { type: 'detail', skillId: first.id }
      }
    }
  })

  onMounted(async () => {
    await skills.initialize(false)
    await skills.loadNpxStatus(true)
    selectDetectedPlatforms()

    if (isTauriRuntime()) {
      const { listen } = await import('@tauri-apps/api/event')
      const unlisten = await listen('skills-changed', async () => {
        await refresh()
      })
      stopSkillsEvent = unlisten
    }
  })

  onUnmounted(() => {
    stopSkillsEvent?.()
    stopSkillsEvent = null
  })

  return {
    // 从 useUnifiedSkills 透传
    allSkills: skills.skills,
    platforms: skills.platforms,
    sources: skills.sources,
    marketplace: skills.marketplace,
    inventoryLoading: skills.inventoryLoading,
    sourcesLoading: skills.sourcesLoading,
    marketplaceLoading: skills.marketplaceLoading,
    mutationLoading: skills.mutationLoading,
    npxStatus: skills.npxStatus,
    install: skills.install,
    syncSkill: skills.syncSkill,
    removeInstallation: skills.removeInstallation,
    addGitSource: skills.addGitSource,
    addLocalSourceRecord: skills.addLocalSourceRecord,
    syncSource: skills.syncSource,
    removeSource: skills.removeSource,
    ensureDetail: skills.ensureDetail,
    ensureContent: skills.ensureContent,
    saveContent: skills.saveContent,
    loadMarketplace: skills.loadMarketplace,
    browseFolder: skills.browseFolder,
    importFromGithub: skills.importFromGithub,
    importFromLocal: skills.importFromLocal,
    importViaNpx: skills.importViaNpx,

    // Manager 新增
    panelMode,
    searchQuery,
    selectedKeys,
    isMultiSelectMode,
    selectedPlatforms,
    groupedSkills,
    filteredGroups,
    stats,
    activeSkill,
    activeGroup,
    effectiveSelectedKeys,
    selectedGroups,

    selectSkill,
    openCreate,
    openImport,
    openImportGithub,
    closePanel,
    toggleMultiSelect,
    selectDetectedPlatforms,
    removeSkill,
    bulkDelete,
    refresh,
  }
}
