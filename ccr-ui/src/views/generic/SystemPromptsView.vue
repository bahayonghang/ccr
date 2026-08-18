<template>
  <PageShell class="system-prompts-view">
    <template #header>
      <PageHeader
        :title="t('systemPrompts.title')"
        :eyebrow="t(`systemPrompts.platforms.${platform}`)"
        :description="t('systemPrompts.description')"
      >
        <template #leading>
          <SIcon
            name="ScrollText"
            size="w-8 h-8"
          />
        </template>
      </PageHeader>
    </template>

    <template #subnav>
      <ModuleSubnav :module="subnavModule" />
    </template>

    <section
      v-if="platform === 'claude'"
      class="system-prompts-view__context"
    >
      <SIcon
        name="Layers"
        size="w-5 h-5"
      />
      <div>
        <strong>{{ t('systemPrompts.claudeHierarchyTitle') }}</strong>
        <p>{{ t('systemPrompts.claudeHierarchyDescription') }}</p>
      </div>
    </section>

    <section
      v-if="platform === 'gemini'"
      class="system-prompts-view__context"
    >
      <SIcon
        name="CircleAlert"
        size="w-5 h-5"
      />
      <p>{{ t('systemPrompts.antigravityNote') }}</p>
    </section>

    <section
      v-if="unsupportedEnvironment"
      class="system-prompts-view__unsupported"
      role="status"
    >
      <SIcon
        name="MonitorOff"
        size="w-5 h-5"
      />
      {{ t('settingsRaw.unsupportedEnvironment') }}
    </section>

    <div
      v-else-if="loading"
      class="system-prompts-view__loading"
    >
      {{ t('systemPrompts.loading') }}
    </div>

    <main
      v-else
      class="system-prompts-view__workspace"
    >
      <aside class="system-prompts-view__files">
        <div class="system-prompts-view__section-heading">
          <div>
            <h2>{{ t('systemPrompts.filesTitle') }}</h2>
            <p>{{ t('systemPrompts.filesDescription') }}</p>
          </div>
          <span>{{ files.length }}</span>
        </div>

        <article
          v-for="file in files"
          :key="file.id"
          class="system-prompts-view__file"
          :class="{ 'system-prompts-view__file--active': selectedFile?.id === file.id }"
        >
          <button
            type="button"
            class="system-prompts-view__file-main"
            :disabled="busy"
            @click="selectFile(file)"
          >
            <SIcon
              :name="file.exists ? 'FileCheck2' : 'FileQuestion'"
              size="w-5 h-5"
            />
            <span>
              <strong>{{ t(file.labelKey) }}</strong>
              <code>{{ file.path }}</code>
            </span>
          </button>
          <div class="system-prompts-view__file-meta">
            <span>{{ file.exists ? t('systemPrompts.exists') : t('systemPrompts.missing') }}</span>
            <span v-if="file.size !== null">{{ t('systemPrompts.bytes', { count: file.size }) }}</span>
            <span v-if="file.mtime">{{ t('systemPrompts.modified', { time: formatTime(file.mtime) }) }}</span>
          </div>
          <button
            v-if="!file.exists"
            type="button"
            class="system-prompts-view__create"
            :disabled="busy"
            @click="createFile(file)"
          >
            <SIcon
              name="FilePlus2"
              size="w-4 h-4"
            />
            {{ creatingId === file.id ? t('systemPrompts.creating') : t('systemPrompts.create') }}
          </button>
        </article>

        <section
          v-if="platform === 'claude'"
          class="system-prompts-view__rules"
        >
          <h3>{{ t('systemPrompts.rulesTitle') }}</h3>
          <p v-if="rules.length === 0">
            {{ t('systemPrompts.rulesEmpty') }}
          </p>
          <div
            v-for="rule in rules"
            :key="rule.path"
            class="system-prompts-view__rule"
          >
            <SIcon
              name="FileText"
              size="w-4 h-4"
            />
            <span>
              <strong>{{ rule.name }}</strong>
              <code>{{ rule.path }}</code>
            </span>
          </div>
        </section>
      </aside>

      <section class="system-prompts-view__editor">
        <div
          v-if="!selectedFile"
          class="system-prompts-view__empty"
        >
          <SIcon
            name="FileText"
            size="w-8 h-8"
          />
          {{ t('systemPrompts.emptySelection') }}
        </div>

        <template v-else-if="selectedFile.exists">
          <header class="system-prompts-view__editor-header">
            <div>
              <strong>{{ t(selectedFile.labelKey) }}</strong>
              <code>{{ selectedFile.path }}</code>
            </div>
            <div class="system-prompts-view__editor-actions">
              <span v-if="dirty">{{ t('systemPrompts.unsaved') }}</span>
              <button
                type="button"
                :disabled="busy"
                @click="reloadSelected"
              >
                <SIcon
                  name="RefreshCw"
                  size="w-4 h-4"
                />
                {{ t('systemPrompts.reload') }}
              </button>
              <button
                type="button"
                class="system-prompts-view__save"
                :disabled="busy || !dirty"
                @click="save"
              >
                <SIcon
                  name="Save"
                  size="w-4 h-4"
                />
                {{ saving ? t('systemPrompts.saving') : t('systemPrompts.save') }}
              </button>
            </div>
          </header>

          <div
            v-if="conflict"
            class="system-prompts-view__message system-prompts-view__message--warning"
            role="alert"
          >
            <div>
              <strong>{{ t('systemPrompts.conflictTitle') }}</strong>
              <p>{{ t('systemPrompts.conflictMessage') }}</p>
            </div>
            <button
              type="button"
              @click="reloadSelected"
            >
              {{ t('systemPrompts.reload') }}
            </button>
          </div>

          <div
            v-if="sizeWarning"
            class="system-prompts-view__message system-prompts-view__message--warning"
            role="status"
          >
            {{ t('systemPrompts.sizeWarning') }}
          </div>

          <div
            v-if="selectedFile.limitHint"
            class="system-prompts-view__message"
            role="note"
          >
            {{ t('systemPrompts.codexLimit') }}
          </div>

          <CodeSourceEditor
            v-model="content"
            language="markdown"
            @save="save"
          />
        </template>
      </section>
    </main>
  </PageShell>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { useI18n } from 'vue-i18n'
