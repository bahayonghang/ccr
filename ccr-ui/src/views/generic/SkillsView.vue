<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden transition-colors duration-500">
    <!-- Enhanced Background -->
    <AnimatedBackground
      contained
      variant="aurora"
    />

    <div class="max-w-[1600px] mx-auto space-y-8 relative z-10">
      <!-- HERO HEADER -->
      <header class="flex flex-col md:flex-row md:items-end justify-between gap-6 animate-slide-up">
        <div class="space-y-2">
          <div class="flex items-center gap-3 mb-1">
            <span class="px-2.5 py-1 rounded-md bg-accent-primary/10 border border-accent-primary/20 text-accent-primary text-xs font-bold uppercase tracking-wider backdrop-blur-md">
              Skills Hub
            </span>
          </div>
          <h1 class="text-4xl md:text-5xl font-bold font-display tracking-tight text-white flex items-center gap-4">
            {{ $t('skills.title') }}
            <span class="w-3 h-3 rounded-full bg-accent-secondary animate-pulse mt-2" />
          </h1>
          <p class="text-white/80 text-lg max-w-2xl">
            {{ $t('skills.help.description') }}
          </p>
        </div>

        <div class="flex items-center gap-3 pb-2">
          <RouterLink
            to="/claude-code"
          >
            <Button
              variant="ghost"
              class="group"
            >
              <SIcon
                name="ArrowLeft"
                size="w-4 h-4"
                class="mr-2 group-hover:-translate-x-1 transition-transform"
              />
              {{ $t('common.back') }}
            </Button>
          </RouterLink>
          <Button
            variant="primary" 
            class="shadow-lg shadow-accent-primary/20 hover:shadow-accent-primary/40"
            @click="handleAdd"
          >
            <SIcon
              name="Plus"
              size="w-5 h-5"
              class="mr-2"
            />
            <span class="font-bold">{{ $t('skills.addSkill') }}</span>
          </Button>
        </div>
      </header>

      <!-- SEARCH & FILTERS -->
      <section 
        class="sticky top-4 z-40 /85 backdrop-blur-md border border-white/10 p-2 rounded-2xl shadow-2xl animate-slide-up flex flex-col xl:flex-row gap-2"
        style="animation-delay: 100ms;"
      >
        <div class="relative flex-1 group min-w-[200px]">
          <SIcon
            name="Search"
            size="w-5 h-5"
            class="absolute left-4 top-1/2 -translate-y-1/2 text-white/50 group-focus-within:text-accent-primary transition-colors"
          />
          <input 
            v-model="searchQuery"
            type="text"
            class="w-full bg-transparent border-none text-white placeholder:text-white/50/50 pl-12 pr-4 py-3 focus:outline-none focus:ring-0 text-sm font-medium"
            :placeholder="$t('skills.searchPlaceholder')"
          >
          <div class="absolute right-3 top-1/2 -translate-y-1/2 flex gap-1">
            <span 
              v-if="filteredSkills.length"
              class="text-[10px] font-mono px-2 py-0.5 rounded glass-surface text-white/50 border border-white/20"
            >
              {{ filteredSkills.length }}
            </span>
            <button 
              v-if="searchQuery"
              class="p-1 hover:bg-white/5 rounded-full text-white/50 transition-colors"
              @click="searchQuery = ''"
            >
              <SIcon
                name="X"
                size="w-3 h-3"
              />
            </button>
          </div>
        </div>

        <div class="h-px xl:h-auto w-full xl:w-px bg-border-subtle mx-2" />

        <div class="flex flex-col md:flex-row gap-2 overflow-x-auto no-scrollbar items-start md:items-center px-2">
          <!-- Source Toggle -->
          <div class="flex bg-white/5/50 p-1 rounded-xl border border-white/5 shrink-0">
            <button
              v-for="opt in sourceOptions" 
              :key="opt.value"
              class="px-3 py-1.5 rounded-lg text-xs font-semibold transition-colors relative"
              :class="selectedSource === opt.value ? 'text-white shadow-sm' : 'text-white/80 hover:text-white'"
              @click="selectedSource = opt.value"
            >
              <div
                v-if="selectedSource === opt.value"
                class="absolute inset-0 glass-surface rounded-lg shadow-sm -z-10"
              />
              {{ opt.label }}
            </button>
          </div>

          <!-- Category Pills -->
          <div class="flex gap-2 pl-2 border-l border-white/5 shrink-0 overflow-x-auto no-scrollbar max-w-[400px]">
            <button
              v-for="cat in availableCategories"
              :key="cat"
              class="px-3 py-1.5 rounded-lg text-xs font-medium border transition-colors whitespace-nowrap"
              :class="selectedCategory === cat ? 'bg-accent-primary/10 border-accent-primary/30 text-accent-primary shadow-[0_0_10px_rgba(var(--color-accent-primary-rgb),0.2)]' : 'bg-white/5/30 border-transparent hover:bg-white/5 text-white/80'"
              @click="toggleCategory(cat)"
            >
              {{ formatCategory(cat) }}
            </button>
          </div>

          <!-- Tags Filter Trigger -->
          <div
            v-if="availableTags.length > 0"
            class="relative group ml-2"
          >
            <button 
              class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium border transition-colors"
              :class="selectedTags.length > 0 ? 'bg-accent-secondary/10 border-accent-secondary/30 text-accent-secondary' : 'bg-white/5/30 border-transparent hover:bg-white/5 text-white/80'"
              @click="showTagsFilter = !showTagsFilter"
            >
              <SIcon
                name="Filter"
                size="w-3.5 h-3.5"
              />
              {{ $t('skills.filter.tags') }}
              <span
                v-if="selectedTags.length"
                class="ml-1 text-[10px] bg-accent-secondary/20 px-1.5 rounded-full"
              >
                {{ selectedTags.length }}
              </span>
            </button>

            <!-- Tags Dropdown -->
            <div 
              v-if="showTagsFilter"
              class="absolute top-full right-0 mt-2 w-64 p-3 bg-white/5/90 backdrop-blur-xl border border-white/10 rounded-xl shadow-xl z-50 grid grid-cols-2 gap-2"
            >
              <div class="col-span-2 flex justify-between items-center mb-1 px-1">
                <span class="text-[10px] font-bold text-white/50 uppercase tracking-wider">Select Tags</span>
                <button 
                  v-if="selectedTags.length" 
                  class="text-[10px] text-accent-primary hover:text-accent-primary-hover" 
                  @click="selectedTags = []"
                >
                  Clear
                </button>
              </div>
              <button
                v-for="tag in availableTags" 
                :key="tag"
                class="text-xs px-2 py-1.5 rounded-lg text-left truncate transition-colors border"
                :class="selectedTags.includes(tag) ? 'bg-accent-secondary/20 border-accent-secondary/30 text-accent-secondary' : 'bg-white/5/50 border-transparent text-white/80 hover:bg-white/5'"
                @click="toggleTag(tag)"
              >
                #{{ tag }}
              </button>
            </div>
             
            <!-- Backdrop for tags dropdown -->
            <div 
              v-if="showTagsFilter" 
              class="fixed inset-0 z-40 bg-transparent" 
              @click="showTagsFilter = false"
            />
          </div>
        </div>
      </section>

      <!-- SKILLS GRID -->
      <section
        v-if="loading"
        class="py-20 flex justify-center"
      >
        <div class="w-8 h-8 rounded-full border-2 border-accent-primary border-t-transparent animate-spin" />
      </section>

      <div
        v-else-if="filteredSkills.length === 0"
        class="py-20 text-center animate-fade-in"
      >
        <div class="w-20 h-20 rounded-full bg-white/5/50 flex items-center justify-center mx-auto mb-4 backdrop-blur-md border border-white/5">
          <SIcon
            :name="searchQuery ? 'Search' : 'Book'"
            size="w-8 h-8"
            class="text-white/50"
          />
        </div>
        <h3 class="text-lg font-bold text-white">
          {{ searchQuery ? $t('skills.noSearchResults') : $t('skills.noSkills') }}
        </h3>
        <p class="text-white/80 text-sm mt-1 mb-4">
          {{ searchQuery ? $t('skills.noSearchResultsHint') : $t('skills.noSkillsHint') }}
        </p>
        <Button
          v-if="searchQuery"
          variant="outline"
          size="sm"
          @click="clearAllFilters"
        >
          {{ $t('skills.clearSearch') }}
        </Button>
      </div>

      <TransitionGroup 
        v-else
        tag="div" 
        name="staggered-grid"
        class="grid grid-cols-1 md:grid-cols-2 2xl:grid-cols-3 gap-6"
      >
        <Card
          v-for="(skill, index) in filteredSkills"
          :key="skill.name"
          variant="glass"
          hover
          glow
          class="group min-h-[280px] flex flex-col p-0 overflow-visible transition-colors duration-300 relative border-white/5"
          :style="{ animationDelay: `${index * 50}ms` }"
          @click="navigateToDetail(skill.name)"
        >
          <!-- Skill Category Decorator -->
          <div 
            class="absolute top-0 right-0 p-3 opacity-20 group-hover:opacity-100 transition-opacity pointer-events-none"
            :class="skill.repository ? 'text-accent-secondary' : 'text-accent-primary'"
          >
            <SIcon
              :name="getSkillIcon(skill.name)"
              size="w-24 h-24"
              class="-mt-8 -mr-8 opacity-10 rotate-12 group-hover:rotate-0 transition-transform duration-500"
            />
          </div>

          <!-- Card Header -->
          <div class="p-6 pb-2 flex items-start justify-between relative z-10">
            <div class="flex items-center gap-4">
              <div
                class="w-14 h-14 rounded-xl flex items-center justify-center text-2xl font-bold font-mono shadow-inner transition-colors duration-300"
                :class="skill.repository ? 'bg-accent-warning/10 text-accent-warning border border-accent-warning/20 group-hover:bg-accent-warning/20' : 'bg-accent-primary/10 text-accent-primary border border-accent-primary/20 group-hover:bg-accent-primary/20'"
              >
                {{ skill.name.charAt(0).toUpperCase() }}
              </div>
              <div>
                <h3
                  class="text-xl font-bold text-white line-clamp-1 group-hover:text-transparent group-hover:bg-clip-text group-hover:bg-gradient-to-r group-hover:from-accent-primary group-hover:to-accent-secondary transition-colors"
                  :title="skill.name"
                >
                  {{ skill.name }}
                </h3>
                <!-- Plugin/User Badge (Moved here) -->
                <div class="flex gap-2 mt-1">
                  <span 
                    v-if="skill.repository"
                    class="text-[10px] font-bold px-2 py-0.5 rounded bg-amber-500/10 text-amber-500 border border-amber-500/20"
                  >
                    PLUGIN
                  </span>
                  <span 
                    v-else
                    class="text-[10px] font-bold px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-500 border border-emerald-500/20"
                  >
                    USER
                  </span>
                </div>
              </div>
            </div>

            <!-- Actions (Visible on hover) -->
            <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-interactive duration-200 translate-x-2 group-hover:translate-x-0">
              <button
                v-if="!skill.repository"
                class="p-1.5 rounded-lg hover:bg-white/5 text-white/50 hover:text-white transition-colors"
                @click.stop="handleEdit(skill)"
              >
                <SIcon
                  name="Edit2"
                  size="w-4 h-4"
                />
              </button>
              <button
                v-if="!skill.repository"
                class="p-1.5 rounded-lg hover:bg-red-500/10 text-white/50 hover:text-red-500 transition-colors"
                @click.stop="handleDelete(skill.name)"
              >
                <SIcon
                  name="Trash2"
                  size="w-4 h-4"
                />
              </button>
            </div>
          </div>

          <!-- Card Body -->
          <div class="p-6 flex-1 relative z-10 flex flex-col">
            <p class="text-sm text-white/80 leading-relaxed line-clamp-4 mb-4 flex-1">
              {{ skill.description || $t('skills.search.noDescription') }}
            </p>

            <!-- Tags -->
            <div class="flex flex-wrap gap-1.5 mt-auto">
              <span 
                v-for="tag in (skill.metadata?.tags || []).slice(0, 4)"
                :key="tag"
                class="text-[10px] px-2 py-1 rounded glass-surface border border-white/20 text-white/50 hover:text-white transition-colors"
              >
                #{{ tag }}
              </span>
              <span
                v-if="(skill.metadata?.tags || []).length > 4"
                class="text-[10px] text-white/50 px-1 self-center"
              >
                +{{ (skill.metadata?.tags?.length || 0) - 4 }}
              </span>
            </div>
          </div>

          <!-- Card Footer (Path) -->
          <div class="px-6 py-3 border-t border-white/5 bg-transparent backdrop-blur-md text-[10px] text-white/50 font-mono truncate opacity-40 group-hover:opacity-80 transition-opacity">
            {{ skill.path }}
          </div>
        </Card>
      </TransitionGroup>


      <!-- ADD MODAL -->
      <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="opacity-0 scale-95"
        enter-to-class="opacity-100 scale-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="opacity-100 scale-100"
        leave-to-class="opacity-0 scale-95"
      >
        <div
          v-if="showModal"
          class="fixed inset-0 z-50 flex items-center justify-center p-4"
        >
          <div
            class="absolute inset-0 bg-black/60 backdrop-blur-md"
            @click="showModal = false"
          />
          
          <div class="relative w-full max-w-2xl /90 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
            <div class="p-6 border-b border-white/5 flex items-center justify-center justify-between">
              <h3 class="text-xl font-bold text-white flex items-center gap-3">
                <div class="w-8 h-8 rounded-lg bg-accent-primary/20 flex items-center justify-center text-accent-primary">
                  <SIcon
                    :name="editingSkill ? 'Edit2' : 'Plus'"
                    size="w-4 h-4"
                  />
                </div>
                {{ editingSkill ? $t('skills.editSkill') : $t('skills.addSkill') }}
              </h3>
              <button
                class="text-white/50 hover:text-white transition-colors"
                @click="showModal = false"
              >
                <SIcon
                  name="X"
                  size="w-5 h-5"
                />
              </button>
            </div>

            <div class="p-6 overflow-y-auto space-y-6 custom-scrollbar">
              <div class="space-y-2">
                <label class="text-sm font-semibold text-white/80">{{ $t('skills.nameLabel') }}</label>
                <input 
                  v-model="formData.name" 
                  :disabled="!!editingSkill"
                  type="text" 
                  placeholder="e.g. data-analysis-pro"
                  class="w-full px-4 py-3 rounded-xl bg-white/5/50 border border-white/10 focus:border-accent-primary focus:ring-1 focus:ring-accent-primary outline-none transition-colors font-mono text-sm"
                >
              </div>

              <div class="space-y-2">
                <div class="flex justify-between">
                  <label class="text-sm font-semibold text-white/80">{{ $t('skills.instructionLabel') }}</label>
                  <span class="text-xs text-white/50">Markdown supported</span>
                </div>
                <textarea 
                  v-model="formData.instruction"
                  rows="12"
                  class="w-full px-4 py-3 rounded-xl bg-white/5/50 border border-white/10 focus:border-accent-primary focus:ring-1 focus:ring-accent-primary outline-none transition-colors font-mono text-sm leading-relaxed custom-scrollbar"
                  placeholder="# Skill Name..."
                />
              </div>
            </div>

            <div class="p-6 border-t border-white/5 bg-white/5/30 flex justify-end gap-3">
              <Button
                variant="ghost"
                @click="showModal = false"
              >
                {{ $t('common.cancel') }}
              </Button>
              <Button
                variant="primary"
                @click="handleSubmit"
              >
                {{ editingSkill ? $t('common.save') : $t('common.add') }}
              </Button>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import { useSkills, type Skill } from '@/composables/useSkills'
