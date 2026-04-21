<template>
  <section class="inventory-layout">
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
                <span
                  class="badge"
                  :class="originBadgeClass(filteredSkills[virtualRow.index])"
                >
                  {{ originBadgeText(filteredSkills[virtualRow.index]) }}
                </span>
                <span
                  v-if="sourceBadgeText(filteredSkills[virtualRow.index])"
                  class="badge badge--subtle"
                  :title="filteredSkills[virtualRow.index]?.sourceLabel || filteredSkills[virtualRow.index]?.sourceRef || ''"
                >
                  {{ sourceBadgeText(filteredSkills[virtualRow.index]) }}
                </span>
                <span
                  v-for="inst in filteredSkills[virtualRow.index]?.installations.slice(0, 2)"
                  :key="inst.id"
                  class="badge"
                >
                  {{ inst.platformName }}
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
        <div class="detail-hero">
          <div class="detail-hero__copy">
            <div class="flex flex-wrap items-center gap-2">
              <span
                class="badge"
                :class="originSummary.badgeClass"
              >{{ originSummary.label }}</span>
              <span
                class="badge badge--subtle"
                :title="sourceSummary.valueTitle"
              >{{ sourceSummary.label }}</span>
            </div>
            <h3>{{ selectedSkill.name }}</h3>
            <p class="detail-hero__subtitle">
              {{ sourceSummary.hint }}
            </p>
          </div>
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

        <div class="detail-summary-grid">
          <article class="detail-card">
            <div class="flex items-center justify-between gap-3">
              <span class="detail-card__eyebrow">Overview</span>
              <button
                v-if="selectedSkill.description && selectedSkill.description.length > 260"
                type="button"
                class="detail-description__toggle"
                @click="showFullDesc = !showFullDesc"
              >
                {{ showFullDesc ? 'Show less' : 'Show more' }}
              </button>
            </div>
            <p
              class="detail-description"
              :class="{ 'detail-description--collapsed': !showFullDesc }"
            >
              {{ selectedSkill.description || 'No structured description found. This skill was discovered from an installed directory.' }}
            </p>
            <div class="detail-chip-row">
              <span
                v-for="tag in selectedSkill.tags"
                :key="tag"
                class="tag-chip"
              >#{{ tag }}</span>
            </div>
          </article>

          <div class="flex flex-col gap-3">
            <article class="detail-card">
              <span class="detail-card__eyebrow">Tracking</span>
              <div class="detail-grid detail-grid--dual">
                <div>
                  <span>Origin</span>
                  <strong>{{ originSummary.label }}</strong>
                  <small>{{ originSummary.caption }}</small>
                </div>
                <div>
                  <span>Source</span>
                  <strong :title="sourceSummary.valueTitle">{{ sourceSummary.label }}</strong>
                  <small>{{ sourceSummary.hint }}</small>
                </div>
              </div>
            </article>
            <article class="detail-card">
              <span class="detail-card__eyebrow">Status</span>
              <div class="detail-grid detail-grid--quad">
                <div>
                  <span>Targets</span>
                  <strong>{{ selectedSkill.lifecycle.targetCount }}</strong>
                  <small>Installed copies</small>
                </div>
                <div>
                  <span>Healthy</span>
                  <strong>{{ selectedSkill.lifecycle.healthyTargetCount }}</strong>
                  <small>Ready to sync</small>
                </div>
                <div>
                  <span>Last sync</span>
                  <strong>{{ formatTimestamp(selectedSkill.lifecycle.lastSyncedAt) }}</strong>
                  <small>Best-known write time</small>
                </div>
                <div>
                  <span>Version</span>
                  <strong>{{ selectedSkill.version || 'N/A' }}</strong>
                  <small>{{ selectedSkill.author || 'Unknown author' }}</small>
                </div>
              </div>
            </article>
          </div>
        </div>

        <div class="detail-card">
          <div class="panel__header">
            <h2 class="panel__title">
              Installations
            </h2>
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
                  <span class="badge badge--subtle">{{ targetStatusMap[inst.id]?.status || 'unknown' }}</span>
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

        <div class="detail-card">
          <div class="panel__header">
            <div>
              <h2 class="panel__title">
                Content Workbench
              </h2>
              <p class="detail-card__subtitle">
                Rendered view strips frontmatter and highlights fenced code blocks.
              </p>
            </div>
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
          <div
            v-if="!editMode"
            class="content-view-switcher"
          >
            <button
              v-for="view in contentViews"
              :key="view.value"
              class="content-view-chip"
              :class="{ 'content-view-chip--active': contentView === view.value }"
              @click="contentView = view.value"
            >
              {{ view.label }}
            </button>
          </div>

          <textarea
            v-if="editMode"
            v-model="editBuffer"
            class="content-editor"
          />

          <template v-else-if="contentView === 'rendered'">
            <div
              v-if="renderedHtml"
              class="rendered-layout"
            >
              <aside
                v-if="tocEntries.length > 1"
                class="content-outline"
              >
                <span class="content-outline__title">Outline</span>
                <button
                  v-for="entry in tocEntries"
                  :key="entry.id"
                  class="content-outline__item"
                  :class="`content-outline__item--lvl-${entry.level}`"
                  @click="scrollToHeading(entry.id)"
                >
                  {{ entry.text }}
                </button>
              </aside>
              <div class="content-surface">
                <div
                  ref="markdownRef"
                  class="prose"
                  v-html="renderedHtml"
                />
              </div>
            </div>
            <pre
              v-else
              class="content-preview"
            >No markdown content loaded.</pre>
          </template>

          <template v-else-if="contentView === 'raw'">
            <div class="content-pane__toolbar">
              <span class="content-pane__label">Raw source</span>
              <button
                class="console-button"
                @click="copyToClipboard(currentContent?.raw ?? '', 'Raw content copied')"
              >
                <SIcon
                  name="Copy"
                  size="w-4 h-4"
                />
                <span>Copy</span>
              </button>
            </div>
            <pre class="content-preview">{{ currentContent?.raw || 'No content loaded.' }}</pre>
          </template>

          <template v-else>
            <div class="content-layout">
              <aside class="content-files">
                <button
                  v-for="file in fileEntries"
                  :key="file.path"
                  data-testid="content-file-row"
                  class="content-file-row"
                  :class="{ 'content-file-row--active': file.path === selectedFilePath }"
                  @click="handleSelectFile(file.path)"
                >
                  <span class="truncate">{{ file.path }}</span>
                </button>
              </aside>
              <div class="content-surface">
                <div class="content-pane__toolbar">
                  <span class="content-pane__label">{{ selectedFilePath || 'No file selected' }}</span>
                  <button
                    class="console-button"
                    :disabled="!selectedFilePreview"
                    @click="copyToClipboard(selectedFilePreview, 'File content copied')"
                  >
                    <SIcon
                      name="Copy"
                      size="w-4 h-4"
                    />
                    <span>Copy</span>
                  </button>
                </div>
                <pre class="content-preview">{{ selectedFilePreview || 'Select a file from the rail to inspect its content.' }}</pre>
              </div>
            </div>
          </template>
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
import { computed, nextTick, ref, watch } from 'vue'
import { renderMarkdown, hljs } from '@/composables/useMarkdownRender'
import SIcon from '@/components/ui/SIcon.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import { useUIStore } from '@/stores/ui'
import type { Platform, SkillRecord } from '@/types/skills'

