<template>
  <section class="sources-layout">
    <!-- 左侧：添加来源 -->
    <aside class="sources-sidebar">
      <section class="panel">
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
              @click="handlePickFolder"
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
    </aside>

    <!-- 右侧：来源列表 -->
    <div class="source-list">
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
            <p class="source-subtitle">
              {{ source.description || source.location }}
            </p>
          </div>
          <div class="source-actions">
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

        <div class="source-meta">
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

      <div
        v-if="sources.length === 0"
        class="empty-state"
      >
        还没有添加任何来源。使用左侧表单添加 Git 仓库或本地目录。
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import { useUIStore } from '@/stores/ui'
import type { Platform } from '@/types/skills'

const props = defineProps<{ selectedPlatforms: Platform[] }>()

const uiStore = useUIStore()
const {
  sources,
  mutationLoading,
  install,
  addGitSource,
  addLocalSourceRecord,
  syncSource,
  removeSource,
  browseFolder,
} = useUnifiedSkills()

const gitSourceUrl = ref('')
const localSourcePath = ref('')

async function handleAddGitSource() {
  try {
    await addGitSource(gitSourceUrl.value.trim())
    gitSourceUrl.value = ''
    uiStore.showSuccess('Git source added')
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleAddLocalSource() {
  try {
    await addLocalSourceRecord(localSourcePath.value.trim())
    localSourcePath.value = ''
    uiStore.showSuccess('Local source added')
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleSyncSource(sourceId: string) {
  try {
    await syncSource(sourceId)
    uiStore.showSuccess('Source synced')
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleRemoveSource(sourceId: string) {
  try {
    await removeSource(sourceId)
    uiStore.showSuccess('Source removed')
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleInstallFromSource(sourceId: string, skillId: string) {
  try {
    await install({
      sourceKind: 'source',
      sourceRef: sourceId,
      sourceSkillId: skillId,
      targetPlatforms: props.selectedPlatforms,
    })
    uiStore.showSuccess('Skill installed from source')
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handlePickFolder() {
  try {
    const path = await browseFolder()
    if (path) localSourcePath.value = path
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}
</script>

<style scoped>
.sources-layout {
  @apply grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)];
}

.sources-sidebar {
  @apply flex flex-col gap-4;
}

.source-list {
  @apply flex flex-col gap-2;
}

.source-subtitle {
  @apply text-sm text-text-secondary;
}

.source-actions {
  @apply flex items-center gap-2;
}

.source-meta {
  @apply flex flex-wrap gap-2;
}

.source-skill-grid {
  @apply flex flex-wrap gap-2;
}

.source-skill-card {
  @apply flex items-center justify-between gap-3 rounded-2xl border border-border-default/55 p-4;

  background: var(--surface-card-bg);
  border-color: var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-1);
}

.source-skill-card strong {
  @apply text-base font-bold text-text-primary;
}

.source-skill-card p {
  @apply mt-1 text-sm text-text-secondary;
}

.empty-state {
  @apply text-xs text-text-muted;
}
</style>
