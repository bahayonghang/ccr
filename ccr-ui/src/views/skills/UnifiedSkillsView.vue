<template>
  <div class="skills-console">
    <PageHeaderCard
      title="Skills 管理中心"
      description="跨平台 AI CLI 工具的统一技能库存、来源和安装工作台。"
      badge="Workspace"
      icon="Package"
      tone="secondary"
    >
      <template #actions>
        <button
          class="console-button"
          :disabled="inventoryLoading || sourcesLoading || marketplaceLoading"
          @click="handleRefresh"
        >
          <SIcon
            name="RefreshCw"
            size="w-4 h-4"
            :class="{ 'animate-spin': inventoryLoading || sourcesLoading || marketplaceLoading }"
          />
          <span>Refresh</span>
        </button>
      </template>

      <div class="skills-header-stats">
        <div class="skills-header-stat">
          <span class="skills-header-stat__label">Logical skills</span>
          <strong class="skills-header-stat__value">{{ stats.logicalSkills }}</strong>
        </div>
        <div class="skills-header-stat">
          <span class="skills-header-stat__label">Sources</span>
          <strong class="skills-header-stat__value">{{ stats.sources }}</strong>
        </div>
        <div class="skills-header-stat">
          <span class="skills-header-stat__label">Installed packages</span>
          <strong class="skills-header-stat__value">{{ stats.installations }}</strong>
        </div>
      </div>
    </PageHeaderCard>

    <AsyncStatePanel
      v-if="runtimeUnavailable"
      state="runtime-unavailable"
      :title="runtimeCopy.title"
      :description="runtimeCopy.description"
      :action-label="runtimeCopy.actionLabel"
      action-icon="ArrowLeft"
      @action="$router.push('/')"
    />

    <template v-else>
      <div class="console-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="console-tab"
          :class="{ 'console-tab--active': routeState.tab === tab.id }"
          @click="setTab(tab.id)"
        >
          <SIcon
            :name="tab.icon"
            size="w-4 h-4"
          />
          <span>{{ tab.label }}</span>
          <span class="console-tab__count">{{ tab.count }}</span>
        </button>
      </div>

      <div class="console-layout">
        <aside class="console-sidebar">
          <section class="panel">
            <div class="panel__header">
              <h2 class="panel__title">
                Targets
              </h2>
              <button
                class="panel__link"
                @click="selectDetectedPlatforms"
              >
                Detected
              </button>
            </div>
            <div class="platform-grid">
              <button
                v-for="platform in platforms"
                :key="platform.id"
                class="platform-chip"
                :class="{
                  'platform-chip--active': selectedPlatforms.includes(platform.id),
                  'platform-chip--disabled': !platform.detected,
                }"
                :disabled="!platform.detected"
                @click="togglePlatform(platform.id)"
              >
                <SIcon
                  :name="platformIcon(platform.id)"
                  size="w-4 h-4"
                />
                <span>{{ platform.displayName }}</span>
                <span class="platform-chip__count">{{ platform.installedCount }}</span>
              </button>
            </div>
          </section>

          <section
            v-if="routeState.tab === 'inventory'"
            class="panel"
          >
            <div class="panel__header">
              <h2 class="panel__title">
                Filters
              </h2>
              <button
                class="panel__link"
                @click="resetInventoryFilters"
              >
                Reset
              </button>
            </div>

            <label class="field">
              <span class="field__label">Search</span>
              <input
                v-model="searchInput"
                class="field__input"
                type="text"
                placeholder="name / tag / category"
              >
            </label>

            <label class="field">
              <span class="field__label">Platform</span>
              <select
                class="field__input"
                :value="routeState.platform"
                @change="updatePlatformFilter(($event.target as HTMLSelectElement).value)"
              >
                <option value="all">All platforms</option>
                <option
                  v-for="platform in platforms"
                  :key="platform.id"
                  :value="platform.id"
                >
                  {{ platform.displayName }}
                </option>
              </select>
            </label>

            <label class="field">
              <span class="field__label">Origin</span>
              <select
                class="field__input"
                :value="routeState.origin"
                @change="updateOriginFilter(($event.target as HTMLSelectElement).value)"
              >
                <option value="all">All origins</option>
                <option
                  v-for="origin in originOptions"
                  :key="origin"
                  :value="origin"
                >
                  {{ origin }}
                </option>
              </select>
            </label>

            <label class="field">
              <span class="field__label">Category</span>
              <select
                v-model="filters.category"
                class="field__input"
              >
                <option :value="null">All categories</option>
                <option
                  v-for="category in categories"
                  :key="category"
                  :value="category"
                >
                  {{ category }}
                </option>
              </select>
            </label>

            <div class="tag-cloud">
              <button
                v-for="tag in tags"
                :key="tag"
                class="tag-chip"
                :class="{ 'tag-chip--active': filters.tags.includes(tag) }"
                @click="toggleTag(tag)"
              >
                {{ tag }}
              </button>
            </div>
          </section>

          <section
            v-else-if="routeState.tab === 'sources'"
            class="panel"
          >
            <div class="panel__header">
              <h2 class="panel__title">
                Add Source
              </h2>
            </div>

            <label class="field">
              <span class="field__label">Git repo</span>
              <input
                v-model="gitSourceUrl"
                class="field__input"
                type="text"
                placeholder="https://github.com/owner/repo"
              >
            </label>
            <button
              class="console-button console-button--primary"
              :disabled="mutationLoading || !gitSourceUrl.trim()"
              @click="handleAddGitSource"
            >
              <SIcon
                name="Github"
                size="w-4 h-4"
              />
              <span>Add Git Source</span>
            </button>

            <label class="field">
              <span class="field__label">Local directory</span>
              <div class="field__row">
                <input
                  v-model="localSourcePath"
                  class="field__input"
                  type="text"
                  placeholder="D:/skills/repo"
                >
                <button
                  class="console-button"
                  @click="handlePickFolder('source')"
                >
                  <SIcon
                    name="FolderOpen"
                    size="w-4 h-4"
                  />
                </button>
              </div>
            </label>
            <button
              class="console-button console-button--primary"
              :disabled="mutationLoading || !localSourcePath.trim()"
              @click="handleAddLocalSource"
            >
              <SIcon
                name="FolderGit2"
                size="w-4 h-4"
              />
              <span>Add Local Source</span>
            </button>
          </section>

          <section
            v-else
            class="panel"
          >
            <div class="panel__header">
              <h2 class="panel__title">
                Manual Install
              </h2>
            </div>

            <label class="field">
              <span class="field__label">GitHub</span>
              <input
                v-model="manualGithub"
                class="field__input"
                type="text"
                placeholder="owner/repo or owner/repo@skill"
              >
            </label>
            <label class="field">
              <span class="field__label">Local path</span>
              <div class="field__row">
                <input
                  v-model="manualLocalPath"
                  class="field__input"
                  type="text"
                  placeholder="D:/skills/local-skill"
                >
                <button
                  class="console-button"
                  @click="handlePickFolder('manual')"
                >
                  <SIcon
                    name="FolderOpen"
                    size="w-4 h-4"
                  />
                </button>
              </div>
            </label>
            <label class="field">
              <span class="field__label">npx package</span>
              <input
                v-model="manualNpx"
                class="field__input"
                type="text"
                placeholder="owner/repo or owner/repo@skill"
              >
            </label>
          </section>

          <section class="panel">
            <div class="panel__header">
              <h2 class="panel__title">
                Activity
              </h2>
            </div>
            <div class="activity-list">
              <div
                v-for="entry in operationLog.slice(0, 8)"
                :key="entry.id"
                class="activity-row"
              >
                <span
                  class="activity-row__status"
                  :class="`activity-row__status--${entry.status}`"
                />
                <div class="activity-row__body">
                  <strong>{{ entry.action }}</strong>
                  <span>{{ entry.target }}</span>
                  <span class="activity-row__meta">{{ formatTime(entry.timestamp) }}</span>
                </div>
              </div>
            </div>
          </section>
        </aside>

        <div class="console-main">
          <section
            v-if="routeState.tab === 'inventory'"
            class="inventory-layout"
          >
            <section class="panel">
              <div class="panel__header">
                <h2 class="panel__title">
                  Inventory
                </h2>
                <span class="panel__link">{{ filteredSkills.length }} results</span>
              </div>

              <div
                ref="inventoryScrollRef"
                class="inventory-list"
              >
                <div :style="{ height: `${rowVirtualizer.getTotalSize()}px`, position: 'relative' }">
                  <div
                    v-for="virtualRow in rowVirtualizer.getVirtualItems()"
                    :key="filteredSkills[virtualRow.index]?.id"
                    class="inventory-row"
                    :style="{ transform: `translateY(${virtualRow.start}px)` }"
                  >
                    <button
                      :ref="measureElement"
                      :data-index="virtualRow.index"
                      class="skill-card"
                      :class="{ 'skill-card--active': filteredSkills[virtualRow.index]?.id === selectedSkill?.id }"
                      @click="handleSelectSkill(filteredSkills[virtualRow.index]?.id)"
                    >
                      <div class="skill-card__head">
                        <div class="min-w-0">
                          <h3 class="truncate">
                            {{ filteredSkills[virtualRow.index]?.name }}
                          </h3>
                          <p
                            class="skill-card__desc"
                            :title="filteredSkills[virtualRow.index]?.description ?? ''"
                          >
                            {{ formatSkillDescription(filteredSkills[virtualRow.index]?.description) }}
                          </p>
                        </div>
                        <span class="console-tab__count">{{ filteredSkills[virtualRow.index]?.installCount }}</span>
                      </div>
                      <div class="skill-card__meta">
                        <span class="badge">{{ filteredSkills[virtualRow.index]?.origin }}</span>
                        <span
                          v-for="installation in filteredSkills[virtualRow.index]?.installations.slice(0, 2)"
                          :key="installation.id"
                          class="badge"
                        >
                          {{ installation.platformName }}
                        </span>
                        <span
                          v-if="(filteredSkills[virtualRow.index]?.installations.length ?? 0) > 2"
                          class="badge"
                        >
                          +{{ (filteredSkills[virtualRow.index]?.installations.length ?? 0) - 2 }}
                        </span>
                      </div>
                    </button>
                  </div>
                </div>
              </div>
            </section>

            <section class="panel">
              <div class="panel__header">
                <h2 class="panel__title">
                  Detail
                </h2>
                <button
                  v-if="selectedSkill"
                  class="console-button console-button--danger"
                  :disabled="mutationLoading"
                  @click="handleRemoveSkill"
                >
                  <SIcon
                    name="Trash2"
                    size="w-4 h-4"
                  />
                  <span>Remove Skill</span>
                </button>
              </div>

              <div
                v-if="selectedSkill"
                class="detail-stack"
              >
                <div class="detail-card">
                  <h3>{{ selectedSkill.name }}</h3>
                  <p
                    class="detail-description"
                    :class="{ 'detail-description--collapsed': !showFullDetailDescription }"
                    :title="selectedSkill.description ?? ''"
                  >
                    {{ selectedSkill.description || 'No description' }}
                  </p>
                  <button
                    v-if="selectedSkill.description && selectedSkill.description.length > 320"
                    type="button"
                    class="detail-description__toggle"
                    @click="showFullDetailDescription = !showFullDetailDescription"
                  >
                    {{ showFullDetailDescription ? 'Show less' : 'Show more' }}
                  </button>
                  <div class="detail-card__grid">
                    <div>
                      <span>Origin</span>
                      <strong>{{ selectedSkill.origin }}</strong>
                    </div>
                    <div>
                      <span>Author</span>
                      <strong>{{ selectedSkill.author || 'Unknown' }}</strong>
                    </div>
                    <div>
                      <span>Version</span>
                      <strong>{{ selectedSkill.version || 'N/A' }}</strong>
                    </div>
                    <div>
                      <span>Category</span>
                      <strong>{{ selectedSkill.category || 'Uncategorized' }}</strong>
                    </div>
                  </div>
                  <div class="skill-card__meta">
                    <span
                      v-for="tag in selectedSkill.tags"
                      :key="tag"
                      class="tag-chip tag-chip--active"
                    >
                      {{ tag }}
                    </span>
                  </div>
                </div>

                <div class="detail-card">
                  <div class="panel__header">
                    <h2 class="panel__title">
                      Installations
                    </h2>
                    <button
                      class="console-button"
                      :disabled="mutationLoading || !selectedSkill || selectedPlatforms.length === 0"
                      @click="handleSyncSelected"
                    >
                      <SIcon
                        name="CopyPlus"
                        size="w-4 h-4"
                      />
                      <span>Sync to selected</span>
                    </button>
                  </div>
                  <div class="installation-list">
                    <div
                      v-for="installation in selectedSkill.installations"
                      :key="installation.id"
                      class="installation-row"
                    >
                      <div class="installation-row__main">
                        <strong>{{ installation.platformName }}</strong>
                        <span>{{ shortenPath(installation.installPath) }}</span>
                      </div>
                      <div class="installation-row__actions">
                        <button
                          class="console-button"
                          @click="handleSelectInstallation(installation.id)"
                        >
                          <SIcon
                            name="Eye"
                            size="w-4 h-4"
                          />
                        </button>
                        <button
                          class="console-button console-button--danger"
                          :disabled="mutationLoading"
                          @click="handleRemoveInstallation(installation.id)"
                        >
                          <SIcon
                            name="Trash2"
                            size="w-4 h-4"
                          />
                        </button>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="detail-card">
                  <div class="panel__header">
                    <h2 class="panel__title">
                      Content
                    </h2>
                    <div class="installation-row__actions">
                      <button
                        class="console-button"
                        @click="setMode(routeState.mode === 'edit' ? 'view' : 'edit')"
                      >
                        <SIcon
                          :name="routeState.mode === 'edit' ? 'Eye' : 'Pencil'"
                          size="w-4 h-4"
                        />
                        <span>{{ routeState.mode === 'edit' ? 'Preview' : 'Edit' }}</span>
                      </button>
                      <button
                        class="console-button console-button--primary"
                        :disabled="mutationLoading || routeState.mode !== 'edit' || !contentDirty || !selectedInstallation"
                        @click="handleSaveContent"
                      >
                        <SIcon
                          name="Save"
                          size="w-4 h-4"
                        />
                        <span>Save</span>
                      </button>
                    </div>
                  </div>

                  <textarea
                    v-if="routeState.mode === 'edit'"
                    v-model="editBuffer"
                    class="content-editor"
                  />
                  <pre
                    v-else
                    class="content-preview"
                  >{{ contentPreview }}</pre>
                </div>
              </div>

              <div
                v-else
                class="empty-state"
              >
                Select a logical skill to inspect it.
              </div>
            </section>
          </section>

          <section
            v-else-if="routeState.tab === 'sources'"
            class="source-list"
          >
            <article
              v-for="source in sources"
              :key="source.id"
              class="panel"
            >
              <div class="panel__header">
                <div>
                  <h2 class="panel__title">
                    {{ source.name }}
                  </h2>
                  <p class="console-header__subtitle">
                    {{ source.description || source.location }}
                  </p>
                </div>
                <div class="installation-row__actions">
                  <button
                    class="console-button"
                    :disabled="mutationLoading"
                    @click="handleSyncSource(source.id)"
                  >
                    <SIcon
                      name="RefreshCw"
                      size="w-4 h-4"
                    />
                  </button>
                  <button
                    class="console-button console-button--danger"
                    :disabled="mutationLoading"
                    @click="handleRemoveSource(source.id)"
                  >
                    <SIcon
                      name="Trash2"
                      size="w-4 h-4"
                    />
                  </button>
                </div>
              </div>

              <div class="skill-card__meta">
                <span class="badge">{{ source.type }}</span>
                <span class="badge">{{ source.health }}</span>
                <span class="badge">{{ source.skillCount }} skills</span>
              </div>

              <div class="source-skill-grid">
                <button
                  v-for="skill in source.skills"
                  :key="`${source.id}:${skill.id}`"
                  class="source-skill-card"
                  :disabled="selectedPlatforms.length === 0 || mutationLoading"
                  @click="handleInstallFromSource(source.id, skill.id)"
                >
                  <div>
                    <strong>{{ skill.name }}</strong>
                    <p>{{ skill.description || 'No description' }}</p>
                  </div>
                  <SIcon
                    name="ArrowRight"
                    size="w-4 h-4"
                  />
                </button>
              </div>
            </article>
          </section>

          <section
            v-else
            class="marketplace-layout"
          >
            <section class="panel">
              <div class="panel__header">
                <h2 class="panel__title">
                  Marketplace
                </h2>
                <div class="field__row">
                  <input
                    v-model="marketplaceSearch"
                    class="field__input"
                    type="text"
                    placeholder="search skills.sh"
                  >
                  <button
                    class="console-button"
                    :disabled="marketplaceLoading"
                    @click="handleMarketplaceSearch"
                  >
                    <SIcon
                      name="Search"
                      size="w-4 h-4"
                    />
                  </button>
                </div>
              </div>

              <div class="marketplace-grid">
                <article
                  v-for="item in marketplace.items"
                  :key="item.package"
                  class="marketplace-card"
                >
                  <div class="marketplace-card__header">
                    <div>
                      <h3>{{ item.package }}</h3>
                      <p>{{ item.description || `${item.owner}/${item.repo}` }}</p>
                    </div>
                    <span class="badge">{{ item.stars ?? 0 }}★</span>
                  </div>
                  <button
                    class="console-button console-button--primary"
                    :disabled="selectedPlatforms.length === 0 || mutationLoading"
                    @click="handleInstallMarketplace(item)"
                  >
                    <SIcon
                      name="Download"
                      size="w-4 h-4"
                    />
                    <span>Install</span>
                  </button>
                </article>
              </div>

              <div class="pagination-row">
                <button
                  class="console-button"
                  :disabled="routeState.page <= 1"
                  @click="updatePage(routeState.page - 1)"
                >
                  Prev
                </button>
                <span class="panel__link">Page {{ routeState.page }}</span>
                <button
                  class="console-button"
                  :disabled="marketplace.items.length < marketplace.pageSize"
                  @click="updatePage(routeState.page + 1)"
                >
                  Next
                </button>
              </div>
            </section>

            <section class="panel manual-actions">
              <div class="panel__header">
                <h2 class="panel__title">
                  Manual Install
                </h2>
              </div>
              <button
                class="console-button console-button--primary"
                :disabled="mutationLoading || !manualGithub.trim() || selectedPlatforms.length === 0"
                @click="installManual('github')"
              >
                <SIcon
                  name="Github"
                  size="w-4 h-4"
                />
                <span>Install GitHub</span>
              </button>
              <button
                class="console-button console-button--primary"
                :disabled="mutationLoading || !manualLocalPath.trim() || selectedPlatforms.length === 0"
                @click="installManual('local')"
              >
                <SIcon
                  name="HardDrive"
                  size="w-4 h-4"
                />
                <span>Install Local</span>
              </button>
              <button
                class="console-button console-button--primary"
                :disabled="mutationLoading || !manualNpx.trim() || selectedPlatforms.length === 0"
                @click="installManual('npx')"
              >
                <SIcon
                  name="Terminal"
                  size="w-4 h-4"
                />
                <span>Install npx</span>
              </button>
              <p class="console-header__subtitle">
                {{ npxStatus?.available ? `npx ${npxStatus.version ?? ''}` : 'npx unavailable' }}
              </p>
            </section>
          </section>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import { useUIStore } from '@/stores/ui'
