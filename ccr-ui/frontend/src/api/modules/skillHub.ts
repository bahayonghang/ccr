import { api } from '../core'

export interface SkillHubMarketplaceItem {
  package: string
  owner: string
  repo: string
  skill?: string
  skills_sh_url: string
}

export interface SkillHubMarketplaceResponse {
  items: SkillHubMarketplaceItem[]
  total: number
  cached: boolean
}

export interface SkillHubAgentSummary {
  id: string
  display_name: string
  global_skills_dir: string
  detected: boolean
  installed_count: number
}

export interface SkillHubInstalledSkill {
  name: string
  description?: string
  skill_dir: string
}

export interface SkillHubOperationResult {
  agent: string
  ok: boolean
  message?: string
}

export interface SkillHubOperationResponse {
  results: SkillHubOperationResult[]
}

export async function getSkillHubTrending(params?: { limit?: number; page?: number }) {
  const response = await api.get('/skill_hub/marketplace/trending', { params })
  return (response.data?.data ?? response.data) as SkillHubMarketplaceResponse
}

export async function searchSkillHubMarketplace(params: { q: string; limit?: number; page?: number }) {
  const response = await api.get('/skill_hub/marketplace/search', { params })
  return (response.data?.data ?? response.data) as SkillHubMarketplaceResponse
}

export async function getSkillHubAgents() {
  const response = await api.get('/skill_hub/agents')
  return (response.data?.data ?? response.data) as SkillHubAgentSummary[]
}

export async function getSkillHubAgentSkills(agent: string) {
  const response = await api.get(`/skill_hub/agents/${encodeURIComponent(agent)}/skills`)
  return (response.data?.data ?? response.data) as SkillHubInstalledSkill[]
}

export async function installSkillHubSkill(payload: { package: string; agents?: string[]; force?: boolean }) {
  const response = await api.post('/skill_hub/install', payload)
  return (response.data?.data ?? response.data) as SkillHubOperationResponse
}

export async function removeSkillHubSkill(payload: { skill: string; agents?: string[] }) {
  const response = await api.post('/skill_hub/remove', payload)
  return (response.data?.data ?? response.data) as SkillHubOperationResponse
}
