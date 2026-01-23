<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="mb-6" />

    <div class="max-w-[1600px] mx-auto">
      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-4">
          <h2 class="text-2xl font-bold text-[var(--color-text-primary)] flex items-center">
            <Book class="w-7 h-7 mr-2 text-[var(--color-danger)]" />
            {{ $t('skills.title') }}
          </h2>
          <span class="px-3 py-1 rounded-full text-sm font-medium bg-[var(--color-danger)]/10 text-[var(--color-danger)] border border-[var(--color-danger)]/20">
            {{ filteredSkills.length }}
          </span>
        </div>
        <div class="flex items-center gap-3">
          <RouterLink
            to="/claude-code"
            class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)] border border-[var(--color-border-default)] hover:bg-[var(--color-bg-surface)]"
          >
            <Home class="w-4 h-4" /><span>{{ $t('common.back') }}</span>
          </RouterLink>
          <button
            class="px-4 py-2 rounded-lg font-medium transition-all hover:scale-105 bg-[var(--color-danger)] text-white shadow-md hover:shadow-lg flex items-center"
            @click="handleAdd"
          >
            <Plus class="w-5 h-5 mr-2" />{{ $t('skills.addSkill') }}
          </button>
        </div>
      </div>

      <!-- Search Bar -->
      <div class="mb-6 glass-effect rounded-xl p-4 border border-white/20 shadow-sm">
        <div class="relative">
          <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-[var(--color-text-muted)]" />
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="$t('skills.searchPlaceholder')"
            class="w-full pl-10 pr-10 py-2.5 rounded-xl bg-[var(--color-bg-surface)]/50 border border-[var(--color-border-default)] hover:bg-[var(--color-bg-surface)] focus:bg-[var(--color-bg-surface)] focus:outline-none focus:ring-2 focus:ring-[var(--color-danger)]/20 text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)] text-sm transition-all"
          >
          <button
            v-if="searchQuery"
            class="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 rounded-full hover:bg-[var(--color-bg-surface)] text-[var(--color-text-muted)] transition-all"
            @click="searchQuery = ''"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Help Box -->
      <div class="mb-8 glass-effect border border-white/20 rounded-xl p-5 flex items-start gap-4 shadow-sm">
        <div class="p-2 bg-[var(--color-info)]/10 rounded-lg text-[var(--color-info)]">
          <Info class="w-6 h-6" />
        </div>
        <div>
          <h3 class="font-bold text-[var(--color-text-primary)] mb-1">
            {{ $t('skills.help.title') }}
          </h3>
          <p class="text-sm text-[var(--color-text-secondary)] mb-2 leading-relaxed">
            {{ $t('skills.help.description') }}
          </p>
          <div class="flex items-center gap-4 text-xs text-[var(--color-text-muted)] font-mono bg-[var(--color-bg-surface)] px-3 py-1.5 rounded-md inline-block border border-[var(--color-border-default)]/50">
            {{ $t('skills.help.structure') }}
          </div>
        </div>
      </div>

      <!-- Skills Grid -->
      <div
        v-if="loading"
        class="text-center py-20 text-[var(--color-text-muted)]"
      >
        <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-[var(--color-danger)]/30 border-t-[var(--color-danger)]" />
        {{ $t('common.loading') }}
      </div>

      <div
        v-else-if="skills.length === 0"
        class="text-center py-20 text-[var(--color-text-muted)]"
      >
        <div class="bg-[var(--color-bg-elevated)] w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
          <Book class="w-10 h-10 opacity-50" />
        </div>
        <p class="text-lg font-medium">
          {{ $t('skills.noSkills') }}
        </p>
        <p class="text-sm mt-2 opacity-70">
          {{ $t('skills.noSkillsHint') }}
        </p>
      </div>

      <div
        v-else-if="filteredSkills.length === 0"
        class="text-center py-20 text-[var(--color-text-muted)]"
      >
        <div class="bg-[var(--color-bg-elevated)] w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
          <Search class="w-10 h-10 opacity-50" />
        </div>
        <p class="text-lg font-medium">
          {{ $t('skills.noSearchResults') }}
        </p>
        <p class="text-sm mt-2 opacity-70">
          {{ $t('skills.noSearchResultsHint') }}
        </p>
        <button
          class="mt-4 px-4 py-2 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/5 rounded-lg transition-colors"
          @click="searchQuery = ''"
        >
          {{ $t('skills.clearSearch') }}
        </button>
      </div>

      <div
        v-else
        class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4"
      >
        <GuofengCard
          v-for="skill in filteredSkills"
          :key="skill.name"
          variant="glass"
          interactive
          pattern
          @click="navigateToDetail(skill.name)"
        >
          <div class="relative z-10">
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-2">
                <h3 class="text-lg font-bold text-[var(--color-text-primary)] group-hover:text-[var(--color-danger)] transition-colors">
                  {{ skill.name }}
                </h3>
              </div>
              <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                <button
                  class="p-1.5 rounded-md text-[var(--color-info)] hover:bg-[var(--color-info)]/10 transition-colors"
                  :title="$t('common.view')"
                  @click.stop="navigateToDetail(skill.name)"
                >
                  <Eye class="w-4 h-4" />
                </button>
                <button
                  class="p-1.5 rounded-md text-[var(--color-info)] hover:bg-[var(--color-info)]/10 transition-colors"
                  :title="$t('common.edit')"
                  @click.stop="handleEdit(skill)"
                >
                  <Edit2 class="w-4 h-4" />
                </button>
                <button
                  class="p-1.5 rounded-md text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors"
                  :title="$t('common.delete')"
                  @click.stop="handleDelete(skill.name)"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>

            <div class="space-y-2 text-sm">
              <div
                v-if="skill.description"
                class="text-[var(--color-text-secondary)] line-clamp-2"
              >
                {{ skill.description }}
              </div>
              <div class="mt-3 pt-3 border-t border-[var(--color-border-default)]/50">
                <p class="text-xs text-[var(--color-text-muted)] font-mono truncate">
                  {{ skill.path }}
                </p>
              </div>
            </div>
          </div>
        </GuofengCard>
      </div>
    </div>

    <!-- Add/Edit Modal -->
    <div
      v-if="showModal"
      class="fixed inset-0 flex items-center justify-center z-50 bg-[var(--color-bg-overlay)]/20 backdrop-blur-sm transition-all"
      @click="showModal = false"
    >
      <div
        class="bg-[var(--color-bg-base)] p-8 rounded-2xl w-full max-w-2xl max-h-[85vh] overflow-y-auto shadow-2xl border border-[var(--color-border-default)] relative"
        @click.stop
      >
        <button
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-[var(--color-bg-surface)] text-[var(--color-text-muted)] transition-colors"
          @click="showModal = false"
        >
          <X class="w-5 h-5" />
        </button>

        <h3 class="text-2xl font-bold mb-6 text-[var(--color-text-primary)] flex items-center">
          <component
            :is="editingSkill ? Edit2 : Plus"
            class="w-6 h-6 mr-2 text-[var(--color-danger)]"
          />
          {{ editingSkill ? $t('skills.editSkill') : $t('skills.addSkill') }}
        </h3>

        <div class="space-y-5">
          <div>
            <label class="block mb-1.5 text-sm font-semibold text-[var(--color-text-secondary)]">{{ $t('skills.nameLabel') }}</label>
            <input
              v-model="formData.name"
              type="text"
              :disabled="!!editingSkill"
              class="w-full px-4 py-2.5 rounded-lg bg-[var(--color-bg-surface)] border border-[var(--color-border-default)] focus:border-[var(--color-danger)] focus:ring-1 focus:ring-[var(--color-danger)] outline-none transition-all disabled:opacity-60 disabled:cursor-not-allowed"
              :placeholder="$t('skills.namePlaceholder')"
            >
          </div>

          <div>
            <label class="block mb-1.5 text-sm font-semibold text-[var(--color-text-secondary)]">{{ $t('skills.instructionLabel') }}</label>
            <textarea
              v-model="formData.instruction"
              rows="10"
              class="w-full px-4 py-3 rounded-lg bg-[var(--color-bg-surface)] border border-[var(--color-border-default)] focus:border-[var(--color-danger)] focus:ring-1 focus:ring-[var(--color-danger)] outline-none transition-all resize-y font-mono text-sm"
              :placeholder="`# Skill Name\n\nDescription of what this skill does.\n\n## Instructions\n\n1. Step one\n2. Step two`"
            />
          </div>
        </div>

        <div class="flex gap-4 mt-8 pt-6 border-t border-[var(--color-border-default)]">
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-all bg-[var(--color-bg-surface)] text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-elevated)] border border-[var(--color-border-default)]"
            @click="showModal = false"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-all bg-[var(--color-danger)] text-white shadow-md hover:shadow-lg hover:-translate-y-0.5"
            @click="handleSubmit"
          >
            {{ editingSkill ? $t('common.save') : $t('common.add') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Book, Plus, Edit2, Trash2, X, Home, Info, Search, Eye } from 'lucide-vue-next'
import GuofengCard from '@/components/common/GuofengCard.vue'
import { useSkills, type Skill } from '@/composables/useSkills'

const router = useRouter()
const { t } = useI18n()
const { skills, loading, listSkills, addSkill, updateSkill, deleteSkill } = useSkills()

const showModal = ref(false)
const editingSkill = ref<Skill | null>(null)
const formData = ref({ name: '', instruction: '' })
const searchQuery = ref('')

// 客户端过滤
const filteredSkills = computed(() => {
  if (!searchQuery.value.trim()) {
    return skills.value
  }
  const query = searchQuery.value.toLowerCase()
  return skills.value.filter(skill =>
    skill.name.toLowerCase().includes(query) ||
    (skill.description && skill.description.toLowerCase().includes(query)) ||
    (skill.instruction && skill.instruction.toLowerCase().includes(query))
  )
})

onMounted(() => {
  listSkills()
})

const navigateToDetail = (name: string) => {
  router.push(`/skills/${encodeURIComponent(name)}`)
}

const handleAdd = () => {
  showModal.value = true
  editingSkill.value = null
  formData.value = { name: '', instruction: '' }
}

const handleEdit = (skill: Skill) => {
  editingSkill.value = skill
  showModal.value = true
  formData.value = { name: skill.name, instruction: skill.instruction }
}

const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.instruction) {
    alert(t('skills.validation.required'))
    return
  }

  try {
    if (editingSkill.value) {
      await updateSkill(editingSkill.value.name, { instruction: formData.value.instruction })
    } else {
      await addSkill({ name: formData.value.name, instruction: formData.value.instruction })
    }
    showModal.value = false
  } catch (err) {
    console.error('Operation failed:', err)
    alert(t('common.operationFailed'))
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('skills.deleteConfirm', { name }))) return
  try {
    await deleteSkill(name)
  } catch (err) {
    console.error('Delete failed:', err)
    alert(t('common.deleteFailed'))
  }
}
</script>
