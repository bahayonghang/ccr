<!-- -->
<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="modelValue"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        @click.self="close"
      >
        <div class="absolute inset-0 bg-black/50 backdrop-blur-md" />

        <div class="modal-content relative flex max-h-[85vh] w-full max-w-3xl flex-col">
          <SkillDetailModalHeader
            :title="skillContent?.name || skill?.name || ''"
            :subtitle="skill?.platformName || ''"
            :platform-color="platformColor"
            :platform-icon="platformIcon"
            :is-edit-mode="isEditMode"
            :preview-title="t('skills.previewMode')"
            :edit-title="t('skills.editMode')"
            @toggle-mode="toggleMode"
            @close="close"
          />

          <SkillDetailContentPanel
            :is-content-loading="isContentLoading"
            :content-error="contentError"
            :skill-content="skillContent"
            :metadata-items="metadataItems"
            :is-edit-mode="isEditMode"
            :edit-buffer="editBuffer"
            :loading-label="t('skills.loadingContent')"
            :retry-label="t('common.retry')"
            :skill-content-label="t('skills.skillContent')"
            :no-content-label="t('skills.noContent')"
            :edit-placeholder="t('skills.instructionLabel')"
            @retry="loadContent"
            @update:edit-buffer="editBuffer = $event"
          />

          <SkillDetailEditFooter
            v-if="isEditMode && skillContent"
            :is-saving="isSaving"
            :cancel-label="t('common.cancel')"
            :save-label="t('skills.saveSkill')"
            @cancel="cancelEdit"
            @save="handleSave"
          />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import SkillDetailContentPanel from '@/components/skills/SkillDetailContentPanel.vue'
import SkillDetailEditFooter from '@/components/skills/SkillDetailEditFooter.vue'
import SkillDetailModalHeader from '@/components/skills/SkillDetailModalHeader.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import type { SkillDetailMetaItem } from '@/types/skillDetailModal'
import type { Platform, SkillContent, UnifiedSkill } from '@/types/skills'
import { PLATFORM_CONFIG } from '@/types/skills'
import { getErrorMessage } from '@/utils/errorHandler'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const GeminiIcon = 'Sparkles'

const props = defineProps<{
  modelValue: boolean
  skill?: UnifiedSkill
  initialMode?: 'view' | 'edit'
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'saved'): void
}>()

const { fetchSkillContent, saveSkillContent } = useUnifiedSkills()
const { t } = useI18n()

const skillContent = ref<SkillContent | null>(null)
const isContentLoading = ref(false)
const contentError = ref<string | null>(null)
const isEditMode = ref(false)
const editBuffer = ref('')
const isSaving = ref(false)

const platformColor = computed(() => {
  if (!props.skill) return '#A78BFA'
  const config = PLATFORM_CONFIG[props.skill.platform as Platform]
  return config?.color || '#A78BFA'
})

const platformIcon = computed(() => {
  const iconMap: Record<string, string> = {
    'claude-code': 'Code2',
    codex: 'Settings',
    gemini: GeminiIcon,
    qwen: 'Zap',
    iflow: 'Activity',
    droid: 'Bot',
  }
  return iconMap[props.skill?.platform || ''] || 'Code2'
})

const sourceIconMap: Record<string, string> = {
  marketplace: 'Store',
  github: 'Github',
  local: 'HardDrive',
}

const sourceLabelMap: Record<string, string> = {
  marketplace: 'Marketplace',
  github: 'GitHub',
  local: 'Local',
}

const metadataItems = computed<SkillDetailMetaItem[]>(() => {
  const items: SkillDetailMetaItem[] = []

  if (skillContent.value?.category) {
    items.push({
      id: 'category',
      icon: 'Folder',
      label: t('skills.categoryLabel'),
      value: skillContent.value.category,
    })
  }

  if (props.skill?.platformName) {
    items.push({
      id: 'platform',
      icon: platformIcon.value,
      label: t('skills.platform'),
      value: props.skill.platformName,
      iconColor: platformColor.value,
    })
  }

  if (props.skill?.version) {
    items.push({
      id: 'version',
      icon: 'Tag',
      label: t('skills.version'),
      value: `v${props.skill.version}`,
    })
  }

  if (props.skill?.author) {
    items.push({
      id: 'author',
      icon: 'User',
      label: t('skills.author'),
      value: props.skill.author,
    })
  }

  if (props.skill?.source) {
    items.push({
      id: 'source',
      icon: sourceIconMap[props.skill.source] || 'HardDrive',
      label: t('skills.sourceLabel'),
      value: sourceLabelMap[props.skill.source] || props.skill.source,
      linkUrl: props.skill.sourceUrl,
    })
  }

  if (props.skill?.installDate) {
    items.push({
      id: 'installDate',
      icon: 'Clock',
      label: t('skills.installedAt'),
      value: formatDate(props.skill.installDate),
    })
  }

  if (skillContent.value?.skillDir) {
    items.push({
      id: 'directory',
      icon: 'FolderOpen',
      label: t('skills.directory'),
      value: shortenPath(skillContent.value.skillDir),
      valueTitle: skillContent.value.skillDir,
      monospace: true,
    })
  }

  return items
})

watch(
  () => props.modelValue,
  (isOpen) => {
    if (isOpen && props.skill) {
      isEditMode.value = props.initialMode === 'edit'
      contentError.value = null
      skillContent.value = null
      loadContent()
    }
  }
)

async function loadContent() {
  if (!props.skill) return

  isContentLoading.value = true
  contentError.value = null

  try {
    skillContent.value = await fetchSkillContent(props.skill.skillDir)
    editBuffer.value = skillContent.value.raw
  } catch (err: unknown) {
    contentError.value = getErrorMessage(err) || 'Failed to load skill content'
  } finally {
    isContentLoading.value = false
  }
}

function toggleMode() {
  if (isEditMode.value) {
    isEditMode.value = false
    return
  }

  if (skillContent.value) {
    editBuffer.value = skillContent.value.raw
  }
  isEditMode.value = true
}

function cancelEdit() {
  if (skillContent.value) {
    editBuffer.value = skillContent.value.raw
  }
  isEditMode.value = false
}

async function handleSave() {
  if (!props.skill || !skillContent.value) return

  isSaving.value = true
  try {
    await saveSkillContent(props.skill.skillDir, editBuffer.value)
    skillContent.value = await fetchSkillContent(props.skill.skillDir)
    editBuffer.value = skillContent.value.raw
    isEditMode.value = false
    emit('saved')
  } catch (err: unknown) {
    contentError.value = getErrorMessage(err) || 'Failed to save'
  } finally {
    isSaving.value = false
  }
}

function close() {
  emit('update:modelValue', false)
}

function shortenPath(path: string): string {
  const segments = path.replace(/\\/g, '/').split('/')
  if (segments.length <= 3) return path
  return `.../${segments.slice(-3).join('/')}`
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}
</script>

<style scoped>
.modal-content {
  @apply overflow-hidden rounded-2xl border border-white/10 bg-black/40 shadow-2xl backdrop-blur-xl;
}

.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}

.modal-fade-enter-active .modal-content,
.modal-fade-leave-active .modal-content {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}

.modal-fade-enter-from .modal-content,
.modal-fade-leave-to .modal-content {
  transform: scale(0.95);
  opacity: 0;
}
</style>