import CodeSourceEditor from '@/components/editor/CodeSourceEditor.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { getCurrentEnvironment, systemPromptsApi } from '@/api'
import { useUIStore } from '@/stores/ui'
import type {
  SystemPromptFile,
  SystemPromptRule,
} from '@/api/domains/systemPrompts'

type SystemPromptPlatform = 'claude' | 'codex' | 'gemini' | 'opencode'

const props = defineProps<{
  platform: SystemPromptPlatform
}>()

const { t, locale } = useI18n()
const uiStore = useUIStore()
const loading = ref(true)
const saving = ref(false)
const creatingId = ref<string | null>(null)
const unsupportedEnvironment = ref(false)
const files = ref<SystemPromptFile[]>([])
const rules = ref<SystemPromptRule[]>([])
const selectedFile = ref<SystemPromptFile | null>(null)
const content = ref('')
const baseline = ref('')
const token = ref('')
const conflict = ref(false)
const sizeWarning = ref(false)

const platform = computed(() => props.platform)
const subnavModule = computed(() => props.platform === 'gemini' ? 'antigravity' : props.platform === 'claude' ? 'claude-code' : props.platform)
const dirty = computed(() => content.value !== baseline.value)
const busy = computed(() => loading.value || saving.value || creatingId.value !== null)

function formatTime(timestamp: number) {
  return new Intl.DateTimeFormat(locale.value, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(timestamp))
}

async function confirmDiscard() {
  if (!dirty.value) return true
  return uiStore.requestConfirm({
    title: t('systemPrompts.discardTitle'),
    message: t('systemPrompts.discardMessage'),
    confirmText: t('systemPrompts.discard'),
    cancelText: t('common.cancel'),
    type: 'warning',
    surface: 'solid',
  })
}

async function loadList() {
  const result = await systemPromptsApi.listSystemPrompts(props.platform)
  if (result.status === 'unsupported_environment') {
    unsupportedEnvironment.value = true
    files.value = []
    rules.value = []
    return
  }
  unsupportedEnvironment.value = false
  files.value = result.files
  rules.value = result.rules
  if (selectedFile.value) {
    selectedFile.value = files.value.find(file => file.id === selectedFile.value?.id) ?? null
  }
}