import { PLATFORM_CONFIG } from '@/types/skills'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import type { Platform, SkillOrigin, SkillsRouteState, SkillsTab } from '@/types/skills'

const router = useRouter()
const route = useRoute()
const uiStore = useUIStore()
const {
  initialize,
  refresh,
  loadMarketplace,
  loadNpxStatus,
  applyRouteState,
  install,
  syncSkill,
  removeInstallation,
  removeSkillRecord,
  addGitSource,
  addLocalSourceRecord,
  syncSource,
  removeSource,
  ensureDetail,
  ensureContent,
  saveContent,
  browseFolder,
  platforms,
  sources,
  marketplace,
  filters,
  routeState,
  selectedSkill,
  selectedInstallation,
  operationLog,
  npxStatus,
  mutationLoading,
  filteredSkills,
  categories,
  tags,
  stats,
  inventoryLoading,
  sourcesLoading,
  marketplaceLoading,
  selectSkill,
} = useUnifiedSkills()

const tabs = computed(() => [
  { id: 'inventory' as SkillsTab, label: 'Inventory', icon: 'Package', count: stats.value.logicalSkills },
  { id: 'sources' as SkillsTab, label: 'Sources', icon: 'FolderGit2', count: stats.value.sources },
  { id: 'marketplace' as SkillsTab, label: 'Marketplace', icon: 'Store', count: marketplace.value.total },
])
const runtimeUnavailable = computed(() => !isTauriRuntime())
const runtimeCopy = computed(() => getRuntimeUnavailableCopy('skills'))

