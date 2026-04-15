<template>
  <div class="skill-form-panel">
    <div class="form-panel-header">
      <h2 class="form-panel-header__title">
        Create Custom Skill
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

    <form
      class="form-panel-body"
      @submit.prevent="handleCreate"
    >
      <div class="form-field">
        <label class="form-field__label">Skill name <span class="text-red-400">*</span></label>
        <input
          v-model="name"
          type="text"
          class="form-field__input"
          placeholder="my-skill"
        >
      </div>

      <div class="form-field">
        <label class="form-field__label">Description</label>
        <textarea
          v-model="description"
          class="form-field__textarea"
          rows="3"
          placeholder="What does this skill do?"
        />
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
            @click="togglePlatform(p.id)"
          >
            {{ p.displayName }}
          </button>
        </div>
      </div>

      <div class="form-field">
        <label class="form-field__label">Skill content (SKILL.md)</label>
        <textarea
          v-model="content"
          class="form-field__textarea form-field__textarea--mono"
          rows="10"
          placeholder="---&#10;name: my-skill&#10;description: ...&#10;---&#10;&#10;Skill instructions here..."
        />
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
          type="submit"
          class="form-btn form-btn--primary"
          :disabled="!name.trim()"
        >
          <SIcon
            name="Check"
            size="w-4 h-4"
          />
          <span>Create</span>
        </button>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { SkillPlatformSummary, Platform } from '@/types/skills'

const props = defineProps<{
  platforms: SkillPlatformSummary[]
  selectedPlatforms: Platform[]
}>()

const emit = defineEmits<{
  cancel: []
  create: [data: { name: string; description: string; content: string; platforms: Platform[] }]
}>()

const name = ref('')
const description = ref('')
const content = ref('')

function togglePlatform(_id: Platform) {
  // 在父组件管理 selectedPlatforms，这里只是触发
  emit('create', { name: name.value, description: description.value, content: content.value, platforms: props.selectedPlatforms })
}

function handleCreate() {
  emit('create', { name: name.value, description: description.value, content: content.value, platforms: props.selectedPlatforms })
}
</script>

<style scoped>
.skill-form-panel {
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
  transition: all var(--motion-subtle-duration) var(--motion-subtle-ease);
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

.form-field__input, .form-field__textarea {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 0.75rem;
  border: 1px solid var(--surface-status-border, rgb(var(--color-border-default-rgb) / 55%));
  background: var(--surface-status-bg);
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  outline: none;
  resize: vertical;
  transition: border-color var(--motion-subtle-duration) var(--motion-subtle-ease), box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease);

}

.form-field__input:focus, .form-field__textarea:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: var(--elevation-2);
}

.form-field__textarea--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
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
  transition: all var(--motion-subtle-duration) var(--motion-subtle-ease);

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
  transition: all var(--motion-subtle-duration) var(--motion-subtle-ease);

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
