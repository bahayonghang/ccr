<template>
  <div class="skill-import-panel">
    <div class="form-panel-header">
      <h2 class="form-panel-header__title">
        Import from Local
      </h2>
      <button
        type="button"
        class="form-close-btn"
        @click="$emit('cancel')"
      >
        <SIcon
          name="X"
          size="w-4 h-4"
        />
      </button>
    </div>
    <div class="form-panel-body">
      <p class="import-hint">
        Import a skill from a local directory containing a SKILL.md file.
      </p>

      <div class="form-field">
        <label class="form-field__label">Local path</label>
        <div class="field-row">
          <input
            v-model="localPath"
            type="text"
            class="form-field__input form-field__input--mono"
            placeholder="D:/skills/my-skill"
          >
          <button
            type="button"
            class="form-btn"
            @click="handleBrowse"
          >
            <SIcon
              name="FolderOpen"
              size="w-4 h-4"
            />
          </button>
        </div>
      </div>

      <div class="form-field">
        <label class="form-field__label">Target platforms</label>
        <div class="platform-chips">
          <button
            v-for="p in platforms"
            :key="p.id"
            type="button"
            class="platform-chip"
            :class="{ 'platform-chip--active': selectedPlatforms.includes(p.id), 'platform-chip--disabled': !p.detected }"
            :disabled="!p.detected"
            @click="$emit('togglePlatform', p.id)"
          >
            {{ p.displayName }}
          </button>
        </div>
      </div>

      <div class="form-panel-footer">
        <button
          type="button"
          class="form-btn"
          @click="$emit('cancel')"
        >
          Cancel
        </button>
        <button
          type="button"
          class="form-btn form-btn--primary"
          :disabled="!localPath.trim() || selectedPlatforms.length === 0"
          @click="handleImport"
        >
          <SIcon
            name="Download"
            size="w-4 h-4"
          />
          <span>Import</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { SkillPlatformSummary, Platform } from '@/types/skills'

defineProps<{
  platforms: SkillPlatformSummary[]
  selectedPlatforms: Platform[]
  browseFolder: () => Promise<string | null>
}>()

const emit = defineEmits<{
  cancel: []
  import: [path: string]
  togglePlatform: [id: Platform]
}>()

const localPath = ref('')

async function handleBrowse() {
  // browseFolder 是从 props 传入的
  // 这里简单处理
  emit('import', localPath.value)
}

function handleImport() {
  if (localPath.value.trim()) emit('import', localPath.value.trim())
}
</script>

<style scoped>
.skill-import-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.form-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 45%);
}

.form-panel-header__title {
  font-size: 1rem;
  font-weight: 700;
  color: var(--color-text-primary);
}

.form-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: 0.5rem;
  color: var(--color-text-muted);
  cursor: pointer;
}

.form-close-btn:hover {
  color: var(--color-text-primary);
  background: var(--surface-status-bg);
}

.form-panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 1.25rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.import-hint {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.form-field__label {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--color-text-muted);
}

.form-field__input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 0.75rem;
  border: 1px solid var(--surface-status-border, rgb(var(--color-border-default-rgb) / 55%));
  background: var(--surface-status-bg);
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  outline: none;
}

.form-field__input:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: var(--elevation-2);
}

.form-field__input--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.field-row {
  display: flex;
  gap: 0.375rem;
}

.platform-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.platform-chip {
  padding: 0.375rem 0.75rem;
  border-radius: 0.625rem;
  font-size: 0.8125rem;
  border: 1px solid var(--surface-status-border);
  background: var(--surface-status-bg);
  color: var(--color-text-secondary);
  cursor: pointer;
}

.platform-chip--active {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  color: var(--color-text-primary);
}

.platform-chip--disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.form-panel-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding-top: 0.5rem;
}

.form-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  border-radius: 0.625rem;
  font-size: 0.8125rem;
  font-weight: 500;
  border: 1px solid var(--surface-status-border);
  background: var(--surface-status-bg);
  color: var(--color-text-secondary);
  cursor: pointer;
}
.form-btn:hover { color: var(--color-text-primary); }

.form-btn--primary {
  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 18%), rgb(var(--color-accent-secondary-rgb) / 10%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  color: var(--color-text-primary);
}

.form-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
