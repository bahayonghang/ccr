<template>
  <div class="modal-header">
    <div class="flex min-w-0 items-center gap-3">
      <div
        class="modal-icon shrink-0"
        :style="{ backgroundColor: `${platformColor}15`, color: platformColor }"
      >
        <SIcon
          :name="platformIcon"
          size="w-5 h-5"
        />
      </div>
      <div class="min-w-0">
        <h2 class="truncate text-lg font-bold text-white">
          {{ title }}
        </h2>
        <p class="truncate text-xs text-text-muted">
          {{ subtitle }}
        </p>
      </div>
    </div>
    <div class="flex shrink-0 items-center gap-1">
      <button
        class="rounded-lg p-2 text-text-muted transition-colors hover:bg-bg-surface/70 hover:text-white"
        :title="isEditMode ? previewTitle : editTitle"
        @click="$emit('toggle-mode')"
      >
        <SIcon
          v-if="isEditMode"
          name="Eye"
          size="w-5 h-5"
        />
        <SIcon
          v-else
          name="Edit3"
          size="w-5 h-5"
        />
      </button>
      <button
        class="rounded-lg p-2 text-text-muted transition-colors hover:bg-bg-surface/70 hover:text-white"
        @click="$emit('close')"
      >
        <SIcon
          name="X"
          size="w-5 h-5"
        />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

defineProps<{
  title: string
  subtitle: string
  platformColor: string
  platformIcon: string
  isEditMode: boolean
  previewTitle: string
  editTitle: string
}>()

defineEmits<{
  (e: 'toggle-mode'): void
  (e: 'close'): void
}>()
</script>

<style scoped>
.modal-header {
  @apply flex items-center justify-between border-b border-border-default/10 p-4 shrink-0;
}

.modal-icon {
  @apply flex h-10 w-10 items-center justify-center rounded-xl;
}
</style>

