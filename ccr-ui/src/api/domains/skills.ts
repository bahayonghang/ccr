/**
 * Skills Domain —— Skills Hub 仓库管理 / 安装 / 版本历史 / 回收站 API
 *
 * 真迁移自 tauri.ts 的 Skills 分组（Legacy + Skills Domain + skills_ext + Legacy aliases）。
 * 对应后端 commands::skills::* 命令。
 *
 * 结构：
 *   1) Legacy（listSkills / addSkill）—— 未迁移至新 Skills Domain 前的最小接口
 *   2) Skills Domain 主接口 —— inventory / detail / content / files / install / sources /
 *      marketplace / npx / pickFolder
 *   3) skills_ext Phase 5 —— 版本历史 / 回收站 / 启用禁用 / 分类分析
 *   4) Legacy aliases —— 向后兼容薄壳，保持旧组件在重构期间能编译
 */

import { invoke } from '@tauri-apps/api/core'
import { asArray, asRecord, type UnknownRecord } from '../_shared'

// ── Legacy Skills ──

/** 列出技能（Legacy 最小接口） */
export const listSkills = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_skills')
}

/** 添加技能（Legacy 最小接口） */
export const addSkill = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  const payload = asRecord(data)
  const name = String(payload.name ?? '')
  const instruction = String(payload.instruction ?? payload.content ?? '')
  return invoke('add_skill', { name, instruction })
}

// ── Skills Domain 主接口 ──

export const skillsInventory = async <T = UnknownRecord>(query?: unknown): Promise<T> => {
  return invoke('skills_inventory', { query: query ?? null })
}

export const skillsDetail = async <T = UnknownRecord>(skillId: string): Promise<T> => {
  return invoke('skills_detail', { skillId })
}

export const skillsContentGet = async <T = UnknownRecord>(
  skillId: string,
  installationId?: string | null,
): Promise<T> => {
  return invoke('skills_content_get', { skillId, installationId: installationId ?? null })
}

export const skillsFilesList = async <T = UnknownRecord>(
  skillId: string,
  installationId?: string | null,
): Promise<T> => {
  return invoke('skills_files_list', { skillId, installationId: installationId ?? null })
}

export const skillsFileGet = async <T = UnknownRecord>(
  skillId: string,
  path: string,
  installationId?: string | null,
): Promise<T> => {
  return invoke('skills_file_get', { skillId, path, installationId: installationId ?? null })
}

export const skillsOnboardingCandidates = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('skills_onboarding_candidates')
}

export const skillsContentSave = async <T = UnknownRecord>(
  skillId: string,
  installationId: string,
  raw: string,
): Promise<T> => {
  return invoke('skills_content_save', { skillId, installationId, raw })
}

export const skillsInstall = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('skills_install', { request })
}

export const skillsPrepareInstall = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('skills_prepare_install', { request })
}

export const skillsSync = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('skills_sync', { request })
}

export const skillsRemoveInstallation = async <T = UnknownRecord>(
  skillId: string,
  installationId: string,
): Promise<T> => {
  return invoke('skills_remove_installation', { skillId, installationId })
}

export const skillsRemoveSkill = async <T = UnknownRecord>(skillId: string): Promise<T> => {
  return invoke('skills_remove_skill', { skillId })
}

export const skillsSourcesList = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('skills_sources_list')
}

export const skillsSourceAddGit = async <T = UnknownRecord>(url: string): Promise<T> => {
  return invoke('skills_source_add_git', { url })
}

export const skillsSourceAddLocal = async <T = UnknownRecord>(path: string): Promise<T> => {
  return invoke('skills_source_add_local', { path })
}

export const skillsSourceSync = async <T = UnknownRecord>(sourceId: string): Promise<T> => {
  return invoke('skills_source_sync', { sourceId })
}

export const skillsSourceRemove = async <T = UnknownRecord>(sourceId: string): Promise<T> => {
  return invoke('skills_source_remove', { sourceId })
}

export const skillsMarketplaceList = async <T = UnknownRecord>(
  query?: string | null,
  page = 1,
  pageSize = 20,
): Promise<T> => {
  return invoke('skills_marketplace_list', { query: query ?? null, page, pageSize })
}