async function loadSelected(file: SystemPromptFile) {
  const result = await systemPromptsApi.getSystemPrompt(props.platform, file.id)
  if (result.status === 'unsupported_environment') {
    unsupportedEnvironment.value = true
    return
  }
  selectedFile.value = { ...file, exists: result.exists, path: result.path }
  content.value = result.content
  baseline.value = result.content
  token.value = result.token
  conflict.value = false
  sizeWarning.value = result.content.length > 64 * 1024
}

async function selectFile(file: SystemPromptFile) {
  if (!file.exists || file.id === selectedFile.value?.id) return
  if (!await confirmDiscard()) return
  try {
    await loadSelected(file)
  } catch (error) {
    uiStore.showError(`${t('systemPrompts.loadFailed')}: ${String(error)}`)
  }
}

async function reloadSelected() {
  if (!selectedFile.value || !await confirmDiscard()) return
  try {
    await loadSelected(selectedFile.value)
  } catch (error) {
    uiStore.showError(`${t('systemPrompts.loadFailed')}: ${String(error)}`)
  }
}

async function createFile(file: SystemPromptFile) {
  if (creatingId.value) return
  creatingId.value = file.id
  try {
    const result = await systemPromptsApi.createSystemPrompt(props.platform, file.id)
    if (result.status === 'unsupported_environment') {
      unsupportedEnvironment.value = true
    } else if (result.status === 'conflict') {
      await loadList()
      const current = files.value.find(item => item.id === file.id)
      if (current) await loadSelected(current)
    } else {
      uiStore.showSuccess(t('systemPrompts.createSuccess'))
      await loadList()
      const created = files.value.find(item => item.id === file.id)
      if (created) await loadSelected(created)
    }
  } catch (error) {
    uiStore.showError(`${t('systemPrompts.createFailed')}: ${String(error)}`)
  } finally {
    creatingId.value = null
  }
}

async function save() {
  if (!selectedFile.value || !dirty.value || saving.value) return
  saving.value = true
  conflict.value = false
  try {
    const result = await systemPromptsApi.saveSystemPrompt(
      props.platform,
      selectedFile.value.id,
      content.value,
      token.value,
    )
    if (result.status === 'unsupported_environment') {
      unsupportedEnvironment.value = true
    } else if (result.status === 'conflict') {
      conflict.value = true
    } else {
      token.value = result.token
      baseline.value = content.value
      sizeWarning.value = result.warning === 'size'
      uiStore.showSuccess(t('systemPrompts.saveSuccess'))
      await loadList()
    }
  } catch (error) {
    uiStore.showError(`${t('systemPrompts.saveFailed')}: ${String(error)}`)
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  try {
    const environment = await getCurrentEnvironment()
    if (environment && environment.env_type !== 'local') {
      unsupportedEnvironment.value = true
      return
    }
    await loadList()
    const firstExisting = files.value.find(file => file.exists)
    if (firstExisting) await loadSelected(firstExisting)
  } catch (error) {
    uiStore.showError(`${t('systemPrompts.loadFailed')}: ${String(error)}`)
  } finally {
    loading.value = false
  }
})

onBeforeRouteLeave(async () => confirmDiscard())
</script>

<style scoped>
.system-prompts-view__context,
.system-prompts-view__unsupported,
.system-prompts-view__editor-header,
.system-prompts-view__editor-actions,
.system-prompts-view__message,
.system-prompts-view__file-main,
.system-prompts-view__create,
.system-prompts-view__rule {
  display: flex;
  align-items: center;
}

.system-prompts-view__context p,
.system-prompts-view__message p,
.system-prompts-view__section-heading h2,
.system-prompts-view__section-heading p {
  margin: 0;
}

.system-prompts-view__section-heading p,
.system-prompts-view__context p {
  color: var(--color-text-muted);
  font-size: 0.8125rem;
}

.system-prompts-view__context,
.system-prompts-view__unsupported {
  gap: 0.75rem;
  padding: 0.8rem 0.95rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: 6px;
  color: var(--color-text-secondary);
  background: var(--color-bg-elevated);
}

