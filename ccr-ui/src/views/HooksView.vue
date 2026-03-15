<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="mb-6" />

    <div class="max-w-[1600px] mx-auto">
      <!-- Header -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div class="flex items-center gap-4">
          <h2 class="text-xl sm:text-2xl font-bold text-text-primary flex items-center">
            <SIcon
              name="Webhook"
              size="w-6 h-6"
              class="sm:w-7 sm:h-7 mr-2 text-accent-secondary"
            />
            Hooks Management
          </h2>
          <span
            class="px-3 py-1 rounded-full text-sm font-medium bg-accent-secondary/10 text-accent-secondary border border-accent-secondary/20"
            aria-label="Total hooks count"
          >
            {{ hooks.length }}
          </span>
        </div>
        <button
          class="w-full sm:w-auto px-4 py-2 rounded-lg font-medium transition-[color,background-color,border-color,transform] hover:scale-105 bg-accent-secondary text-white shadow-md hover:shadow-lg flex items-center justify-center min-h-[44px]"
          aria-label="Add new hook"
          @click="handleAdd"
        >
          <SIcon
            name="Plus"
            size="w-5 h-5"
            class="mr-2"
          />Add Hook
        </button>
      </div>

      <!-- Hook Type Tabs -->
      <div
        class="mb-6 flex gap-2 overflow-x-auto pb-2 scrollbar-thin md:flex-wrap md:overflow-x-visible md:pb-0"
        role="tablist"
        aria-label="Filter hooks by type"
      >
        <button
          v-for="type in hookTypes"
          :key="type"
          role="tab"
          :aria-selected="selectedType === type"
          class="px-4 py-2 rounded-lg font-medium text-sm transition-colors min-h-[44px] whitespace-nowrap flex-shrink-0"
          :class="selectedType === type ? 'bg-accent-secondary text-white shadow-md' : 'bg-bg-elevated text-text-secondary border border-border-default hover:bg-bg-surface'"
          @click="selectedType = type"
        >
          {{ type }}
          <span class="ml-2 opacity-70">({{ getHooksByType(type).length }})</span>
        </button>
      </div>

      <!-- Hooks Grid -->
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
        <span>Loading...</span>
      </div>

      <div
        v-else-if="filteredHooks.length === 0"
        class="text-center py-20 text-text-muted"
        role="status"
        aria-live="polite"
      >
        <div class="bg-bg-elevated w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
          <SIcon
            name="Webhook"
            size="w-10 h-10"
            class="opacity-50"
          />
        </div>
        <p class="text-lg font-medium">
          No {{ selectedType }} hooks found
        </p>
      </div>

      <div
        v-else
        class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4"
        role="list"
        aria-label="Hooks list"
      >
        <Card
          v-for="hook in filteredHooks"
          :key="hook.name"
          variant="glass"
          interactive
          pattern
        >
          <div
            class="relative z-10"
            role="listitem"
            tabindex="0"
            @keydown.enter="handleEdit(hook)"
          >
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-2">
                <h3 class="text-lg font-bold text-text-primary">
                  {{ hook.name }}
                </h3>
                <span
                  class="px-2 py-0.5 rounded text-xs font-medium"
                  :class="getHookTypeColor(hook.hook_type)"
                >
                  {{ hook.hook_type }}
                </span>
              </div>
              <div class="flex gap-1">
                <button
                  class="p-1.5 rounded-md transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                  :class="hook.enabled !== false ? 'text-accent-success hover:bg-accent-success/10' : 'text-text-muted hover:bg-bg-surface'"
                  :aria-label="hook.enabled !== false ? `Disable hook ${hook.name}` : `Enable hook ${hook.name}`"
                  :aria-pressed="hook.enabled !== false"
                  @click.stop="handleToggle(hook.name)"
                >
                  <SIcon
                    name="Power"
                    size="w-4 h-4"
                  />
                  <span class="sr-only">{{ hook.enabled !== false ? 'Enabled' : 'Disabled' }}</span>
                </button>
                <button
                  class="p-1.5 rounded-md text-accent-secondary hover:bg-accent-secondary/10 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                  :aria-label="`Edit hook ${hook.name}`"
                  @click.stop="handleEdit(hook)"
                >
                  <SIcon
                    name="Edit2"
                    size="w-4 h-4"
                  />
                </button>
                <button
                  class="p-1.5 rounded-md text-accent-danger hover:bg-accent-danger/10 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                  :aria-label="`Delete hook ${hook.name}`"
                  @click.stop="handleDelete(hook.name)"
                >
                  <SIcon
                    name="Trash2"
                    size="w-4 h-4"
                  />
                </button>
              </div>
            </div>

            <div class="space-y-2 text-sm">
              <div class="bg-bg-surface rounded-lg p-3 border border-border-default/50">
                <p class="text-xs text-text-muted mb-1 font-semibold">
                  Command:
                </p>
                <code class="text-xs font-mono text-text-primary block break-all">{{ hook.command }}</code>
                <div
                  v-if="hook.args && hook.args.length > 0"
                  class="mt-2"
                >
                  <p class="text-xs text-text-muted mb-1 font-semibold">
                    Args:
                  </p>
                  <code class="text-xs font-mono text-text-secondary">{{ hook.args.join(' ') }}</code>
                </div>
              </div>
            </div>

            <!-- Status indicator with text -->
            <div class="mt-3 flex items-center gap-2">
              <span
                class="w-2 h-2 rounded-full"
                :class="hook.enabled !== false ? 'bg-accent-success' : 'bg-text-muted'"
                aria-hidden="true"
              />
              <span
                class="text-xs font-medium"
                :class="hook.enabled !== false ? 'text-accent-success' : 'text-text-muted'"
              >
                {{ hook.enabled !== false ? 'Enabled' : 'Disabled' }}
              </span>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <!-- Add/Edit Modal -->
    <Teleport to="body">
      <div
        v-if="showModal"
        ref="modalOverlay"
        class="fixed inset-0 flex items-center justify-center z-50 bg-black/20 backdrop-blur-md transition-colors"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="editingHook ? 'modal-title-edit' : 'modal-title-add'"
        @click="closeModal"
        @keydown.esc="closeModal"
      >
        <div
          ref="modalContent"
          class="bg-bg-elevated p-8 rounded-2xl w-full max-w-2xl max-h-[85vh] overflow-y-auto shadow-2xl border border-border-default relative"
          @click.stop
        >
          <button
            ref="closeButton"
            class="absolute top-4 right-4 p-2 rounded-full hover:bg-bg-surface text-text-muted transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
            aria-label="Close modal"
            @click="closeModal"
          >
            <SIcon
              name="X"
              size="w-5 h-5"
            />
          </button>

          <h3
            :id="editingHook ? 'modal-title-edit' : 'modal-title-add'"
            class="text-2xl font-bold mb-6 text-text-primary flex items-center"
          >
            <SIcon
              :name="editingHook ? 'Edit2' : 'Plus'"
              size="w-6 h-6"
              class="mr-2 text-accent-secondary"
            />
            {{ editingHook ? 'Edit Hook' : 'Add Hook' }}
          </h3>

          <div class="space-y-5">
            <div>
              <label
                for="hook-name"
                class="block mb-1.5 text-sm font-semibold text-text-secondary"
              >Name</label>
              <input
                id="hook-name"
                ref="firstInput"
                v-model="formData.name"
                type="text"
                :disabled="!!editingHook"
                class="w-full px-4 py-2.5 rounded-lg bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-1 focus:ring-accent-secondary outline-none transition-colors disabled:opacity-60 disabled:cursor-not-allowed"
                placeholder="my-hook"
                aria-required="true"
              >
            </div>

            <div>
              <label
                for="hook-type"
                class="block mb-1.5 text-sm font-semibold text-text-secondary"
              >Hook Type</label>
              <select
                id="hook-type"
                v-model="formData.hook_type"
                class="w-full px-4 py-2.5 rounded-lg bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-1 focus:ring-accent-secondary outline-none transition-colors"
                aria-required="true"
              >
                <option
                  v-for="type in hookTypesWithoutAll"
                  :key="type"
                  :value="type"
                >
                  {{ type }}
                </option>
              </select>
            </div>

            <div>
              <label
                for="hook-command"
                class="block mb-1.5 text-sm font-semibold text-text-secondary"
              >Command</label>
              <input
                id="hook-command"
                v-model="formData.command"
                type="text"
                class="w-full px-4 py-2.5 rounded-lg bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-1 focus:ring-accent-secondary outline-none transition-colors font-mono text-sm"
                placeholder="/usr/bin/notify-send"
                aria-required="true"
              >
            </div>

            <div>
              <label
                for="hook-args"
                class="block mb-1.5 text-sm font-semibold text-text-secondary"
              >Arguments (one per line)</label>
              <textarea
                id="hook-args"
                v-model="argsText"
                rows="4"
                class="w-full px-4 py-3 rounded-lg bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-1 focus:ring-accent-secondary outline-none transition-colors resize-y font-mono text-sm"
                placeholder="--urgency=normal&#10;Tool executed"
                aria-describedby="hook-args-hint"
              />
              <p
                id="hook-args-hint"
                class="text-xs text-text-muted mt-1"
              >
                Enter each argument on a separate line
              </p>
            </div>

            <div class="flex items-center gap-3">
              <input
                id="hook-enabled"
                v-model="formData.enabled"
                type="checkbox"
                class="w-4 h-4 rounded border-border-default text-accent-secondary focus:ring-accent-secondary"
              >
              <label
                for="hook-enabled"
                class="text-sm font-semibold text-text-secondary cursor-pointer"
              >Enabled</label>
            </div>
          </div>

          <div class="flex gap-4 mt-8 pt-6 border-t border-border-default">
            <button
              class="flex-1 px-6 py-3 rounded-lg font-medium transition-colors bg-bg-surface text-text-secondary hover:bg-bg-elevated border border-border-default min-h-[44px]"
              @click="closeModal"
            >
              Cancel
            </button>
            <button
              ref="lastButton"
              class="flex-1 px-6 py-3 rounded-lg font-medium transition-[color,background-color,border-color,transform] bg-accent-secondary text-white shadow-md hover:shadow-lg hover:-translate-y-0.5 min-h-[44px]"
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
                Saving...
              </span>
              <span v-else>{{ editingHook ? 'Save' : 'Add' }}</span>
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import Card from '@/components/ui/Card.vue'
import { listHooks, addHook, updateHook, deleteHook, toggleHook } from '@/api'
import { useUIStore } from '@/store'
import type { Hook, HookType } from '@/types'
import { logger } from '@/utils/logger'

