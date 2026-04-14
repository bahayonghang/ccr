<template>
  <div class="glass-effect rounded-3xl p-6 border border-border-default/25">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-xl bg-gradient-to-br from-accent-success/20 to-accent-primary/20">
          <SIcon
            name="GitFork"
            size="w-5 h-5"
            class="text-accent-success"
          />
        </div>
        <div>
          <h3 class="text-base font-bold text-text-primary">
            {{ $t('skills.repositories.title') }}
          </h3>
          <p class="text-xs text-text-muted">
            {{ $t('skills.repositories.subtitle') }}
          </p>
        </div>
      </div>
      <button
        class="px-3 py-1.5 text-xs rounded-lg bg-gradient-to-r from-accent-success to-accent-primary text-white font-medium flex items-center gap-1.5"
        @click="showAddModal = true"
      >
        <SIcon
          name="Plus"
          size="w-3.5 h-3.5"
        />
        {{ $t('skills.repositories.add') }}
      </button>
    </div>

    <!-- Loading -->
    <div
      v-if="loading"
      class="flex justify-center py-8"
    >
      <SIcon
        name="Loader2"
        size="w-6 h-6"
        class="animate-spin text-accent-success"
      />
    </div>

    <!-- Empty State -->
    <div
      v-else-if="repositories.length === 0"
      class="text-center py-8"
    >
      <SIcon
        name="GitFork"
        size="w-12 h-12"
        class="mx-auto text-text-muted/30 mb-3"
      />
      <p class="text-sm text-text-muted">
        {{ $t('skills.repositories.empty') }}
      </p>
      <p class="text-xs text-text-muted/70 mt-1">
        {{ $t('skills.repositories.emptyHint') }}
      </p>
    </div>

    <!-- Repository List -->
    <div
      v-else
      class="space-y-3"
    >
      <div
        v-for="repo in repositories"
        :key="repo.name"
        class="group p-4 rounded-xl bg-bg-surface/50 border border-transparent hover:border-accent-success/30 transition-colors"
      >
        <div class="flex items-start justify-between">
          <div class="flex items-center gap-3">
            <div
              class="p-2 rounded-lg"
              :class="repo.is_official ? 'bg-accent-warning/10' : 'bg-bg-base'"
            >
              <SIcon
                :name="repo.is_official ? 'Star' : 'Github'"
                size="w-4 h-4"
                :class="repo.is_official ? 'text-accent-warning' : 'text-text-muted'"
              />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <span class="text-sm font-bold text-text-primary">{{ repo.name }}</span>
                <span
                  v-if="repo.is_official"
                  class="text-[10px] px-1.5 py-0.5 rounded-full bg-accent-warning/10 text-accent-warning"
                >
                  {{ $t('skills.repositories.official') }}
                </span>
              </div>
              <p class="text-xs text-text-muted mt-0.5">
                {{ repo.description || repo.url }}
              </p>
            </div>
          </div>
          
          <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              class="p-1.5 rounded-lg hover:bg-accent-success/10 text-text-muted hover:text-accent-success transition-colors"
              :title="$t('skills.repositories.scan')"
              @click="scanRepository(repo.name)"
            >
              <SIcon
                name="RefreshCw"
                size="w-4 h-4"
                :class="{ 'animate-spin': scanningRepo === repo.name }"
              />
            </button>
            <button
              v-if="!repo.is_official"
              class="p-1.5 rounded-lg hover:bg-accent-danger/10 text-text-muted hover:text-accent-danger transition-colors"
              :title="$t('common.delete')"
              @click="removeRepository(repo.name)"
            >
              <SIcon
                name="Trash2"
                size="w-4 h-4"
              />
            </button>
          </div>
        </div>

        <!-- Stats Row -->
        <div class="flex items-center gap-4 mt-3 pt-3 border-t border-border-default/10">
          <div class="flex items-center gap-1.5 text-xs text-text-muted">
            <SIcon
              name="GitBranch"
              size="w-3.5 h-3.5"
            />
            <span>{{ repo.branch }}</span>
          </div>
          <div class="flex items-center gap-1.5 text-xs text-text-muted">
            <SIcon
              name="Package"
              size="w-3.5 h-3.5"
            />
            <span>{{ repo.skill_count || 0 }} {{ $t('skills.repositories.skills') }}</span>
          </div>
          <div
            v-if="repo.last_synced"
            class="flex items-center gap-1.5 text-xs text-text-muted"
          >
            <SIcon
              name="Clock"
              size="w-3.5 h-3.5"
            />
            <span>{{ formatTime(repo.last_synced) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Repository Modal -->
    <Teleport to="body">
      <div
        v-if="showAddModal"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-md"
        @click.self="showAddModal = false"
      >
        <div class="w-full max-w-md m-4 bg-bg-elevated rounded-2xl border border-border-default/15 overflow-hidden">
          <div class="flex items-center justify-between p-4 border-b border-border-default/15">
            <h3 class="text-base font-bold text-text-primary">
              {{ $t('skills.repositories.addTitle') }}
            </h3>
            <button
              class="p-2 rounded-lg hover:bg-bg-surface"
              @click="showAddModal = false"
            >
              <SIcon
                name="X"
                size="w-4 h-4"
                class="text-text-muted"
              />
            </button>
          </div>

          <div class="p-4 space-y-4">
            <div>
              <label class="block text-xs text-text-muted mb-1.5">{{ $t('skills.repositories.name') }}</label>
              <input
                v-model="newRepo.name"
                type="text"
                class="w-full px-3 py-2 text-sm rounded-lg bg-bg-base border border-border-default text-text-primary focus:outline-none focus:border-accent-success"
                placeholder="my-skills"
              >
            </div>
            <div>
              <label class="block text-xs text-text-muted mb-1.5">{{ $t('skills.repositories.url') }}</label>
              <input
                v-model="newRepo.url"
                type="text"
                class="w-full px-3 py-2 text-sm rounded-lg bg-bg-base border border-border-default text-text-primary focus:outline-none focus:border-accent-success"
                placeholder="https://github.com/owner/repo"
              >
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-xs text-text-muted mb-1.5">{{ $t('skills.repositories.branch') }}</label>
                <input
                  v-model="newRepo.branch"
                  type="text"
                  class="w-full px-3 py-2 text-sm rounded-lg bg-bg-base border border-border-default text-text-primary focus:outline-none focus:border-accent-success"
                  placeholder="main"
                >
              </div>
              <div>
                <label class="block text-xs text-text-muted mb-1.5">{{ $t('skills.repositories.description') }}</label>
                <input
                  v-model="newRepo.description"
                  type="text"
                  class="w-full px-3 py-2 text-sm rounded-lg bg-bg-base border border-border-default text-text-primary focus:outline-none focus:border-accent-success"
                  :placeholder="$t('common.optional')"
                >
              </div>
            </div>
          </div>

          <div class="flex items-center justify-end gap-3 p-4 border-t border-border-default/15">
            <button
              class="px-4 py-2 text-xs rounded-lg bg-bg-surface text-text-secondary"
              @click="showAddModal = false"
            >
              {{ $t('common.cancel') }}
            </button>
            <button
              class="px-4 py-2 text-xs rounded-lg bg-gradient-to-r from-accent-success to-accent-primary text-white font-medium"
              :disabled="!newRepo.name || !newRepo.url || adding"
              @click="addRepository"
            >
              <SIcon
                v-if="adding"
                name="Loader2"
                size="w-3.5 h-3.5"
                class="inline animate-spin mr-1.5"
              />
              {{ $t('skills.repositories.add') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { listSkillRepositories, addSkillRepository, removeSkillRepository, type SkillRepository } from '@/api'
import { logger } from '@/utils/logger'

const { t } = useI18n({ useScope: 'global' })

const emit = defineEmits<{
  (e: 'scan', repoName: string): void
}>()

const loading = ref(true)
const repositories = ref<SkillRepository[]>([])
const showAddModal = ref(false)
const adding = ref(false)
const scanningRepo = ref<string | null>(null)

const newRepo = ref({
  name: '',
  url: '',
  branch: 'main',
  description: ''
})

onMounted(async () => {
  await loadRepositories()
})

const loadRepositories = async () => {
  try {
    loading.value = true
    repositories.value = await listSkillRepositories()
  } catch (error) {
    logger.error('Failed to load repositories:', error)
  } finally {
    loading.value = false
  }
}

const addRepository = async () => {
  if (!newRepo.value.name || !newRepo.value.url) return

  try {
    adding.value = true
    await addSkillRepository({
      name: newRepo.value.name,
      url: newRepo.value.url,
      branch: newRepo.value.branch || 'main',
      description: newRepo.value.description || undefined
    })
    showAddModal.value = false
    newRepo.value = { name: '', url: '', branch: 'main', description: '' }
    await loadRepositories()
  } catch (error) {
    logger.error('Failed to add repository:', error)
  } finally {
    adding.value = false
  }
}

const removeRepository = async (name: string) => {
  if (!confirm(t('skills.repositories.confirmDelete', { name }))) return

  try {
    await removeSkillRepository(name)
    await loadRepositories()
  } catch (error) {
    logger.error('Failed to remove repository:', error)
  }
}

const scanRepository = async (name: string) => {
  scanningRepo.value = name
  emit('scan', name)
  // Simulate scan delay
  setTimeout(() => {
    scanningRepo.value = null
  }, 2000)
}

const formatTime = (timestamp: string) => {
  const date = new Date(timestamp)
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}
</script>

