<template>
  <BaseModal
    :model-value="modelValue"
    :title="editingName ? t('codex.agents.editAgent') : t('codex.agents.addAgent')"
    size="xl"
    surface="solid"
    content-class="codex-agent-editor-modal max-h-[calc(100vh-3rem)]"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="space-y-5">
      <div
        v-if="parseError"
        class="rounded-2xl border border-amber-400/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-100"
      >
        <div class="font-semibold">
          {{ t('codex.agents.diagnosticsTitle') }}
        </div>
        <div class="mt-1 whitespace-pre-wrap break-words">
          {{ parseError }}
        </div>
      </div>

      <div class="flex flex-wrap gap-2">
        <button
          type="button"
          class="rounded-xl border px-3 py-2 text-sm transition-colors"
          :class="useRawToml ? panelButtonClass.inactive : panelButtonClass.active"
          @click="useRawToml = false"
        >
          {{ t('codex.agents.structuredEditor') }}
        </button>
        <button
          type="button"
          class="rounded-xl border px-3 py-2 text-sm transition-colors"
          :class="useRawToml ? panelButtonClass.active : panelButtonClass.inactive"
          @click="useRawToml = true"
        >
          {{ t('codex.agents.rawEditor') }}
        </button>
      </div>

      <div
        v-if="!useRawToml"
        class="space-y-4"
      >
        <div class="grid gap-4 md:grid-cols-2">
          <label class="space-y-2 text-sm text-text-secondary">
            <span class="font-semibold text-text-primary">{{ t('codex.agents.form.name') }} *</span>
            <input
              v-model="form.name"
              type="text"
              class="codex-agent-input"
            >
          </label>

          <label class="space-y-2 text-sm text-text-secondary">
            <span class="font-semibold text-text-primary">{{ t('codex.agents.form.model') }}</span>
            <input
              v-if="availableModels.length === 0"
              v-model="form.model"
              type="text"
              class="codex-agent-input"
              placeholder="gpt-5.4"
            >
            <select
              v-else
              v-model="form.model"
              class="codex-agent-input"
            >
              <option value="">
                {{ t('codex.agents.followParentSession') }}
              </option>
              <option
                v-for="model in availableModels"
                :key="model"
                :value="model"
              >
                {{ model }}
              </option>
            </select>
          </label>
        </div>

        <label class="space-y-2 text-sm text-text-secondary">
          <span class="font-semibold text-text-primary">{{ t('codex.agents.form.description') }} *</span>
          <textarea
            v-model="form.description"
            rows="2"
            class="codex-agent-textarea"
          />
        </label>

        <label class="space-y-2 text-sm text-text-secondary">
          <span class="font-semibold text-text-primary">{{ t('codex.agents.form.developerInstructions') }} *</span>
          <textarea
            v-model="form.developerInstructions"
            rows="8"
            class="codex-agent-textarea font-mono text-[13px]"
          />
        </label>

        <div class="grid gap-4 md:grid-cols-3">
          <label class="space-y-2 text-sm text-text-secondary">
            <span class="font-semibold text-text-primary">{{ t('codex.agents.form.reasoningEffort') }}</span>
            <select
              v-model="form.modelReasoningEffort"
              class="codex-agent-input"
            >
              <option value="">
                {{ t('codex.agents.followParentSession') }}
              </option>
              <option value="low">
                low
              </option>
              <option value="medium">
                medium
              </option>
              <option value="high">
                high
              </option>
              <option value="xhigh">
                xhigh
              </option>
            </select>
          </label>

          <label class="space-y-2 text-sm text-text-secondary">
            <span class="font-semibold text-text-primary">{{ t('codex.agents.form.sandboxMode') }}</span>
            <input
              v-model="form.sandboxMode"
              type="text"
              class="codex-agent-input"
              placeholder="workspace-write"
            >
          </label>

          <label class="space-y-2 text-sm text-text-secondary">
            <span class="font-semibold text-text-primary">{{ t('codex.agents.form.nicknameCandidates') }}</span>
            <input
              v-model="nicknameCandidatesText"
              type="text"
              class="codex-agent-input"
              placeholder="Atlas, Delta, Echo"
            >
          </label>
        </div>

        <div>
          <label class="space-y-2 text-sm text-text-secondary">
            <span class="font-semibold text-text-primary">{{ t('codex.agents.mcpServersLabel') }}</span>
            <textarea
              v-model="mcpServersJson"
              rows="6"
              class="codex-agent-textarea font-mono text-[13px]"
              placeholder="{&quot;docs&quot;:{&quot;url&quot;:&quot;https://developers.openai.com/mcp&quot;}}"
            />
          </label>
        </div>
      </div>

      <div
        v-else
        class="space-y-3"
      >
        <div class="rounded-2xl border border-border-default/50 bg-bg-elevated px-4 py-3 text-sm text-text-secondary">
          {{ t('codex.agents.rawEditorDescription') }}
        </div>
        <textarea
          v-model="rawToml"
          rows="20"
          class="codex-agent-textarea font-mono text-[13px]"
        />
      </div>

      <div
        v-if="errorMessage"
        class="rounded-2xl border border-rose-400/30 bg-rose-500/10 px-4 py-3 text-sm text-rose-100"
      >
        {{ errorMessage }}
      </div>
    </div>

    <template #footer>
      <button
        type="button"
        class="rounded-xl border border-border-default/70 px-4 py-2 text-sm text-text-secondary transition-colors hover:bg-bg-surface"
        @click="emit('update:modelValue', false)"
      >
        {{ t('common.cancel') }}
      </button>
      <button
        type="button"
        class="rounded-xl bg-accent-primary px-4 py-2 text-sm font-semibold text-[color:var(--color-accent-primary-contrast)] shadow-lg shadow-accent-primary/20 transition-transform hover:scale-[1.02]"
        @click="handleSubmit"
      >
        {{ editingName ? t('common.save') : t('codex.agents.editorCreate') }}
      </button>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { getErrorMessage } from '@/utils/errorHandler'
