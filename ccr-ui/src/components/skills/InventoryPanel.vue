<template>
  <section class="inventory-layout">
    <!-- 左侧：技能列表（虚拟滚动） -->
    <section class="panel">
      <div class="panel__header">
        <h2 class="panel__title">
          Inventory
        </h2>
        <span class="panel__link">{{ filteredSkills.length }} results</span>
      </div>

      <div
        ref="scrollRef"
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
                    {{ formatDescription(filteredSkills[virtualRow.index]?.description) }}
                  </p>
                </div>
                <span class="count-badge">{{ filteredSkills[virtualRow.index]?.installCount }}</span>
              </div>
              <div class="skill-card__meta">
                <span class="badge">{{ filteredSkills[virtualRow.index]?.origin }}</span>
                <span
                  v-for="inst in filteredSkills[virtualRow.index]?.installations.slice(0, 2)"
                  :key="inst.id"
                  class="badge"
                >
                  {{ inst.platformName }}
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

    <!-- 右侧：技能详情 -->
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
        <!-- 基本信息 -->
        <div class="detail-card">
          <h3>{{ selectedSkill.name }}</h3>
          <p
            class="detail-description"
            :class="{ 'detail-description--collapsed': !showFullDesc }"
            :title="selectedSkill.description ?? ''"
          >
            {{ selectedSkill.description || 'No description' }}
          </p>
          <button
            v-if="selectedSkill.description && selectedSkill.description.length > 320"
            type="button"
            class="detail-description__toggle"
            @click="showFullDesc = !showFullDesc"
          >
            {{ showFullDesc ? 'Show less' : 'Show more' }}
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
              Lifecycle
            </h2>
          </div>
          <div class="detail-card__grid">
            <div>
              <span>Targets</span>
              <strong>{{ selectedSkill.lifecycle.targetCount }}</strong>
            </div>
            <div>
              <span>Healthy</span>
              <strong>{{ selectedSkill.lifecycle.healthyTargetCount }}</strong>
            </div>
            <div>
              <span>Last sync</span>
              <strong>{{ formatTimestamp(selectedSkill.lifecycle.lastSyncedAt) }}</strong>
            </div>
            <div>
              <span>Source</span>
              <strong>{{ selectedSkill.lifecycle.sourceLabel || selectedSkill.lifecycle.sourceRef || 'N/A' }}</strong>
            </div>
          </div>
        </div>

        <!-- 安装列表 -->
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
              v-for="inst in selectedSkill.installations"
              :key="inst.id"
              class="installation-row"
            >
              <div class="installation-row__main">
                <div class="installation-row__title">
                  <strong>{{ inst.platformName }}</strong>
                  <span
                    v-if="inst.isPrimary"
                    class="badge"
                  >Primary</span>
                  <span class="badge">{{ targetStatusMap[inst.id]?.status || 'unknown' }}</span>
                </div>
                <span>{{ shortenPath(inst.installPath) }}</span>
                <span class="installation-row__meta">
                  {{ targetStatusMap[inst.id]?.syncedAt ? `Synced ${formatTimestamp(targetStatusMap[inst.id]?.syncedAt)}` : 'Not synced yet' }}
                </span>
              </div>
              <div class="installation-row__actions">
                <button
                  class="console-button"
                  @click="handleSelectInstallation(inst.id)"
                >
                  <SIcon
                    name="Eye"
                    size="w-4 h-4"
                  />
                </button>
                <button
                  class="console-button console-button--danger"
                  :disabled="mutationLoading"
                  @click="handleRemoveInstallation(inst.id)"
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

        <!-- 内容编辑 -->
        <div class="detail-card">
          <div class="panel__header">
            <h2 class="panel__title">
              Content
            </h2>
            <div class="installation-row__actions">
              <button
                class="console-button"
                @click="toggleMode"
              >
                <SIcon
                  :name="editMode ? 'Eye' : 'Pencil'"
                  size="w-4 h-4"
                />
                <span>{{ editMode ? 'Preview' : 'Edit' }}</span>
              </button>
              <button
                class="console-button console-button--primary"
                :disabled="mutationLoading || !editMode || !contentDirty || !selectedInstallation"
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

          <div class="content-layout">
            <aside class="content-files">
              <button
                v-for="file in currentFiles.filter((entry) => !entry.isDir)"
                :key="file.path"
                data-testid="content-file-row"
                class="content-file-row"
                :class="{ 'content-file-row--active': file.path === selectedFilePath }"
                @click="handleSelectFile(file.path)"
              >
                <span class="truncate">{{ file.path }}</span>
              </button>
            </aside>

            <div class="content-body">
              <textarea
                v-if="editMode"
                v-model="editBuffer"
                class="content-editor"
              />
              <pre
                v-else-if="selectedFilePath && selectedFilePreview"
                class="content-preview"
              >{{ selectedFilePreview }}</pre>
              <pre
                v-else
                class="content-preview"
              >{{ contentPreview }}</pre>
            </div>
          </div>
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
</template>

