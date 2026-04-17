<template>
  <div class="skills-manager-view">
    <!-- P1-6 接入：Trash 抽屉入口 + 面板 -->
    <div class="skills-manager-view__trash-bar">
      <button
        type="button"
        class="skills-manager-view__trash-toggle"
        :aria-expanded="showTrash"
        @click="showTrash = !showTrash"
      >
        {{ showTrash ? '▼' : '▶' }} Trash
      </button>
    </div>
    <SkillTrashPanel
      v-if="showTrash"
      class="skills-manager-view__trash-panel"
      @restored="handleRestored"
      @purged="handlePurged"
    />

    <MasterDetailLayout list-width="22rem">
      <template #list>
        <SkillListPanel
          :groups="filteredGroups"
          :search-query="searchQuery"
          :selected-keys="effectiveSelectedKeys"
          :is-multi-select-mode="isMultiSelectMode"
          :loading="inventoryLoading ?? false"
          :stats="stats"
          @update:search-query="searchQuery = $event"
          @select="selectSkill"
          @create="openCreate"
          @import="openImport"
          @import-github="openImportGithub"
          @refresh="refresh()"
          @toggle-multi-select="toggleMultiSelect"
          @bulk-delete="handleBulkDelete"
        />
      </template>

      <template #detail>
        <SkillCreatePanel
          v-if="panelMode.type === 'create'"
          :platforms="allPlatforms"
          :selected-platforms="selectedPlatforms"
          @cancel="closePanel"
          @create="handleCreateSkill"
        />

        <SkillImportPanel
          v-else-if="panelMode.type === 'import'"
          :platforms="allPlatforms"
          :selected-platforms="selectedPlatforms"
          :browse-folder="browseFolder"
          @cancel="closePanel"
          @import="handleImportLocal"
          @toggle-platform="handleTogglePlatform"
        />

        <SkillImportGithubPanel
          v-else-if="panelMode.type === 'import-github'"
          :platforms="allPlatforms"
          :selected-platforms="selectedPlatforms"
          @cancel="closePanel"
          @import="handleImportGithub"
          @toggle-platform="handleTogglePlatform"
        />

        <SkillDetailPanel
          v-else
          :skill="activeSkill"
          :selected-platforms="selectedPlatforms"
          :ensure-content="ensureContent"
          @remove="handleRemoveSkill"
          @remove-installation="handleRemoveInstallation"
          @sync="handleSyncSkill"
        />
      </template>
    </MasterDetailLayout>

    <BulkDeleteDialog
      :is-open="showBulkDeleteDialog"
      :items="bulkDeleteItems"
      resource-label="skill"
      :loading="bulkDeleting"
      @confirm="confirmBulkDelete"
      @cancel="showBulkDeleteDialog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import MasterDetailLayout from '@/components/common/MasterDetailLayout.vue'
import BulkDeleteDialog from '@/components/common/BulkDeleteDialog.vue'
import type { BulkDeleteItem } from '@/components/common/BulkDeleteDialog.vue'
import SkillListPanel from '@/components/skills/SkillListPanel.vue'
import SkillDetailPanel from '@/components/skills/SkillDetailPanel.vue'
import SkillCreatePanel from '@/components/skills/SkillCreatePanel.vue'
import SkillImportPanel from '@/components/skills/SkillImportPanel.vue'
import SkillImportGithubPanel from '@/components/skills/SkillImportGithubPanel.vue'
import SkillTrashPanel from '@/components/skills/SkillTrashPanel.vue'
import { useSkillsManager } from '@/composables/useSkillsManager'
import { useUIStore } from '@/stores/ui'
import type { SkillRecord, Platform } from '@/types/skills'

const uiStore = useUIStore()

