<template>
  <section class="marketplace-layout">
    <!-- 左侧：市场搜索+列表 -->
    <section class="panel">
      <div class="panel__header">
        <h2 class="panel__title">
          Marketplace
        </h2>
        <div class="field__row">
          <input
            v-model="searchQuery"
            class="field__input"
            type="text"
            placeholder="search skills.sh"
            @keyup.enter="handleSearch"
          >
          <button
            class="console-button"
            :disabled="marketplaceLoading"
            @click="handleSearch"
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
          :disabled="page <= 1"
          @click="$emit('update:page', page - 1)"
        >
          Prev
        </button>
        <span class="panel__link">Page {{ page }}</span>
        <button
          class="console-button"
          :disabled="marketplace.items.length < marketplace.pageSize"
          @click="$emit('update:page', page + 1)"
        >
          Next
        </button>
      </div>
    </section>

    <!-- 右侧：手动安装 -->
    <section class="panel manual-actions">
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
        :disabled="mutationLoading || !manualLocalPath.trim() || selectedPlatforms.length === 0"
        @click="installManual('local')"
      >
        <SIcon
          name="HardDrive"
          size="w-4 h-4"
        />
        <span>Install Local</span>
      </button>

      <label class="field">
        <span class="field__label">npx package</span>
        <input
          v-model="manualNpx"
          class="field__input"
          type="text"
          placeholder="owner/repo or owner/repo@skill"
        >
      </label>
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

      <p class="npx-status">
        <span
          class="npx-dot"
          :class="npxStatus?.available ? 'npx-dot--ok' : 'npx-dot--off'"
        />
        {{ npxStatus?.available ? `npx ${npxStatus.version ?? ''}` : 'npx unavailable' }}
      </p>
    </section>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import { useUIStore } from '@/stores/ui'
import type { Platform } from '@/types/skills'

const props = defineProps<{
  selectedPlatforms: Platform[]
  searchInitial?: string
  page: number
}>()

const emit = defineEmits<{
  'update:page': [page: number]
  'update:search': [query: string]
}>()

const uiStore = useUIStore()
const {
  marketplace,
  npxStatus,
  mutationLoading,
  marketplaceLoading,
  install,
  loadMarketplace,
  browseFolder,
} = useUnifiedSkills()

const searchQuery = ref(props.searchInitial ?? '')
const manualGithub = ref('')
const manualLocalPath = ref('')
const manualNpx = ref('')

async function handleSearch() {
  emit('update:search', searchQuery.value.trim())
  emit('update:page', 1)
  await loadMarketplace(true)
}

async function handleInstallMarketplace(item: { package: string; skill?: string }) {
  try {
    await install({
      sourceKind: 'marketplace',
      sourceRef: item.package,
      sourceSkillId: item.skill,
      targetPlatforms: props.selectedPlatforms,
    })
    uiStore.showSuccess('Marketplace skill installed')
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function installManual(kind: 'github' | 'local' | 'npx') {
  const sourceRef = kind === 'github' ? manualGithub.value : kind === 'local' ? manualLocalPath.value : manualNpx.value
  try {
    await install({
      sourceKind: kind,
      sourceRef: sourceRef.trim(),
      targetPlatforms: props.selectedPlatforms,
    })
    if (kind === 'github') manualGithub.value = ''
    if (kind === 'local') manualLocalPath.value = ''
    if (kind === 'npx') manualNpx.value = ''
    uiStore.showSuccess(`Installed via ${kind}`)
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handlePickFolder() {
  try {
    const path = await browseFolder()
    if (path) manualLocalPath.value = path
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}
</script>

<style scoped>
.marketplace-layout {
  @apply grid gap-4 xl:grid-cols-[minmax(320px,420px)_minmax(0,1fr)];
}

.marketplace-grid {
  @apply flex flex-wrap gap-2;
}

.marketplace-card {
  @apply rounded-2xl border border-border-default/55 p-4;

  background: var(--surface-card-bg);
  border-color: var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-1);
}

.marketplace-card__header {
  @apply flex items-center justify-between gap-3;
}

.marketplace-card__header h3 {
  @apply text-base font-bold text-text-primary;
}

.marketplace-card__header p {
  @apply mt-1 text-xs text-text-muted;
}

.manual-actions {
  @apply flex flex-col gap-2;
}

.pagination-row {
  @apply flex items-center justify-between gap-3;
}

.npx-status {
  @apply flex items-center gap-2 text-sm text-text-secondary;
}

.npx-dot {
  @apply inline-block h-2.5 w-2.5 rounded-full;
}

.npx-dot--ok {
  @apply bg-emerald-400;
}

.npx-dot--off {
  @apply bg-rose-400;
}

@media (width <= 1279px) {
  .marketplace-layout {
    grid-template-columns: 1fr;
  }
}
</style>