type ContentView = 'rendered' | 'raw' | 'files'
type TocEntry = { id: string; text: string; level: number }

const props = defineProps<{ selectedPlatforms: Platform[] }>()
const emit = defineEmits<{
  'select': [skillId: string | null]
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
const markdownRef = ref<HTMLElement | null>(null)
const showFullDesc = ref(false)
const editMode = ref(false)
const contentView = ref<ContentView>('rendered')
const editBuffer = ref('')
const currentContent = ref<Awaited<ReturnType<typeof ensureContent>> | null>(null)
const currentFiles = ref<Awaited<ReturnType<typeof ensureFiles>>>([])
const selectedFilePath = ref<string | null>(null)
const selectedFileContent = ref<Awaited<ReturnType<typeof ensureFileContent>> | null>(null)
const tocEntries = ref<TocEntry[]>([])

const rowVirtualizer = useVirtualizer(computed(() => ({
  count: filteredSkills.value.length,
  getScrollElement: () => scrollRef.value,
  estimateSize: () => 140,
  overscan: 6,
})))

const contentViews = [
  { value: 'rendered' as ContentView, label: 'Rendered' },
  { value: 'raw' as ContentView, label: 'Raw' },
  { value: 'files' as ContentView, label: 'Files' },
]

const contentDirty = computed(() => currentContent.value != null && editBuffer.value !== currentContent.value.raw)
const markdownSource = computed(() => stripFrontmatter(currentContent.value?.raw ?? ''))
const renderedHtml = computed(() => renderMarkdown(markdownSource.value))
const fileEntries = computed(() => currentFiles.value.filter((entry) => !entry.isDir))
const selectedFilePreview = computed(() => selectedFileContent.value?.content ?? '')
const targetStatusMap = computed(() => Object.fromEntries((selectedSkill.value?.targets ?? []).map((target) => [target.id, target])))
const originSummary = computed(() => selectedSkill.value ? originMeta(selectedSkill.value) : { label: 'Unknown', caption: 'No skill selected', badgeClass: 'badge--unknown' })
const sourceSummary = computed(() => selectedSkill.value ? sourceMeta(selectedSkill.value) : { label: 'N/A', hint: 'Select a skill to inspect tracking metadata.', valueTitle: '' })

const measureElement = (element: unknown) => {
  rowVirtualizer.value.measureElement(element instanceof Element ? element : null)
}

function formatDescription(value?: string) {
  const description = value?.trim()
  if (!description) return 'No description'
  return description.length <= 280 ? description : `${description.slice(0, 277).trimEnd()}...`
}

function stripFrontmatter(raw: string) {
  const normalized = raw.replace(/\r\n/g, '\n').trimStart()
  const lines = normalized.split('\n')
  if (lines[0]?.trim() !== '---') return raw
  for (let index = 1; index < lines.length; index += 1) {
    if (lines[index].trim() === '---') {
      return lines.slice(index + 1).join('\n').trim()
    }
  }
  return raw
}

function shortenPath(path: string) {
  const normalized = path.replace(/\\/g, '/')
  const parts = normalized.split('/')
  return parts.length <= 4 ? normalized : `.../${parts.slice(-4).join('/')}`
}

function shortenSource(value?: string) {
  if (!value) return ''
  if (/^https?:\/\//.test(value)) {
    return value.replace(/^https?:\/\//, '').replace(/\/$/, '').slice(0, 42)
  }
  return value.length <= 42 ? value : `${value.slice(0, 39)}...`
}

function formatTimestamp(value?: number) {
  return value ? new Date(value).toLocaleString() : 'Never'
}

function originMeta(skill: SkillRecord) {
  switch (skill.origin) {
    case 'marketplace': return { label: 'Marketplace', caption: 'Installed from a tracked marketplace skill.', badgeClass: 'badge--marketplace' }
    case 'github': return { label: 'GitHub', caption: 'Installed directly from a GitHub repository.', badgeClass: 'badge--github' }
    case 'repo': return { label: 'Repo source', caption: 'Installed from a tracked source repository.', badgeClass: 'badge--repo' }
    case 'local': return { label: 'Local import', caption: skill.sourceRef ? 'Installed from a tracked local source.' : 'Imported from a local directory.', badgeClass: 'badge--local' }
    case 'npx': return { label: 'npx install', caption: 'Installed through an npx workflow.', badgeClass: 'badge--npx' }
    default: return { label: 'Legacy install', caption: 'Discovered by scanning installed skill directories.', badgeClass: 'badge--unknown' }
  }
}

function sourceMeta(skill: SkillRecord) {
  const tracked = skill.sourceLabel || skill.sourceRef
  return tracked
    ? { label: shortenSource(tracked), hint: 'Tracked source metadata is available for this installation.', valueTitle: tracked }
    : { label: 'Untracked source', hint: 'This skill was found in a platform skills directory and does not yet have tracked source metadata.', valueTitle: '' }
}

function originBadgeText(skill?: SkillRecord) {
  return skill ? originMeta(skill).label : 'Unknown'
}

function originBadgeClass(skill?: SkillRecord) {
  return skill ? originMeta(skill).badgeClass : 'badge--unknown'
}

function sourceBadgeText(skill?: SkillRecord) {
  if (!skill) return ''
  const tracked = skill.sourceLabel || skill.sourceRef
  if (!tracked) return skill.origin === 'unknown' ? 'Untracked source' : ''
  return shortenSource(tracked)
}

function toggleMode() {
  editMode.value = !editMode.value
  emit('update:mode', editMode.value ? 'edit' : 'view')
}

function confirmDiscardChanges() {
  return !contentDirty.value || window.confirm('Discard unsaved skill content changes?')
}

function handleSelectSkill(skillId?: string) {
  if (!skillId || !confirmDiscardChanges()) return
  emit('select', skillId)
  selectSkill(skillId, null)
  void ensureDetail(skillId, true)
}

function handleSelectInstallation(installationId: string) {
  if (!selectedSkill.value || installationId === selectedInstallation.value?.id || !confirmDiscardChanges()) return
  selectSkill(selectedSkill.value.id, installationId)
}

async function loadSelectedContent() {
  if (!selectedSkill.value || !selectedInstallation.value) {
    currentContent.value = null
    currentFiles.value = []
    selectedFilePath.value = null
    selectedFileContent.value = null
    editBuffer.value = ''
    tocEntries.value = []
    return
  }

  currentContent.value = await ensureContent(selectedSkill.value.id, selectedInstallation.value.id, true)
  currentFiles.value = await ensureFiles(selectedSkill.value.id, selectedInstallation.value.id, true)
  const preferred =
    fileEntries.value.find((file) => file.path.toLowerCase().endsWith('skill.md')) ??
    fileEntries.value.find((file) => file.path.toLowerCase().endsWith('.md')) ??
    fileEntries.value[0]
  selectedFilePath.value = preferred?.path ?? null
  selectedFileContent.value = selectedFilePath.value
    ? await ensureFileContent(selectedSkill.value.id, selectedFilePath.value, selectedInstallation.value.id, true)
    : null
  editBuffer.value = currentContent.value?.raw ?? ''
  editMode.value = false
  contentView.value = 'rendered'
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
    if (selectedFilePath.value?.toLowerCase().endsWith('skill.md')) {
      selectedFileContent.value = {
        skillId: saved.skillId,
        installationId: saved.installationId,
        path: selectedFilePath.value,
        content: saved.raw,
      }
    }
    editMode.value = false
    contentView.value = 'rendered'
    uiStore.showSuccess('Skill content saved')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleSyncSelected() {
  if (!selectedSkill.value || !selectedInstallation.value || props.selectedPlatforms.length === 0) return
  try {
    await syncSkill({ skillId: selectedSkill.value.id, installationId: selectedInstallation.value.id, targetPlatforms: props.selectedPlatforms })
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
    selectSkill(null, null)
    emit('select', null)
    currentContent.value = null
    currentFiles.value = []
    selectedFilePath.value = null
    selectedFileContent.value = null
    editBuffer.value = ''
    tocEntries.value = []
    uiStore.showSuccess('Skill removed')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

function slugifyHeading(text: string) {
  return text.trim().toLowerCase().replace(/[^\w\u4e00-\u9fff\s-]/g, '').replace(/\s+/g, '-').replace(/-+/g, '-')
}

function extractCodeLanguage(element: HTMLElement) {
  const match = [...element.classList].find((name) => name.startsWith('language-') || name.startsWith('lang-'))
  return match ? match.replace(/^language-/, '').replace(/^lang-/, '') || 'code' : 'code'
}

async function copyToClipboard(value: string, successMessage: string) {
  if (!value) return
  if (!navigator.clipboard?.writeText) {
    uiStore.showError('Clipboard is unavailable in this runtime')
    return
  }
  try {
    await navigator.clipboard.writeText(value)
    uiStore.showSuccess(successMessage)
  } catch {
    uiStore.showError('Clipboard is unavailable in this runtime')
  }
}

function scrollToHeading(id: string) {
  markdownRef.value?.querySelector<HTMLElement>(`[id="${id}"]`)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function enhanceRenderedContent() {
  const container = markdownRef.value
  if (!container) return

  container.querySelectorAll('.markdown-toolbar').forEach((node) => node.remove())
  const seen = new Map<string, number>()
  const nextToc: TocEntry[] = []

  container.querySelectorAll<HTMLElement>('h1, h2, h3, h4').forEach((heading) => {
    const text = heading.textContent?.trim()
    if (!text) return
    const base = slugifyHeading(text) || 'section'
    const count = (seen.get(base) ?? 0) + 1
    seen.set(base, count)
    const id = count === 1 ? base : `${base}-${count}`
    heading.id = id
    nextToc.push({ id, text, level: Number.parseInt(heading.tagName.slice(1), 10) })
  })
  tocEntries.value = nextToc

  container.querySelectorAll<HTMLElement>('pre code').forEach((block) => {
    hljs.highlightElement(block)
    const pre = block.closest('pre')
    if (!pre?.parentElement) return
    const toolbar = document.createElement('div')
    toolbar.className = 'markdown-toolbar'
    const label = document.createElement('span')
    label.className = 'markdown-toolbar__label'
    label.textContent = extractCodeLanguage(block)
    const button = document.createElement('button')
    button.type = 'button'
    button.className = 'markdown-toolbar__copy'
    button.textContent = 'Copy'
    button.addEventListener('click', () => {
      void copyToClipboard(block.innerText || block.textContent || '', 'Code block copied')
    })
    toolbar.append(label, button)
    pre.parentElement.insertBefore(toolbar, pre)
  })
}

watch([selectedSkill, selectedInstallation], () => {
  void loadSelectedContent()
}, { immediate: true })

watch(selectedSkill, () => {
  showFullDesc.value = false
})

watch([renderedHtml, editMode, contentView], async () => {
  if (editMode.value || contentView.value !== 'rendered' || !renderedHtml.value) {
    tocEntries.value = []
    return
  }
  await nextTick()
  enhanceRenderedContent()
})
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
}

.skill-card--active {
  background: linear-gradient(
    135deg,
    rgb(var(--color-accent-primary-rgb) / 14%),
    rgb(var(--color-accent-secondary-rgb) / 8%)
  );
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

.skill-card__meta,
.detail-chip-row {
  @apply flex flex-wrap gap-2;
}

.count-badge,
.badge {
  @apply inline-flex items-center rounded-full border px-2.5 py-1 text-[11px] font-medium;

  background: rgb(var(--color-bg-base-rgb) / 58%);
  border-color: rgb(var(--color-border-subtle-rgb) / 35%);
}

.count-badge,
.badge--subtle {
  @apply text-text-secondary;
}

.badge {
  @apply text-text-primary;
}

.badge--marketplace {
  background: rgb(251 191 36 / 12%);
  border-color: rgb(251 191 36 / 26%);
}

.badge--github {
  background: rgb(96 165 250 / 12%);
  border-color: rgb(96 165 250 / 24%);
}

.badge--repo {
  background: rgb(167 139 250 / 14%);
  border-color: rgb(167 139 250 / 26%);
}

.badge--local {
  background: rgb(52 211 153 / 12%);
  border-color: rgb(52 211 153 / 22%);
}

.badge--npx {
  background: rgb(244 114 182 / 12%);
  border-color: rgb(244 114 182 / 22%);
}

.badge--unknown {
  background: rgb(148 163 184 / 12%);
  border-color: rgb(148 163 184 / 22%);
}

.detail-stack {
  @apply flex flex-col gap-3;
}

.detail-hero,
.detail-card {
  @apply rounded-2xl border border-border-default/55 p-4;

  background: var(--surface-card-bg);
  border-color: var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-1);
}

.detail-hero {
  @apply flex flex-wrap items-start justify-between gap-4;

  background:
    radial-gradient(circle at top right, rgb(var(--color-accent-primary-rgb) / 16%), transparent 38%),
    radial-gradient(circle at bottom left, rgb(var(--color-accent-secondary-rgb) / 12%), transparent 34%),
    var(--surface-card-bg);
}

.detail-hero__copy {
  @apply flex min-w-0 flex-1 flex-col gap-2;
}

.detail-hero h3 {
  @apply text-2xl font-bold text-text-primary;
}

.detail-hero__subtitle,
.detail-card__subtitle {
  @apply text-sm leading-6 text-text-secondary;
}

.detail-summary-grid {
  @apply grid gap-3 xl:grid-cols-[minmax(0,1.25fr)_minmax(0,0.95fr)];
}

.detail-card__eyebrow,
.content-pane__label,
.content-outline__title {
  @apply text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.detail-description {
  @apply text-sm leading-7 text-text-primary;
}

.detail-description--collapsed {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.detail-description__toggle {
  @apply text-xs font-medium text-accent-secondary hover:underline;
}

.detail-grid {
  @apply mt-3 grid gap-3;
}

.detail-grid--dual {
  @apply md:grid-cols-2;
}

.detail-grid--quad {
  @apply md:grid-cols-2 2xl:grid-cols-4;
}

.detail-grid div {
  @apply rounded-2xl border border-border-default/45 p-3;

  background-color: rgb(var(--color-bg-base-rgb) / 55%);
}

.detail-grid span {
  @apply mb-1 block text-xs uppercase tracking-[0.12em] text-text-muted;
}

.detail-grid strong {
  @apply block text-sm text-text-primary;
}

.detail-grid small,
.installation-row__main span {
  @apply mt-1 block text-xs leading-5 text-text-muted;
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

.installation-row__title,
.installation-row__actions,
.content-view-switcher {
  @apply flex items-center gap-2;
}

.content-view-chip {
  @apply rounded-full border border-border-default/45 px-3 py-1.5 text-xs font-medium text-text-secondary;

  background-color: rgb(var(--color-bg-base-rgb) / 45%);
}

.content-view-chip--active,
.tag-chip,
.content-file-row--active {
  @apply text-text-primary;

  background: linear-gradient(
    180deg,
    rgb(var(--color-accent-primary-rgb) / 18%),
    rgb(var(--color-accent-secondary-rgb) / 10%)
  );
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
}

.rendered-layout {
  @apply grid gap-3 xl:grid-cols-[180px_minmax(0,1fr)];
}

.content-outline,
.content-files,
.content-surface {
  @apply rounded-2xl border border-border-default/45 p-3;

  background-color: rgb(var(--color-bg-base-rgb) / 48%);
}

.content-outline {
  @apply flex h-fit flex-col gap-1;
}

.content-outline__item {
  @apply rounded-xl px-2.5 py-2 text-left text-xs text-text-secondary;
}

.content-outline__item--lvl-2 {
  @apply pl-4;
}

.content-outline__item--lvl-3,
.content-outline__item--lvl-4 {
  @apply pl-6;
}

.content-layout {
  @apply grid gap-3 xl:grid-cols-[220px_minmax(0,1fr)];
}

.content-files {
  @apply flex max-h-[360px] flex-col gap-2 p-2;
}

.content-file-row {
  @apply rounded-xl px-3 py-2 text-left text-xs text-text-secondary;
}

.content-pane__toolbar {
  @apply mb-3 flex items-center justify-between gap-3;
}

.content-editor {
  @apply min-h-[360px] w-full resize-y rounded-2xl border border-border-default/45 bg-transparent p-4 text-sm leading-7 text-text-primary outline-none;

  font-family: var(--font-mono);
}

.content-preview {
  @apply whitespace-pre-wrap break-words rounded-2xl border border-border-default/35 p-4 text-sm leading-7 text-text-primary;

  background-color: rgb(var(--color-bg-overlay-rgb) / 45%);
  font-family: var(--font-mono);
}

.prose {
  @apply max-w-none text-sm leading-7 text-text-primary;
}

.prose :deep(h1),
.prose :deep(h2),
.prose :deep(h3),
.prose :deep(h4) {
  @apply scroll-mt-20 font-semibold text-text-primary;
}

.prose :deep(h1) {
  @apply mt-6 text-2xl;
}

.prose :deep(h2) {
  @apply mt-5 text-xl;
}

.prose :deep(h3) {
  @apply mt-4 text-lg;
}

.prose :deep(p),
.prose :deep(ul),
.prose :deep(ol) {
  @apply my-3;
}

.prose :deep(ul),
.prose :deep(ol) {
  @apply pl-6;
}

.prose :deep(code) {
  @apply rounded-md px-1.5 py-0.5 text-xs text-accent-primary;

  background-color: rgb(var(--color-bg-overlay-rgb) / 65%);
}

.prose :deep(pre) {
  @apply my-0 overflow-auto rounded-b-2xl px-4 py-4 text-xs leading-6;

  background-color: rgb(var(--color-bg-overlay-rgb) / 72%);
}

.prose :deep(pre code) {
  @apply bg-transparent p-0 text-text-primary;
}

.prose :deep(blockquote) {
  @apply my-4 border-l-2 pl-4 italic text-text-secondary;

  border-color: rgb(var(--color-accent-primary-rgb) / 25%);
}

.prose :deep(a) {
  @apply text-accent-primary hover:underline;
}

.prose :deep(table) {
  @apply my-4 w-full text-left text-xs;

  border-collapse: collapse;
}

.prose :deep(th),
.prose :deep(td) {
  @apply border border-border-default/30 px-3 py-2;
}

.markdown-toolbar {
  @apply flex items-center justify-between rounded-t-2xl border border-b-0 border-border-default/35 px-4 py-2 text-[11px] font-medium text-text-secondary;

  background-color: rgb(var(--color-bg-overlay-rgb) / 55%);
}

.markdown-toolbar__label {
  @apply uppercase tracking-[0.16em];
}

.markdown-toolbar__copy {
  @apply rounded-full border border-border-default/35 px-3 py-1 text-[11px] text-text-primary;

  background-color: rgb(var(--color-bg-base-rgb) / 45%);
}

.tag-chip {
  @apply inline-flex items-center rounded-xl border px-3 py-1.5 text-xs;
}

.empty-state {
  @apply text-xs text-text-muted;
}

@media (width <= 1279px) {
  .inventory-layout,
  .detail-summary-grid,
  .rendered-layout,
  .content-layout {
    grid-template-columns: 1fr;
  }
}
</style>