const {
  inventoryLoading,
  panelMode,
  searchQuery,
  filteredGroups,
  activeSkill,
  effectiveSelectedKeys,
  selectedGroups,
  isMultiSelectMode,
  selectedPlatforms,
  platforms,
  stats,
  selectSkill,
  openCreate,
  openImport,
  openImportGithub,
  closePanel,
  toggleMultiSelect,
  bulkDelete,
  removeSkill,
  removeInstallation,
  syncSkill,
  refresh,
  ensureContent,
  importFromGithub,
  importFromLocal,
  browseFolder,
} = useSkillsManager()

const allPlatforms = computed(() => platforms?.value ?? [])

// 批量删除
const showBulkDeleteDialog = ref(false)
const bulkDeleting = ref(false)

// P1-6：回收站抽屉状态
const showTrash = ref(false)

function handleRestored(path: string) {
  uiStore.showSuccess(`Restored skill to ${path}`)
  void refresh()
}

function handlePurged(_trashId: string) {
  uiStore.showSuccess('Trash entry purged')
}

const bulkDeleteItems = computed<BulkDeleteItem[]>(() =>
  selectedGroups.value.map(s => ({ key: s.id, label: s.name, badge: s.origin })),
)

function handleBulkDelete() { showBulkDeleteDialog.value = true }

async function confirmBulkDelete() {
  bulkDeleting.value = true
  try {
    await bulkDelete()
    showBulkDeleteDialog.value = false
    uiStore.showSuccess('Deleted selected skills')
  } catch (err) { uiStore.showError(err instanceof Error ? err.message : String(err)) }
  finally { bulkDeleting.value = false }
}

async function handleRemoveSkill(skillId: string) {
  try { await removeSkill(skillId); uiStore.showSuccess('Skill removed') }
  catch (err) { uiStore.showError(err instanceof Error ? err.message : String(err)) }
}

async function handleRemoveInstallation(skillId: string, installationId: string) {
  try { await removeInstallation(skillId, installationId); uiStore.showSuccess('Installation removed') }
  catch (err) { uiStore.showError(err instanceof Error ? err.message : String(err)) }
}

async function handleSyncSkill(skill: SkillRecord) {
  if (selectedPlatforms.value.length === 0 || !skill.installations[0]) return
  try {
    await syncSkill({ skillId: skill.id, installationId: skill.installations[0].id, targetPlatforms: selectedPlatforms.value })
    uiStore.showSuccess('Skill synced')
  } catch (err) { uiStore.showError(err instanceof Error ? err.message : String(err)) }
}

function handleTogglePlatform(id: Platform) {
  selectedPlatforms.value = selectedPlatforms.value.includes(id)
    ? selectedPlatforms.value.filter(p => p !== id)
    : [...selectedPlatforms.value, id]
}

async function handleCreateSkill(data: { name: string; description: string; content: string; platforms: Platform[] }) {
  // TODO: 实现自定义 skill 创建
  uiStore.showSuccess(`Created skill: ${data.name}`)
  closePanel()
  await refresh()
}

async function handleImportLocal(path: string) {
  try {
    await importFromLocal({ sourcePath: path, agents: selectedPlatforms.value })
    uiStore.showSuccess('Skill imported')
    closePanel()
    await refresh()
  } catch (err) { uiStore.showError(err instanceof Error ? err.message : String(err)) }
}

async function handleImportGithub(url: string) {
  try {
    await importFromGithub({ url, agents: selectedPlatforms.value })
    uiStore.showSuccess('Skill imported from GitHub')
    closePanel()
    await refresh()
  } catch (err) { uiStore.showError(err instanceof Error ? err.message : String(err)) }
}
</script>

<style scoped>
.skills-manager-view {
  height: 100%;
  overflow: hidden;
}

.skills-manager-view__trash-bar {
  @apply flex items-center justify-end px-3 py-1;
}

.skills-manager-view__trash-toggle {
  @apply rounded-full border border-border-default/55 px-3 py-1 text-xs text-text-secondary;

  background: var(--surface-status-bg);
}

.skills-manager-view__trash-toggle[aria-expanded="true"] {
  @apply text-text-primary;

  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
}

.skills-manager-view__trash-panel {
  @apply mx-3 my-2;
}
</style>
