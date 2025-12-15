<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="mb-6" />

    <div class="max-w-[1600px] mx-auto">
      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-4">
          <h2 class="text-2xl font-bold text-guofeng-text-primary flex items-center">
            <Book class="w-7 h-7 mr-2 text-guofeng-red" />
            {{ $t('skills.title') }}
          </h2>
          <span class="px-3 py-1 rounded-full text-sm font-medium bg-guofeng-red/10 text-guofeng-red border border-guofeng-red/20">
            {{ skills.length }}
          </span>
        </div>
        <div class="flex items-center gap-3">
          <RouterLink
            to="/claude"
            class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors bg-guofeng-bg-secondary text-guofeng-text-secondary border border-guofeng-border hover:bg-guofeng-bg-tertiary"
          >
            <Home class="w-4 h-4" /><span>{{ $t('common.back') }}</span>
          </RouterLink>
          <button
            class="px-4 py-2 rounded-lg font-medium transition-all hover:scale-105 bg-guofeng-red text-white shadow-md hover:shadow-lg flex items-center"
            @click="handleAdd"
          >
            <Plus class="w-5 h-5 mr-2" />{{ $t('skills.addSkill') }}
          </button>
        </div>
      </div>

      <!-- Help Box -->
      <div class="mb-8 glass-effect border border-white/20 rounded-xl p-5 flex items-start gap-4 shadow-sm">
        <div class="p-2 bg-guofeng-blue/10 rounded-lg text-guofeng-blue">
          <Info class="w-6 h-6" />
        </div>
        <div>
          <h3 class="font-bold text-guofeng-text-primary mb-1">
            {{ $t('skills.help.title') }}
          </h3>
          <p class="text-sm text-guofeng-text-secondary mb-2 leading-relaxed">
            {{ $t('skills.help.description') }}
          </p>
          <div class="flex items-center gap-4 text-xs text-guofeng-text-muted font-mono bg-guofeng-bg-tertiary px-3 py-1.5 rounded-md inline-block border border-guofeng-border/50">
            {{ $t('skills.help.structure') }}
          </div>
        </div>
      </div>

      <!-- Skills Grid -->
      <div
        v-if="loading"
        class="text-center py-20 text-guofeng-text-muted"
      >
        <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-guofeng-red/30 border-t-guofeng-red" />
        {{ $t('common.loading') }}
      </div>
      
      <div
        v-else-if="skills.length === 0"
        class="text-center py-20 text-guofeng-text-muted"
      >
        <div class="bg-guofeng-bg-secondary w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
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
        v-else
        class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4"
      >
        <GuofengCard
          v-for="skill in skills"
          :key="skill.name"
          variant="glass"
          interactive
          pattern
          @click="handleEdit(skill)"
        >
          <div class="relative z-10">
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-2">
                <h3 class="text-lg font-bold text-guofeng-text-primary group-hover:text-guofeng-red transition-colors">
                  {{ skill.name }}
                </h3>
              </div>
              <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                <button
                  class="p-1.5 rounded-md text-guofeng-blue hover:bg-guofeng-blue/10 transition-colors"
                  :title="$t('common.edit')"
                  @click.stop="handleEdit(skill)"
                >
                  <Edit2 class="w-4 h-4" />
                </button>
                <button
                  class="p-1.5 rounded-md text-guofeng-red hover:bg-guofeng-red/10 transition-colors"
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
                class="text-guofeng-text-secondary line-clamp-2"
              >
                {{ skill.description }}
              </div>
              <div class="mt-3 pt-3 border-t border-guofeng-border/50">
                <p class="text-xs text-guofeng-text-muted font-mono truncate">
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
      class="fixed inset-0 flex items-center justify-center z-50 bg-guofeng-ink/20 backdrop-blur-sm transition-all"
      @click="showModal = false"
    >
      <div
        class="bg-guofeng-bg p-8 rounded-2xl w-full max-w-2xl max-h-[85vh] overflow-y-auto shadow-2xl border border-guofeng-border relative"
        @click.stop
      >
        <button 
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-guofeng-bg-tertiary text-guofeng-text-muted transition-colors"
          @click="showModal = false"
        >
          <X class="w-5 h-5" />
        </button>

        <h3 class="text-2xl font-bold mb-6 text-guofeng-text-primary flex items-center">
          <component
            :is="editingSkill ? Edit2 : Plus"
            class="w-6 h-6 mr-2 text-guofeng-red"
          />
          {{ editingSkill ? $t('skills.editSkill') : $t('skills.addSkill') }}
        </h3>

        <div class="space-y-5">
          <div>
            <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary">{{ $t('skills.nameLabel') }}</label>
            <input
              v-model="formData.name"
              type="text"
              :disabled="!!editingSkill"
              class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all disabled:opacity-60 disabled:cursor-not-allowed"
              :placeholder="$t('skills.namePlaceholder')"
            >
          </div>

          <div>
            <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary">{{ $t('skills.instructionLabel') }}</label>
            <textarea
              v-model="formData.instruction"
              rows="10"
              class="w-full px-4 py-3 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all resize-y font-mono text-sm"
              :placeholder="`# Skill Name\n\nDescription of what this skill does.\n\n## Instructions\n\n1. Step one\n2. Step two`"
            />
          </div>
        </div>

        <div class="flex gap-4 mt-8 pt-6 border-t border-guofeng-border">
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-all bg-guofeng-bg-tertiary text-guofeng-text-secondary hover:bg-guofeng-bg-secondary border border-guofeng-border"
            @click="showModal = false"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-all bg-guofeng-red text-white shadow-md hover:shadow-lg hover:-translate-y-0.5"
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
import { ref, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Book, Plus, Edit2, Trash2, X, Home, Info } from 'lucide-vue-next'
import GuofengCard from '@/components/common/GuofengCard.vue'
import { useSkills, type Skill } from '@/composables/useSkills'

const { t } = useI18n()
const { skills, loading, listSkills, addSkill, updateSkill, deleteSkill } = useSkills()

const showModal = ref(false)
const editingSkill = ref<Skill | null>(null)
const formData = ref({ name: '', instruction: '' })

onMounted(() => {
  listSkills()
})

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
