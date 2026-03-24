<template>
  <div class="skill-hub-view">
    <div class="skill-hub-view__shell">
      <div class="skill-hub-view__header">
        <div class="skill-hub-view__header-main">
          <div class="skill-hub-view__title-row">
            <h2 class="skill-hub-view__title">
              Skill Hub
            </h2>
            <span class="skill-hub-view__count-badge">
              {{ marketplace.total }}
            </span>
          </div>
          <p class="skill-hub-view__subtitle">
            浏览 skills.sh 并一键安装到本机多个 Agent 的全局目录
          </p>
        </div>
        <div class="skill-hub-view__header-actions">
          <RouterLink
            to="/skills"
            class="skill-hub-view__local-link"
          >
            <SIcon
              name="Book"
              size="w-4 h-4"
            /><span>本地技能</span>
          </RouterLink>
        </div>
      </div>

      <div class="skill-hub-view__layout">
        <div class="skill-hub-view__sidebar">
          <div class="skill-hub-view__panel glass-effect">
            <div class="skill-hub-view__panel-header">
              <div class="skill-hub-view__panel-title">
                <SIcon
                  name="Boxes"
                  size="w-4 h-4"
                  class="skill-hub-view__panel-title-icon"
                />
                Agents
              </div>
            </div>

            <div
              v-if="agentsLoading"
              class="skill-hub-view__panel-state"
            >
              加载中...
            </div>

            <div
              v-else
              class="skill-hub-view__agent-list"
            >
              <button
                v-for="agent in agents"
                :key="agent.id"
                :class="[
                  'skill-hub-view__agent-button',
                  selectedAgent === agent.id
                    ? 'skill-hub-view__agent-button--active'
                    : 'skill-hub-view__agent-button--inactive',
                ]"
                @click="selectAgent(agent.id)"
              >
                <div class="skill-hub-view__agent-row">
                  <div class="skill-hub-view__agent-meta">
                    <div class="skill-hub-view__agent-name">
                      {{ agent.display_name }}
                    </div>
                    <div class="skill-hub-view__agent-path">
                      {{ agent.global_skills_dir || '未配置路径' }}
                    </div>
                  </div>
                  <div class="skill-hub-view__agent-badges">
                    <span
                      :class="[
                        'skill-hub-view__agent-status',
                        agent.detected
                          ? 'skill-hub-view__agent-status--detected'
                          : 'skill-hub-view__agent-status--undetected',
                      ]"
                    >
                      {{ agent.detected ? '已检测' : '未检测' }}
                    </span>
                    <span class="skill-hub-view__agent-count">
                      {{ agent.installed_count }}
                    </span>
                  </div>
                </div>
              </button>
            </div>
          </div>
        </div>

        <div class="skill-hub-view__content">
          <div class="skill-hub-view__toolbar glass-effect">
            <div class="skill-hub-view__toolbar-row">
              <div class="skill-hub-view__search-shell">
                <SIcon
                  name="Search"
                  size="w-5 h-5"
                  class="skill-hub-view__search-icon"
                />
                <input
                  v-model="searchQuery"
                  type="text"
                  placeholder="搜索 skills.sh（owner/repo 或 skill 名）"
                  class="skill-hub-view__search-input"
                  @keydown.enter="runSearch"
                >
                <button
                  v-if="searchQuery"
                  class="skill-hub-view__search-clear"
                  @click="clearSearch"
                >
                  <SIcon
                    name="X"
                    size="w-4 h-4"
                  />
                </button>
              </div>

              <div class="skill-hub-view__toolbar-actions">
                <button
                  class="skill-hub-view__toolbar-button skill-hub-view__toolbar-button--secondary"
                  @click="reloadAll"
                >
                  刷新
                </button>
                <button
                  class="skill-hub-view__toolbar-button skill-hub-view__toolbar-button--primary"
                  :disabled="marketplaceLoading || installLoading"
                  @click="runSearch"
                >
                  {{ searchQuery ? '搜索' : '热门' }}
                </button>
              </div>
            </div>

            <div class="skill-hub-view__toolbar-meta">
              <span class="skill-hub-view__toolbar-meta-item">
                <SIcon
                  name="Zap"
                  size="w-3.5 h-3.5"
                  class="skill-hub-view__toolbar-meta-icon skill-hub-view__toolbar-meta-icon--success"
                />
                默认全局安装
              </span>
              <span class="skill-hub-view__toolbar-meta-item">
                <SIcon
                  name="ShieldCheck"
                  size="w-3.5 h-3.5"
                  class="skill-hub-view__toolbar-meta-icon skill-hub-view__toolbar-meta-icon--accent"
                />
                解析 SKILL.md frontmatter
              </span>
              <span
                v-if="marketplace.cached"
                class="skill-hub-view__toolbar-meta-item"
              >
                <SIcon
                  name="Clock"
                  size="w-3.5 h-3.5"
                />
                缓存命中
              </span>
            </div>
          </div>

          <div class="skill-hub-view__section glass-effect">
            <div class="skill-hub-view__section-header">
              <div class="skill-hub-view__section-title">
                <SIcon
                  name="Store"
                  size="w-4 h-4"
                  class="skill-hub-view__section-icon"
                />
                Marketplace
              </div>
              <div class="skill-hub-view__section-meta">
                {{ selectedAgent }}
              </div>
            </div>

            <div
              v-if="marketplaceLoading"
              class="p-8 text-center text-[var(--color-text-muted)]"
            >
              <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-[var(--color-success)]/30 border-t-[var(--color-success)]" />
              加载中...
            </div>

            <div
              v-else-if="marketplace.items.length === 0"
              class="p-10 text-center text-[var(--color-text-muted)]"
            >
              <div class="bg-[var(--color-bg-elevated)] w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
                <SIcon
                  name="Store"
                  size="w-10 h-10"
                  class="opacity-50"
                />
              </div>
              <p class="text-lg font-medium">
                没有结果
              </p>
            </div>

            <div
              v-else
              class="divide-y divide-white/10"
            >
              <div
                v-for="item in marketplace.items"
                :key="item.skills_sh_url"
                class="p-4 flex flex-col sm:flex-row sm:items-center gap-3"
              >
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2 min-w-0">
                    <div class="text-sm font-semibold text-[var(--color-text-primary)] font-mono truncate">
                      {{ item.package }}
                    </div>
                    <a
                      :href="item.skills_sh_url"
                      target="_blank"
                      rel="noreferrer"
                      class="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text-secondary)] transition-colors flex-shrink-0"
                    >
                      查看
                    </a>
                  </div>
                  <div class="mt-1 text-xs text-[var(--color-text-muted)] truncate">
                    {{ item.owner }}/{{ item.repo }}<span v-if="item.skill"> · {{ item.skill }}</span>
                  </div>
                </div>

                <div class="flex items-center gap-2 justify-end">
                  <span
                    v-if="item.skill && installedSkillNames.has(item.skill)"
                    class="px-2 py-1 text-xs rounded-full border border-[var(--color-success)]/30 text-[var(--color-success)] bg-[var(--color-success)]/10"
                  >
                    已安装
                  </span>

                  <button
                    class="px-3 py-2 rounded-lg text-sm font-semibold transition-colors duration-200 border"
                    :class="item.skill ? 'border-[var(--color-success)]/30 text-white bg-[var(--color-success)] hover:bg-[var(--color-success-hover)]' : 'border-[var(--color-border-default)] text-[var(--color-text-muted)] bg-[var(--color-bg-elevated)] cursor-not-allowed'"
                    :disabled="installLoading || !item.skill"
                    @click="item.skill && installItem(item.package)"
                  >
                    安装
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div class="glass-effect rounded-xl border border-white/20 shadow-sm overflow-hidden">
            <div class="p-4 border-b border-white/10 flex items-center justify-between gap-3">
              <div class="flex items-center gap-2 text-sm font-semibold text-[var(--color-text-primary)]">
                <SIcon
                  name="Package"
                  size="w-4 h-4"
                  class="text-[var(--color-accent-primary)]"
                />
                Installed
              </div>
              <div class="text-xs text-[var(--color-text-muted)] font-mono">
                {{ installedSkills.length }}
              </div>
            </div>

            <div
              v-if="installedLoading"
              class="p-8 text-center text-[var(--color-text-muted)]"
            >
              <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-[var(--color-accent-primary)]/30 border-t-[var(--color-accent-primary)]" />
              加载中...
            </div>

            <div
              v-else-if="installedSkills.length === 0"
              class="p-10 text-center text-[var(--color-text-muted)]"
            >
              <div class="bg-[var(--color-bg-elevated)] w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
                <SIcon
                  name="Package"
                  size="w-10 h-10"
                  class="opacity-50"
                />
              </div>
              <p class="text-lg font-medium">
                当前 Agent 没有安装技能
              </p>
            </div>

            <div
              v-else
              class="divide-y divide-white/10"
            >
              <div
                v-for="s in installedSkills"
                :key="s.skill_dir"
                class="p-4 flex items-start gap-3"
              >
                <div class="min-w-0 flex-1">
                  <div class="text-sm font-semibold text-[var(--color-text-primary)] font-mono truncate">
                    {{ s.name }}
                  </div>
                  <div
                    v-if="s.description"
                    class="mt-1 text-xs text-[var(--color-text-secondary)] line-clamp-2"
                  >
                    {{ s.description }}
                  </div>
                  <div class="mt-1 text-xs text-[var(--color-text-muted)] truncate">
                    {{ s.skill_dir }}
                  </div>
                </div>

                <div class="flex items-center gap-2 flex-shrink-0">
                  <button
                    class="px-3 py-2 rounded-lg text-sm font-semibold transition-colors border border-[var(--color-danger)]/30 text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10"
                    :disabled="removeLoading"
                    @click="removeInstalledSkill(s.name)"
                  >
                    卸载
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, onMounted, ref } from 'vue'
import { useUIStore } from '@/stores/ui'
import {
  getSkillHubAgents,
  getSkillHubAgentSkills,
  getSkillHubTrending,
  installSkillHubSkill,
  removeSkillHubSkill,
  searchSkillHubMarketplace,
  type SkillHubAgentSummary,
  type SkillHubInstalledSkill,
  type SkillHubMarketplaceResponse
} from '@/api'
const ui = useUIStore()