const uiStore = useUIStore()
const hooks = ref<Hook[]>([])
const loading = ref(false)
const saving = ref(false)
const showModal = ref(false)
const editingHook = ref<Hook | null>(null)
const selectedType = ref<HookType | 'All'>('All')

// Modal refs for focus trap
const modalOverlay = ref<HTMLElement | null>(null)
const modalContent = ref<HTMLElement | null>(null)
const firstInput = ref<HTMLInputElement | null>(null)
const lastButton = ref<HTMLButtonElement | null>(null)
const closeButton = ref<HTMLButtonElement | null>(null)

const hookTypes: (HookType | 'All')[] = ['All', 'PreToolUse', 'PostToolUse', 'Stop', 'SessionStart', 'SessionEnd', 'Error']
const hookTypesWithoutAll: HookType[] = ['PreToolUse', 'PostToolUse', 'Stop', 'SessionStart', 'SessionEnd', 'Error']

const formData = ref({
  name: '',
  hook_type: 'PreToolUse' as HookType,
  command: '',
  enabled: true
})

const argsText = ref('')

const filteredHooks = computed(() => {
  if (selectedType.value === 'All') return hooks.value
  return hooks.value.filter(h => h.hook_type === selectedType.value)
})

const getHooksByType = (type: HookType | 'All') => {
  if (type === 'All') return hooks.value
  return hooks.value.filter(h => h.hook_type === type)
}

