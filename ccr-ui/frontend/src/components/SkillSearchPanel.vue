<template>
  <div class="glass-effect rounded-3xl p-6 border border-white/20">
    <!-- Header with Search -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-xl bg-gradient-to-br from-guofeng-purple/20 to-guofeng-indigo/20">
          <Sparkles class="w-5 h-5 text-guofeng-purple" />
        </div>
        <div>
          <h3 class="text-base font-bold text-guofeng-text-primary">
            {{ $t('skills.search.title') }}
          </h3>
          <p class="text-xs text-guofeng-text-muted">
            {{ skills.length }} {{ $t('skills.search.skillsFound') }}
          </p>
        </div>
      </div>

      <!-- Search Input -->
      <div class="flex items-center gap-3">
        <div class="relative">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-guofeng-text-muted" />
          <input
            v-model="searchQuery"
            type="text"
            class="w-64 pl-9 pr-4 py-2 text-sm rounded-xl bg-guofeng-bg-tertiary border border-guofeng-border text-guofeng-text-primary placeholder:text-guofeng-text-muted focus:outline-none focus:border-guofeng-purple"
            :placeholder="$t('skills.search.placeholder')"
          >
        </div>
        <button
          v-if="searchQuery || selectedTags.length > 0 || selectedRepo"
          class="p-2 rounded-lg hover:bg-guofeng-bg-tertiary text-guofeng-text-muted hover:text-guofeng-red transition-colors"
          @click="clearFilters"
        >
          <XCircle class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Filter Chips -->
    <div class="flex flex-wrap gap-2 mb-6">
      <!-- Repository Filter -->
      <div class="flex items-center gap-2">
        <span class="text-xs text-guofeng-text-muted">{{ $t('skills.search.repository') }}:</span>
        <button
          v-for="repo in availableRepos"
          :key="repo"
          class="px-2.5 py-1 text-xs rounded-lg transition-all"
          :class="selectedRepo === repo 
            ? 'bg-guofeng-purple/20 text-guofeng-purple border border-guofeng-purple/30' 
            : 'bg-guofeng-bg-tertiary text-guofeng-text-muted hover:text-guofeng-text-primary border border-transparent'"
          @click="toggleRepo(repo)"
        >
          {{ repo || $t('skills.search.local') }}
        </button>
      </div>

      <!-- Tag Filter -->
      <div
        v-if="availableTags.length > 0"
        class="flex items-center gap-2 ml-4"
      >
        <span class="text-xs text-guofeng-text-muted">{{ $t('skills.search.tags') }}:</span>
        <button
          v-for="tag in availableTags.slice(0, 8)"
          :key="tag"
          class="px-2.5 py-1 text-xs rounded-lg transition-all"
          :class="selectedTags.includes(tag) 
            ? 'bg-guofeng-indigo/20 text-guofeng-indigo border border-guofeng-indigo/30' 
            : 'bg-guofeng-bg-tertiary text-guofeng-text-muted hover:text-guofeng-text-primary border border-transparent'"
          @click="toggleTag(tag)"
        >
          {{ tag }}
        </button>
        <span
          v-if="availableTags.length > 8"
          class="text-xs text-guofeng-text-muted"
        >
          +{{ availableTags.length - 8 }}
        </span>
      </div>
    </div>

    <!-- Loading -->
    <div
      v-if="loading"
      class="flex justify-center py-8"
    >
      <Loader2 class="w-6 h-6 animate-spin text-guofeng-purple" />
    </div>

    <!-- Empty State -->
    <div
      v-else-if="filteredSkills.length === 0"
      class="text-center py-8"
    >
      <Search class="w-12 h-12 mx-auto text-guofeng-text-muted/30 mb-3" />
      <p class="text-sm text-guofeng-text-muted">
        {{ $t('skills.search.noResults') }}
      </p>
      <p
        v-if="searchQuery || selectedTags.length > 0"
        class="text-xs text-guofeng-text-muted/70 mt-1"
      >
        {{ $t('skills.search.tryDifferent') }}
      </p>
    </div>

    <!-- Skills Grid -->
    <div
      v-else
      class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
    >
      <div
        v-for="skill in filteredSkills"
        :key="skill.name"
        class="group p-4 rounded-xl bg-guofeng-bg-tertiary/50 border border-transparent hover:border-guofeng-purple/30 cursor-pointer transition-all"
        @click="selectSkill(skill)"
      >
        <div class="flex items-start justify-between mb-2">
          <div class="flex items-center gap-2">
            <component 
              :is="skill.is_remote ? Cloud : HardDrive" 
              class="w-4 h-4" 
              :class="skill.is_remote ? 'text-guofeng-cyan' : 'text-guofeng-emerald'"
            />
            <span class="text-sm font-bold text-guofeng-text-primary">{{ skill.name }}</span>
          </div>
          <button
            v-if="!skill.is_remote"
            class="p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-guofeng-red/10 text-guofeng-text-muted hover:text-guofeng-red transition-all"
            @click.stop="deleteSkill(skill.name)"
          >
            <Trash2 class="w-3.5 h-3.5" />
          </button>
        </div>

        <p class="text-xs text-guofeng-text-muted line-clamp-2 mb-3">
          {{ skill.description || $t('skills.search.noDescription') }}
        </p>

        <!-- Metadata -->
        <div class="flex flex-wrap items-center gap-2">
          <span 
            v-if="skill.metadata?.author" 
            class="text-[10px] px-1.5 py-0.5 rounded bg-guofeng-bg-primary text-guofeng-text-muted"
          >
            {{ skill.metadata.author }}
          </span>
          <span 
            v-for="tag in (skill.metadata?.tags || []).slice(0, 3)" 
            :key="tag"
            class="text-[10px] px-1.5 py-0.5 rounded bg-guofeng-indigo/10 text-guofeng-indigo"
          >
            {{ tag }}
          </span>
          <span 
            v-if="skill.repository" 
            class="text-[10px] px-1.5 py-0.5 rounded bg-guofeng-cyan/10 text-guofeng-cyan"
          >
            {{ skill.repository }}
          </span>
        </div>
      </div>
    </div>

    <!-- Skill Detail Modal -->
    <Teleport to="body">
      <div
        v-if="selectedSkillData"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
        @click.self="selectedSkillData = null"
      >
        <div class="w-full max-w-2xl max-h-[80vh] m-4 bg-guofeng-bg-secondary rounded-2xl border border-white/10 overflow-hidden flex flex-col">
          <div class="flex items-center justify-between p-4 border-b border-white/10">
            <div class="flex items-center gap-3">
              <component 
                :is="selectedSkillData.is_remote ? Cloud : HardDrive" 
                class="w-5 h-5"
                :class="selectedSkillData.is_remote ? 'text-guofeng-cyan' : 'text-guofeng-emerald'"
              />
              <div>
                <h3 class="text-base font-bold text-guofeng-text-primary">
                  {{ selectedSkillData.name }}
                </h3>
                <p class="text-xs text-guofeng-text-muted">
                  {{ selectedSkillData.path }}
                </p>
              </div>
            </div>
            <button
              class="p-2 rounded-lg hover:bg-guofeng-bg-tertiary"
              @click="selectedSkillData = null"
            >
              <X class="w-4 h-4 text-guofeng-text-muted" />
            </button>
          </div>
          <div class="flex-1 overflow-y-auto p-4">
            <pre class="text-xs text-guofeng-text-secondary whitespace-pre-wrap font-mono bg-guofeng-bg-primary/50 p-4 rounded-xl">{{ selectedSkillData.instruction }}</pre>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Sparkles,
  Search,
  XCircle,
  Loader2,
  Cloud,
  HardDrive,
  Trash2,
  X
} from 'lucide-vue-next'
import { listSkills, deleteSkill as apiDeleteSkill, type Skill } from '@/api/client'

