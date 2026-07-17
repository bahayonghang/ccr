<template>
  <BaseModal
    :model-value="true"
    :persistent="true"
    :show-close="false"
    size="full"
    scrollable
    surface="solid"
    content-class="profiles-raw-editor-modal !rounded-md"
  >
    <template #header="{ titleId }">
      <div class="profiles-raw-editor__header">
        <div class="profiles-raw-editor__heading">
          <SIcon
            name="FileCode2"
            size="w-5 h-5"
          />
          <div>
            <h2 :id="titleId">
              {{ t('profilesRaw.title') }}
            </h2>
            <code>{{ filePath || t('settingsRaw.pathPending') }}</code>
          </div>
        </div>
        <div class="profiles-raw-editor__actions">
          <span
            v-if="dirty"
            class="profiles-raw-editor__dirty"
          >{{ t('profilesRaw.unsaved') }}</span>
          <button
            type="button"
            class="profiles-raw-editor__button"
            :disabled="loading || saving"
            @click="reload"
          >
            <SIcon
              name="RefreshCw"
              size="w-4 h-4"
            />
            {{ t('profilesRaw.reload') }}
          </button>
          <button
            type="button"
            class="profiles-raw-editor__button profiles-raw-editor__button--primary"
            :disabled="loading || saving || !dirty"
            @click="save"
          >
            <SIcon
              name="Save"
              size="w-4 h-4"
            />
            {{ saving ? t('profilesRaw.saving') : t('profilesRaw.save') }}
          </button>
          <button
            type="button"
            class="profiles-raw-editor__icon-button"
            :aria-label="t('profilesRaw.close')"
            :title="t('profilesRaw.close')"
            @click="close"
          >
            <SIcon
              name="X"
              size="w-5 h-5"
            />
          </button>
        </div>
      </div>
    </template>

    <div
      class="profiles-raw-editor__notice"
      role="note"
    >
      <SIcon
        name="ShieldAlert"
        size="w-4 h-4"
      />
      <span>{{ t('profilesRaw.plaintextNotice') }}</span>
    </div>

    <div
      v-if="conflict"
      class="profiles-raw-editor__message"
      role="alert"
    >
      <div>
        <strong>{{ t('profilesRaw.conflictTitle') }}</strong>
        <p>{{ t('profilesRaw.conflictMessage') }}</p>
      </div>
      <button
        type="button"
        @click="reload"
      >
        {{ t('profilesRaw.reload') }}
      </button>
    </div>

    <div
      v-if="unsupportedEnvironment"
      class="profiles-raw-editor__message"
      role="status"
    >
      {{ t('settingsRaw.unsupportedEnvironment') }}
    </div>

    <div
      v-if="loading"
      class="profiles-raw-editor__loading"
    >
      {{ t('profilesRaw.loading') }}
    </div>
    <CodeSourceEditor
      v-else-if="!unsupportedEnvironment"
      v-model="content"
      language="toml"
      :error-marker="errorMarker"
      @save="save"
    />
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import CodeSourceEditor, { type EditorErrorMarker } from '@/components/editor/CodeSourceEditor.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { RawFileGetResult, RawProfilesSaveResult } from '@/api/domains/configRawTypes'
import { useUIStore } from '@/stores/ui'