const originOptions: SkillOrigin[] = ['marketplace', 'github', 'repo', 'local', 'npx', 'unknown']
const selectedPlatforms = ref<Platform[]>([])
const inventoryScrollRef = ref<HTMLElement | null>(null)
const searchInput = ref('')
const marketplaceSearch = ref('')
const gitSourceUrl = ref('')
const localSourcePath = ref('')
const manualGithub = ref('')
const manualLocalPath = ref('')
const manualNpx = ref('')
const currentContent = ref<Awaited<ReturnType<typeof ensureContent>> | null>(null)
const editBuffer = ref('')
const showFullDetailDescription = ref(false)
let stopSkillsEvent: null | (() => void) = null
const rowVirtualizer = useVirtualizer(computed(() => ({
  count: filteredSkills.value.length,
  getScrollElement: () => inventoryScrollRef.value,
  estimateSize: () => 140,
  overscan: 6,
})))
const contentDirty = computed(() => currentContent.value != null && editBuffer.value !== currentContent.value.raw)
const contentPreview = computed(() => {
  const raw = currentContent.value?.raw ?? ''
  return raw ? stripFrontmatter(raw) : 'No content loaded.'
})

const measureElement = (element: unknown) => {
  rowVirtualizer.value.measureElement(element instanceof Element ? element : null)
}

