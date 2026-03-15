<template>
  <div class="glass-effect rounded-3xl p-6 border border-white/20">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-xl bg-gradient-to-br from-accent-secondary/20 to-accent-primary/20">
          <SIcon
            name="FileText"
            size="w-5 h-5"
            class="text-accent-secondary"
          />
        </div>
        <div>
          <h3 class="text-base font-bold text-text-primary">
            {{ $t('prompts.builtin.title') }}
          </h3>
          <p class="text-xs text-text-muted">
            {{ $t('prompts.builtin.subtitle') }}
          </p>
        </div>
      </div>
      <button
        class="text-xs px-3 py-1.5 flex items-center gap-1.5 rounded-lg transition-colors"
        :class="isExpanded 
          ? 'bg-accent-secondary/10 text-accent-secondary' 
          : 'bg-bg-surface text-text-secondary hover:text-accent-secondary'"
        @click="isExpanded = !isExpanded"
      >
        <SIcon
          :name="isExpanded ? 'ChevronUp' : 'ChevronDown'"
          size="w-3.5 h-3.5"
        />
        {{ isExpanded ? $t('common.collapse') : $t('common.expand') }}
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
        class="animate-spin text-accent-secondary"
      />
    </div>

    <!-- Prompts Grid (Collapsed View) -->
    <div
      v-else-if="!isExpanded"
      class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3"
    >
      <div
        v-for="prompt in prompts"
        :key="prompt.id"
        class="group p-3 rounded-xl bg-bg-surface/50 border border-transparent hover:border-accent-secondary/30 cursor-pointer transition-colors"
        @click="selectPrompt(prompt)"
      >
        <div class="flex items-center gap-2 mb-1.5">
          <SIcon
            :name="getCategoryIcon(prompt.category)"
            size="w-4 h-4"
            class="text-accent-secondary"
          />
          <span class="text-xs font-medium text-text-primary truncate">{{ prompt.name }}</span>
        </div>
        <p class="text-[10px] text-text-muted line-clamp-2">
          {{ prompt.description }}
        </p>
      </div>
    </div>

    <!-- Prompts Grid (Expanded View) -->
    <div
      v-else
      class="space-y-4"
    >
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="prompt in prompts"
          :key="prompt.id"
          class="group p-4 rounded-xl bg-bg-surface/50 border border-transparent hover:border-accent-secondary/30 cursor-pointer transition-colors"
          @click="selectPrompt(prompt)"
        >
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <SIcon
                :name="getCategoryIcon(prompt.category)"
                size="w-5 h-5"
                class="text-accent-secondary"
              />
              <span class="text-sm font-bold text-text-primary">{{ prompt.name }}</span>
            </div>
            <span class="text-[10px] px-2 py-0.5 rounded-full bg-accent-secondary/10 text-accent-secondary">
              {{ getCategoryLabel(prompt.category) }}
            </span>
          </div>
          <p class="text-xs text-text-muted mb-3">
            {{ prompt.description }}
          </p>
          <div class="flex flex-wrap gap-1">
            <span
              v-for="tag in prompt.tags"
              :key="tag"
              class="text-[10px] px-1.5 py-0.5 rounded bg-bg-base text-text-muted"
            >
              {{ tag }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Preview Modal -->
    <Teleport to="body">
      <div
        v-if="selectedPrompt"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-md"
        @click.self="selectedPrompt = null"
      >
        <div class="w-full max-w-3xl max-h-[80vh] m-4 bg-bg-elevated rounded-2xl border border-white/10 overflow-hidden flex flex-col">
          <!-- Modal Header -->
          <div class="flex items-center justify-between p-4 border-b border-white/10">
            <div class="flex items-center gap-3">
              <SIcon
                :name="getCategoryIcon(selectedPrompt.category)"
                size="w-5 h-5"
                class="text-accent-secondary"
              />
              <div>
                <h3 class="text-base font-bold text-text-primary">
                  {{ selectedPrompt.name }}
                </h3>
                <p class="text-xs text-text-muted">
                  {{ selectedPrompt.description }}
                </p>
              </div>
            </div>
            <button
              class="p-2 rounded-lg hover:bg-bg-surface transition-colors"
              @click="selectedPrompt = null"
            >
              <SIcon
                name="X"
                size="w-4 h-4"
                class="text-text-muted"
              />
            </button>
          </div>

          <!-- Modal Content -->
          <div class="flex-1 overflow-y-auto p-4">
            <pre class="text-xs text-text-secondary whitespace-pre-wrap font-mono bg-bg-base/50 p-4 rounded-xl">{{ selectedPrompt.content }}</pre>
          </div>

          <!-- Modal Footer -->
          <div class="flex items-center justify-end gap-3 p-4 border-t border-white/10">
            <button
              class="px-4 py-2 text-xs rounded-lg bg-bg-surface text-text-secondary hover:text-text-primary transition-colors"
              @click="copyToClipboard(selectedPrompt.content)"
            >
              <SIcon
                name="Copy"
                size="w-3.5 h-3.5"
                class="inline mr-1.5"
              />
              {{ $t('common.copy') }}
            </button>
            <button
              class="px-4 py-2 text-xs rounded-lg bg-gradient-to-r from-accent-secondary to-accent-primary text-white font-medium"
              @click="applyPrompt(selectedPrompt)"
            >
              <SIcon
                name="Check"
                size="w-3.5 h-3.5"
                class="inline mr-1.5"
              />
              {{ $t('prompts.builtin.apply') }}
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
import { listBuiltinPrompts, type BuiltinPrompt } from '@/api'
import { copyToClipboard } from '@/utils/codexHelpers'
import { logger } from '@/utils/logger'

const { t } = useI18n({ useScope: 'global' })

const emit = defineEmits<{
  (e: 'apply', prompt: BuiltinPrompt): void
}>()

const loading = ref(true)
const prompts = ref<BuiltinPrompt[]>([])
const isExpanded = ref(false)
const selectedPrompt = ref<BuiltinPrompt | null>(null)

onMounted(async () => {
  try {
    prompts.value = await listBuiltinPrompts()
  } catch (error) {
    logger.error('Failed to load builtin prompts:', error)
  } finally {
    loading.value = false
  }
})

const getCategoryIcon = (category: string) => {
  const icons: Record<string, string> = {
    code_review: 'Code',
    debugging: 'Bug',
    refactoring: 'RefreshCw',
    testing: 'TestTube',
    documentation: 'FileCode',
    security: 'Shield'
  }
  return icons[category] || 'FileText'
}

const getCategoryLabel = (category: string) => {
  const labels: Record<string, string> = {
    code_review: t('prompts.builtin.categories.codeReview'),
    debugging: t('prompts.builtin.categories.debugging'),
    refactoring: t('prompts.builtin.categories.refactoring'),
    testing: t('prompts.builtin.categories.testing'),
    documentation: t('prompts.builtin.categories.documentation'),
    security: t('prompts.builtin.categories.security')
  }
  return labels[category] || category
}

const selectPrompt = (prompt: BuiltinPrompt) => {
  selectedPrompt.value = prompt
}


const applyPrompt = (prompt: BuiltinPrompt) => {
  emit('apply', prompt)
  selectedPrompt.value = null
}
</script>