const agents = ref<SkillHubAgentSummary[]>([])
const agentsLoading = ref(false)
const selectedAgent = ref('claude-code')

const installedSkills = ref<SkillHubInstalledSkill[]>([])
const installedLoading = ref(false)

const marketplace = ref<SkillHubMarketplaceResponse>({ items: [], total: 0, cached: false })
const marketplaceLoading = ref(false)

const searchQuery = ref('')
const installLoading = ref(false)
const removeLoading = ref(false)

interface SkillHubOperationEntry {
  ok?: boolean
  message?: string
}

interface SkillHubOperationResult {
  results?: SkillHubOperationEntry[]
}

const installedSkillNames = computed(() => {
  return new Set(installedSkills.value.map(s => s.name))
})

async function loadAgents() {
  agentsLoading.value = true
  try {
    const data = await getSkillHubAgents<SkillHubAgentSummary[]>()
    agents.value = data
    if (!agents.value.some(a => a.id === selectedAgent.value) && agents.value.length > 0) {
      selectedAgent.value = agents.value[0].id
    }
  } catch (e) {
    const message = e instanceof Error ? e.message : '加载 Agents 失败'
    ui.showError(message)
  } finally {
    agentsLoading.value = false
  }
}

async function loadInstalled() {
  installedLoading.value = true
  try {
    installedSkills.value = await getSkillHubAgentSkills<SkillHubInstalledSkill[]>(selectedAgent.value)
  } catch (e) {
    installedSkills.value = []
    const message = e instanceof Error ? e.message : '加载已安装技能失败'
    ui.showError(message)
  } finally {
    installedLoading.value = false
  }
}