.system-prompts-view__workspace {
  display: grid;
  grid-template-columns: minmax(18rem, 24rem) minmax(0, 1fr);
  min-height: 38rem;
  border-top: 1px solid var(--color-border-subtle);
}

.system-prompts-view__files {
  padding: 1rem 1rem 1rem 0;
  border-right: 1px solid var(--color-border-subtle);
}

.system-prompts-view__editor {
  min-width: 0;
  padding: 1rem 0 1rem 1rem;
}

.system-prompts-view__section-heading {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.system-prompts-view__section-heading h2 {
  color: var(--color-text-primary);
  font-size: 0.95rem;
}

.system-prompts-view__section-heading > span {
  color: var(--color-text-muted);
  font-variant-numeric: tabular-nums;
}

.system-prompts-view__file {
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--color-border-subtle);
}

.system-prompts-view__file--active {
  background: color-mix(in srgb, var(--color-accent-primary) 5%, transparent);
}

.system-prompts-view__file-main {
  width: 100%;
  gap: 0.65rem;
  padding: 0;
  border: 0;
  color: var(--color-text-primary);
  text-align: left;
  background: transparent;
  cursor: pointer;
}

.system-prompts-view__file-main > span,
.system-prompts-view__rule > span,
.system-prompts-view__editor-header > div:first-child {
  min-width: 0;
}

.system-prompts-view code {
  display: block;
  overflow: hidden;
  margin-top: 0.2rem;
  color: var(--color-text-muted);
  font-size: 0.72rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.system-prompts-view__file-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
  margin: 0.45rem 0 0 1.65rem;
  color: var(--color-text-muted);
  font-size: 0.7rem;
}

.system-prompts-view__create {
  gap: 0.35rem;
  min-height: 2rem;
  margin: 0.65rem 0 0 1.65rem;
  padding: 0 0.65rem;
  border: 1px solid var(--color-border-default);
  border-radius: 5px;
  color: var(--color-text-secondary);
  background: var(--color-bg-elevated);
  cursor: pointer;
}

.system-prompts-view__rules {
  margin-top: 1.25rem;
}

.system-prompts-view__rules h3 {
  color: var(--color-text-primary);
  font-size: 0.875rem;
}

.system-prompts-view__rules > p {
  color: var(--color-text-muted);
  font-size: 0.75rem;
}

.system-prompts-view__rule {
  gap: 0.5rem;
  padding: 0.55rem 0;
  border-bottom: 1px solid var(--color-border-subtle);
  font-size: 0.75rem;
}

.system-prompts-view__editor-header {
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.system-prompts-view__editor-actions {
  flex: 0 0 auto;
  gap: 0.5rem;
}

.system-prompts-view__editor-actions > span {
  color: var(--color-warning);
  font-size: 0.75rem;
}

.system-prompts-view__editor-actions button,
.system-prompts-view__message button {
  display: inline-flex;
  gap: 0.35rem;
  align-items: center;
  min-height: 2.25rem;
  padding: 0 0.7rem;
  border: 1px solid var(--color-border-default);
  border-radius: 5px;
  color: var(--color-text-secondary);
  background: var(--color-bg-elevated);
  cursor: pointer;
}

.system-prompts-view__editor-actions .system-prompts-view__save {
  border-color: var(--color-accent-primary);
  color: var(--color-accent-primary-contrast);
  background: var(--color-accent-primary);
}

.system-prompts-view button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.system-prompts-view__message {
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
  padding: 0.7rem 0.85rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: 6px;
  color: var(--color-text-secondary);
  background: var(--color-bg-elevated);
  font-size: 0.8rem;
}

.system-prompts-view__message--warning {
  border-color: color-mix(in srgb, var(--color-warning) 35%, var(--color-border-subtle));
}

.system-prompts-view__empty,
.system-prompts-view__loading {
  display: grid;
  min-height: 28rem;
  place-items: center;
  color: var(--color-text-muted);
}

@media (width <= 900px) {
  .system-prompts-view__workspace {
    grid-template-columns: 1fr;
  }

  .system-prompts-view__files {
    padding-right: 0;
    border-right: 0;
  }

  .system-prompts-view__editor {
    padding-left: 0;
  }

  .system-prompts-view__editor-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .system-prompts-view__editor-actions {
    width: 100%;
    flex-wrap: wrap;
  }
}
</style>
