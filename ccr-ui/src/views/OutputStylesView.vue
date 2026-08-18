<template>
  <PageShell>
    <template #header>
      <PageHeader :title="$t('outputStyles.pageTitle')">
        <template #status>
          <span>{{ outputStyles.length }}</span>
        </template>
        <template #actions>
          <Button
            @click="handleAdd"
          >
            <SIcon
              name="Plus"
              size="w-5 h-5"
              class="mr-2"
            />{{ $t('outputStyles.addStyle') }}
          </Button>
        </template>
      </PageHeader>
    </template>
    <template #subnav>
      <ModuleSubnav module="claude-code" />
    </template>

    <!-- Search Bar -->
    <div class="mb-6">
      <div class="relative max-w-md">
        <SIcon
          name="Search"
          size="w-4 h-4"
          class="absolute left-3 top-1/2 transform -translate-y-1/2 text-text-muted"
        />
        <input
          v-model="searchQuery"
          type="search"
          :placeholder="$t('outputStyles.searchPlaceholder')"
          :aria-label="$t('outputStyles.searchPlaceholder')"
          class="w-full pl-10 pr-10 py-2.5 rounded-xl transition-colors focus:outline-none focus:ring-2 focus:ring-accent-secondary/20 bg-bg-elevated border border-border-default hover:bg-bg-surface text-text-primary placeholder-text-muted text-sm"
        >
        <button
          v-if="searchQuery"
          class="absolute right-2 top-1/2 transform -translate-y-1/2 rounded-full hover:bg-bg-base/35 text-text-muted transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
          :aria-label="$t('common.clearSearch')"
          @click="searchQuery = ''"
        >
          <SIcon
            name="X"
            size="w-3 h-3"
          />
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div
      v-if="loading"
      class="text-center py-20 text-text-muted"
      role="status"
      aria-live="polite"
    >
      <div
        class="loading-spinner mx-auto mb-4 w-8 h-8 border-accent-secondary/30 border-t-accent-secondary"
        aria-hidden="true"
      />
      <span>{{ $t('common.loading') }}</span>
    </div>

    <!-- Empty State -->
    <div
      v-else-if="filteredStyles.length === 0"
      class="text-center py-20 text-text-muted"
      role="status"
      aria-live="polite"
    >
      <div class="bg-bg-elevated w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
        <SIcon
          name="Palette"
          size="w-10 h-10"
          class="opacity-50"
        />
      </div>
      <p class="text-lg font-medium">
        {{ searchQuery ? $t('outputStyles.noResults') : $t('outputStyles.noStyles') }}
      </p>
      <p
        v-if="!searchQuery"
        class="text-sm mt-2 text-text-muted"
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
      <Card
        v-for="style in filteredStyles"
        :key="style.name"
        variant="glass"
        pattern
      >
        <article
          class="relative z-10 h-full"
          role="listitem"
        >
          <div class="flex items-start justify-between mb-3">
            <div class="flex items-center gap-2">
              <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-accent-secondary/10 to-accent-secondary/10 flex items-center justify-center text-lg shadow-sm border border-border-default/25">
                <SIcon
                  name="Palette"
                  size="w-5 h-5"
                  class="text-accent-secondary"
                />
              </div>
              <h3 class="text-lg font-bold text-text-primary">
                {{ style.name }}
              </h3>
            </div>
            <div class="flex gap-1">
              <button
                class="p-1.5 rounded-md text-text-secondary hover:text-accent-secondary hover:bg-accent-secondary/10 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                :aria-label="$t('common.view') + ': ' + style.name"
                @click.stop="handleView(style)"
              >
                <SIcon
                  name="Eye"
                  size="w-4 h-4"
                />
              </button>
              <button
                class="p-1.5 rounded-md text-text-secondary hover:text-accent-secondary hover:bg-accent-secondary/10 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                :aria-label="$t('common.edit') + ': ' + style.name"
                @click.stop="handleEdit(style)"
              >
                <SIcon
                  name="Edit2"
                  size="w-4 h-4"
                />
              </button>
              <button
                class="p-1.5 rounded-md text-text-secondary hover:text-accent-danger hover:bg-accent-danger/10 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                :aria-label="$t('common.delete') + ': ' + style.name"
                @click.stop="handleDelete(style.name)"
              >
                <SIcon
                  name="Trash2"
                  size="w-4 h-4"
                />
              </button>
            </div>
          </div>

          <button
            type="button"
            class="block w-full rounded-2xl text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-secondary/30"
            :aria-label="$t('common.view') + ': ' + style.name"
            @click="handleView(style)"
          >
            <div class="bg-bg-elevated rounded-lg p-3 border border-border-default/30">
              <p class="text-xs text-text-muted mb-1 font-semibold">
                {{ $t('outputStyles.preview') }}:
              </p>
              <pre class="text-xs font-mono text-text-secondary line-clamp-4 whitespace-pre-wrap break-words">{{ previewContent(style.content) }}</pre>
            </div>

            <div class="mt-3 flex items-center justify-between text-xs text-text-muted">
              <span>{{ style.content.length }} {{ $t('outputStyles.characters') }}</span>
              <span>{{ style.content.split('\n').length }} {{ $t('outputStyles.lines') }}</span>
            </div>
          </button>
        </article>
      </Card>
    </div>

    <!-- View Modal -->
    <Teleport to="body">
      <div
        v-if="showViewModal && viewingStyle"
        class="fixed inset-0 flex items-center justify-center z-50 bg-black/20 backdrop-blur-md transition-colors p-4"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="'view-modal-title'"
        @click="showViewModal = false"
        @keydown.esc="showViewModal = false"
      >
        <div
          class="glass-effect p-8 rounded-3xl w-full max-w-4xl max-h-[85vh] overflow-y-auto shadow-2xl border border-border-default/30 relative"
          @click.stop
        >
          <button
            class="absolute top-4 right-4 p-2 rounded-full hover:bg-bg-surface text-text-muted transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
            :aria-label="$t('common.close')"
            @click="showViewModal = false"
          >
            <SIcon
              name="X"
              size="w-5 h-5"
            />
          </button>

          <div class="flex items-center justify-between mb-6">
            <h3
              id="view-modal-title"
              class="text-2xl font-bold text-text-primary flex items-center"
            >
              <SIcon
                name="Palette"
                size="w-6 h-6"
                class="mr-2 text-accent-secondary"
              />
              {{ viewingStyle.name }}
            </h3>
            <button
              class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-bg-surface hover:bg-bg-elevated text-text-secondary flex items-center gap-1.5 min-h-[44px]"
              :aria-label="copied ? $t('common.copied') : $t('common.copy')"
              @click="copyContent"
            >
              <SIcon
                name="Copy"
                size="w-3.5 h-3.5"
              />
              {{ copied ? $t('common.copied') : $t('common.copy') }}
            </button>
          </div>

          <pre class="bg-bg-elevated rounded-xl p-4 overflow-auto max-h-[500px] border border-border-default/30">
            <code class="text-sm font-mono text-text-primary whitespace-pre-wrap break-words leading-relaxed">{{ viewingStyle.content }}</code>
          </pre>

          <div class="flex gap-3 mt-6 pt-4 border-t border-border-default/50">
            <button
              class="flex-1 px-4 py-2.5 rounded-xl font-medium transition-colors bg-accent-secondary/10 text-accent-secondary hover:bg-accent-secondary/20 flex items-center justify-center gap-2 min-h-[44px]"
              @click="handleEditFromView"
            >
              <SIcon
                name="Edit2"
                size="w-4 h-4"
              />
              {{ $t('common.edit') }}
            </button>
            <button
              class="flex-1 px-4 py-2.5 rounded-xl font-medium transition-colors bg-bg-surface text-text-secondary hover:bg-bg-overlay border border-border-default min-h-[44px]"
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
        class="fixed inset-0 flex items-center justify-center z-50 bg-black/20 backdrop-blur-md transition-colors p-4"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="editingStyle ? 'edit-modal-title' : 'add-modal-title'"
        @click="closeEditModal"
        @keydown.esc="closeEditModal"
      >
        <div
          ref="editModalContent"
          class="glass-effect p-8 rounded-3xl w-full max-w-3xl max-h-[85vh] overflow-y-auto shadow-2xl border border-border-default/30 relative"
          @click.stop
        >
          <button
            class="absolute top-4 right-4 p-2 rounded-full hover:bg-bg-surface text-text-muted transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
            :aria-label="$t('common.close')"
            @click="closeEditModal"
          >
            <SIcon
              name="X"
              size="w-5 h-5"
            />
          </button>

          <h3
            :id="editingStyle ? 'edit-modal-title' : 'add-modal-title'"
            class="text-2xl font-bold mb-6 text-text-primary flex items-center"
          >
            <SIcon
              :name="editingStyle ? 'Edit2' : 'Plus'"
              size="w-6 h-6"
              class="mr-2 text-accent-secondary"
            />
            {{ editingStyle ? $t('outputStyles.editStyle') : $t('outputStyles.addStyle') }}
          </h3>

          <div class="space-y-5">
            <div>
              <label
                for="style-name"
                class="block mb-1.5 text-sm font-semibold text-text-secondary"
              >{{ $t('outputStyles.nameLabel') }}</label>
              <input
                id="style-name"
                ref="firstInput"
                v-model="formData.name"
                type="text"
                :disabled="!!editingStyle"
                class="w-full px-4 py-2.5 rounded-lg bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-1 focus:ring-accent-secondary outline-none transition-colors disabled:opacity-60 disabled:cursor-not-allowed"
                :placeholder="$t('outputStyles.namePlaceholder')"
                aria-required="true"
              >
            </div>

            <div>
              <label
                for="style-content"
                class="block mb-1.5 text-sm font-semibold text-text-secondary"
              >{{ $t('outputStyles.contentLabel') }}</label>
              <textarea
                id="style-content"
                v-model="formData.content"
                rows="15"
                class="w-full px-4 py-3 rounded-lg bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-1 focus:ring-accent-secondary outline-none transition-colors resize-y font-mono text-sm leading-relaxed"
                :placeholder="$t('outputStyles.contentPlaceholder')"
                aria-required="true"
              />
            </div>
          </div>

          <div class="flex gap-4 mt-8 pt-6 border-t border-border-default/50">
            <button
              class="flex-1 px-6 py-3 rounded-xl font-bold transition-colors bg-bg-surface text-text-secondary hover:bg-bg-overlay border border-border-default min-h-[44px]"
              @click="closeEditModal"
            >
              {{ $t('common.cancel') }}
            </button>
            <button
              class="flex-1 px-6 py-3 rounded-xl font-bold transition-[color,background-color,border-color,transform] bg-accent-secondary text-[color:var(--color-accent-primary-contrast)] shadow-lg shadow-accent-secondary/20 hover:shadow-xl hover:shadow-accent-secondary/30 hover:-translate-y-0.5 min-h-[44px]"
              :disabled="saving"
              @click="handleSubmit"
            >
              <span
                v-if="saving"
                class="flex items-center justify-center"
              >
                <span
                  class="loading-spinner w-4 h-4 mr-2 border-border-default/30 border-t-white"
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
  </PageShell>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import {
  listOutputStyles,
  createOutputStyle,
  updateOutputStyle,
  deleteOutputStyle
} from '@/api'
import { useUIStore } from '@/stores/ui'
import type { OutputStyle } from '@/types'
import { logger } from '@/utils/logger'
import { copyText } from '@/utils/clipboard'

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

const previewContent = (content: string) =>
  content.length > 300 ? `${content.slice(0, 300)}...` : content

onMounted(async () => {
  await loadStyles()
})

const loadStyles = async () => {
  loading.value = true
  try {
    outputStyles.value = await listOutputStyles()
  } catch (err) {
    logger.error('Failed to load output styles:', err)
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
  const confirmed = await uiStore.requestConfirm({
    title: t('common.delete'),
    message: t('outputStyles.deleteConfirm', { name }),
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    type: 'danger'
  })
  if (!confirmed) return

  try {
    await deleteOutputStyle(name)
    await loadStyles()
    uiStore.showSuccess(t('common.deleteSuccess'))
  } catch (err) {
    logger.error('Failed to delete output style:', err)
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
    logger.error('Failed to save output style:', err)
    uiStore.showError(t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}

const copyContent = async () => {
  if (!viewingStyle.value) return

  try {
    if (!(await copyText(viewingStyle.value.content))) throw new Error('clipboard copy failed')
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    logger.error('Failed to copy:', err)
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