<script setup lang="ts">
import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, ref, watch } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import { useUIStore } from '@/stores/ui'
import type { Platform } from '@/types/skills'

const props = defineProps<{
  selectedPlatforms: Platform[]
}>()

const emit = defineEmits<{
  'select': [skillId: string]
  'update:mode': [mode: 'view' | 'edit']
}>()

const uiStore = useUIStore()
const {
  filteredSkills,
  selectedSkill,
  selectedInstallation,
  mutationLoading,
  selectSkill,
  ensureDetail,
  ensureContent,
  ensureFiles,
  ensureFileContent,
  saveContent,
  syncSkill,
  removeInstallation,
  removeSkillRecord,
} = useUnifiedSkills()

const scrollRef = ref<HTMLElement | null>(null)
const showFullDesc = ref(false)
const editMode = ref(false)
const editBuffer = ref('')
const currentContent = ref<Awaited<ReturnType<typeof ensureContent>> | null>(null)
const currentFiles = ref<Awaited<ReturnType<typeof ensureFiles>>>([])
const selectedFilePath = ref<string | null>(null)
const selectedFileContent = ref<Awaited<ReturnType<typeof ensureFileContent>> | null>(null)

const rowVirtualizer = useVirtualizer(computed(() => ({
  count: filteredSkills.value.length,
  getScrollElement: () => scrollRef.value,
  estimateSize: () => 140,
  overscan: 6,
})))

const contentDirty = computed(() => currentContent.value != null && editBuffer.value !== currentContent.value.raw)
const contentPreview = computed(() => {
  const raw = currentContent.value?.raw ?? ''
  return raw ? stripFrontmatter(raw) : 'No content loaded.'
})
const selectedFilePreview = computed(() => selectedFileContent.value?.content ?? '')
const targetStatusMap = computed(() => {
  const entries = selectedSkill.value?.targets ?? []
  return Object.fromEntries(entries.map((target) => [target.id, target]))
})

const measureElement = (element: unknown) => {
  rowVirtualizer.value.measureElement(element instanceof Element ? element : null)
}

function formatDescription(value?: string) {
  const description = value?.trim()
  if (!description) return 'No description'
  return description.length <= 280 ? description : description.slice(0, 280).trimEnd() + '…'
}

function stripFrontmatter(raw: string) {
  const normalized = raw.replace(/\r\n/g, '\n').trimStart()
  const lines = normalized.split('\n')
  if (lines[0]?.trim() !== '---') return raw
  for (let i = 1; i < lines.length; i += 1) {
    if (lines[i].trim() === '---') return lines.slice(i + 1).join('\n').trim()
  }
  return raw
}

function shortenPath(path: string) {
  const normalized = path.replace(/\\/g, '/')
  const parts = normalized.split('/')
  return parts.length <= 4 ? normalized : `.../${parts.slice(-4).join('/')}`
}

function formatTimestamp(value?: number) {
  if (!value) return 'Never'
  return new Date(value).toLocaleString()
}

function toggleMode() {
  editMode.value = !editMode.value
  emit('update:mode', editMode.value ? 'edit' : 'view')
}

function confirmDiscardChanges() {
  if (!contentDirty.value) {
    return true
  }

  return window.confirm('Discard unsaved skill content changes?')
}

function handleSelectSkill(skillId?: string) {
  if (!skillId) return
  if (!confirmDiscardChanges()) return
  emit('select', skillId)
  selectSkill(skillId, null)
  void ensureDetail(skillId, true)
}

function handleSelectInstallation(installationId: string) {
  if (!selectedSkill.value) return
  if (installationId === selectedInstallation.value?.id) return
  if (!confirmDiscardChanges()) return
  selectSkill(selectedSkill.value.id, installationId)
}

async function loadSelectedContent() {
  if (!selectedSkill.value || !selectedInstallation.value) {
    currentContent.value = null
    currentFiles.value = []
    selectedFilePath.value = null
    selectedFileContent.value = null
    editBuffer.value = ''
    return
  }
  currentContent.value = await ensureContent(selectedSkill.value.id, selectedInstallation.value.id, true)
  currentFiles.value = await ensureFiles(selectedSkill.value.id, selectedInstallation.value.id, true)
  selectedFilePath.value = currentFiles.value.find((file) => file.path.toLowerCase().endsWith('skill.md'))?.path ?? null
  selectedFileContent.value = selectedFilePath.value
    ? await ensureFileContent(selectedSkill.value.id, selectedFilePath.value, selectedInstallation.value.id, true)
    : null
  editBuffer.value = currentContent.value?.raw ?? ''
}

