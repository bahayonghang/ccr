<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="max-w-[1600px] mx-auto">
      <!-- Breadcrumb -->
      <Breadcrumb :items="breadcrumbs" />

      <!-- Header -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div class="flex items-center gap-4">
          <h2 class="text-xl sm:text-2xl font-bold text-guofeng-text-primary flex items-center">
            <Palette
              class="w-6 h-6 sm:w-7 sm:h-7 mr-2 text-guofeng-indigo"
              aria-hidden="true"
            />
            {{ $t('outputStyles.pageTitle') }}
          </h2>
          <span
            class="px-3 py-1 rounded-full text-sm font-medium bg-guofeng-indigo/10 text-guofeng-indigo border border-guofeng-indigo/20"
            :aria-label="$t('outputStyles.totalCount', { count: outputStyles.length })"
          >
            {{ outputStyles.length }}
          </span>
        </div>
        <button
          class="w-full sm:w-auto px-4 py-2 rounded-lg font-medium transition-all hover:scale-105 bg-guofeng-indigo text-white shadow-md hover:shadow-lg flex items-center justify-center min-h-[44px]"
          :aria-label="$t('outputStyles.addStyle')"
          @click="handleAdd"
        >
          <Plus
            class="w-5 h-5 mr-2"
            aria-hidden="true"
          />{{ $t('outputStyles.addStyle') }}
        </button>
      </div>

      <!-- Search Bar -->
      <div class="mb-6">
        <div class="relative max-w-md">
          <Search
            class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-guofeng-text-muted"
            aria-hidden="true"
          />
          <input
            v-model="searchQuery"
            type="search"
            :placeholder="$t('outputStyles.searchPlaceholder')"
            :aria-label="$t('outputStyles.searchPlaceholder')"
            class="w-full pl-10 pr-10 py-2.5 rounded-xl transition-all focus:outline-none focus:ring-2 focus:ring-guofeng-indigo/20 bg-guofeng-bg-tertiary/50 border border-guofeng-border hover:bg-guofeng-bg-tertiary text-guofeng-text-primary placeholder-guofeng-text-muted text-sm"
          >
          <button
            v-if="searchQuery"
            class="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 rounded-full hover:bg-black/10 text-guofeng-text-muted transition-all min-h-[28px] min-w-[28px] flex items-center justify-center"
            :aria-label="$t('common.clearSearch')"
            @click="searchQuery = ''"
          >
            <X
              class="w-3 h-3"
              aria-hidden="true"
            />
          </button>
        </div>
      </div>

      <!-- Loading State -->
      <div
        v-if="loading"
        class="text-center py-20 text-guofeng-text-muted"
        role="status"
        aria-live="polite"
      >
        <div
          class="loading-spinner mx-auto mb-4 w-8 h-8 border-guofeng-indigo/30 border-t-guofeng-indigo"
          aria-hidden="true"
        />
        <span>{{ $t('common.loading') }}</span>
      </div>

      <!-- Empty State -->
      <div
        v-else-if="filteredStyles.length === 0"
        class="text-center py-20 text-guofeng-text-muted"
        role="status"
        aria-live="polite"
      >
        <div class="bg-guofeng-bg-secondary w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
          <Palette
            class="w-10 h-10 opacity-50"
            aria-hidden="true"
          />
        </div>
        <p class="text-lg font-medium">
          {{ searchQuery ? $t('outputStyles.noResults') : $t('outputStyles.noStyles') }}
        </p>
        <p
          v-if="!searchQuery"
          class="text-sm mt-2 text-guofeng-text-muted"
        >
          {{ $t('outputStyles.noStylesHint') }}
        </p>
      </div>

      <!-- Styles Grid -->
      <div
        v-else
        class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4"
        role="list"
        :aria-label="$t('outputStyles.stylesList')"
      >
        <GuofengCard
          v-for="style in filteredStyles"
          :key="style.name"
          variant="glass"
          interactive
          pattern
          role="listitem"
          tabindex="0"
          @click="handleView(style)"
          @keydown.enter="handleView(style)"
        >
          <div class="relative z-10">
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-2">
                <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-guofeng-indigo/10 to-guofeng-blue/10 flex items-center justify-center text-lg shadow-sm border border-white/20">
                  <Palette
                    class="w-5 h-5 text-guofeng-indigo"
                    aria-hidden="true"
                  />
                </div>
                <h3 class="text-lg font-bold text-guofeng-text-primary">
                  {{ style.name }}
                </h3>
              </div>
              <div class="flex gap-1">
                <button
                  class="p-1.5 rounded-md text-guofeng-text-secondary hover:text-guofeng-indigo hover:bg-guofeng-indigo/10 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                  :aria-label="$t('common.view') + ': ' + style.name"
                  @click.stop="handleView(style)"
                >
                  <Eye
                    class="w-4 h-4"
                    aria-hidden="true"
                  />
                </button>
                <button
                  class="p-1.5 rounded-md text-guofeng-text-secondary hover:text-guofeng-blue hover:bg-guofeng-blue/10 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                  :aria-label="$t('common.edit') + ': ' + style.name"
                  @click.stop="handleEdit(style)"
                >
                  <Edit2
                    class="w-4 h-4"
                    aria-hidden="true"
                  />
                </button>
                <button
                  class="p-1.5 rounded-md text-guofeng-text-secondary hover:text-guofeng-red hover:bg-guofeng-red/10 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                  :aria-label="$t('common.delete') + ': ' + style.name"
                  @click.stop="handleDelete(style.name)"
                >
                  <Trash2
                    class="w-4 h-4"
                    aria-hidden="true"
                  />
                </button>
              </div>
            </div>

            <!-- Preview -->
            <div class="bg-guofeng-bg-tertiary/50 rounded-lg p-3 border border-guofeng-border/30">
              <p class="text-xs text-guofeng-text-muted mb-1 font-semibold">
                {{ $t('outputStyles.preview') }}:
              </p>
              <pre class="text-xs font-mono text-guofeng-text-secondary line-clamp-4 whitespace-pre-wrap break-words">{{ style.content.slice(0, 300) }}{{ style.content.length > 300 ? '...' : '' }}</pre>
            </div>

            <div class="mt-3 flex items-center justify-between text-xs text-guofeng-text-muted">
              <span>{{ style.content.length }} {{ $t('outputStyles.characters') }}</span>
              <span>{{ style.content.split('\n').length }} {{ $t('outputStyles.lines') }}</span>
            </div>
          </div>
        </GuofengCard>
      </div>
    </div>

    <!-- View Modal -->
    <Teleport to="body">
      <div
        v-if="showViewModal && viewingStyle"
        class="fixed inset-0 flex items-center justify-center z-50 bg-guofeng-ink/20 backdrop-blur-sm transition-all p-4"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="'view-modal-title'"
        @click="showViewModal = false"
        @keydown.esc="showViewModal = false"
      >
        <div
          class="glass-effect p-8 rounded-3xl w-full max-w-4xl max-h-[85vh] overflow-y-auto shadow-2xl border border-white/30 relative"
          @click.stop
        >
          <button
            class="absolute top-4 right-4 p-2 rounded-full hover:bg-guofeng-bg-tertiary text-guofeng-text-muted transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
            :aria-label="$t('common.close')"
            @click="showViewModal = false"
          >
            <X
              class="w-5 h-5"
              aria-hidden="true"
            />
          </button>

          <div class="flex items-center justify-between mb-6">
            <h3
              id="view-modal-title"
              class="text-2xl font-bold text-guofeng-text-primary flex items-center"
            >
              <Palette
                class="w-6 h-6 mr-2 text-guofeng-indigo"
                aria-hidden="true"
              />
              {{ viewingStyle.name }}
            </h3>
            <button
              class="px-3 py-1.5 rounded-lg text-xs font-medium transition-all bg-guofeng-bg-tertiary hover:bg-guofeng-bg-secondary text-guofeng-text-secondary flex items-center gap-1.5 min-h-[44px]"
              :aria-label="copied ? $t('common.copied') : $t('common.copy')"
              @click="copyContent"
            >
              <Copy
                class="w-3.5 h-3.5"
                aria-hidden="true"
              />
              {{ copied ? $t('common.copied') : $t('common.copy') }}
            </button>
          </div>

          <pre class="bg-guofeng-bg-tertiary/50 rounded-xl p-4 overflow-auto max-h-[500px] border border-guofeng-border/30">
            <code class="text-sm font-mono text-guofeng-text-primary whitespace-pre-wrap break-words leading-relaxed">{{ viewingStyle.content }}</code>
          </pre>

          <div class="flex gap-3 mt-6 pt-4 border-t border-guofeng-border/50">
            <button
              class="flex-1 px-4 py-2.5 rounded-xl font-medium transition-all bg-guofeng-blue/10 text-guofeng-blue hover:bg-guofeng-blue/20 flex items-center justify-center gap-2 min-h-[44px]"
              @click="handleEditFromView"
            >
              <Edit2
                class="w-4 h-4"
                aria-hidden="true"
              />
              {{ $t('common.edit') }}
            </button>
            <button
              class="flex-1 px-4 py-2.5 rounded-xl font-medium transition-all bg-white text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary border border-guofeng-border min-h-[44px]"
              @click="showViewModal = false"
            >
              {{ $t('common.close') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Add/Edit Modal -->
    <Teleport to="body">
      <div
        v-if="showModal"
        ref="editModalOverlay"
        class="fixed inset-0 flex items-center justify-center z-50 bg-guofeng-ink/20 backdrop-blur-sm transition-all p-4"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="editingStyle ? 'edit-modal-title' : 'add-modal-title'"
        @click="closeEditModal"
        @keydown.esc="closeEditModal"
      >
        <div
          ref="editModalContent"
          class="glass-effect p-8 rounded-3xl w-full max-w-3xl max-h-[85vh] overflow-y-auto shadow-2xl border border-white/30 relative"
          @click.stop
        >
          <button
            class="absolute top-4 right-4 p-2 rounded-full hover:bg-guofeng-bg-tertiary text-guofeng-text-muted transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
            :aria-label="$t('common.close')"
            @click="closeEditModal"
          >
            <X
              class="w-5 h-5"
              aria-hidden="true"
            />
          </button>

          <h3
            :id="editingStyle ? 'edit-modal-title' : 'add-modal-title'"
            class="text-2xl font-bold mb-6 text-guofeng-text-primary flex items-center"
          >
            <component
              :is="editingStyle ? Edit2 : Plus"
              class="w-6 h-6 mr-2 text-guofeng-indigo"
              aria-hidden="true"
            />
            {{ editingStyle ? $t('outputStyles.editStyle') : $t('outputStyles.addStyle') }}
          </h3>

          <div class="space-y-5">
            <div>
              <label
                for="style-name"
                class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary"
              >{{ $t('outputStyles.nameLabel') }}</label>
              <input
                id="style-name"
                ref="firstInput"
                v-model="formData.name"
                type="text"
                :disabled="!!editingStyle"
                class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-indigo focus:ring-1 focus:ring-guofeng-indigo outline-none transition-all disabled:opacity-60 disabled:cursor-not-allowed"
                :placeholder="$t('outputStyles.namePlaceholder')"
                aria-required="true"
              >
            </div>

            <div>
              <label
                for="style-content"
                class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary"
              >{{ $t('outputStyles.contentLabel') }}</label>
              <textarea
                id="style-content"
                v-model="formData.content"
                rows="15"
                class="w-full px-4 py-3 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-indigo focus:ring-1 focus:ring-guofeng-indigo outline-none transition-all resize-y font-mono text-sm leading-relaxed"
                :placeholder="$t('outputStyles.contentPlaceholder')"
                aria-required="true"
              />
            </div>
          </div>

          <div class="flex gap-4 mt-8 pt-6 border-t border-guofeng-border/50">
            <button
              class="flex-1 px-6 py-3 rounded-xl font-bold transition-all bg-white text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary border border-guofeng-border min-h-[44px]"
              @click="closeEditModal"
            >
              {{ $t('common.cancel') }}
            </button>
            <button
              class="flex-1 px-6 py-3 rounded-xl font-bold transition-all bg-guofeng-indigo text-white shadow-lg shadow-guofeng-indigo/20 hover:shadow-xl hover:shadow-guofeng-indigo/30 hover:-translate-y-0.5 min-h-[44px]"
              :disabled="saving"
              @click="handleSubmit"
            >
              <span
                v-if="saving"
                class="flex items-center justify-center"
              >
                <span
                  class="loading-spinner w-4 h-4 mr-2 border-white/30 border-t-white"
                  aria-hidden="true"
                />
                {{ $t('common.saving') }}
              </span>
              <span v-else>{{ editingStyle ? $t('common.save') : $t('outputStyles.create') }}</span>
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { Palette, Plus, Edit2, Trash2, Search, X, Eye, Copy, Home, Code2 } from 'lucide-vue-next'
import Breadcrumb from '@/components/common/Breadcrumb.vue'
import GuofengCard from '@/components/common/GuofengCard.vue'
import {
  listOutputStyles,
  createOutputStyle,
  updateOutputStyle,
  deleteOutputStyle
} from '@/api/client'
import { useUIStore } from '@/store'
import type { OutputStyle } from '@/types'

const { t } = useI18n()
const uiStore = useUIStore()

const outputStyles = ref<OutputStyle[]>([])
const loading = ref(true)
const searchQuery = ref('')
const showModal = ref(false)
const showViewModal = ref(false)
const editingStyle = ref<OutputStyle | null>(null)
const viewingStyle = ref<OutputStyle | null>(null)
const formData = ref({ name: '', content: '' })
const saving = ref(false)
const copied = ref(false)

// Modal refs for focus trap
const editModalOverlay = ref<HTMLElement | null>(null)
const editModalContent = ref<HTMLElement | null>(null)
const firstInput = ref<HTMLInputElement | null>(null)

const breadcrumbs = computed(() => [
  { label: t('common.home'), to: '/', icon: Home },
  { label: t('claudeCode.title'), to: '/claude-code', icon: Code2 },
  { label: t('outputStyles.pageTitle') }
])

const filteredStyles = computed(() => {
  if (!searchQuery.value.trim()) {
    return outputStyles.value
  }
  const query = searchQuery.value.toLowerCase()
  return outputStyles.value.filter(style =>
    style.name.toLowerCase().includes(query) ||
    style.content.toLowerCase().includes(query)
  )
})

onMounted(async () => {
  await loadStyles()
})

const loadStyles = async () => {
  loading.value = true
  try {
    outputStyles.value = await listOutputStyles()
  } catch (err) {
    console.error('Failed to load output styles:', err)
    uiStore.showError(t('common.loadFailed'))
  } finally {
    loading.value = false
  }
}

const handleAdd = () => {
  editingStyle.value = null
  formData.value = { name: '', content: '' }
  showModal.value = true
}

const handleEdit = (style: OutputStyle) => {
  editingStyle.value = style
  formData.value = { name: style.name, content: style.content }
  showModal.value = true
}

const handleView = (style: OutputStyle) => {
  viewingStyle.value = style
  showViewModal.value = true
}

const handleEditFromView = () => {
  if (viewingStyle.value) {
    showViewModal.value = false
    handleEdit(viewingStyle.value)
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('outputStyles.deleteConfirm', { name }))) return

  try {
    await deleteOutputStyle(name)
    await loadStyles()
    uiStore.showSuccess(t('common.deleteSuccess'))
  } catch (err) {
    console.error('Failed to delete output style:', err)
    uiStore.showError(t('common.deleteFailed'))
  }
}

const handleSubmit = async () => {
  if (!formData.value.name.trim() || !formData.value.content.trim()) {
    uiStore.showWarning(t('outputStyles.validation.required'))
    return
  }

  saving.value = true
  try {
    if (editingStyle.value) {
      await updateOutputStyle(editingStyle.value.name, { content: formData.value.content })
      uiStore.showSuccess(t('common.saveSuccess'))
    } else {
      await createOutputStyle(formData.value)
      uiStore.showSuccess(t('outputStyles.createSuccess'))
    }
    showModal.value = false
    await loadStyles()
  } catch (err) {
    console.error('Failed to save output style:', err)
    uiStore.showError(t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}

const copyContent = async () => {
  if (!viewingStyle.value) return

  try {
    await navigator.clipboard.writeText(viewingStyle.value.content)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

// Close edit modal function
const closeEditModal = () => {
  showModal.value = false
}

// Focus trap for edit modal
watch(showModal, async (isOpen) => {
  if (isOpen) {
    await nextTick()
    // Focus the first input when modal opens
    firstInput.value?.focus()

    // Add focus trap event listener
    document.addEventListener('keydown', handleEditModalFocusTrap)
  } else {
    // Remove focus trap event listener
    document.removeEventListener('keydown', handleEditModalFocusTrap)
  }
})

const handleEditModalFocusTrap = (e: KeyboardEvent) => {
  if (e.key !== 'Tab') return

  const focusableElements = editModalContent.value?.querySelectorAll(
    'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
  )
  if (!focusableElements || focusableElements.length === 0) return

  const firstElement = focusableElements[0] as HTMLElement
  const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement

  if (e.shiftKey) {
    // Shift + Tab
    if (document.activeElement === firstElement) {
      e.preventDefault()
      lastElement.focus()
    }
  } else {
    // Tab
    if (document.activeElement === lastElement) {
      e.preventDefault()
      firstElement.focus()
    }
  }
}
</script>
