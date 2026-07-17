<template>
  <section class="config-source-panel">
    <header class="config-source-panel__header">
      <div class="config-source-panel__path">
        <span>{{ t('settingsRaw.filePath') }}</span>
        <code>{{ filePath || t('settingsRaw.pathPending') }}</code>
      </div>
      <div class="config-source-panel__actions">
        <span
          v-if="dirty"
          class="config-source-panel__dirty"
        >{{ t('settingsRaw.unsaved') }}</span>
        <button
          type="button"
          class="config-source-panel__button"
          :disabled="loading || saving"
          @click="reload"
        >
          <SIcon
            name="RefreshCw"
            size="w-4 h-4"
          />
          {{ t('settingsRaw.reload') }}
        </button>
        <button
          type="button"
          class="config-source-panel__button config-source-panel__button--primary"
          :disabled="loading || saving || !dirty"
          @click="save"
        >
          <SIcon
            name="Save"
            size="w-4 h-4"
          />
          {{ saving ? t('settingsRaw.saving') : t('settingsRaw.save') }}
        </button>
      </div>
    </header>

    <div
      class="config-source-panel__notice"
      role="note"
    >
      <SIcon
        name="ShieldAlert"
        size="w-4 h-4"
      />
      <span>{{ t('settingsRaw.plaintextNotice') }}</span>
    </div>

    <div
      v-if="conflict"
      class="config-source-panel__message config-source-panel__message--warning"
      role="alert"
    >
      <div>
        <strong>{{ t('settingsRaw.conflictTitle') }}</strong>
        <p>{{ t('settingsRaw.conflictMessage') }}</p>
      </div>
      <button
        type="button"
        class="config-source-panel__text-button"
        @click="reload"
      >
        {{ t('settingsRaw.reload') }}
      </button>
    </div>

    <div
      v-if="unsupportedEnvironment"
      class="config-source-panel__message config-source-panel__message--warning"
      role="status"
    >
      {{ t('settingsRaw.unsupportedEnvironment') }}
    </div>

    <div
      v-if="loading"
      class="config-source-panel__loading"
    >
      {{ t('settingsRaw.loading') }}
    </div>
    <CodeSourceEditor
      v-else-if="!unsupportedEnvironment"
      v-model="content"
      :language="language"
      :error-marker="errorMarker"
      @save="save"
    />

    <section
      class="config-source-panel__layers"
      aria-labelledby="config-layers-title"
    >
      <div class="config-source-panel__layers-heading">
        <div>
          <h3 id="config-layers-title">
            {{ t('settingsRaw.layersTitle') }}
          </h3>
          <p>{{ t('settingsRaw.layersDescription') }}</p>
        </div>
        <span>{{ layers.length }}</span>
      </div>
      <div class="config-source-panel__layer-list">
        <div
          v-for="layer in layers"
          :key="`${layer.id}-${layer.path ?? layer.label}`"
          class="config-source-panel__layer"
        >
          <SIcon
            :name="layer.exists ? 'FileCheck2' : 'FileQuestion'"
            size="w-4 h-4"
          />
          <div>
            <strong>{{ layer.label }}</strong>
            <code v-if="layer.path">{{ layer.path }}</code>
            <span v-else>{{ t('settingsRaw.projectContextRequired') }}</span>
          </div>
          <span class="config-source-panel__layer-state">
            {{ layer.editable ? t('settingsRaw.editable') : t('settingsRaw.readOnly') }}
          </span>
        </div>
      </div>
    </section>
  </section>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { useI18n } from 'vue-i18n'
import CodeSourceEditor, { type EditorErrorMarker } from './CodeSourceEditor.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useUIStore } from '@/stores/ui'
import type {
  ConfigLayer,
  ConfigLayersResult,
  RawFileGetResult,
  RawFileSaveResult,
} from '@/api/domains/configRawTypes'

const props = defineProps<{
  language: 'json' | 'toml'
  getRaw: () => Promise<RawFileGetResult>
  saveRaw: (content: string, token: string) => Promise<RawFileSaveResult>
  listLayers: () => Promise<ConfigLayersResult>
}>()

const emit = defineEmits<{
  saved: []
  close: []
  'dirty-change': [dirty: boolean]
}>()

const { t } = useI18n()
const uiStore = useUIStore()
const loading = ref(true)
const saving = ref(false)
const content = ref('')
const baseline = ref('')
const token = ref('')
const filePath = ref('')
const layers = ref<ConfigLayer[]>([])
const conflict = ref(false)
const unsupportedEnvironment = ref(false)
const errorMarker = ref<EditorErrorMarker | null>(null)
const dirty = computed(() => content.value !== baseline.value)

watch(dirty, value => emit('dirty-change', value), { immediate: true })

async function confirmDiscard() {
  if (!dirty.value) return true
  return uiStore.requestConfirm({
    title: t('settingsRaw.discardTitle'),
    message: t('settingsRaw.discardMessage'),
    confirmText: t('settingsRaw.discard'),
    cancelText: t('common.cancel'),
    type: 'warning',
    surface: 'solid',
  })
}

