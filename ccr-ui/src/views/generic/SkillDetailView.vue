<template>
  <div class="min-h-full p-5 transition-colors duration-300">
    <div class="max-w-[1200px] mx-auto">
      <ModuleSubnav
        module="skills"
        class="mb-6"
      />

      <div class="mb-6 flex items-center gap-3">
        <RouterLink
          to="/skills"
          class="inline-flex min-h-[44px] items-center gap-2 rounded-xl border border-border-default/60 bg-[var(--color-bg-surface)]/70 px-4 py-2 text-sm font-medium text-[var(--color-text-secondary)] transition-colors hover:border-[var(--color-danger)]/30 hover:bg-[var(--color-bg-elevated)] hover:text-[var(--color-text-primary)]"
        >
          <SIcon
            name="ArrowLeft"
            size="w-4 h-4"
          />
          {{ $t('common.back') }}
        </RouterLink>
        <span class="text-sm text-[var(--color-text-muted)]">
          {{ skill?.name || $t('common.loading') }}
        </span>
      </div>

      <!-- Loading State -->
      <div
        v-if="loading"
        class="text-center py-20 text-[var(--color-text-muted)]"
      >
        <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-[var(--color-danger)]/30 border-t-[var(--color-danger)]" />
        {{ $t('common.loading') }}
      </div>

      <!-- Error State -->
      <div
        v-else-if="error"
        class="text-center py-20"
      >
        <div class="bg-[var(--color-danger)]/10 w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
          <SIcon
            name="AlertCircle"
            size="w-10 h-10"
            class="text-[var(--color-danger)]"
          />
        </div>
        <p class="text-lg font-medium text-[var(--color-text-primary)]">
          {{ $t('skills.loadError') }}
        </p>
        <p class="text-sm mt-2 text-[var(--color-text-muted)]">
          {{ error }}
        </p>
        <RouterLink
          to="/skills"
          class="mt-4 inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-[var(--color-bg-elevated)] hover:bg-[var(--color-bg-surface)] transition-colors"
        >
          <SIcon
            name="ArrowLeft"
            size="w-4 h-4"
          />
          {{ $t('common.back') }}
        </RouterLink>
      </div>

      <!-- Skill Detail -->
      <div v-else-if="skill">
        <PageHeaderCard
          :title="skill.name"
          :description="skill.description || undefined"
          icon="Book"
          tone="danger"
          class="mb-6"
        >
          <template #meta>
            <span class="inline-flex items-center gap-1.5 rounded-full border border-[var(--color-border-default)]/50 bg-[var(--color-bg-surface)] px-3 py-1 text-xs text-[var(--color-text-muted)]">
              <SIcon
                name="FolderOpen"
                size="w-3.5 h-3.5"
              />
              <span class="font-mono truncate max-w-[300px]">{{ skill.path }}</span>
            </span>
          </template>

          <template #actions>
            <button
              class="px-4 py-2 rounded-lg font-medium text-sm transition-colors bg-[var(--color-info)]/10 text-[var(--color-info)] hover:bg-[var(--color-info)]/20 flex items-center gap-2"
              @click="handleEdit"
            >
              <SIcon
                name="Edit2"
                size="w-4 h-4"
              />
              {{ $t('common.edit') }}
            </button>
            <button
              class="px-4 py-2 rounded-lg font-medium text-sm transition-colors bg-[var(--color-danger)]/10 text-[var(--color-danger)] hover:bg-[var(--color-danger)]/20 flex items-center gap-2"
              @click="handleDelete"
            >
              <SIcon
                name="Trash2"
                size="w-4 h-4"
              />
              {{ $t('common.delete') }}
            </button>
          </template>
        </PageHeaderCard>

        <!-- Instruction Content -->
        <div class="glass-effect rounded-2xl p-6 border border-white/20 shadow-sm">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-bold text-[var(--color-text-primary)] flex items-center gap-2">
              <SIcon
                name="FileText"
                size="w-5 h-5"
                class="text-[var(--color-danger)]"
              />
              {{ $t('skills.instructionLabel') }}
            </h2>
            <button
              class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-[var(--color-bg-surface)] hover:bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)] flex items-center gap-1.5"
              @click="copyInstruction"
            >
              <SIcon
                name="Copy"
                size="w-3.5 h-3.5"
              />
              {{ copied ? $t('common.copied') : $t('common.copy') }}
            </button>
          </div>

          <div class="relative">
            <pre class="bg-[var(--color-bg-surface)]/50 rounded-xl p-4 overflow-auto max-h-[600px] border border-[var(--color-border-default)]/30">
              <code class="text-sm font-mono text-[var(--color-text-primary)] whitespace-pre-wrap break-words leading-relaxed">{{ skill.instruction }}</code>
            </pre>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit Modal -->
    <div
      v-if="showEditModal"
      class="fixed inset-0 flex items-center justify-center z-50 bg-[var(--color-bg-overlay)]/20 backdrop-blur-md transition-opacity"
      @click="showEditModal = false"
    >
      <div
        class="bg-[var(--color-bg-base)] p-8 rounded-2xl w-full max-w-2xl max-h-[85vh] overflow-y-auto shadow-2xl border border-[var(--color-border-default)] relative"
        @click.stop
      >
        <button
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-[var(--color-bg-surface)] text-[var(--color-text-muted)] transition-colors"
          @click="showEditModal = false"
        >
          <SIcon
            name="X"
            size="w-5 h-5"
          />
        </button>

        <h3 class="text-2xl font-bold mb-6 text-[var(--color-text-primary)] flex items-center">
          <SIcon
            name="Edit2"
            size="w-6 h-6"
            class="mr-2 text-[var(--color-danger)]"
          />
          {{ $t('skills.editSkill') }}
        </h3>

        <div class="space-y-5">
          <div>
            <label class="block mb-1.5 text-sm font-semibold text-[var(--color-text-secondary)]">{{ $t('skills.nameLabel') }}</label>
            <input
              :value="skill?.name"
              type="text"
              disabled
              class="w-full px-4 py-2.5 rounded-lg bg-[var(--color-bg-surface)] border border-[var(--color-border-default)] opacity-60 cursor-not-allowed"
            >
          </div>

          <div>
            <label class="block mb-1.5 text-sm font-semibold text-[var(--color-text-secondary)]">{{ $t('skills.instructionLabel') }}</label>
            <textarea
              v-model="editInstruction"
              rows="12"
              class="w-full px-4 py-3 rounded-lg bg-[var(--color-bg-surface)] border border-[var(--color-border-default)] focus:border-[var(--color-danger)] focus:ring-1 focus:ring-[var(--color-danger)] outline-none transition-[border-color,box-shadow] resize-y font-mono text-sm"
            />
          </div>
        </div>

        <div class="flex gap-4 mt-8 pt-6 border-t border-[var(--color-border-default)]">
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-colors bg-[var(--color-bg-surface)] text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-elevated)] border border-[var(--color-border-default)]"
            @click="showEditModal = false"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-[box-shadow,transform] bg-[var(--color-danger)] text-white shadow-md hover:shadow-lg hover:-translate-y-0.5"
            :disabled="saving"
            @click="handleSave"
          >
            {{ saving ? $t('common.saving') : $t('common.save') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import { useSkills, type Skill } from '@/composables/useSkills'
import { extractStringParam } from '@/types/router'
import { logger } from '@/utils/logger'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const { getSkill, updateSkill, deleteSkill, loading, error } = useSkills()

const skill = ref<Skill | null>(null)
const showEditModal = ref(false)
const editInstruction = ref('')
const saving = ref(false)
const copied = ref(false)

onMounted(async () => {
  const name = extractStringParam(route.params.name)
  if (name) {
    try {
      skill.value = await getSkill(name)
    } catch (err) {
      logger.error('Failed to load skill:', err)
    }
  }
})

const handleEdit = () => {
  if (skill.value) {
    editInstruction.value = skill.value.instruction
    showEditModal.value = true
  }
}

const handleSave = async () => {
  if (!skill.value || !editInstruction.value.trim()) return

  saving.value = true
  try {
    await updateSkill(skill.value.name, { instruction: editInstruction.value })
    skill.value.instruction = editInstruction.value
    showEditModal.value = false
  } catch (err) {
    logger.error('Failed to update skill:', err)
    alert(t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}

const handleDelete = async () => {
  if (!skill.value) return

  if (!confirm(t('skills.deleteConfirm', { name: skill.value.name }))) return

  try {
    await deleteSkill(skill.value.name)
    router.push('/skills')
  } catch (err) {
    logger.error('Failed to delete skill:', err)
    alert(t('common.deleteFailed'))
  }
}

const copyInstruction = async () => {
  if (!skill.value) return

  try {
    await navigator.clipboard.writeText(skill.value.instruction)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    logger.error('Failed to copy:', err)
  }
}
</script>
