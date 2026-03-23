<template>
  <div class="min-h-full p-6 transition-colors duration-300">
    <div class="mx-auto max-w-6xl space-y-6">
      <ModuleSubnav module="qoder" />

      <PageHeaderCard
        :title="t('qoder.plugins.title')"
        icon="Webhook"
        :badge="String(hooks.length)"
        tone="primary"
      >
        <template #meta>
          <span class="inline-flex items-center gap-2 rounded-full border border-[var(--color-accent-primary)]/20 bg-[var(--color-accent-primary)]/10 px-3 py-1 text-sm font-medium text-[var(--color-accent-primary)]">
            Notification
          </span>
        </template>

        <template #actions>
          <button
            class="min-h-[44px] rounded-xl bg-[var(--color-accent-primary)] px-4 py-2.5 text-sm font-medium text-white shadow-lg shadow-[var(--color-accent-primary)]/20 transition-[color,background-color,border-color,transform] hover:scale-105 hover:shadow-[var(--color-accent-primary)]/30"
            @click="openCreateModal"
          >
            <span class="inline-flex items-center gap-2">
              <SIcon
                name="Plus"
                size="w-4 h-4"
              />
              Add Hook
            </span>
          </button>
        </template>

        <p class="text-sm text-[var(--color-text-secondary)]">
          {{ t('qoder.plugins.description') }}
        </p>
      </PageHeaderCard>

      <div
        v-if="loading"
        class="rounded-3xl border border-white/20 bg-white/40 py-20 text-center text-[var(--color-text-muted)] shadow-sm backdrop-blur-md"
      >
        <div class="loading-spinner mx-auto mb-4 h-8 w-8 border-[var(--color-accent-primary)]/30 border-t-[var(--color-accent-primary)]" />
        {{ t('common.loading') }}
      </div>

      <div
        v-else-if="hooks.length === 0"
        class="rounded-3xl border border-dashed border-white/20 bg-white/30 py-20 text-center shadow-sm backdrop-blur-md"
      >
        <div class="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-[var(--color-bg-elevated)]">
          <SIcon
            name="Webhook"
            size="w-10 h-10"
            class="text-[var(--color-text-muted)] opacity-40"
          />
        </div>
        <p class="text-lg font-bold text-[var(--color-text-primary)]">
          {{ t('qoder.plugins.emptyState') }}
        </p>
        <p class="mt-2 text-sm text-[var(--color-text-muted)]">
          {{ t('qoder.plugins.configHint') }}
        </p>
      </div>

      <div
        v-else
        class="grid grid-cols-1 gap-5 xl:grid-cols-2"
      >
        <Card
          v-for="hook in hooks"
          :key="hook.id"
          variant="glass"
          pattern
        >
          <div class="relative z-10 flex h-full flex-col gap-4">
            <div class="flex items-start justify-between gap-4">
              <div class="min-w-0">
                <div class="mb-2 flex items-center gap-2">
                  <h2 class="truncate text-base font-bold text-[var(--color-text-primary)]">
                    {{ hook.name }}
                  </h2>
                  <span class="rounded-full border border-[var(--color-accent-primary)]/20 bg-[var(--color-accent-primary)]/10 px-2 py-0.5 text-xs font-medium text-[var(--color-accent-primary)]">
                    {{ hook.event }}
                  </span>
                </div>
                <p class="text-sm text-[var(--color-text-secondary)]">
                  {{ t('qoder.plugins.configHint') }}
                </p>
              </div>

              <div class="flex items-center gap-1">
                <button
                  class="min-h-[44px] min-w-[44px] rounded-xl text-[var(--color-text-secondary)] transition-colors hover:bg-[var(--color-accent-primary)]/10 hover:text-[var(--color-accent-primary)]"
                  @click="openEditModal(hook)"
                >
                  <SIcon
                    name="Edit2"
                    size="w-4 h-4"
                  />
                </button>
                <button
                  class="min-h-[44px] min-w-[44px] rounded-xl text-[var(--color-text-secondary)] transition-colors hover:bg-[var(--color-danger)]/10 hover:text-[var(--color-danger)]"
                  @click="handleDelete(hook)"
                >
                  <SIcon
                    name="Trash2"
                    size="w-4 h-4"
                  />
                </button>
              </div>
            </div>

            <div class="rounded-2xl border border-[var(--color-border-default)] bg-[var(--color-bg-surface)]/80 p-4">
              <p class="mb-2 text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)]">
                Command
              </p>
              <code class="block break-all whitespace-pre-wrap text-sm text-[var(--color-text-primary)]">{{ hook.command }}</code>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="showModal"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/30 p-4 backdrop-blur-sm"
        @click.self="closeModal"
      >
        <div class="w-full max-w-2xl rounded-3xl border border-white/20 bg-white/90 p-6 shadow-2xl backdrop-blur-xl">
          <div class="mb-6 flex items-center justify-between">
            <div>
              <h2 class="text-xl font-bold text-[var(--color-text-primary)]">
                {{ editingHook ? t('qoder.plugins.editPlugin') : t('qoder.plugins.addPlugin') }}
              </h2>
              <p class="mt-1 text-sm text-[var(--color-text-secondary)]">
                {{ t('qoder.plugins.description') }}
              </p>
            </div>
            <button
              class="min-h-[44px] min-w-[44px] rounded-xl text-[var(--color-text-secondary)] transition-colors hover:bg-black/5 hover:text-[var(--color-text-primary)]"
              @click="closeModal"
            >
              <SIcon
                name="X"
                size="w-4 h-4"
              />
            </button>
          </div>

          <div class="space-y-4">
            <div>
              <label class="mb-2 block text-xs font-bold uppercase tracking-wider text-[var(--color-text-secondary)]">{{ t('qoder.plugins.configLabel') }} *</label>
              <textarea
                v-model="form.command"
                rows="5"
                class="w-full rounded-2xl border border-[var(--color-border-default)] bg-white/70 px-4 py-3 font-mono text-sm text-[var(--color-text-primary)] outline-none transition-colors focus:border-[var(--color-accent-primary)] focus:ring-4 focus:ring-[var(--color-accent-primary)]/10"
                :placeholder="t('qoder.plugins.configPlaceholder')"
              />
            </div>
          </div>

          <div class="mt-8 flex gap-4 border-t border-[var(--color-border-default)]/50 pt-6">
            <button
              class="flex-1 rounded-xl border border-[var(--color-border-default)] bg-white px-6 py-3.5 font-medium text-[var(--color-text-secondary)] transition-colors hover:bg-[var(--color-bg-surface)]"
              @click="closeModal"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              class="flex-1 rounded-xl bg-[var(--color-accent-primary)] px-6 py-3.5 font-medium text-white shadow-lg shadow-[var(--color-accent-primary)]/20 transition-[color,background-color,border-color,transform] hover:-translate-y-0.5 hover:shadow-[var(--color-accent-primary)]/30"
              :disabled="saving"
              @click="handleSubmit"
            >
              {{ saving ? `${t('common.saving')}...` : (editingHook ? t('common.save') : t('common.add')) }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import Card from '@/components/ui/Card.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { addQoderHook, deleteQoderHook, listQoderHooks, updateQoderHook } from '@/api'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'

interface QoderHookItem {
  id: number
  name: string
  event: string
  command: string
}

const uiStore = useUIStore()
const { t } = useI18n()

const hooks = ref<QoderHookItem[]>([])
const loading = ref(false)
const saving = ref(false)
const showModal = ref(false)
const editingHook = ref<QoderHookItem | null>(null)
const form = ref({
  command: '',
})

const loadHooks = async () => {
  loading.value = true
  try {
    const response = await listQoderHooks<{ hooks?: QoderHookItem[] }>()
    hooks.value = response.hooks ?? []
  } catch (error) {
    logger.error('Failed to load qoder hooks', error)
    uiStore.showError('Failed to load Qoder hooks')
  } finally {
    loading.value = false
  }
}

const openCreateModal = () => {
  editingHook.value = null
  form.value = { command: '' }
  showModal.value = true
}

const openEditModal = (hook: QoderHookItem) => {
  editingHook.value = hook
  form.value = { command: hook.command }
  showModal.value = true
}

const closeModal = () => {
  showModal.value = false
  editingHook.value = null
}

const handleSubmit = async () => {
  if (!form.value.command.trim()) {
    uiStore.showWarning('Hook command is required')
    return
  }

  saving.value = true
  try {
    if (editingHook.value) {
      await updateQoderHook(editingHook.value.id, { command: form.value.command.trim() })
      uiStore.showSuccess(t('qoder.plugins.messages.updateSuccess'))
    } else {
      await addQoderHook({ command: form.value.command.trim() })
      uiStore.showSuccess(t('qoder.plugins.messages.addSuccess'))
    }
    closeModal()
    await loadHooks()
  } catch (error) {
    logger.error('Failed to save qoder hook', error)
    uiStore.showError(error instanceof Error ? error.message : 'Failed to save Qoder hook')
  } finally {
    saving.value = false
  }
}

const handleDelete = async (hook: QoderHookItem) => {
  const confirmed = await uiStore.requestConfirm({
    title: 'Delete hook',
    message: `Delete ${hook.name}?`,
    confirmText: 'Delete',
    cancelText: 'Cancel',
    type: 'danger',
  })

  if (!confirmed) return

  try {
    await deleteQoderHook(hook.id)
    uiStore.showSuccess(t('qoder.plugins.messages.deleteSuccess'))
    await loadHooks()
  } catch (error) {
    logger.error('Failed to delete qoder hook', error)
    uiStore.showError(error instanceof Error ? error.message : 'Failed to delete Qoder hook')
  }
}

onMounted(() => {
  loadHooks()
})
</script>