const getHookTypeColor = (type: HookType) => {
  const colors: Record<HookType, string> = {
    PreToolUse: 'bg-accent-secondary/10 text-accent-secondary border border-accent-secondary/20',
    PostToolUse: 'bg-accent-success/10 text-accent-success border border-accent-success/20',
    Stop: 'bg-accent-danger/10 text-accent-danger border border-accent-danger/20',
    SessionStart: 'bg-accent-primary/10 text-accent-primary border border-accent-primary/20',
    SessionEnd: 'bg-accent-warning/10 text-accent-warning border border-accent-warning/20',
    Error: 'bg-accent-danger/10 text-accent-danger border border-accent-danger/20'
  }
  return colors[type] || 'bg-bg-elevated text-text-secondary'
}

const loadHooks = async () => {
  loading.value = true
  try {
    hooks.value = await listHooks()
  } catch (err) {
    logger.error('Failed to load hooks:', err)
    uiStore.showError('Failed to load hooks')
  } finally {
    loading.value = false
  }
}

const handleAdd = () => {
  showModal.value = true
  editingHook.value = null
  formData.value = { name: '', hook_type: 'PreToolUse', command: '', enabled: true }
  argsText.value = ''
}

const handleEdit = (hook: Hook) => {
  editingHook.value = hook
  showModal.value = true
  formData.value = {
    name: hook.name,
    hook_type: hook.hook_type,
    command: hook.command,
    enabled: hook.enabled !== false
  }
  argsText.value = hook.args?.join('\n') || ''
}