async function loadTrending() {
  marketplaceLoading.value = true
  try {
    marketplace.value = await getSkillHubTrending<SkillHubMarketplaceResponse>()
  } catch (e) {
    marketplace.value = { items: [], total: 0, cached: false }
    const message = e instanceof Error ? e.message : '加载 Marketplace 失败'
    ui.showError(message)
  } finally {
    marketplaceLoading.value = false
  }
}

async function runSearch() {
  if (!searchQuery.value.trim()) {
    await loadTrending()
    return
  }
  marketplaceLoading.value = true
  try {
    marketplace.value = await searchSkillHubMarketplace<SkillHubMarketplaceResponse>(searchQuery.value.trim())
  } catch (e) {
    marketplace.value = { items: [], total: 0, cached: false }
    const message = e instanceof Error ? e.message : '搜索失败'
    ui.showError(message)
  } finally {
    marketplaceLoading.value = false
  }
}

function clearSearch() {
  searchQuery.value = ''
  loadTrending()
}

async function reloadAll() {
  await Promise.all([loadAgents(), loadInstalled(), runSearch()])
}

async function selectAgent(agentId: string) {
  if (selectedAgent.value === agentId) return
  selectedAgent.value = agentId
  await loadInstalled()
}

async function installItem(pkg: string) {
  installLoading.value = true
  try {
    const res = await installSkillHubSkill<SkillHubOperationResult>({ package: pkg, agents: [selectedAgent.value], force: false })
    const r = res.results?.[0]
    if (r && !r.ok) {
      throw new Error(r.message || '安装失败')
    }
    ui.showSuccess('安装完成')
    await Promise.all([loadAgents(), loadInstalled()])
  } catch (e) {
    const message = e instanceof Error ? e.message : '安装失败'
    ui.showError(message)
  } finally {
    installLoading.value = false
  }
}

