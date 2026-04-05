import { storeToRefs } from 'pinia'
import { ref } from 'vue'
import { useSkillsStore } from '@/stores/skills'
import {
  skillsPickFolder,
  skillsSourceAddLocal,
} from '@/api'
import type { InstallProgress, NpxStatus, Platform, SkillContent, SkillOperationResponse, SkillsInstallRequest, SkillsRouteState, SkillsSyncRequest } from '@/types/skills'

type LegacyInstallRequest = {
  package: string
  agents: string[]
  force?: boolean
}

type LegacyRemoveRequest = {
  skill: string
  agents: string[]
}

type GithubImportRequest = {
  url: string
  agents: string[]
  force?: boolean
}

type LocalImportRequest = {
  sourcePath: string
  agents: string[]
  skillName?: string
}

type NpxImportRequest = {
  package: string
  agents: string[]
  global?: boolean
}

type BatchInstallRequest = {
  packages: string[]
  agents: string[]
  force?: boolean
}

export function useUnifiedSkills() {
  const store = useSkillsStore()
  const refs = storeToRefs(store)
  const { skills, platforms } = refs
  const isInstalling = ref(false)
  const installProgress = ref<InstallProgress | null>(null)

  async function initialize(preloadMarketplace = false) {
    await Promise.all([
      store.loadInventory(),
      store.loadSources(),
      preloadMarketplace ? store.loadMarketplace() : Promise.resolve(),
    ])
  }

  async function refresh(includeMarketplace = false) {
    await Promise.all([
      store.loadInventory(true),
      store.loadSources(true),
      includeMarketplace ? store.loadMarketplace(true) : Promise.resolve(),
    ])
  }

  async function fetchAllSkills(force = false) {
    return store.loadInventory(force)
  }

  async function fetchMarketplaceTrending(_pageSize = 20, page = 1, force = false) {
    store.setRouteState({ page })
    if (store.routeState.q && !force) {
      store.setRouteState({ q: '' })
    }
    return store.loadMarketplace(force)
  }

  async function searchMarketplace(query: string, page = 1) {
    store.setRouteState({ q: query, page })
    return store.loadMarketplace(true)
  }

  async function fetchPlatforms() {
    if (platforms.value.length === 0) {
      await store.loadInventory()
    }
    return platforms.value
  }

  async function fetchSkillContent(skillIdOrPath: string, installationId?: string | null): Promise<SkillContent> {
    const byId = skills.value.find((skill) => skill.id === skillIdOrPath)
    const targetSkill = byId ?? skills.value.find((skill) =>
      skill.installations.some((installation) => installation.installPath === skillIdOrPath))
    if (!targetSkill) {
      throw new Error(`Skill not found: ${skillIdOrPath}`)
    }
    const targetInstallation = installationId
      ? targetSkill.installations.find((installation) => installation.id === installationId)
      : targetSkill.installations.find((installation) => installation.installPath === skillIdOrPath)
        ?? targetSkill.installations.find((installation) => installation.isPrimary)
        ?? targetSkill.installations[0]
    const content = await store.ensureContent(targetSkill.id, targetInstallation?.id, true)
    if (!content) {
      throw new Error(`Skill content not found: ${skillIdOrPath}`)
    }
    return content
  }

  async function saveSkillContent(skillIdOrPath: string, installationIdOrRaw: string, maybeRaw?: string) {
    const byId = skills.value.find((skill) => skill.id === skillIdOrPath)
    const targetSkill = byId ?? skills.value.find((skill) =>
      skill.installations.some((installation) => installation.installPath === skillIdOrPath))
    if (!targetSkill) {
      throw new Error(`Skill not found: ${skillIdOrPath}`)
    }

    if (maybeRaw == null) {
      const installation = targetSkill.installations.find((item) => item.installPath === skillIdOrPath)
        ?? targetSkill.installations.find((item) => item.isPrimary)
        ?? targetSkill.installations[0]
      if (!installation) {
        throw new Error(`Installation not found: ${skillIdOrPath}`)
      }
      return store.saveContent(targetSkill.id, installation.id, installationIdOrRaw)
    }

    return store.saveContent(targetSkill.id, installationIdOrRaw, maybeRaw)
  }

  async function installSkill(request: LegacyInstallRequest): Promise<SkillOperationResponse> {
    return store.install({
      sourceKind: 'marketplace',
      sourceRef: request.package,
      targetPlatforms: request.agents as Platform[],
      force: request.force,
    })
  }

  async function removeSkill(request: LegacyRemoveRequest): Promise<SkillOperationResponse> {
    const targetSkill = skills.value.find((skill) =>
      skill.installations.some((installation) => installation.installPath === request.skill))
    if (!targetSkill) {
      throw new Error(`Skill not found: ${request.skill}`)
    }
    return store.removeSkillRecord(targetSkill.id)
  }

  async function importFromGithub(request: GithubImportRequest) {
    return store.install({
      sourceKind: 'github',
      sourceRef: request.url,
      targetPlatforms: request.agents as Platform[],
      force: request.force,
    })
  }

  async function importFromLocal(request: LocalImportRequest) {
    return store.install({
      sourceKind: 'local',
      sourceRef: request.sourcePath,
      sourceSkillId: request.skillName,
      targetPlatforms: request.agents as Platform[],
    })
  }

  async function importViaNpx(request: NpxImportRequest) {
    return store.install({
      sourceKind: 'npx',
      sourceRef: request.package,
      targetPlatforms: request.agents as Platform[],
      force: request.global,
    })
  }

  async function batchInstall(request: BatchInstallRequest) {
    const results = []
    for (const pkg of request.packages) {
      const response = await store.install({
        sourceKind: 'marketplace',
        sourceRef: pkg,
        targetPlatforms: request.agents as Platform[],
        force: request.force,
      })
      results.push({
        package: pkg,
        ok: response.results.every((item) => item.ok),
        message: response.results.find((item) => !item.ok)?.message,
      })
    }
    return {
      total: request.packages.length,
      successCount: results.filter((item) => item.ok).length,
      failCount: results.filter((item) => !item.ok).length,
      results,
    }
  }

  async function checkNpxStatus(): Promise<NpxStatus> {
    return store.loadNpxStatus(true)
  }

  async function browseFolder(): Promise<string | null> {
    const result = await skillsPickFolder<{ path?: string | null }>()
    return typeof result.path === 'string' ? result.path : null
  }

  async function addLocalSource(path: string) {
    const source = await skillsSourceAddLocal(path)
    await store.loadSources(true)
    return source
  }

  function applyRouteState(state: Partial<SkillsRouteState>) {
    store.setRouteState(state)
  }

  function setInstalling(value: boolean) {
    isInstalling.value = value
  }

  function setInstallProgress(value: InstallProgress | null) {
    installProgress.value = value
  }

  return {
    ...refs,
    isInstalling,
    installProgress,
    initialize,
    refresh,
    fetchAllSkills,
    fetchMarketplaceTrending,
    searchMarketplace,
    fetchPlatforms,
    fetchSkillContent,
    saveSkillContent,
    installSkill,
    removeSkill,
    importFromGithub,
    importFromLocal,
    importViaNpx,
    batchInstall,
    checkNpxStatus,
    browseFolder,
    loadMarketplace: store.loadMarketplace,
    loadNpxStatus: store.loadNpxStatus,
    addLocalSource,
    applyRouteState,
    setInstalling,
    setInstallProgress,
    install: (request: SkillsInstallRequest) => store.install(request),
    syncSkill: (request: SkillsSyncRequest) => store.syncSkill(request),
    removeInstallation: store.removeInstallation,
    removeSkillRecord: store.removeSkillRecord,
    addGitSource: store.addGitSource,
    addLocalSourceRecord: store.addLocalSource,
    syncSource: store.syncSource,
    removeSource: store.removeSource,
    ensureDetail: store.ensureDetail,
    ensureContent: store.ensureContent,
    selectSkill: store.selectSkill,
    saveContent: store.saveContent,
  }
}