const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.command) {
    uiStore.showWarning('Name and command are required')
    return
  }

  const args = argsText.value.trim() ? argsText.value.split('\n').filter(a => a.trim()) : []

  saving.value = true
  try {
    const request = {
      name: formData.value.name,
      hook_type: formData.value.hook_type,
      command: formData.value.command,
      args: args.length > 0 ? args : undefined,
      enabled: formData.value.enabled
    }

    if (editingHook.value) {
      await updateHook(editingHook.value.name, request)
      uiStore.showSuccess('Hook updated successfully')
    } else {
      await addHook(request)
      uiStore.showSuccess('Hook created successfully')
    }

    closeModal()
    await loadHooks()
  } catch (err) {
    logger.error('Operation failed:', err)
    uiStore.showError('Operation failed')
  } finally {
    saving.value = false
  }
}

// Close modal function
const closeModal = () => {
  showModal.value = false
}

// Focus trap for modal
watch(showModal, async (isOpen) => {
  if (isOpen) {
    await nextTick()
    // Focus the first input when modal opens
    firstInput.value?.focus()

    // Add focus trap event listener
    document.addEventListener('keydown', handleFocusTrap)
  } else {
    // Remove focus trap event listener
    document.removeEventListener('keydown', handleFocusTrap)
  }
})

const handleFocusTrap = (e: KeyboardEvent) => {
  if (e.key !== 'Tab') return

  const focusableElements = modalContent.value?.querySelectorAll(
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

const handleToggle = async (name: string) => {
  try {
    await toggleHook(name)
    await loadHooks()
    uiStore.showSuccess('Hook toggled successfully')
  } catch (err) {
    logger.error('Toggle failed:', err)
    uiStore.showError('Toggle failed')
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(`Delete hook "${name}"?`)) return
  try {
    await deleteHook(name)
    await loadHooks()
    uiStore.showSuccess('Hook deleted successfully')
  } catch (err) {
    logger.error('Delete failed:', err)
    uiStore.showError('Delete failed')
  }
}

onMounted(() => {
  loadHooks()
})
</script>