async function removeInstalledSkill(skillName: string) {
  removeLoading.value = true
  try {
    const res = await removeSkillHubSkill<SkillHubOperationResult>(skillName)
    const r = res.results?.[0]
    if (r && !r.ok) {
      throw new Error(r.message || '卸载失败')
    }
    ui.showSuccess('已卸载')
    await Promise.all([loadAgents(), loadInstalled()])
  } catch (e) {
    const message = e instanceof Error ? e.message : '卸载失败'
    ui.showError(message)
  } finally {
    removeLoading.value = false
  }
}

onMounted(async () => {
  await loadAgents()
  await Promise.all([loadInstalled(), loadTrending()])
})
</script>

<style scoped>
.skill-hub-view,
.skill-hub-view__shell,
.skill-hub-view__header-main,
.skill-hub-view__content,
.skill-hub-view__agent-list {
  display: flex;
  flex-direction: column;
}

.skill-hub-view {
  min-height: 100%;
  padding: 1.25rem;
  transition: color 0.3s ease, background-color 0.3s ease;
}

.skill-hub-view__shell {
  max-width: 1600px;
  margin: 0 auto;
  gap: 1.5rem;
}

.skill-hub-view__header,
.skill-hub-view__title-row,
.skill-hub-view__header-actions,
.skill-hub-view__local-link,
.skill-hub-view__panel-title,
.skill-hub-view__agent-row,
.skill-hub-view__agent-badges {
  display: flex;
  align-items: center;
}

.skill-hub-view__header {
  justify-content: space-between;
  align-items: flex-start;
  gap: 1rem;
}

.skill-hub-view__header-main {
  min-width: 0;
}

.skill-hub-view__title-row {
  gap: 0.75rem;
}

.skill-hub-view__title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-mono, 'Maple Mono', monospace);
  font-size: 1.5rem;
  line-height: 2rem;
  font-weight: 700;
  color: var(--color-text-primary);
}

