<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="mb-6" />

    <div class="max-w-[1600px] mx-auto">
      <!-- Header -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div class="flex items-center gap-4">
          <h2 class="text-xl sm:text-2xl font-bold text-guofeng-text-primary flex items-center">
            <Webhook
              class="w-6 h-6 sm:w-7 sm:h-7 mr-2 text-guofeng-red"
              aria-hidden="true"
            />
            Hooks Management
          </h2>
          <span
            class="px-3 py-1 rounded-full text-sm font-medium bg-guofeng-red/10 text-guofeng-red border border-guofeng-red/20"
            aria-label="Total hooks count"
          >
            {{ hooks.length }}
          </span>
        </div>
        <button
          class="w-full sm:w-auto px-4 py-2 rounded-lg font-medium transition-all hover:scale-105 bg-guofeng-red text-white shadow-md hover:shadow-lg flex items-center justify-center min-h-[44px]"
          aria-label="Add new hook"
          @click="handleAdd"
        >
          <Plus
            class="w-5 h-5 mr-2"
            aria-hidden="true"
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
          class="px-4 py-2 rounded-lg font-medium text-sm transition-all min-h-[44px] whitespace-nowrap flex-shrink-0"
          :class="selectedType === type ? 'bg-guofeng-red text-white shadow-md' : 'bg-guofeng-bg-secondary text-guofeng-text-secondary border border-guofeng-border hover:bg-guofeng-bg-tertiary'"
          @click="selectedType = type"
        >
          {{ type }}
          <span class="ml-2 opacity-70">({{ getHooksByType(type).length }})</span>
        </button>
      </div>

      <!-- Hooks Grid -->
      <div
        v-if="loading"
        class="text-center py-20 text-guofeng-text-muted"
        role="status"
        aria-live="polite"
      >
        <div
          class="loading-spinner mx-auto mb-4 w-8 h-8 border-guofeng-red/30 border-t-guofeng-red"
          aria-hidden="true"
        />
        <span>Loading...</span>
      </div>

      <div
        v-else-if="filteredHooks.length === 0"
        class="text-center py-20 text-guofeng-text-muted"
        role="status"
        aria-live="polite"
      >
        <div class="bg-guofeng-bg-secondary w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
          <Webhook
            class="w-10 h-10 opacity-50"
            aria-hidden="true"
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
        <GuofengCard
          v-for="hook in filteredHooks"
          :key="hook.name"
          variant="glass"
          interactive
          pattern
          role="listitem"
          tabindex="0"
          @keydown.enter="handleEdit(hook)"
        >
          <div class="relative z-10">
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-2">
                <h3 class="text-lg font-bold text-guofeng-text-primary">
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
                  :class="hook.enabled !== false ? 'text-guofeng-green hover:bg-guofeng-green/10' : 'text-guofeng-text-muted hover:bg-guofeng-bg-tertiary'"
                  :aria-label="hook.enabled !== false ? `Disable hook ${hook.name}` : `Enable hook ${hook.name}`"
                  :aria-pressed="hook.enabled !== false"
                  @click.stop="handleToggle(hook.name)"
                >
                  <Power
                    class="w-4 h-4"
                    aria-hidden="true"
                  />
                  <span class="sr-only">{{ hook.enabled !== false ? 'Enabled' : 'Disabled' }}</span>
                </button>
                <button
                  class="p-1.5 rounded-md text-guofeng-blue hover:bg-guofeng-blue/10 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                  :aria-label="`Edit hook ${hook.name}`"
                  @click.stop="handleEdit(hook)"
                >
                  <Edit2
                    class="w-4 h-4"
                    aria-hidden="true"
                  />
                </button>
                <button
                  class="p-1.5 rounded-md text-guofeng-red hover:bg-guofeng-red/10 transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                  :aria-label="`Delete hook ${hook.name}`"
                  @click.stop="handleDelete(hook.name)"
                >
                  <Trash2
                    class="w-4 h-4"
                    aria-hidden="true"
                  />
                </button>
              </div>
            </div>

            <div class="space-y-2 text-sm">
              <div class="bg-guofeng-bg-tertiary rounded-lg p-3 border border-guofeng-border/50">
                <p class="text-xs text-guofeng-text-muted mb-1 font-semibold">
                  Command:
                </p>
                <code class="text-xs font-mono text-guofeng-text-primary block break-all">{{ hook.command }}</code>
                <div
                  v-if="hook.args && hook.args.length > 0"
                  class="mt-2"
                >
                  <p class="text-xs text-guofeng-text-muted mb-1 font-semibold">
                    Args:
                  </p>
                  <code class="text-xs font-mono text-guofeng-text-secondary">{{ hook.args.join(' ') }}</code>
                </div>
              </div>
            </div>

            <!-- Status indicator with text -->
            <div class="mt-3 flex items-center gap-2">
              <span
                class="w-2 h-2 rounded-full"
                :class="hook.enabled !== false ? 'bg-guofeng-green' : 'bg-guofeng-text-muted'"
                aria-hidden="true"
              />
              <span
                class="text-xs font-medium"
                :class="hook.enabled !== false ? 'text-guofeng-green' : 'text-guofeng-text-muted'"
              >
                {{ hook.enabled !== false ? 'Enabled' : 'Disabled' }}
              </span>
            </div>
          </div>
        </GuofengCard>
      </div>
    </div>

    <!-- Add/Edit Modal -->
    <Teleport to="body">
      <div
        v-if="showModal"
        ref="modalOverlay"
        class="fixed inset-0 flex items-center justify-center z-50 bg-guofeng-ink/20 backdrop-blur-sm transition-all"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="editingHook ? 'modal-title-edit' : 'modal-title-add'"
        @click="closeModal"
        @keydown.esc="closeModal"
      >
        <div
          ref="modalContent"
          class="bg-guofeng-bg p-8 rounded-2xl w-full max-w-2xl max-h-[85vh] overflow-y-auto shadow-2xl border border-guofeng-border relative"
          @click.stop
        >
          <button
            ref="closeButton"
            class="absolute top-4 right-4 p-2 rounded-full hover:bg-guofeng-bg-tertiary text-guofeng-text-muted transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
            aria-label="Close modal"
            @click="closeModal"
          >
            <X
              class="w-5 h-5"
              aria-hidden="true"
            />
          </button>

          <h3
            :id="editingHook ? 'modal-title-edit' : 'modal-title-add'"
            class="text-2xl font-bold mb-6 text-guofeng-text-primary flex items-center"
          >
            <component
              :is="editingHook ? Edit2 : Plus"
              class="w-6 h-6 mr-2 text-guofeng-red"
              aria-hidden="true"
            />
            {{ editingHook ? 'Edit Hook' : 'Add Hook' }}
          </h3>

          <div class="space-y-5">
            <div>
              <label
                for="hook-name"
                class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary"
              >Name</label>
              <input
                id="hook-name"
                ref="firstInput"
                v-model="formData.name"
                type="text"
                :disabled="!!editingHook"
                class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all disabled:opacity-60 disabled:cursor-not-allowed"
                placeholder="my-hook"
                aria-required="true"
              >
            </div>

            <div>
              <label
                for="hook-type"
                class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary"
              >Hook Type</label>
              <select
                id="hook-type"
                v-model="formData.hook_type"
                class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all"
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
                class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary"
              >Command</label>
              <input
                id="hook-command"
                v-model="formData.command"
                type="text"
                class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all font-mono text-sm"
                placeholder="/usr/bin/notify-send"
                aria-required="true"
              >
            </div>

            <div>
              <label
                for="hook-args"
                class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary"
              >Arguments (one per line)</label>
              <textarea
                id="hook-args"
                v-model="argsText"
                rows="4"
                class="w-full px-4 py-3 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all resize-y font-mono text-sm"
                placeholder="--urgency=normal&#10;Tool executed"
                aria-describedby="hook-args-hint"
              />
              <p
                id="hook-args-hint"
                class="text-xs text-guofeng-text-muted mt-1"
              >
                Enter each argument on a separate line
              </p>
            </div>

            <div class="flex items-center gap-3">
              <input
                id="hook-enabled"
                v-model="formData.enabled"
                type="checkbox"
                class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
              >
              <label
                for="hook-enabled"
                class="text-sm font-semibold text-guofeng-text-secondary cursor-pointer"
              >Enabled</label>
            </div>
          </div>

          <div class="flex gap-4 mt-8 pt-6 border-t border-guofeng-border">
            <button
              class="flex-1 px-6 py-3 rounded-lg font-medium transition-all bg-guofeng-bg-tertiary text-guofeng-text-secondary hover:bg-guofeng-bg-secondary border border-guofeng-border min-h-[44px]"
              @click="closeModal"
            >
              Cancel
            </button>
            <button
              ref="lastButton"
              class="flex-1 px-6 py-3 rounded-lg font-medium transition-all bg-guofeng-red text-white shadow-md hover:shadow-lg hover:-translate-y-0.5 min-h-[44px]"
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
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { Webhook, Plus, Edit2, Trash2, X, Power } from 'lucide-vue-next'
import GuofengCard from '@/components/common/GuofengCard.vue'
import { listHooks, addHook, updateHook, deleteHook, toggleHook } from '@/api/client'
import { useUIStore } from '@/store'
import type { Hook, HookType } from '@/types'

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
    PreToolUse: 'bg-guofeng-blue/10 text-guofeng-blue border border-guofeng-blue/20',
    PostToolUse: 'bg-guofeng-green/10 text-guofeng-green border border-guofeng-green/20',
    Stop: 'bg-guofeng-red/10 text-guofeng-red border border-guofeng-red/20',
    SessionStart: 'bg-guofeng-purple/10 text-guofeng-purple border border-guofeng-purple/20',
    SessionEnd: 'bg-guofeng-orange/10 text-guofeng-orange border border-guofeng-orange/20',
    Error: 'bg-guofeng-red/10 text-guofeng-red border border-guofeng-red/20'
  }
  return colors[type] || 'bg-guofeng-bg-secondary text-guofeng-text-secondary'
}

const loadHooks = async () => {
  loading.value = true
  try {
    hooks.value = await listHooks()
  } catch (err) {
    console.error('Failed to load hooks:', err)
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
    console.error('Operation failed:', err)
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
    console.error('Toggle failed:', err)
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
    console.error('Delete failed:', err)
    uiStore.showError('Delete failed')
  }
}

onMounted(() => {
  loadHooks()
})
</script>