import { logger } from '@/utils/logger'

const router = useRouter()
const { t } = useI18n()
const { skills, loading, listSkills, addSkill, updateSkill, deleteSkill } = useSkills()

// --- State ---
const showModal = ref(false)
const editingSkill = ref<Skill | null>(null)
const formData = ref({ name: '', instruction: '' })
const searchQuery = ref('')
const selectedCategory = ref<string | null>(null)
const selectedSource = ref<'all' | 'user' | 'plugin'>('all')
const selectedTags = ref<string[]>([])
const showTagsFilter = ref(false)

const sourceOptions = computed(() => [
  { label: t('skills.filter.allSources'), value: 'all' as const },
  { label: t('skills.filter.userSkill'), value: 'user' as const },
  { label: t('skills.filter.pluginSkill'), value: 'plugin' as const }
])

// --- Helpers ---
const availableCategories = computed(() => {
  const categories = new Set<string>()
  let hasUncategorized = false
  
  skills.value.forEach(skill => {
    if (skill.metadata?.category) {
      categories.add(skill.metadata.category)
    } else {
      hasUncategorized = true
    }
  })
  
  const sorted = Array.from(categories).sort()
  if (hasUncategorized) {
    sorted.push('uncategorized')
  }
  return sorted
})

const availableTags = computed(() => {
  const tags = new Set<string>()
  skills.value.forEach(skill => {
    skill.metadata?.tags?.forEach(tag => tags.add(tag))
  })
  return Array.from(tags).sort()
})