function platformIcon(platformId: Platform) {
  return PLATFORM_CONFIG[platformId]?.icon ?? 'Code2'
}

function stripFrontmatter(raw: string) {
  const normalized = raw.replace(/\r\n/g, '\n')
  const trimmed = normalized.trimStart()
  const lines = trimmed.split('\n')

  if (lines[0]?.trim() !== '---') return raw

  for (let i = 1; i < lines.length; i += 1) {
    if (lines[i].trim() === '---') {
      return lines.slice(i + 1).join('\n').trim()
    }
  }

  return raw
}

function formatSkillDescription(value?: string) {
  const description = value?.trim()
  if (!description) return 'No description'
  const maxLength = 280
  if (description.length <= maxLength) return description
  return description.slice(0, maxLength).trimEnd() + '…'
}

function normalizeRouteState(query: Record<string, unknown>): SkillsRouteState {
  return {
    tab: query.tab === 'sources' || query.tab === 'marketplace' ? query.tab : 'inventory',
    selected: typeof query.selected === 'string' ? query.selected : null,
    mode: query.mode === 'edit' ? 'edit' : 'view',
    platform: typeof query.platform === 'string' && query.platform in PLATFORM_CONFIG ? query.platform as Platform : 'all',
    origin: typeof query.origin === 'string' && originOptions.includes(query.origin as SkillOrigin) ? query.origin as SkillOrigin : 'all',
    q: typeof query.q === 'string' ? query.q : '',
    page: typeof query.page === 'string' ? Math.max(Number(query.page) || 1, 1) : 1,
    source: typeof query.source === 'string' ? query.source : null,
  }
}