async function handleSelectFile(path: string) {
  if (!selectedSkill.value || !selectedInstallation.value) return
  selectedFilePath.value = path
  selectedFileContent.value = await ensureFileContent(selectedSkill.value.id, path, selectedInstallation.value.id, true)
}

async function handleSaveContent() {
  if (!selectedSkill.value || !selectedInstallation.value) return
  try {
    const saved = await saveContent(selectedSkill.value.id, selectedInstallation.value.id, editBuffer.value)
    currentContent.value = saved
    editBuffer.value = saved.raw
    editMode.value = false
    uiStore.showSuccess('Skill content saved')
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleSyncSelected() {
  if (!selectedSkill.value || !selectedInstallation.value || props.selectedPlatforms.length === 0) return
  try {
    await syncSkill({
      skillId: selectedSkill.value.id,
      installationId: selectedInstallation.value.id,
      targetPlatforms: props.selectedPlatforms,
    })
    uiStore.showSuccess('Skill synced to selected platforms')
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleRemoveInstallation(installationId: string) {
  if (!selectedSkill.value) return
  try {
    await removeInstallation(selectedSkill.value.id, installationId)
    uiStore.showSuccess('Installation removed')
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleRemoveSkill() {
  if (!selectedSkill.value) return
  try {
    await removeSkillRecord(selectedSkill.value.id)
    currentContent.value = null
    editBuffer.value = ''
    uiStore.showSuccess('Skill removed')
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

watch([selectedSkill, selectedInstallation], () => { void loadSelectedContent() }, { immediate: true })
watch(selectedSkill, () => { showFullDesc.value = false })
</script>

<style scoped>
.inventory-layout {
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

.skill-card__head {
  @apply flex items-center justify-between gap-3;
}

.skill-card__head h3 {
  @apply text-base font-bold text-text-primary;
}

.skill-card__desc {
  @apply mt-1 text-sm text-text-secondary;

  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.skill-card__meta {
  @apply flex flex-wrap gap-2;
}

.count-badge {
  @apply ml-auto rounded-full px-2 py-0.5 text-xs text-text-secondary;

  background-color: rgb(var(--color-bg-base-rgb) / 68%);
}

.detail-stack {
  @apply flex flex-col gap-2;
}

.detail-card {
  @apply rounded-2xl border border-border-default/55 p-4;

  background: var(--surface-card-bg);
  border-color: var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-1);
}

.detail-card h3 {
  @apply text-base font-bold text-text-primary;
}

.detail-card p {
  @apply mt-1 text-sm text-text-secondary;
}

.detail-description--collapsed {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.detail-description__toggle {
  @apply mt-1 text-xs font-medium text-accent-secondary hover:text-accent-secondary hover:underline;
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
  @apply flex items-center justify-between gap-3 rounded-2xl border border-border-default/45 p-3;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  backdrop-filter: var(--surface-status-blur);
}

.installation-row__main {
  @apply flex min-w-0 flex-col gap-1;
}

.installation-row__title {
  @apply flex items-center gap-2;
}

.installation-row__main span {
  @apply text-xs text-text-muted;
}

.installation-row__actions {
  @apply flex items-center gap-2;
}

.content-layout {
  @apply grid gap-3 xl:grid-cols-[220px_minmax(0,1fr)];
}

.content-files {
  @apply flex max-h-[320px] flex-col gap-2 overflow-auto rounded-2xl border border-border-default/45 p-2;

  background-color: rgb(var(--color-bg-base-rgb) / 45%);
}

.content-file-row {
  @apply rounded-xl px-3 py-2 text-left text-xs text-text-secondary;

  background: transparent;
}

.content-file-row--active {
  @apply text-text-primary;

  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 18%), rgb(var(--color-accent-secondary-rgb) / 10%));
}

.content-body {
  @apply min-h-[300px] w-full rounded-2xl border border-border-default/45 p-3 text-sm leading-6 text-text-primary;

  background-color: rgb(var(--color-bg-base-rgb) / 55%);
  font-family: var(--font-mono);
}

.content-editor {
  @apply outline-none;

  resize: vertical;
}

.content-preview {
  @apply whitespace-pre-wrap break-words;
}

.tag-chip {
  @apply inline-flex items-center gap-2 rounded-xl border border-border-default/55 px-3 py-2 text-sm text-text-secondary;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  box-shadow: var(--elevation-1);
}

.tag-chip--active {
  @apply text-text-primary;

  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 18%), rgb(var(--color-accent-secondary-rgb) / 10%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
}

.empty-state {
  @apply text-xs text-text-muted;
}

@media (width <= 1279px) {
  .inventory-layout {
    grid-template-columns: 1fr;
  }
}
</style>