const props = defineProps<{
  getRaw: () => Promise<RawFileGetResult>
  saveRaw: (content: string, token: string, force?: boolean) => Promise<RawProfilesSaveResult>
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
const conflict = ref(false)
const unsupportedEnvironment = ref(false)
const errorMarker = ref<EditorErrorMarker | null>(null)
const dirty = computed(() => content.value !== baseline.value)

watch(dirty, value => emit('dirty-change', value), { immediate: true })

async function confirmDiscard() {
  if (!dirty.value) return true
  return uiStore.requestConfirm({
    title: t('profilesRaw.discardTitle'),
    message: t('profilesRaw.discardMessage'),
    confirmText: t('profilesRaw.discard'),
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
    const result = await props.getRaw()
    if (result.status === 'unsupported_environment') {
      unsupportedEnvironment.value = true
      return
    }
    unsupportedEnvironment.value = false
    content.value = result.content
    baseline.value = result.content
    token.value = result.token
    filePath.value = result.path
  } catch (error) {
    uiStore.showError(`${t('profilesRaw.loadFailed')}: ${String(error)}`)
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
    let result = await props.saveRaw(content.value, token.value, false)
    if (result.status === 'activation_conflict') {
      saving.value = false
      const confirmed = await uiStore.requestConfirm({
        title: t('profilesRaw.activationTitle'),
        message: t('profilesRaw.activationMessage', { name: result.current }),
        confirmText: t('profilesRaw.activationConfirm'),
        cancelText: t('common.cancel'),
        type: 'danger',
        surface: 'solid',
      })
      if (!confirmed) return
      saving.value = true
      result = await props.saveRaw(content.value, token.value, true)
    }

    if (result.status === 'saved') {
      token.value = result.token
      baseline.value = content.value
      uiStore.showSuccess(t('profilesRaw.saveSuccess', { count: result.profiles_count }))
      emit('saved')
    } else if (result.status === 'conflict') {
      conflict.value = true
    } else if (result.status === 'invalid') {
      errorMarker.value = {
        line: result.line ?? 1,
        column: result.column,
        message: result.message,
      }
    } else if (result.status === 'unsupported_environment') {
      unsupportedEnvironment.value = true
    }
  } catch (error) {
    uiStore.showError(`${t('profilesRaw.saveFailed')}: ${String(error)}`)
  } finally {
    saving.value = false
  }
}

async function close() {
  if (await confirmDiscard()) emit('close')
}

onMounted(load)
onBeforeRouteLeave(confirmDiscard)
onBeforeUnmount(() => emit('dirty-change', false))
</script>

<style scoped>
.profiles-raw-editor__header,
.profiles-raw-editor__heading,
.profiles-raw-editor__actions,
.profiles-raw-editor__notice,
.profiles-raw-editor__message {
  display: flex;
  align-items: center;
}

.profiles-raw-editor__header {
  justify-content: space-between;
  gap: 1rem;
}

.profiles-raw-editor__heading {
  min-width: 0;
  gap: 0.75rem;
}

.profiles-raw-editor__heading h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 1rem;
  letter-spacing: 0;
}

.profiles-raw-editor__heading code {
  display: block;
  overflow: hidden;
  margin-top: 0.25rem;
  color: var(--text-muted);
  font-size: 0.75rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.profiles-raw-editor__actions {
  flex: 0 0 auto;
  gap: 0.5rem;
}

.profiles-raw-editor__button,
.profiles-raw-editor__icon-button,
.profiles-raw-editor__message button {
  border: 1px solid var(--border-default);
  color: var(--text-secondary);
  background: var(--bg-secondary);
  cursor: pointer;
}

.profiles-raw-editor__button {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  min-height: 2.25rem;
  padding: 0 0.75rem;
  border-radius: 5px;
}

.profiles-raw-editor__button--primary {
  border-color: var(--accent-primary);
  color: var(--text-on-accent);
  background: var(--accent-primary);
}

.profiles-raw-editor__button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.profiles-raw-editor__icon-button {
  display: grid;
  width: 2.25rem;
  height: 2.25rem;
  padding: 0;
  border-radius: 5px;
  place-items: center;
}

.profiles-raw-editor__dirty {
  color: var(--color-warning);
  font-size: 0.75rem;
}

.profiles-raw-editor__notice,
.profiles-raw-editor__message {
  gap: 0.65rem;
  margin-bottom: 1rem;
  padding: 0.75rem 0.9rem;
  border: 1px solid color-mix(in srgb, var(--color-warning) 35%, var(--border-subtle));
  border-radius: 6px;
  color: var(--text-secondary);
  background: var(--bg-secondary);
  font-size: 0.8125rem;
}

.profiles-raw-editor__message {
  justify-content: space-between;
}

.profiles-raw-editor__message p {
  margin: 0.2rem 0 0;
}

.profiles-raw-editor__message button {
  border: 0;
  color: var(--accent-primary);
  background: transparent;
  font-weight: 600;
}

.profiles-raw-editor__loading {
  display: grid;
  min-height: 28rem;
  place-items: center;
  color: var(--text-muted);
}

@media (width <= 760px) {
  .profiles-raw-editor__header {
    align-items: flex-start;
    flex-direction: column;
  }

  .profiles-raw-editor__actions {
    width: 100%;
    flex-wrap: wrap;
  }
}
</style>
