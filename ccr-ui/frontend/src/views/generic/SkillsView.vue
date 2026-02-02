<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden transition-colors duration-500">
    <!-- Enhanced Background -->
    <AnimatedBackground variant="aurora" />

    <div class="max-w-[1600px] mx-auto space-y-8 relative z-10">
      
      <!-- HERO HEADER -->
      <header class="flex flex-col md:flex-row md:items-end justify-between gap-6 animate-slide-up">
        <div class="space-y-2">
          <div class="flex items-center gap-3 mb-1">
            <span class="px-2.5 py-1 rounded-md bg-accent-primary/10 border border-accent-primary/20 text-accent-primary text-xs font-bold uppercase tracking-wider backdrop-blur-md">
              Skills Hub
            </span>
          </div>
          <h1 class="text-4xl md:text-5xl font-bold font-display tracking-tight text-text-primary flex items-center gap-4">
            {{ $t('skills.title') }}
            <span class="w-3 h-3 rounded-full bg-accent-secondary animate-pulse mt-2"></span>
          </h1>
          <p class="text-text-secondary text-lg max-w-2xl">
            {{ $t('skills.help.description') }}
          </p>
        </div>

        <div class="flex items-center gap-3 pb-2">
           <RouterLink
            to="/claude-code"
          >
            <Button variant="ghost" class="group">
              <ArrowLeft class="w-4 h-4 mr-2 group-hover:-translate-x-1 transition-transform" />
              {{ $t('common.back') }}
            </Button>
          </RouterLink>
          <Button
            variant="solid" 
            class="shadow-lg shadow-accent-primary/20 hover:shadow-accent-primary/40"
            @click="handleAdd"
          >
            <Plus class="w-5 h-5 mr-2" />
            <span class="font-bold">{{ $t('skills.addSkill') }}</span>
          </Button>
        </div>
      </header>

      <!-- SEARCH & FILTERS -->
      <section 
        class="sticky top-4 z-40 bg-bg-base/60 backdrop-blur-xl border border-white/10 p-2 rounded-2xl shadow-2xl animate-slide-up flex flex-col md:flex-row gap-2"
        style="animation-delay: 100ms;"
      >
        <div class="relative flex-1 group">
          <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-text-muted group-focus-within:text-accent-primary transition-colors" />
          <input 
            v-model="searchQuery"
            type="text"
            class="w-full bg-transparent border-none text-text-primary placeholder:text-text-muted/50 pl-12 pr-4 py-3 focus:outline-none focus:ring-0 text-sm font-medium"
            :placeholder="$t('skills.searchPlaceholder')"
          />
          <div class="absolute right-3 top-1/2 -translate-y-1/2 flex gap-1">
            <span 
              v-if="filteredSkills.length"
              class="text-[10px] font-mono px-2 py-0.5 rounded bg-bg-elevated text-text-muted border border-border-default"
            >
              {{ filteredSkills.length }}
            </span>
            <button 
              v-if="searchQuery"
              @click="searchQuery = ''"
              class="p-1 hover:bg-bg-elevated rounded-full text-text-muted transition-colors"
            >
              <X class="w-3 h-3" />
            </button>
          </div>
        </div>

        <div class="h-px md:h-full w-full md:w-px bg-border-subtle mx-2" />

        <div class="flex items-center gap-2 px-2 overflow-x-auto no-scrollbar">
          <!-- Source Toggle -->
          <div class="flex bg-bg-surface/50 p-1 rounded-xl border border-border-subtle shrink-0">
             <button
              v-for="opt in sourceOptions" 
              :key="opt.value"
              @click="selectedSource = opt.value"
              class="px-3 py-1.5 rounded-lg text-xs font-semibold transition-all relative"
              :class="selectedSource === opt.value ? 'text-text-primary shadow-sm' : 'text-text-secondary hover:text-text-primary'"
             >
                <div v-if="selectedSource === opt.value" class="absolute inset-0 bg-bg-elevated rounded-lg shadow-sm -z-10" />
                {{ opt.label }}
             </button>
          </div>

          <!-- Category Pills -->
          <div class="flex gap-2 pl-2 border-l border-border-subtle shrink-0">
            <button
               v-for="cat in availableCategories"
               :key="cat"
               @click="toggleCategory(cat)"
               class="px-3 py-1.5 rounded-lg text-xs font-medium border transition-all whitespace-nowrap"
               :class="selectedCategory === cat 
                 ? 'bg-accent-primary/10 border-accent-primary/30 text-accent-primary shadow-[0_0_10px_rgba(var(--color-accent-primary-rgb),0.2)]' 
                 : 'bg-bg-surface/30 border-transparent hover:bg-bg-surface text-text-secondary'"
            >
              {{ formatCategory(cat) }}
            </button>
          </div>
        </div>
      </section>

      <!-- SKILLS GRID -->
      <section v-if="loading" class="py-20 flex justify-center">
         <div class="w-8 h-8 rounded-full border-2 border-accent-primary border-t-transparent animate-spin" />
      </section>

      <div v-else-if="filteredSkills.length === 0" class="py-20 text-center animate-fade-in">
        <div class="w-20 h-20 rounded-full bg-bg-elevated/50 flex items-center justify-center mx-auto mb-4 backdrop-blur-md border border-white/5">
          <component :is="searchQuery ? Search : Book" class="w-8 h-8 text-text-muted" />
        </div>
        <h3 class="text-lg font-bold text-text-primary">{{ searchQuery ? $t('skills.noSearchResults') : $t('skills.noSkills') }}</h3>
        <p class="text-text-secondary text-sm mt-1 mb-4">{{ searchQuery ? $t('skills.noSearchResultsHint') : $t('skills.noSkillsHint') }}</p>
        <Button v-if="searchQuery" variant="outline" size="sm" @click="clearAllFilters">
          {{ $t('skills.clearSearch') }}
        </Button>
      </div>

      <TransitionGroup 
        v-else
        tag="div" 
        name="staggered-grid"
        class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5"
      >
        <Card
          v-for="(skill, index) in filteredSkills"
          :key="skill.name"
          variant="glass"
          hover
          glow
          class="group min-h-[220px] flex flex-col p-0 overflow-visible transition-all duration-300 relative border-white/5"
          :style="{ animationDelay: `${index * 50}ms` }"
          @click="navigateToDetail(skill.name)"
        >
          <!-- Skill Category Decorator -->
          <div 
             class="absolute top-0 right-0 p-3 opacity-20 group-hover:opacity-100 transition-opacity pointer-events-none"
             :class="skill.repository ? 'text-accent-secondary' : 'text-accent-primary'"
          >
             <component :is="getSkillIcon(skill.name)" class="w-24 h-24 -mt-8 -mr-8 opacity-10 rotate-12 group-hover:rotate-0 transition-transform duration-500" />
          </div>

          <!-- Card Header -->
          <div class="p-5 pb-0 flex items-start justify-between relative z-10">
            <div class="w-12 h-12 rounded-xl flex items-center justify-center text-xl font-bold font-mono shadow-inner transition-colors duration-300"
               :class="skill.repository 
                 ? 'bg-accent-warning/10 text-accent-warning border border-accent-warning/20 group-hover:bg-accent-warning/20' 
                 : 'bg-accent-primary/10 text-accent-primary border border-accent-primary/20 group-hover:bg-accent-primary/20'"
            >
              {{ skill.name.charAt(0).toUpperCase() }}
            </div>

            <!-- Actions (Visible on hover) -->
            <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-all duration-200 translate-x-2 group-hover:translate-x-0">
               <button v-if="!skill.repository" @click.stop="handleEdit(skill)" class="p-1.5 rounded-lg hover:bg-bg-elevated text-text-muted hover:text-text-primary transition-colors">
                  <Edit2 class="w-4 h-4" />
               </button>
               <button v-if="!skill.repository" @click.stop="handleDelete(skill.name)" class="p-1.5 rounded-lg hover:bg-red-500/10 text-text-muted hover:text-red-500 transition-colors">
                  <Trash2 class="w-4 h-4" />
               </button>
            </div>
          </div>

          <!-- Card Body -->
          <div class="p-5 flex-1 relative z-10 flex flex-col">
            <h3 class="text-lg font-bold text-text-primary mb-2 line-clamp-1 group-hover:text-transparent group-hover:bg-clip-text group-hover:bg-gradient-to-r group-hover:from-accent-primary group-hover:to-accent-secondary transition-all">
              {{ skill.name }}
            </h3>
            
            <p class="text-sm text-text-secondary leading-relaxed line-clamp-2 mb-4 flex-1">
              {{ skill.description || $t('skills.search.noDescription') }}
            </p>

            <!-- Tags -->
            <div class="flex flex-wrap gap-1.5 mt-auto">
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
               <span 
                  v-for="tag in (skill.metadata?.tags || []).slice(0, 2)"
                  :key="tag"
                  class="text-[10px] px-2 py-0.5 rounded bg-bg-surface border border-border-default text-text-muted"
               >
                  #{{ tag }}
               </span>
            </div>
          </div>

          <!-- Card Footer (Path) -->
          <div class="px-5 py-3 border-t border-white/5 bg-black/10 backdrop-blur-sm text-[10px] text-text-muted font-mono truncate opacity-60 group-hover:opacity-100 transition-opacity">
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
        <div v-if="showModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="showModal = false" />
          
          <div class="relative w-full max-w-2xl bg-bg-base/90 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
            <div class="p-6 border-b border-white/5 flex items-center justify-between">
               <h3 class="text-xl font-bold text-text-primary flex items-center gap-3">
                 <div class="w-8 h-8 rounded-lg bg-accent-primary/20 flex items-center justify-center text-accent-primary">
                   <component :is="editingSkill ? Edit2 : Plus" class="w-4 h-4" />
                 </div>
                 {{ editingSkill ? $t('skills.editSkill') : $t('skills.addSkill') }}
               </h3>
               <button @click="showModal = false" class="text-text-muted hover:text-text-primary transition-colors">
                 <X class="w-5 h-5" />
               </button>
            </div>

            <div class="p-6 overflow-y-auto space-y-6 custom-scrollbar">
              <div class="space-y-2">
                 <label class="text-sm font-semibold text-text-secondary">{{ $t('skills.nameLabel') }}</label>
                 <input 
                   v-model="formData.name" 
                   :disabled="!!editingSkill"
                   type="text" 
                   placeholder="e.g. data-analysis-pro"
                   class="w-full px-4 py-3 rounded-xl bg-bg-surface/50 border border-border-default focus:border-accent-primary focus:ring-1 focus:ring-accent-primary outline-none transition-all font-mono text-sm"
                 />
              </div>

              <div class="space-y-2">
                 <div class="flex justify-between">
                    <label class="text-sm font-semibold text-text-secondary">{{ $t('skills.instructionLabel') }}</label>
                    <span class="text-xs text-text-muted">Markdown supported</span>
                 </div>
                 <textarea 
                   v-model="formData.instruction"
                   rows="12"
                   class="w-full px-4 py-3 rounded-xl bg-bg-surface/50 border border-border-default focus:border-accent-primary focus:ring-1 focus:ring-accent-primary outline-none transition-all font-mono text-sm leading-relaxed custom-scrollbar"
                   placeholder="# Skill Name..."
                 />
              </div>
            </div>

            <div class="p-6 border-t border-white/5 bg-bg-surface/30 flex justify-end gap-3">
               <Button variant="ghost" @click="showModal = false">
                 {{ $t('common.cancel') }}
               </Button>
               <Button variant="solid" @click="handleSubmit">
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
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { 
  Book, Plus, Edit2, Trash2, X, ArrowLeft, Search, 
  Tag, Layers, Code2, Zap, Terminal, Box, Feather,
  Cpu, Globe, Database, PenTool
} from 'lucide-vue-next'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import { useSkills, type Skill } from '@/composables/useSkills'

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