function replaceRoute(patch: Partial<SkillsRouteState>) {
  const nextState = {
    ...routeState.value,
    ...patch,
  }
  applyRouteState(nextState)
  const query: Record<string, string> = {}
  if (nextState.tab !== 'inventory') query.tab = nextState.tab
  if (nextState.selected) query.selected = nextState.selected
  if (nextState.mode !== 'view') query.mode = nextState.mode
  if (nextState.platform !== 'all') query.platform = nextState.platform
  if (nextState.origin !== 'all') query.origin = nextState.origin
  if (nextState.q) query.q = nextState.q
  if (nextState.page > 1) query.page = String(nextState.page)
  if (nextState.source) query.source = nextState.source
  void router.replace({ path: '/skills', query })
}

function setTab(tab: SkillsTab) {
  replaceRoute({ tab, page: 1 })
}

function updatePlatformFilter(value: string) {
  replaceRoute({ platform: value as SkillsRouteState['platform'] })
}

function updateOriginFilter(value: string) {
  replaceRoute({ origin: value as SkillsRouteState['origin'] })
}

function updatePage(page: number) {
  replaceRoute({ page })
}

function resetInventoryFilters() {
  filters.value = { search: '', platform: 'all', origin: 'all', category: null, tags: [], source: 'all' }
  searchInput.value = ''
  replaceRoute({ platform: 'all', origin: 'all', q: '' })
}