.skill-hub-view__count-badge {
  border: 1px solid rgb(var(--color-success-rgb) / 20%);
  border-radius: 9999px;
  background: rgb(var(--color-success-rgb) / 10%);
  padding: 0.25rem 0.75rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
  color: var(--color-success);
}

.skill-hub-view__subtitle {
  margin-top: 0.25rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--color-text-secondary);
}

.skill-hub-view__header-actions {
  gap: 0.75rem;
  flex-shrink: 0;
}

.skill-hub-view__local-link {
  gap: 0.5rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.5rem;
  background: var(--color-bg-elevated);
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  transition: background-color 0.2s ease;
}

.skill-hub-view__local-link:hover {
  background: var(--color-bg-surface);
}

.skill-hub-view__layout {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 1rem;
}

.skill-hub-view__panel {
  overflow: hidden;
  border: 1px solid rgb(255 255 255 / 20%);
  border-radius: 0.75rem;
  box-shadow: 0 1px 2px rgb(15 23 42 / 8%);
}

.skill-hub-view__panel-header {
  padding: 1rem;
  border-bottom: 1px solid rgb(255 255 255 / 10%);
}

.skill-hub-view__panel-title {
  gap: 0.5rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.skill-hub-view__panel-title-icon {
  color: var(--color-accent-primary);
}

.skill-hub-view__panel-state {
  padding: 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--color-text-muted);
}

.skill-hub-view__agent-list {
  gap: 0.5rem;
  padding: 0.5rem;
}

.skill-hub-view__agent-button {
  width: 100%;
  border: 1px solid;
  border-radius: 0.5rem;
  padding: 0.75rem;
  text-align: left;
  transition: background-color 0.2s ease, border-color 0.2s ease;
}

.skill-hub-view__agent-button--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 30%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
}

.skill-hub-view__agent-button--inactive {
  border-color: var(--color-border-default);
  background: var(--color-bg-elevated);
}

.skill-hub-view__agent-button--inactive:hover {
  background: var(--color-bg-surface);
}

.skill-hub-view__agent-row {
  justify-content: space-between;
  gap: 0.75rem;
}

.skill-hub-view__agent-meta {
  min-width: 0;
}

.skill-hub-view__agent-name,
.skill-hub-view__agent-count {
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.skill-hub-view__agent-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 600;
  color: var(--color-text-primary);
}

.skill-hub-view__agent-path,
.skill-hub-view__agent-status {
  font-size: 0.75rem;
  line-height: 1rem;
}

.skill-hub-view__agent-path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text-muted);
}

.skill-hub-view__agent-badges {
  gap: 0.5rem;
  flex-shrink: 0;
}

.skill-hub-view__agent-status {
  border: 1px solid;
  border-radius: 9999px;
  padding: 0.25rem 0.5rem;
}

.skill-hub-view__agent-status--detected {
  border-color: rgb(var(--color-success-rgb) / 30%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--color-success);
}

.skill-hub-view__agent-status--undetected {
  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
  color: var(--color-text-muted);
}

.skill-hub-view__agent-count {
  font-family: var(--font-mono, 'Maple Mono', monospace);
  color: var(--color-text-secondary);
}

.skill-hub-view__content {
  gap: 1rem;
}

.skill-hub-view__toolbar,
.skill-hub-view__section {
  overflow: hidden;
  border: 1px solid rgb(255 255 255 / 20%);
  border-radius: 0.75rem;
  box-shadow: 0 1px 2px rgb(15 23 42 / 8%);
}

.skill-hub-view__toolbar {
  padding: 1rem;
}

.skill-hub-view__toolbar-row,
.skill-hub-view__toolbar-actions,
.skill-hub-view__toolbar-meta,
.skill-hub-view__toolbar-meta-item,
.skill-hub-view__section-header,
.skill-hub-view__section-title,
.skill-hub-view__search-clear,
.skill-hub-view__toolbar-button {
  display: flex;
  align-items: center;
}