const formatCategory = (category: string) => {
  if (category === 'uncategorized') return t('skills.category.uncategorized')
  return category.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')
}

const toggleCategory = (cat: string) => {
  selectedCategory.value = selectedCategory.value === cat ? null : cat
}

const toggleTag = (tag: string) => {
  if (selectedTags.value.includes(tag)) {
    selectedTags.value = selectedTags.value.filter(t => t !== tag)
  } else {
    selectedTags.value = [...selectedTags.value, tag]
  }
}

// Icon mapper
const getSkillIcon = (name: string) => {
   const n = name.toLowerCase()
   if (n.includes('write') || n.includes('doc')) return 'PenTool'
   if (n.includes('data') || n.includes('sql')) return 'Database'
   if (n.includes('web') || n.includes('browser')) return 'Globe'
   if (n.includes('code') || n.includes('dev')) return 'Code2'
   if (n.includes('agent')) return 'Zap'
   return 'Box'
}

// --- Filtering ---
const filteredSkills = computed(() => {
  let result = skills.value

  // 1. Source Filter
  if (selectedSource.value === 'user') result = result.filter(s => !s.repository)
  else if (selectedSource.value === 'plugin') result = result.filter(s => !!s.repository)

  // 2. Category Filter
  if (selectedCategory.value) {
    if (selectedCategory.value === 'uncategorized') {
      result = result.filter(s => !s.metadata?.category)
    } else {
      result = result.filter(s => s.metadata?.category === selectedCategory.value)
    }
  }

  // 3. Tags Filter (OR logic - show if has ANY selected tag)
  if (selectedTags.value.length > 0) {
    result = result.filter(s => 
      s.metadata?.tags?.some(tag => selectedTags.value.includes(tag))
    )
  }

  // 4. Search Query
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(s => 
      s.name.toLowerCase().includes(q) || 
      (s.description?.toLowerCase().includes(q)) ||
      (s.metadata?.tags || []).some(t => t.toLowerCase().includes(q))
    )
  }
  return result
})