async function load() {
  loading.value = true
  conflict.value = false
  errorMarker.value = null
  try {
    const [raw, layerResult] = await Promise.all([props.getRaw(), props.listLayers()])
    if (raw.status === 'unsupported_environment') {
      unsupportedEnvironment.value = true
      return
    }
    unsupportedEnvironment.value = false
    content.value = raw.content
    baseline.value = raw.content
    token.value = raw.token
    filePath.value = raw.path
    layers.value = 'layers' in layerResult ? layerResult.layers : []
  } catch (error) {
    uiStore.showError(`${t('settingsRaw.loadFailed')}: ${String(error)}`)
  } finally {
    loading.value = false
  }
}

async function reload() {
  if (!await confirmDiscard()) return
  await load()
}

async function save() {
  if (!dirty.value || saving.value) return
  saving.value = true
  conflict.value = false
  errorMarker.value = null
  try {
    const result = await props.saveRaw(content.value, token.value)
    if (result.status === 'saved') {
      token.value = result.token
      baseline.value = content.value
      uiStore.showSuccess(t('settingsRaw.saveSuccess'))
      emit('saved')
    } else if (result.status === 'conflict') {
      conflict.value = true
    } else if (result.status === 'invalid') {
      errorMarker.value = {
        line: result.line ?? 1,
        column: result.column,
        message: result.message,
      }
    } else {
      unsupportedEnvironment.value = true
    }
  } catch (error) {
    uiStore.showError(`${t('settingsRaw.saveFailed')}: ${String(error)}`)
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  const confirmed = await uiStore.requestConfirm({
    title: t('settingsRaw.warningTitle'),
    message: t('settingsRaw.warningMessage'),
    confirmText: t('settingsRaw.continue'),
    cancelText: t('common.cancel'),
    type: 'warning',
    surface: 'solid',
  })
  if (!confirmed) {
    emit('close')
    return
  }
  await load()
})

onBeforeRouteLeave(async () => confirmDiscard())
onBeforeUnmount(() => emit('dirty-change', false))
</script>

<style scoped>
.config-source-panel {
  display: grid;
  gap: 1rem;
}

.config-source-panel__header,
.config-source-panel__actions,
.config-source-panel__notice,
.config-source-panel__message,
.config-source-panel__layers-heading,
.config-source-panel__layer {
  display: flex;
  align-items: center;
}

.config-source-panel__header {
  justify-content: space-between;
  gap: 1rem;
}

.config-source-panel__path {
  min-width: 0;
}

.config-source-panel__path span,
.config-source-panel__layers-heading p,
.config-source-panel__layer span {
  color: var(--text-muted);
  font-size: 0.75rem;
}

.config-source-panel__path code,
.config-source-panel__layer code {
  display: block;
  overflow: hidden;
  margin-top: 0.2rem;
  color: var(--text-secondary);
  font-size: 0.75rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.config-source-panel__actions {
  flex: 0 0 auto;
  gap: 0.5rem;
}

.config-source-panel__button,
.config-source-panel__text-button {
  border: 0;
  color: var(--text-secondary);
  background: transparent;
  cursor: pointer;
}

.config-source-panel__button {
  display: inline-flex;
  gap: 0.4rem;
  align-items: center;
  min-height: 2.25rem;
  padding: 0 0.75rem;
  border: 1px solid var(--border-default);
  border-radius: 5px;
  background: var(--bg-secondary);
}

.config-source-panel__button--primary {
  border-color: var(--accent-primary);
  color: var(--text-on-accent);
  background: var(--accent-primary);
}

.config-source-panel__button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.config-source-panel__dirty {
  color: var(--color-warning);
  font-size: 0.75rem;
}

.config-source-panel__notice,
.config-source-panel__message {
  gap: 0.65rem;
  padding: 0.75rem 0.9rem;
  border: 1px solid var(--border-subtle);
  border-radius: 6px;
  color: var(--text-secondary);
  background: var(--bg-secondary);
  font-size: 0.8125rem;
}

.config-source-panel__message {
  justify-content: space-between;
}

.config-source-panel__message p {
  margin: 0.2rem 0 0;
}

.config-source-panel__message--warning {
  border-color: color-mix(in srgb, var(--color-warning) 35%, var(--border-subtle));
}

.config-source-panel__text-button {
  color: var(--accent-primary);
  font-weight: 600;
}

.config-source-panel__loading {
  display: grid;
  min-height: 28rem;
  place-items: center;
  border: 1px solid var(--border-default);
  border-radius: 6px;
  color: var(--text-muted);
}

.config-source-panel__layers {
  padding-top: 0.25rem;
}

.config-source-panel__layers-heading {
  justify-content: space-between;
  margin-bottom: 0.65rem;
}

.config-source-panel__layers-heading h3,
.config-source-panel__layers-heading p {
  margin: 0;
}

.config-source-panel__layers-heading h3 {
  color: var(--text-primary);
  font-size: 0.95rem;
}

.config-source-panel__layer-list {
  border-top: 1px solid var(--border-subtle);
}

.config-source-panel__layer {
  display: grid;
  grid-template-columns: 1rem minmax(0, 1fr) auto;
  gap: 0.65rem;
  padding: 0.7rem 0;
  border-bottom: 1px solid var(--border-subtle);
}

.config-source-panel__layer strong {
  color: var(--text-primary);
  font-size: 0.8125rem;
}

.config-source-panel__layer-state {
  white-space: nowrap;
}

@media (width <= 720px) {
  .config-source-panel__header {
    align-items: flex-start;
    flex-direction: column;
  }

  .config-source-panel__actions {
    width: 100%;
    flex-wrap: wrap;
  }
}
</style>