export const skillsMarketplaceDetail = async <T = UnknownRecord>(packageId: string): Promise<T> => {
  return invoke('skills_marketplace_detail', { packageId })
}

export const skillsNpxStatus = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('skills_npx_status')
}

export const skillsNpxCapabilities = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('skills_npx_capabilities')
}

export const skillsPickFolder = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('skills_pick_folder')
}

// ── skills_ext Phase 5：版本历史 / 回收站 / 启用禁用 ──

export const skillsVersionList = async <T = UnknownRecord>(installPath: string): Promise<T> => {
  return invoke('skills_version_list', { installPath })
}

export const skillsVersionGet = async <T = UnknownRecord>(
  installPath: string,
  versionId: string,
): Promise<T> => {
  return invoke('skills_version_get', { installPath, versionId })
}

export const skillsVersionSnapshot = async <T = UnknownRecord>(
  installPath: string,
  skillName: string,
  message: string,
  source: 'auto' | 'manual' = 'manual',
): Promise<T> => {
  return invoke('skills_version_snapshot', { installPath, skillName, message, source })
}

export const skillsVersionDiff = async <T = UnknownRecord>(
  installPath: string,
  oldId: string,
  newId: string,
): Promise<T> => {
  return invoke('skills_version_diff', { installPath, oldId, newId })
}

export const skillsVersionRollback = async <T = UnknownRecord>(
  installPath: string,
  versionId: string,
): Promise<T> => {
  return invoke('skills_version_rollback', { installPath, versionId })
}

export const skillsTrashList = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('skills_trash_list')
}

export const skillsTrashSoftDelete = async <T = UnknownRecord>(
  installPath: string,
  skillName: string,
): Promise<T> => {
  return invoke('skills_trash_soft_delete', { installPath, skillName })
}

export const skillsTrashRestore = async <T = UnknownRecord>(trashId: string): Promise<T> => {
  return invoke('skills_trash_restore', { trashId })
}

export const skillsTrashPurge = async (trashId: string): Promise<boolean> => {
  return invoke('skills_trash_purge', { trashId })
}

export const skillsToggleSet = async (skillName: string, enabled: boolean): Promise<boolean> => {
  return invoke('skills_toggle_set', { skillName, enabled })
}

export const skillsToggleListDisabled = async (): Promise<string[]> => {
  return invoke('skills_toggle_list_disabled')
}

export const skillsTaxonomyAnalyze = async <T = UnknownRecord>(
  items: unknown[],
): Promise<T> => {
  return invoke('skills_taxonomy_analyze', { items })
}

// ── Legacy aliases（向后兼容薄壳，重构期内保留，待旧组件迁移完毕后移除） ──

export const deleteSkill = async <T = UnknownRecord>(skillId: string): Promise<T> => {
  return skillsRemoveSkill(skillId)
}

export const getSkillDetail = async <T = UnknownRecord>(skillId: string): Promise<T> => {
  return skillsDetail(skillId)
}

export const updateSkillContent = async <T = UnknownRecord>(
  skillId: string,
  raw: string,
): Promise<T> => {
  const detail = asRecord(await skillsDetail(skillId))
  const installations = Array.isArray(detail.installations) ? detail.installations : []
  const installationId = String(asRecord(installations[0]).id ?? '')
  return skillsContentSave(skillId, installationId, raw)
}

export const listSkillRepositories = async <T = UnknownRecord>(): Promise<T> => {
  return skillsSourcesList()
}

export const addSkillRepository = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  const payload = asRecord(data)
  if (typeof payload.url === 'string' && payload.url.trim()) {
    return skillsSourceAddGit(payload.url)
  }
  if (typeof payload.path === 'string' && payload.path.trim()) {
    return skillsSourceAddLocal(payload.path)
  }
  throw new Error('Repository url/path is required')
}

export const removeSkillRepository = async <T = UnknownRecord>(sourceId: string): Promise<T> => {
  return skillsSourceRemove(sourceId)
}

