<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="max-w-[1600px] mx-auto space-y-6">
      <div class="flex items-start justify-between gap-4">
        <div class="min-w-0">
          <div class="flex items-center gap-3">
            <h2 class="text-2xl font-bold text-[var(--color-text-primary)] font-mono truncate">
              Skill Hub
            </h2>
            <span
              class="px-3 py-1 rounded-full text-sm font-medium bg-[var(--color-success)]/10 text-[var(--color-success)] border border-[var(--color-success)]/20"
            >
              {{ marketplace.total }}
            </span>
          </div>
          <p class="mt-1 text-sm text-[var(--color-text-secondary)]">
            浏览 skills.sh 并一键安装到本机多个 Agent 的全局目录
          </p>
        </div>
        <div class="flex items-center gap-3 flex-shrink-0">
          <RouterLink
            to="/skills"
            class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)] border border-[var(--color-border-default)] hover:bg-[var(--color-bg-surface)]"
          >
            <Book class="w-4 h-4" /><span>本地技能</span>
          </RouterLink>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <div class="lg:col-span-3">
          <div class="glass-effect rounded-xl border border-white/20 shadow-sm overflow-hidden">
            <div class="p-4 border-b border-white/10">
              <div class="flex items-center gap-2 text-sm font-semibold text-[var(--color-text-primary)]">
                <Boxes class="w-4 h-4 text-[var(--color-accent-primary)]" />
                Agents
              </div>
            </div>

            <div
              v-if="agentsLoading"
              class="p-4 text-sm text-[var(--color-text-muted)]"
            >
              加载中...
            </div>

            <div
              v-else
              class="p-2 space-y-2"
            >
              <button
                v-for="agent in agents"
                :key="agent.id"
                class="w-full text-left px-3 py-3 rounded-lg border transition-all duration-200"
                :class="selectedAgent === agent.id ? 'bg-[var(--color-accent-primary)]/10 border-[var(--color-accent-primary)]/30' : 'bg-[var(--color-bg-elevated)] border-[var(--color-border-default)] hover:bg-[var(--color-bg-surface)]'"
                @click="selectAgent(agent.id)"
              >
                <div class="flex items-center justify-between gap-3">
                  <div class="min-w-0">
                    <div class="text-sm font-semibold text-[var(--color-text-primary)] truncate">
                      {{ agent.display_name }}
                    </div>
                    <div class="text-xs text-[var(--color-text-muted)] truncate">
                      {{ agent.global_skills_dir || '未配置路径' }}
                    </div>
                  </div>
                  <div class="flex items-center gap-2 flex-shrink-0">
                    <span
                      class="text-xs px-2 py-1 rounded-full border"
                      :class="agent.detected ? 'border-[var(--color-success)]/30 text-[var(--color-success)] bg-[var(--color-success)]/10' : 'border-[var(--color-border-default)] text-[var(--color-text-muted)] bg-[var(--color-bg-surface)]'"
                    >
                      {{ agent.detected ? '已检测' : '未检测' }}
                    </span>
                    <span class="text-xs font-mono text-[var(--color-text-secondary)]">
                      {{ agent.installed_count }}
                    </span>
                  </div>
                </div>
              </button>
            </div>
          </div>
        </div>

        <div class="lg:col-span-9 space-y-4">
          <div class="glass-effect rounded-xl border border-white/20 shadow-sm p-4">
            <div class="flex flex-col sm:flex-row gap-3 sm:items-center sm:justify-between">
              <div class="relative flex-1">
                <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-[var(--color-text-muted)]" />
                <input
                  v-model="searchQuery"
                  type="text"
                  placeholder="搜索 skills.sh（owner/repo 或 skill 名）"
                  class="w-full pl-10 pr-10 py-2.5 rounded-xl bg-[var(--color-bg-surface)]/50 border border-[var(--color-border-default)] hover:bg-[var(--color-bg-surface)] focus:bg-[var(--color-bg-surface)] focus:outline-none focus:ring-2 focus:ring-[var(--color-success)]/20 text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] text-sm transition-all"
                  @keydown.enter="runSearch"
                >
                <button
                  v-if="searchQuery"
                  class="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 rounded-full hover:bg-[var(--color-bg-surface)] text-[var(--color-text-muted)] transition-all"
                  @click="clearSearch"
                >
                  <X class="w-4 h-4" />
                </button>
              </div>

              <div class="flex items-center gap-2">
                <button
                  class="px-4 py-2 rounded-lg font-medium transition-colors bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)] border border-[var(--color-border-default)] hover:bg-[var(--color-bg-surface)] text-sm"
                  @click="reloadAll"
                >
                  刷新
                </button>
                <button
                  class="px-4 py-2 rounded-lg font-semibold transition-all duration-200 bg-[var(--color-success)] text-white shadow-md hover:shadow-lg text-sm"
                  :disabled="marketplaceLoading || installLoading"
                  @click="runSearch"
                >
                  {{ searchQuery ? '搜索' : '热门' }}
                </button>
              </div>
            </div>

            <div class="mt-3 flex items-center gap-3 text-xs text-[var(--color-text-muted)]">
              <span class="inline-flex items-center gap-1">
                <Zap class="w-3.5 h-3.5 text-[var(--color-success)]" />
                默认全局安装
              </span>
              <span class="inline-flex items-center gap-1">
                <ShieldCheck class="w-3.5 h-3.5 text-[var(--color-accent-primary)]" />
                解析 SKILL.md frontmatter
              </span>
              <span
                v-if="marketplace.cached"
                class="inline-flex items-center gap-1"
              >
                <Clock class="w-3.5 h-3.5" />
                缓存命中
              </span>
            </div>
          </div>

          <div class="glass-effect rounded-xl border border-white/20 shadow-sm overflow-hidden">
            <div class="p-4 border-b border-white/10 flex items-center justify-between gap-3">
              <div class="flex items-center gap-2 text-sm font-semibold text-[var(--color-text-primary)]">
                <Store class="w-4 h-4 text-[var(--color-success)]" />
                Marketplace
              </div>
              <div class="text-xs text-[var(--color-text-muted)] font-mono">
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
                <Store class="w-10 h-10 opacity-50" />
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
                    class="px-3 py-2 rounded-lg text-sm font-semibold transition-all duration-200 border"
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
                <Package class="w-4 h-4 text-[var(--color-accent-primary)]" />
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
                <Package class="w-10 h-10 opacity-50" />
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
} from '@/api/modules'
import { Book, Boxes, Clock, Package, Search, ShieldCheck, Store, X, Zap } from 'lucide-vue-next'

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

