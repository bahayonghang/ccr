<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="max-w-[1200px] mx-auto">
      <!-- Breadcrumb -->
      <Breadcrumb :items="breadcrumbs" />

      <!-- Loading State -->
      <div
        v-if="loading"
        class="text-center py-20 text-guofeng-text-muted"
      >
        <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-guofeng-red/30 border-t-guofeng-red" />
        {{ $t('common.loading') }}
      </div>

      <!-- Error State -->
      <div
        v-else-if="error"
        class="text-center py-20"
      >
        <div class="bg-guofeng-red/10 w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
          <AlertCircle class="w-10 h-10 text-guofeng-red" />
        </div>
        <p class="text-lg font-medium text-guofeng-text-primary">
          {{ $t('skills.loadError') }}
        </p>
        <p class="text-sm mt-2 text-guofeng-text-muted">
          {{ error }}
        </p>
        <RouterLink
          to="/skills"
          class="mt-4 inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-guofeng-bg-secondary hover:bg-guofeng-bg-tertiary transition-colors"
        >
          <ArrowLeft class="w-4 h-4" />
          {{ $t('common.back') }}
        </RouterLink>
      </div>

      <!-- Skill Detail -->
      <div v-else-if="skill">
        <!-- Header -->
        <div class="glass-effect rounded-2xl p-6 mb-6 border border-white/20 shadow-sm">
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-3 mb-2">
                <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-guofeng-red/10 to-guofeng-blue/10 flex items-center justify-center text-xl shadow-sm border border-white/20">
                  <Book class="w-6 h-6 text-guofeng-red" />
                </div>
                <div>
                  <h1 class="text-2xl font-bold text-guofeng-text-primary">
                    {{ skill.name }}
                  </h1>
                  <p
                    v-if="skill.description"
                    class="text-sm text-guofeng-text-secondary mt-1"
                  >
                    {{ skill.description }}
                  </p>
                </div>
              </div>

              <div class="flex items-center gap-3 mt-4 text-xs text-guofeng-text-muted">
                <div class="flex items-center gap-1.5 bg-guofeng-bg-tertiary px-2.5 py-1 rounded-md border border-guofeng-border/50">
                  <FolderOpen class="w-3.5 h-3.5" />
                  <span class="font-mono truncate max-w-[300px]">{{ skill.path }}</span>
                </div>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <button
                class="px-4 py-2 rounded-lg font-medium text-sm transition-all bg-guofeng-blue/10 text-guofeng-blue hover:bg-guofeng-blue/20 flex items-center gap-2"
                @click="handleEdit"
              >
                <Edit2 class="w-4 h-4" />
                {{ $t('common.edit') }}
              </button>
              <button
                class="px-4 py-2 rounded-lg font-medium text-sm transition-all bg-guofeng-red/10 text-guofeng-red hover:bg-guofeng-red/20 flex items-center gap-2"
                @click="handleDelete"
              >
                <Trash2 class="w-4 h-4" />
                {{ $t('common.delete') }}
              </button>
            </div>
          </div>
        </div>

        <!-- Instruction Content -->
        <div class="glass-effect rounded-2xl p-6 border border-white/20 shadow-sm">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-bold text-guofeng-text-primary flex items-center gap-2">
              <FileText class="w-5 h-5 text-guofeng-red" />
              {{ $t('skills.instructionLabel') }}
            </h2>
            <button
              class="px-3 py-1.5 rounded-lg text-xs font-medium transition-all bg-guofeng-bg-tertiary hover:bg-guofeng-bg-secondary text-guofeng-text-secondary flex items-center gap-1.5"
              @click="copyInstruction"
            >
              <Copy class="w-3.5 h-3.5" />
              {{ copied ? $t('common.copied') : $t('common.copy') }}
            </button>
          </div>

          <div class="relative">
            <pre class="bg-guofeng-bg-tertiary/50 rounded-xl p-4 overflow-auto max-h-[600px] border border-guofeng-border/30">
              <code class="text-sm font-mono text-guofeng-text-primary whitespace-pre-wrap break-words leading-relaxed">{{ skill.instruction }}</code>
            </pre>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit Modal -->
    <div
      v-if="showEditModal"
      class="fixed inset-0 flex items-center justify-center z-50 bg-guofeng-ink/20 backdrop-blur-sm transition-all"
      @click="showEditModal = false"
    >
      <div
        class="bg-guofeng-bg p-8 rounded-2xl w-full max-w-2xl max-h-[85vh] overflow-y-auto shadow-2xl border border-guofeng-border relative"
        @click.stop
      >
        <button
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-guofeng-bg-tertiary text-guofeng-text-muted transition-colors"
          @click="showEditModal = false"
        >
          <X class="w-5 h-5" />
        </button>

        <h3 class="text-2xl font-bold mb-6 text-guofeng-text-primary flex items-center">
          <Edit2 class="w-6 h-6 mr-2 text-guofeng-red" />
          {{ $t('skills.editSkill') }}
        </h3>

        <div class="space-y-5">
          <div>
            <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary">{{ $t('skills.nameLabel') }}</label>
            <input
              :value="skill?.name"
              type="text"
              disabled
              class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border opacity-60 cursor-not-allowed"
            >
          </div>

          <div>
            <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary">{{ $t('skills.instructionLabel') }}</label>
            <textarea
              v-model="editInstruction"
              rows="12"
              class="w-full px-4 py-3 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all resize-y font-mono text-sm"
            />
          </div>
        </div>

        <div class="flex gap-4 mt-8 pt-6 border-t border-guofeng-border">
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-all bg-guofeng-bg-tertiary text-guofeng-text-secondary hover:bg-guofeng-bg-secondary border border-guofeng-border"
            @click="showEditModal = false"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-all bg-guofeng-red text-white shadow-md hover:shadow-lg hover:-translate-y-0.5"
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
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Book, Edit2, Trash2, ArrowLeft, FolderOpen, FileText, Copy, X, AlertCircle, Home } from 'lucide-vue-next'
import Breadcrumb from '@/components/common/Breadcrumb.vue'
import { useSkills, type Skill } from '@/composables/useSkills'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const { getSkill, updateSkill, deleteSkill, loading, error } = useSkills()

const skill = ref<Skill | null>(null)
const showEditModal = ref(false)
const editInstruction = ref('')
const saving = ref(false)
const copied = ref(false)

const breadcrumbs = computed(() => [
  { label: t('common.home'), to: '/', icon: Home },
  { label: t('skills.title'), to: '/skills', icon: Book },
  { label: skill.value?.name || t('common.loading') }
])

onMounted(async () => {
  const name = route.params.name as string
  if (name) {
    try {
      skill.value = await getSkill(name)
    } catch (err) {
      console.error('Failed to load skill:', err)
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
    console.error('Failed to update skill:', err)
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
    console.error('Failed to delete skill:', err)
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
    console.error('Failed to copy:', err)
  }
}
</script>