const clearAllFilters = () => {
  searchQuery.value = ''
  selectedCategory.value = null
  selectedSource.value = 'all'
  selectedTags.value = []
}

// --- Actions ---
const navigateToDetail = (name: string) => router.push(`/skills/${encodeURIComponent(name)}`)

const handleAdd = () => {
  editingSkill.value = null
  formData.value = { name: '', instruction: '' }
  showModal.value = true
}

const handleEdit = (skill: Skill) => {
  editingSkill.value = skill
  formData.value = { name: skill.name, instruction: skill.instruction || '' }
  showModal.value = true
}

const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.instruction) return
  
  try {
    if (editingSkill.value) {
      await updateSkill(editingSkill.value.name, { instruction: formData.value.instruction })
    } else {
      await addSkill({ name: formData.value.name, instruction: formData.value.instruction })
    }
    showModal.value = false
  } catch (err) {
    logger.error('Failed to submit skill form', err)
  }
}

const handleDelete = async (name: string) => {
  if (confirm(t('skills.deleteConfirm', { name }))) {
    await deleteSkill(name)
  }
}

onMounted(listSkills)
</script>

<style scoped>
/* Custom Scrollbar for Modal content */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgb(255 255 255 / 2%);
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgb(255 255 255 / 10%);
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgb(255 255 255 / 20%);
}

.no-scrollbar::-webkit-scrollbar {
  display: none;
}

.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