function togglePlatform(platform: Platform) {
  selectedPlatforms.value = selectedPlatforms.value.includes(platform)
    ? selectedPlatforms.value.filter((item) => item !== platform)
    : [...selectedPlatforms.value, platform]
}

function selectDetectedPlatforms() {
  selectedPlatforms.value = platforms.value.filter((platform) => platform.detected).map((platform) => platform.id)
}

function toggleTag(tag: string) {
  filters.value = {
    ...filters.value,
    tags: filters.value.tags.includes(tag)
      ? filters.value.tags.filter((item) => item !== tag)
      : [...filters.value.tags, tag],
  }
}

async function handleRefresh() {
  if (runtimeUnavailable.value) return
  await refresh(routeState.value.tab === 'marketplace')
  if (routeState.value.tab === 'marketplace') {
    await loadMarketplace(true)
  }
}

function handleSelectSkill(skillId?: string) {
  if (!skillId) return
  replaceRoute({ selected: skillId })
  selectSkill(skillId, null)
  void ensureDetail(skillId, true)
}

function handleSelectInstallation(installationId: string) {
  if (!selectedSkill.value) return
  selectSkill(selectedSkill.value.id, installationId)
}

function setMode(mode: 'view' | 'edit') {
  replaceRoute({ mode })
}

async function loadSelectedContent() {
  if (runtimeUnavailable.value) {
    currentContent.value = null
    editBuffer.value = ''
    return
  }
  if (!selectedSkill.value || !selectedInstallation.value) {
    currentContent.value = null
    editBuffer.value = ''
    return
  }
  currentContent.value = await ensureContent(selectedSkill.value.id, selectedInstallation.value.id, true)
  editBuffer.value = currentContent.value?.raw ?? ''
}

