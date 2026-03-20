<template>
  <ConfirmModal
    v-if="confirmDialog"
    :is-open="true"
    :title="confirmDialog.title"
    :message="confirmDialog.message"
    :confirm-text="confirmDialog.confirmText"
    :cancel-text="confirmDialog.cancelText"
    :type="confirmDialog.type"
    :surface="confirmDialog.surface"
    @confirm="handleConfirm"
    @cancel="handleCancel"
    @update:is-open="handleOpenChange"
  />
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useUIStore } from '@/store'
import ConfirmModal from '@/components/ConfirmModal.vue'

const uiStore = useUIStore()
const { confirmDialog } = storeToRefs(uiStore)

const handleConfirm = () => {
  uiStore.resolveConfirmDialog(true)
}

const handleCancel = () => {
  uiStore.resolveConfirmDialog(false)
}

const handleOpenChange = (isOpen: boolean) => {
  if (!isOpen) {
    handleCancel()
  }
}
</script>
