<template>
  <div class="glass-card p-6 transition-[transform,box-shadow] duration-300 hover:scale-[1.01]">
    <div class="mb-4 flex items-center gap-3">
      <div
        class="rounded-2xl p-3"
        :style="{ background: 'rgb(var(--color-warning-rgb) / 10%)' }"
      >
        <SIcon
          name="Layers"
          size="w-6 h-6"
          :style="{ color: 'var(--accent-warning)' }"
        />
      </div>
      <h2
        class="text-2xl font-bold"
        :style="{ color: 'var(--text-primary)' }"
      >
        {{ $t('sync.batchOperations.title') }}
      </h2>
    </div>

    <p
      class="mb-4 text-sm"
      :style="{ color: 'var(--text-secondary)' }"
    >
      {{ $t('sync.batchOperations.description') }}
    </p>

    <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
      <button
        :disabled="batchOperating || foldersCount === 0"
        class="glass-card flex items-center justify-center gap-3 rounded-xl px-6 py-4 font-bold transition-[color,background-color,border-color,transform] duration-300 hover:scale-105 disabled:cursor-not-allowed disabled:opacity-50"
        :style="{ background: 'rgb(var(--color-success-rgb) / 10%)', color: 'var(--accent-success)' }"
        @click="pushAllFolders"
      >
        <SIcon
          name="Upload"
          size="w-5 h-5"
        />
        {{ $t('sync.batchOperations.uploadAll') }}
      </button>
      <button
        :disabled="batchOperating || foldersCount === 0"
        class="glass-card flex items-center justify-center gap-3 rounded-xl px-6 py-4 font-bold transition-[color,background-color,border-color,transform] duration-300 hover:scale-105 disabled:cursor-not-allowed disabled:opacity-50"
        :style="{ background: 'rgb(var(--color-accent-secondary-rgb) / 10%)', color: 'var(--accent-secondary)' }"
        @click="pullAllFolders"
      >
        <SIcon
          name="Download"
          size="w-5 h-5"
        />
        {{ $t('sync.batchOperations.downloadAll') }}
      </button>
      <button
        :disabled="batchOperating || foldersCount === 0"
        class="glass-card flex items-center justify-center gap-3 rounded-xl px-6 py-4 font-bold transition-[color,background-color,border-color,transform] duration-300 hover:scale-105 disabled:cursor-not-allowed disabled:opacity-50"
        :style="{ background: 'rgb(var(--color-info-rgb) / 10%)', color: 'var(--accent-info)' }"
        @click="getAllFoldersStatus"
      >
        <SIcon
          name="Info"
          size="w-5 h-5"
        />
        {{ $t('sync.batchOperations.viewStatus') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

interface Props {
  batchOperating: boolean
  foldersCount: number
  pushAllFolders: () => void
  pullAllFolders: () => void
  getAllFoldersStatus: () => void
}

defineProps<Props>()
</script>