export const scanSkillRepository = async <T = UnknownRecord>(sourceId: string): Promise<T> => {
  return skillsSourceSync(sourceId)
}

export const getSkillHubTrending = async <T = UnknownRecord>(
  page = 1,
  pageSize = 20,
): Promise<T> => {
  return skillsMarketplaceList(null, page, pageSize)
}

export const searchSkillHubMarketplace = async <T = UnknownRecord>(
  query: string,
  page = 1,
  pageSize = 20,
): Promise<T> => {
  return skillsMarketplaceList(query, page, pageSize)
}

export const getSkillHubAgents = async <T = UnknownRecord>(): Promise<T> => {
  const inventory = asRecord(await skillsInventory())
  return (inventory.platforms ?? []) as T
}

export const getSkillHubAgentSkills = async <T = UnknownRecord>(agentName: string): Promise<T> => {
  return skillsInventory({ platform: agentName })
}

export const installSkillHubSkill = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  const payload = asRecord(data)
  const agents = asArray(payload.agents).filter(
    (value): value is string => typeof value === 'string',
  )
  return skillsInstall({
    source_kind: 'marketplace',
    source_ref: String(payload.url ?? payload.package ?? ''),
    source_skill_id: typeof payload.skill === 'string' ? payload.skill : null,
    target_platforms: agents,
    force: Boolean(payload.force),
  })
}

export const removeSkillHubSkill = async <T = UnknownRecord>(skillId: string): Promise<T> => {
  return skillsRemoveSkill(skillId)
}

export const getSkillHubUnified = async <T = UnknownRecord>(platform?: string): Promise<T> => {
  return skillsInventory(platform ? { platform } : null)
}

export const getSkillHubSkillContent = async <T = UnknownRecord>(
  skillId: string,
  installationId?: string | null,
): Promise<T> => {
  return skillsContentGet(skillId, installationId ?? null)
}

export const saveSkillHubSkillContent = async <T = UnknownRecord>(
  skillId: string,
  installationIdOrContent: string,
  maybeContent?: string,
): Promise<T> => {
  if (maybeContent == null) {
    return updateSkillContent(skillId, installationIdOrContent)
  }

  return skillsContentSave(skillId, installationIdOrContent, maybeContent)
}

export const importSkillFromGithub = async <T = UnknownRecord>(
  url: string,
  agents: string[],
  force = false,
): Promise<T> => {
  return skillsInstall({
    source_kind: 'github',
    source_ref: url,
    target_platforms: agents,
    force,
  })
}

export const importSkillFromLocal = async <T = UnknownRecord>(
  sourcePath: string,
  agents: string[],
  skillName?: string,
): Promise<T> => {
  return skillsInstall({
    source_kind: 'local',
    source_ref: sourcePath,
    source_skill_id: skillName ?? null,
    target_platforms: agents,
    force: false,
  })
}

export const importSkillViaNpx = async <T = UnknownRecord>(
  packageName: string,
  agents: string[],
  global = false,
): Promise<T> => {
  return skillsInstall({
    source_kind: 'npx',
    source_ref: packageName,
    target_platforms: agents,
    force: global,
  })
}

export const batchInstallSkills = async <T = UnknownRecord>(
  packages: string[],
  agents: string[],
  force = false,
): Promise<T> => {
  const results = await Promise.all(
    packages.map((pkg) =>
      skillsInstall({
        source_kind: 'marketplace',
        source_ref: pkg,
        target_platforms: agents,
        force,
      }),
    ),
  )
  return {
    total: packages.length,
    success_count: results.filter((item) =>
      asArray(asRecord(item).results).every((row) => Boolean(asRecord(row).ok)),
    ).length,
    fail_count: results.filter(
      (item) => !asArray(asRecord(item).results).every((row) => Boolean(asRecord(row).ok)),
    ).length,
    results: results.flatMap((item) => asArray(asRecord(item).results)),
  } as T
}

export const checkNpxAvailability = async <T = UnknownRecord>(): Promise<T> => {
  return skillsNpxStatus()
}

export const browseForFolder = async <T = UnknownRecord>(): Promise<T> => {
  return skillsPickFolder()
}