const sourceOptions = computed(() => [
  { label: t('skills.filter.allSources'), value: 'all' },
  { label: t('skills.filter.userSkill'), value: 'user' },
  { label: t('skills.filter.pluginSkill'), value: 'plugin' }
])

// --- Helpers ---
const availableCategories = computed(() => {
  const categories = new Set<string>()
  skills.value.forEach(skill => {
    if (skill.metadata?.category) categories.add(skill.metadata.category)
  })
  return Array.from(categories).sort()
})

const formatCategory = (category: string) => {
  return category.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')
}

const toggleCategory = (cat: string) => {
  selectedCategory.value = selectedCategory.value === cat ? null : cat
}

// Icon mapper
const getSkillIcon = (name: string) => {
   const n = name.toLowerCase()
   if (n.includes('write') || n.includes('doc')) return PenTool
   if (n.includes('data') || n.includes('sql')) return Database
   if (n.includes('web') || n.includes('browser')) return Globe
   if (n.includes('code') || n.includes('dev')) return Code2
   if (n.includes('agent')) return Zap
   return Box
}

// --- Filtering ---
const filteredSkills = computed(() => {
  let result = skills.value

  if (selectedSource.value === 'user') result = result.filter(s => !s.repository)
  else if (selectedSource.value === 'plugin') result = result.filter(s => !!s.repository)

  if (selectedCategory.value) {
    result = result.filter(s => s.metadata?.category === selectedCategory.value)
  }

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
    console.error(err)
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
  background: rgba(255,255,255,0.02);
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255,255,255,0.1);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255,255,255,0.2);
}

.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>