.skill-hub-view__toolbar-row {
  gap: 0.75rem;
  justify-content: space-between;
}

.skill-hub-view__search-shell {
  position: relative;
  flex: 1 1 auto;
}

.skill-hub-view__search-icon,
.skill-hub-view__search-clear {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  color: var(--color-text-muted);
}

.skill-hub-view__search-icon {
  left: 0.75rem;
}

.skill-hub-view__search-input {
  width: 100%;
  border: 1px solid var(--color-border-default);
  border-radius: 0.75rem;
  background: rgb(var(--color-bg-surface-rgb) / 50%);
  padding: 0.625rem 2.5rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--color-text-primary);
  transition: background-color 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
}

.skill-hub-view__search-input::placeholder {
  color: var(--color-text-muted);
}

.skill-hub-view__search-input:hover,
.skill-hub-view__search-input:focus {
  background: var(--color-bg-surface);
}

.skill-hub-view__search-input:focus {
  outline: none;
  box-shadow: 0 0 0 2px rgb(var(--color-success-rgb) / 20%);
}

.skill-hub-view__search-clear {
  right: 0.75rem;
  padding: 0.25rem;
  border-radius: 9999px;
  transition: background-color 0.2s ease;
}

.skill-hub-view__search-clear:hover {
  background: var(--color-bg-surface);
}

.skill-hub-view__toolbar-actions {
  gap: 0.5rem;
}

.skill-hub-view__toolbar-button {
  justify-content: center;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  transition: background-color 0.2s ease, box-shadow 0.2s ease, opacity 0.2s ease;
}

.skill-hub-view__toolbar-button:disabled {
  opacity: 0.5;
}

.skill-hub-view__toolbar-button--secondary {
  border: 1px solid var(--color-border-default);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
}

.skill-hub-view__toolbar-button--secondary:hover {
  background: var(--color-bg-surface);
}

.skill-hub-view__toolbar-button--primary {
  background: var(--color-success);
  color: white;
  font-weight: 600;
  box-shadow: 0 4px 12px rgb(var(--color-success-rgb) / 24%);
}

.skill-hub-view__toolbar-button--primary:hover:not(:disabled) {
  box-shadow: 0 8px 16px rgb(var(--color-success-rgb) / 28%);
}

.skill-hub-view__toolbar-meta {
  gap: 0.75rem;
  margin-top: 0.75rem;
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--color-text-muted);
}

.skill-hub-view__toolbar-meta-item {
  gap: 0.25rem;
}

.skill-hub-view__toolbar-meta-icon--success {
  color: var(--color-success);
}

.skill-hub-view__toolbar-meta-icon--accent {
  color: var(--color-accent-primary);
}

.skill-hub-view__section-header {
  justify-content: space-between;
  gap: 0.75rem;
  padding: 1rem;
  border-bottom: 1px solid rgb(255 255 255 / 10%);
}

.skill-hub-view__section-title,
.skill-hub-view__section-meta {
  font-size: 0.75rem;
  line-height: 1rem;
}

.skill-hub-view__section-title {
  gap: 0.5rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.skill-hub-view__section-icon {
  color: var(--color-success);
}

.skill-hub-view__section-meta {
  font-family: var(--font-mono, 'Maple Mono', monospace);
  color: var(--color-text-muted);
}

@media (width >= 1024px) {
  .skill-hub-view__layout {
    grid-template-columns: repeat(12, minmax(0, 1fr));
  }

  .skill-hub-view__sidebar {
    grid-column: span 3 / span 3;
  }

  .skill-hub-view__content {
    grid-column: span 9 / span 9;
  }
}

@media (width <= 767px) {
  .skill-hub-view__header {
    flex-direction: column;
  }

  .skill-hub-view__toolbar-row,
  .skill-hub-view__toolbar-actions,
  .skill-hub-view__toolbar-meta {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