const { t } = useI18n({ useScope: 'global' })

const loading = ref(true)
const skills = ref<Skill[]>([])
const searchQuery = ref('')
const selectedTags = ref<string[]>([])
const selectedRepo = ref<string | null>(null)
const selectedSkillData = ref<Skill | null>(null)

// 可用的仓库列表
const availableRepos = computed(() => {
  const repos = new Set<string>()
  repos.add('') // 本地
  skills.value.forEach(s => {
    if (s.repository) repos.add(s.repository)
  })
  return Array.from(repos)
})

// 可用的标签列表
const availableTags = computed(() => {
  const tags = new Set<string>()
  skills.value.forEach(s => {
    s.metadata?.tags?.forEach(tag => tags.add(tag))
  })
  return Array.from(tags).sort()
})

// 过滤后的技能列表
const filteredSkills = computed(() => {
  return skills.value.filter(skill => {
    // 搜索查询过滤
    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase()
      const matchName = skill.name.toLowerCase().includes(query)
      const matchDesc = skill.description?.toLowerCase().includes(query)
      const matchAuthor = skill.metadata?.author?.toLowerCase().includes(query)
      const matchTags = skill.metadata?.tags?.some(t => t.toLowerCase().includes(query))
      if (!matchName && !matchDesc && !matchAuthor && !matchTags) return false
    }

    // 仓库过滤
    if (selectedRepo.value !== null) {
      if (selectedRepo.value === '') {
        // 本地技能
        if (skill.is_remote) return false
      } else {
        if (skill.repository !== selectedRepo.value) return false
      }
    }

    // 标签过滤
    if (selectedTags.value.length > 0) {
      const skillTags = skill.metadata?.tags || []
      if (!selectedTags.value.some(t => skillTags.includes(t))) return false
    }

    return true
  })
})

onMounted(async () => {
  await loadSkills()
})

const loadSkills = async () => {
  try {
    loading.value = true
    skills.value = await listSkills()
  } catch (error) {
    console.error('Failed to load skills:', error)
  } finally {
    loading.value = false
  }
}

const toggleTag = (tag: string) => {
  const index = selectedTags.value.indexOf(tag)
  if (index === -1) {
    selectedTags.value.push(tag)
  } else {
    selectedTags.value.splice(index, 1)
  }
}

const toggleRepo = (repo: string) => {
  selectedRepo.value = selectedRepo.value === repo ? null : repo
}

const clearFilters = () => {
  searchQuery.value = ''
  selectedTags.value = []
  selectedRepo.value = null
}

const selectSkill = (skill: Skill) => {
  selectedSkillData.value = skill
}

const deleteSkill = async (name: string) => {
  if (!confirm(t('skills.deleteConfirm', { name }))) return
  try {
    await apiDeleteSkill(name)
    await loadSkills()
  } catch (error) {
    console.error('Failed to delete skill:', error)
  }
}
</script>