const installedSkillNames = computed(() => {
  return new Set(installedSkills.value.map(s => s.name))
})

async function loadAgents() {
  agentsLoading.value = true
  try {
    const data = await getSkillHubAgents()
    agents.value = data
    if (!agents.value.some(a => a.id === selectedAgent.value) && agents.value.length > 0) {
      selectedAgent.value = agents.value[0].id
    }
  } catch (e: any) {
    ui.showError(e?.message || '加载 Agents 失败')
  } finally {
    agentsLoading.value = false
  }
}

async function loadInstalled() {
  installedLoading.value = true
  try {
    installedSkills.value = await getSkillHubAgentSkills(selectedAgent.value)
  } catch (e: any) {
    installedSkills.value = []
    ui.showError(e?.message || '加载已安装技能失败')
  } finally {
    installedLoading.value = false
  }
}

async function loadTrending() {
  marketplaceLoading.value = true
  try {
    marketplace.value = await getSkillHubTrending({ limit: 50, page: 1 })
  } catch (e: any) {
    marketplace.value = { items: [], total: 0, cached: false }
    ui.showError(e?.message || '加载 Marketplace 失败')
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
    marketplace.value = await searchSkillHubMarketplace({ q: searchQuery.value.trim(), limit: 50, page: 1 })
  } catch (e: any) {
    marketplace.value = { items: [], total: 0, cached: false }
    ui.showError(e?.message || '搜索失败')
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
    const res = await installSkillHubSkill({ package: pkg, agents: [selectedAgent.value], force: false })
    const r = res.results?.[0]
    if (r && !r.ok) {
      throw new Error(r.message || '安装失败')
    }
    ui.showSuccess('安装完成')
    await Promise.all([loadAgents(), loadInstalled()])
  } catch (e: any) {
    ui.showError(e?.message || '安装失败')
  } finally {
    installLoading.value = false
  }
}

async function removeInstalledSkill(skillName: string) {
  removeLoading.value = true
  try {
    const res = await removeSkillHubSkill({ skill: skillName, agents: [selectedAgent.value] })
    const r = res.results?.[0]
    if (r && !r.ok) {
      throw new Error(r.message || '卸载失败')
    }
    ui.showSuccess('已卸载')
    await Promise.all([loadAgents(), loadInstalled()])
  } catch (e: any) {
    ui.showError(e?.message || '卸载失败')
  } finally {
    removeLoading.value = false
  }
}

onMounted(async () => {
  await loadAgents()
  await Promise.all([loadInstalled(), loadTrending()])
})
</script>