import BaseModal from '@/components/common/BaseModal.vue'
import type { CodexAgent, CodexAgentRequest } from '@/types'

const props = defineProps<{
  modelValue: boolean
  agent?: CodexAgent | null
  availableModels?: string[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  save: [payload: CodexAgentRequest]
}>()
const { t } = useI18n()

const form = reactive<CodexAgentRequest>({
  name: '',
  description: '',
  developerInstructions: '',
  model: '',
  modelReasoningEffort: '',
  sandboxMode: '',
})

const nicknameCandidatesText = ref('')
const mcpServersJson = ref('')
const rawToml = ref('')
const useRawToml = ref(false)
const errorMessage = ref('')

const editingName = computed(() => props.agent?.name ?? '')
const parseError = computed(() => props.agent?.parseError ?? null)
const availableModels = computed(() => props.availableModels ?? [])

const panelButtonClass = {
  active: 'border-accent-primary/40 bg-accent-primary/10 text-accent-primary',
  inactive: 'border-border-default/60 bg-bg-elevated text-text-secondary hover:bg-bg-surface',
}

function resetForm() {
  form.name = props.agent?.name ?? ''
  form.description = props.agent?.description ?? ''
  form.developerInstructions = props.agent?.developerInstructions ?? ''
  form.model = props.agent?.model ?? ''
  form.modelReasoningEffort = props.agent?.modelReasoningEffort ?? ''
  form.sandboxMode = props.agent?.sandboxMode ?? ''
  nicknameCandidatesText.value = (props.agent?.nicknameCandidates ?? []).join(', ')
  mcpServersJson.value = props.agent?.mcpServers ? JSON.stringify(props.agent.mcpServers, null, 2) : ''
  rawToml.value = props.agent?.rawToml ?? ''
  useRawToml.value = !!props.agent?.parseError
  errorMessage.value = ''
}

watch(
  () => [props.modelValue, props.agent] as const,
  () => {
    if (props.modelValue) {
      resetForm()
    }
  },
  { immediate: true }
)

function parseOptionalJson(label: string, value: string) {
  if (!value.trim()) {
    return undefined
  }

  try {
    return JSON.parse(value)
  } catch (error) {
    throw new Error(t('codex.agents.invalidJsonWithLabel', {
      label,
      error: getErrorMessage(error),
    }))
  }
}

function handleSubmit() {
  errorMessage.value = ''

  try {
    if (useRawToml.value) {
      if (!rawToml.value.trim()) {
        throw new Error(t('codex.agents.rawTomlEmpty'))
      }

      emit('save', {
        rawToml: rawToml.value,
      })
      return
    }

    if (!form.name?.trim() || !form.description?.trim() || !form.developerInstructions?.trim()) {
      throw new Error(t('codex.agents.validation.required'))
    }

    emit('save', {
      name: form.name.trim(),
      description: form.description?.trim() || null,
      developerInstructions: form.developerInstructions?.trim() || null,
      model: form.model?.trim() || null,
      modelReasoningEffort: form.modelReasoningEffort?.trim() || null,
      sandboxMode: form.sandboxMode?.trim() || null,
      nicknameCandidates: nicknameCandidatesText.value
        .split(',')
        .map(item => item.trim())
        .filter(Boolean),
      mcpServers: parseOptionalJson('mcp_servers JSON', mcpServersJson.value) ?? null,
    })
  } catch (error) {
    errorMessage.value = getErrorMessage(error)
  }
}
</script>

<style>
.codex-agent-editor-modal {
  --agent-shell-bg: var(--surface-modal-bg);
  --agent-shell-border: var(--surface-modal-border);
  --agent-shell-shadow: var(--surface-modal-shadow);
  --agent-panel-bg: var(--color-bg-surface);
  --agent-panel-bg-hover: var(--color-bg-overlay);
  --agent-panel-muted-bg: var(--color-bg-elevated);
  --agent-input-bg: var(--color-bg-surface);
  --agent-input-bg-hover: var(--color-bg-overlay);
  --agent-input-bg-focus: var(--color-bg-surface);
  --agent-input-border: var(--color-border-subtle);
  --agent-input-border-strong: rgb(var(--color-accent-primary-rgb) / 34%);
  --agent-hairline: var(--color-border-default);
  --agent-hairline-soft: var(--color-border-subtle);
  --agent-ink: var(--color-text-primary);
  --agent-ink-muted: var(--color-text-secondary);
  --agent-ink-soft: var(--color-text-muted);
  --agent-placeholder: var(--color-text-ghost);
  --agent-ring: 0 0 0 3px rgb(var(--color-accent-primary-rgb) / 14%);

  background: var(--agent-shell-bg) !important;
  border: 1px solid var(--agent-shell-border) !important;
  box-shadow: var(--agent-shell-shadow) !important;
  color: var(--agent-ink);
}

.codex-agent-editor-modal .text-text-primary,
.codex-agent-editor-modal h2 {
  color: var(--agent-ink) !important;
}

.codex-agent-editor-modal .text-text-secondary {
  color: var(--agent-ink-muted) !important;
}

.codex-agent-editor-modal .text-text-muted {
  color: var(--agent-ink-soft) !important;
}

.codex-agent-editor-modal [class*='border-t'] {
  border-color: var(--agent-hairline-soft) !important;
}

.codex-agent-editor-modal button {
  color: inherit;
}

.codex-agent-editor-modal button:not(.bg-accent-primary) {
  border-color: var(--agent-hairline-soft);
  background: var(--agent-panel-bg);
  color: var(--agent-ink-muted);
}

.codex-agent-editor-modal button:not(.bg-accent-primary):hover {
  border-color: var(--agent-hairline);
  background: var(--agent-panel-bg-hover);
  color: var(--agent-ink);
}

.codex-agent-editor-modal .bg-accent-primary {
  box-shadow: none;
}

.codex-agent-editor-modal .rounded-2xl.border.border-border-default\/50 {
  background: var(--agent-panel-muted-bg) !important;
}

.codex-agent-input {
  width: 100%;
  min-height: 2.75rem;
  border-radius: 0.875rem;
  border: 1px solid var(--agent-input-border);
  background: var(--agent-input-bg);
  padding: 0.75rem 0.875rem;
  color: var(--agent-ink);
  transition:
    background-color 150ms ease,
    border-color 150ms ease,
    box-shadow 150ms ease;
}

.codex-agent-input:hover,
.codex-agent-textarea:hover {
  background: var(--agent-input-bg-hover);
}

.codex-agent-input::placeholder,
.codex-agent-textarea::placeholder {
  color: var(--agent-placeholder);
}

.codex-agent-input option {
  background: var(--agent-panel-bg);
  color: var(--agent-ink);
}

.codex-agent-input:focus,
.codex-agent-textarea:focus {
  outline: none;
  border-color: var(--agent-input-border-strong);
  background: var(--agent-input-bg-focus);
  box-shadow: var(--agent-ring);
}

.codex-agent-textarea {
  width: 100%;
  border-radius: 1rem;
  border: 1px solid var(--agent-input-border);
  background: var(--agent-input-bg);
  padding: 0.875rem 1rem;
  color: var(--agent-ink);
  resize: vertical;
}
</style>