async function handleSaveContent() {
  if (!selectedSkill.value || !selectedInstallation.value) return
  try {
    const saved = await saveContent(selectedSkill.value.id, selectedInstallation.value.id, editBuffer.value)
    currentContent.value = saved
    editBuffer.value = saved.raw
    setMode('view')
    uiStore.showSuccess('Skill content saved')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleSyncSelected() {
  if (!selectedSkill.value || !selectedInstallation.value || selectedPlatforms.value.length === 0) return
  try {
    await syncSkill({
      skillId: selectedSkill.value.id,
      installationId: selectedInstallation.value.id,
      targetPlatforms: selectedPlatforms.value,
    })
    uiStore.showSuccess('Skill synced to selected platforms')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleRemoveInstallation(installationId: string) {
  if (!selectedSkill.value) return
  try {
    await removeInstallation(selectedSkill.value.id, installationId)
    uiStore.showSuccess('Installation removed')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleRemoveSkill() {
  if (!selectedSkill.value) return
  try {
    await removeSkillRecord(selectedSkill.value.id)
    replaceRoute({ selected: null, mode: 'view' })
    currentContent.value = null
    editBuffer.value = ''
    uiStore.showSuccess('Skill removed')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleAddGitSource() {
  try {
    await addGitSource(gitSourceUrl.value.trim())
    gitSourceUrl.value = ''
    uiStore.showSuccess('Git source added')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleAddLocalSource() {
  try {
    await addLocalSourceRecord(localSourcePath.value.trim())
    localSourcePath.value = ''
    uiStore.showSuccess('Local source added')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleSyncSource(sourceId: string) {
  try {
    await syncSource(sourceId)
    uiStore.showSuccess('Source synced')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleRemoveSource(sourceId: string) {
  try {
    await removeSource(sourceId)
    uiStore.showSuccess('Source removed')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleInstallFromSource(sourceId: string, skillId: string) {
  try {
    await install({
      sourceKind: 'source',
      sourceRef: sourceId,
      sourceSkillId: skillId,
      targetPlatforms: selectedPlatforms.value,
    })
    uiStore.showSuccess('Skill installed from source')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleMarketplaceSearch() {
  replaceRoute({ q: marketplaceSearch.value.trim(), page: 1 })
  await loadMarketplace(true)
}

async function handleInstallMarketplace(item: { package: string; skill?: string }) {
  try {
    await install({
      sourceKind: 'marketplace',
      sourceRef: item.package,
      sourceSkillId: item.skill,
      targetPlatforms: selectedPlatforms.value,
    })
    uiStore.showSuccess('Marketplace skill installed')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function installManual(kind: 'github' | 'local' | 'npx') {
  const sourceRef = kind === 'github' ? manualGithub.value : kind === 'local' ? manualLocalPath.value : manualNpx.value
  try {
    await install({
      sourceKind: kind,
      sourceRef: sourceRef.trim(),
      targetPlatforms: selectedPlatforms.value,
    })
    if (kind === 'github') manualGithub.value = ''
    if (kind === 'local') manualLocalPath.value = ''
    if (kind === 'npx') manualNpx.value = ''
    uiStore.showSuccess(`Installed via ${kind}`)
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handlePickFolder(target: 'source' | 'manual') {
  try {
    const path = await browseFolder()
    if (!path) return
    if (target === 'source') {
      localSourcePath.value = path
    } else {
      manualLocalPath.value = path
    }
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString()
}

function shortenPath(path: string) {
  const normalized = path.replace(/\\/g, '/')
  const parts = normalized.split('/')
  return parts.length <= 4 ? normalized : `.../${parts.slice(-4).join('/')}`
}

watch(
  () => route.query,
  (query) => {
    const normalized = normalizeRouteState(query as Record<string, unknown>)
    applyRouteState(normalized)
    searchInput.value = normalized.q
    marketplaceSearch.value = normalized.q

    if (runtimeUnavailable.value) {
      selectSkill(null, null)
      return
    }

    if (normalized.selected) {
      selectSkill(normalized.selected, null)
      void ensureDetail(normalized.selected, true)
    } else {
      selectSkill(null, null)
    }
    if (normalized.tab === 'marketplace') {
      void loadMarketplace(true)
    }
  },
  { immediate: true }
)

watch(searchInput, (value) => {
  window.clearTimeout(searchTimer)
  searchTimer = window.setTimeout(() => {
    filters.value = {
      ...filters.value,
      search: value.trim(),
    }
    replaceRoute({ q: value.trim() })
  }, 300)
})

watch([selectedSkill, selectedInstallation], () => {
  void loadSelectedContent()
})

watch(selectedSkill, () => {
  showFullDetailDescription.value = false
})

watch(
  platforms,
  (currentPlatforms) => {
    if (selectedPlatforms.value.length === 0 && currentPlatforms.length > 0) {
      selectDetectedPlatforms()
    }
  },
  { immediate: true }
)

let searchTimer = 0

onMounted(async () => {
  if (runtimeUnavailable.value) {
    return
  }

  await initialize(false)
  await loadNpxStatus(true)
  if (routeState.value.tab === 'marketplace') {
    await loadMarketplace()
  }

  if (isTauriRuntime()) {
    const { listen } = await import('@tauri-apps/api/event')
    stopSkillsEvent = await listen('skills-changed', async () => {
      await refresh(routeState.value.tab === 'marketplace')
    })
  }
})

onUnmounted(() => {
  stopSkillsEvent?.()
  stopSkillsEvent = null
})
</script>

<style scoped>
.skills-console {
  @apply flex flex-col gap-5 px-4 py-4;
}

.panel,
.console-tabs {
  @apply rounded-3xl p-4;

  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  box-shadow: var(--elevation-2);
  backdrop-filter: var(--surface-workspace-blur);
}

.skills-header-stats {
  @apply grid gap-3 md:grid-cols-3;
}

.skills-header-stat {
  @apply rounded-2xl border border-border-default/55 px-4 py-3;

  background-color: rgb(var(--color-bg-elevated-rgb) / 72%);
  backdrop-filter: blur(14px);
}

.skills-header-stat__label {
  @apply block text-[11px] uppercase tracking-[0.12em] text-text-muted;
}

.skills-header-stat__value {
  @apply mt-2 block text-lg font-semibold tracking-tight text-text-primary;
}

.console-header__eyebrow,
.panel__title,
.field__label {
  @apply text-xs font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.console-header__title {
  @apply text-3xl font-black text-text-primary;
}

.console-header__subtitle {
  @apply text-sm text-text-secondary;
}

.console-tabs {
  @apply flex flex-wrap gap-2 p-2;
}

.console-tab,
.console-button,
.platform-chip,
.tag-chip {
  @apply inline-flex items-center gap-2 rounded-xl border border-border-default/55 px-3 py-2 text-sm text-text-secondary;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  box-shadow: var(--elevation-1);
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.console-tab--active,
.platform-chip--active,
.tag-chip--active,
.console-button--primary {
  @apply text-text-primary;

  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 18%), rgb(var(--color-accent-secondary-rgb) / 10%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  box-shadow: var(--elevation-2);
}

.platform-chip--disabled {
  @apply opacity-40;
}

.console-layout {
  @apply grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)];
}

.console-sidebar {
  @apply flex flex-col gap-4;
}

.panel__header {
  @apply mb-3 flex items-center justify-between gap-2;
}

.panel__link {
  @apply text-xs text-text-muted;
}

.platform-grid,
.tag-cloud,
.activity-list {
  @apply flex flex-col gap-2;
}

.field {
  @apply flex flex-col gap-2;
}

.field__input {
  @apply w-full rounded-2xl border border-border-default/55 px-3 py-2 text-sm text-text-primary outline-none placeholder:text-text-muted/70;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  box-shadow: var(--elevation-1);
  backdrop-filter: var(--surface-status-blur);
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.field__input:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
}

.field__input:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: var(--elevation-2);
}

.field__row {
  @apply flex gap-2;
}

.platform-chip__count,
.console-tab__count {
  @apply ml-auto rounded-full px-2 py-0.5 text-xs text-text-secondary;

  background-color: rgb(var(--color-bg-base-rgb) / 68%);
}

.inventory-layout,
.marketplace-layout {
  @apply grid gap-4 xl:grid-cols-[minmax(320px,420px)_minmax(0,1fr)];
}

.inventory-list {
  @apply h-[68vh] overflow-auto rounded-2xl border border-border-default/55 p-2;

  background: rgb(var(--color-bg-base-rgb) / 42%);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 6%);
}

.inventory-row {
  @apply absolute left-0 top-0 w-full;
}

.skill-card {
  @apply flex w-full flex-col gap-3 rounded-2xl border border-border-default/55 p-4 text-left;

  background: var(--surface-card-bg);
  border-color: var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-1);
  transition:
    transform var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.skill-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--elevation-2);
}

.skill-card--active {
  background: linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 14%), rgb(var(--color-accent-secondary-rgb) / 8%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
}

.skill-card__desc {
  @apply mt-1 text-sm text-text-secondary;

  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.detail-description__toggle {
  @apply mt-1 text-xs font-medium text-accent-secondary hover:text-accent-secondary hover:underline;
}

.detail-description--collapsed {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.skill-card__head,
.marketplace-card__header,
.pagination-row,
.source-skill-card,
.installation-row {
  @apply flex items-center justify-between gap-3;
}

.skill-card__head h3,
.detail-card h3,
.source-skill-card strong,
.marketplace-card__header h3 {
  @apply text-base font-bold text-text-primary;
}

.skill-card__head p,
.detail-card p,
.source-skill-card p,
.marketplace-card__header p {
  @apply mt-1 text-sm text-text-secondary;
}

.skill-card__meta,
.detail-stack,
.manual-actions,
.source-list,
.source-skill-grid,
.marketplace-grid {
  @apply flex flex-wrap gap-2;
}

.detail-stack,
.manual-actions,
.source-list {
  @apply flex-col;
}

.detail-card,
.marketplace-card,
.source-skill-card {
  @apply rounded-2xl border border-border-default/55 p-4;

  background: var(--surface-card-bg);
  border-color: var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-1);
}

.detail-card__grid {
  @apply mt-3 grid gap-3 md:grid-cols-2 xl:grid-cols-4;
}

.detail-card__grid div {
  @apply rounded-2xl border border-border-default/45 p-3;

  background-color: rgb(var(--color-bg-base-rgb) / 55%);
}

.detail-card__grid span {
  @apply mb-1 block text-xs uppercase tracking-[0.12em] text-text-muted;
}

.detail-card__grid strong {
  @apply text-sm text-text-primary;
}

.installation-list {
  @apply flex flex-col gap-2;
}

.installation-row {
  @apply rounded-2xl border border-border-default/45 p-3;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  backdrop-filter: var(--surface-status-blur);
}

.installation-row__main {
  @apply flex min-w-0 flex-col gap-1;
}

.installation-row__main span,
.marketplace-card__header p,
.empty-state {
  @apply text-xs text-text-muted;
}

.installation-row__actions {
  @apply flex items-center gap-2;
}

.content-editor,
.content-preview {
  @apply min-h-[300px] w-full rounded-2xl border border-border-default/45 p-3 text-sm leading-6 text-text-primary;

  background-color: rgb(var(--color-bg-base-rgb) / 55%);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.content-editor {
  @apply outline-none;

  resize: vertical;
}

.content-preview {
  @apply whitespace-pre-wrap break-words;
}

.badge {
  @apply inline-flex items-center rounded-full border border-border-default/45 px-2.5 py-1 text-xs text-text-secondary;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
}

.activity-row {
  @apply flex items-start gap-3 rounded-2xl border border-white/10 bg-white/5 p-3;
}

.activity-row__status {
  @apply mt-1 h-2.5 w-2.5 rounded-full bg-white/25;
}

.activity-row__status--success {
  @apply bg-emerald-400;
}

.activity-row__status--error {
  @apply bg-rose-400;
}

.activity-row__status--pending {
  @apply bg-amber-300;
}

.activity-row__body {
  @apply flex flex-col gap-0.5 text-xs text-white/60;
}

.activity-row__body strong {
  @apply text-sm text-white/80;
}

@media (width <= 1279px) {
  .console-layout,
  .inventory-layout,
  .marketplace-layout {
    grid-template-columns: 1fr;
  }
}
</style>
