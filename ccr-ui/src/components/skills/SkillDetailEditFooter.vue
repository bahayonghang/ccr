<template>
  <div class="modal-footer">
    <button
      class="btn-secondary"
      @click="$emit('cancel')"
    >
      {{ cancelLabel }}
    </button>
    <button
      class="btn-primary"
      :disabled="isSaving"
      @click="$emit('save')"
    >
      <SIcon
        v-if="isSaving"
        name="Loader2"
        size="w-4 h-4"
        class="animate-spin"
      />
      <SIcon
        v-else
        name="Save"
        size="w-4 h-4"
      />
      <span>{{ saveLabel }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

defineProps<{
  isSaving: boolean
  cancelLabel: string
  saveLabel: string
}>()

defineEmits<{
  (e: 'cancel'): void
  (e: 'save'): void
}>()
</script>

<style scoped>
.modal-footer {
  @apply flex items-center justify-end gap-2 border-t border-white/5 p-4 shrink-0;

  background: rgb(0 0 0 / 20%);
}

.btn-secondary {
  @apply px-4 py-2 rounded-xl text-sm font-semibold text-white/80 transition-colors hover:bg-white/5 hover:text-white;
}

.btn-primary {
  @apply flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-bold text-white bg-accent-primary transition-colors disabled:cursor-not-allowed disabled:opacity-50;
}

.btn-primary:hover {
  background: rgb(var(--color-accent-primary-rgb) / 90%);
}
</style>
