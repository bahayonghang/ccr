<template>
  <div class="glass-card p-6 transition-[transform,box-shadow] duration-300 hover:scale-[1.01]">
    <div class="mb-6 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div
          class="rounded-2xl p-3"
          :style="{ background: 'rgb(var(--color-info-rgb) / 10%)' }"
        >
          <SIcon
            name="Folders"
            size="w-6 h-6"
            :style="{ color: 'var(--accent-info)' }"
          />
        </div>
        <h2
          class="text-2xl font-bold"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ $t('sync.enabledFolders.title') }}
        </h2>
      </div>
      <button
        class="glass-card flex items-center gap-2 rounded-xl px-4 py-2.5 transition-[color,background-color,border-color,transform] duration-300 hover:scale-105"
        :style="{ background: 'rgb(var(--color-info-rgb) / 10%)', color: 'var(--accent-info)' }"
        @click="refreshFolders"
      >
        <SIcon
          name="RefreshCw"
          size="w-4 h-4"
          :class="{ 'animate-spin': refreshingFolders }"
        />
        <span class="font-medium">{{ $t('sync.enabledFolders.refresh') }}</span>
      </button>
    </div>

    <div
      v-if="folders.length === 0"
      class="py-12 text-center"
    >
      <div
        class="inline-block rounded-2xl p-4"
        :style="{ background: 'rgb(var(--color-gray-rgb) / 10%)' }"
      >
        <SIcon
          name="FolderOpen"
          size="w-16 h-16"
          :style="{ color: 'var(--text-muted)' }"
        />
      </div>
      <p
        class="mt-4 text-lg"
        :style="{ color: 'var(--text-secondary)' }"
      >
        {{ $t('sync.enabledFolders.noFolders') }}
      </p>
      <p
        class="mt-2 text-sm"
        :style="{ color: 'var(--text-muted)' }"
      >
        {{ $t('sync.enabledFolders.noFoldersHint') }}
      </p>
    </div>

    <div
      v-else
      class="space-y-4"
    >
      <div
        v-for="(folder, index) in folders"
        :key="folder.name"
        class="glass-card rounded-xl p-5 transition-[transform,box-shadow] duration-300 hover:scale-[1.01]"
        :style="{ animationDelay: `${index * 0.05}s` }"
      >
        <div class="mb-4 flex items-start justify-between">
          <div class="flex-1">
            <div class="mb-2 flex items-center gap-3">
              <h4
                class="text-xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ folder.name }}
              </h4>
              <span
                class="rounded-full px-3 py-1 text-sm font-medium"
                :style="{
                  background: folder.enabled ? 'rgb(var(--color-success-rgb) / 15%)' : 'rgb(var(--color-gray-rgb) / 15%)',
                  color: folder.enabled ? 'var(--accent-success)' : 'var(--text-muted)'
                }"
              >
                {{ folder.enabled ? $t('sync.enabledFolders.enabled') : $t('sync.enabledFolders.disabled') }}
              </span>
            </div>
            <p
              v-if="folder.description"
              class="mb-2 text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              {{ folder.description }}
            </p>
            <div class="grid grid-cols-1 gap-2 text-sm md:grid-cols-2">
              <div
                class="flex items-center gap-2"
                :style="{ color: 'var(--text-secondary)' }"
              >
                <SIcon
                  name="Folder"
                  size="w-4 h-4"
                />
                <span class="font-mono">{{ folder.localPath }}</span>
              </div>
              <div
                class="flex items-center gap-2"
                :style="{ color: 'var(--text-secondary)' }"
              >
                <SIcon
                  name="Cloud"
                  size="w-4 h-4"
                />
                <span class="font-mono">{{ folder.remotePath }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="flex flex-wrap gap-2">
          <button
            class="glass-card flex items-center gap-2 rounded-lg px-4 py-2 font-medium transition-[color,background-color,border-color,transform] duration-300 hover:scale-105"
            :style="{ background: 'rgb(var(--color-accent-primary-rgb) / 10%)', color: 'var(--accent-primary)' }"
            @click="toggleFolder(folder.name, folder.enabled)"
          >
            <SIcon
              name="ToggleLeft"
              size="w-4 h-4"
            />
            {{ folder.enabled ? $t('sync.operations.disable') : $t('sync.operations.enable') }}
          </button>
          <button
            :disabled="!folder.enabled"
            class="glass-card flex items-center gap-2 rounded-lg px-4 py-2 font-medium transition-[color,background-color,border-color,transform] duration-300 hover:scale-105 disabled:cursor-not-allowed disabled:opacity-50"
            :style="{ background: 'rgb(var(--color-success-rgb) / 10%)', color: 'var(--accent-success)' }"
            @click="pushFolder(folder.name)"
          >
            <SIcon
              name="Upload"
              size="w-4 h-4"
            />
            {{ $t('sync.operations.upload') }}
          </button>
          <button
            :disabled="!folder.enabled"
            class="glass-card flex items-center gap-2 rounded-lg px-4 py-2 font-medium transition-[color,background-color,border-color,transform] duration-300 hover:scale-105 disabled:cursor-not-allowed disabled:opacity-50"
            :style="{ background: 'rgb(var(--color-accent-secondary-rgb) / 10%)', color: 'var(--accent-secondary)' }"
            @click="pullFolder(folder.name)"
          >
            <SIcon
              name="Download"
              size="w-4 h-4"
            />
            {{ $t('sync.operations.download') }}
          </button>
          <button
            class="glass-card flex items-center gap-2 rounded-lg px-4 py-2 font-medium transition-[color,background-color,border-color,transform] duration-300 hover:scale-105"
            :style="{ background: 'rgb(var(--color-info-rgb) / 10%)', color: 'var(--accent-info)' }"
            @click="getFolderStatus(folder.name)"
          >
            <SIcon
              name="Info"
              size="w-4 h-4"
            />
            {{ $t('sync.operations.status') }}
          </button>
          <button
            class="glass-card flex items-center gap-2 rounded-lg px-4 py-2 font-medium transition-[color,background-color,border-color,transform] duration-300 hover:scale-105"
            :style="{ background: 'rgb(var(--color-danger-rgb) / 10%)', color: 'var(--accent-danger)' }"
            @click="removeFolder(folder.name)"
          >
            <SIcon
              name="Trash2"
              size="w-4 h-4"
            />
            {{ $t('sync.operations.delete') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { SyncManagedFolder } from '@/types/syncSelection'

interface Props {
  folders: SyncManagedFolder[]
  refreshingFolders: boolean
  refreshFolders: () => void
  toggleFolder: (name: string, currentEnabled: boolean) => void
  pushFolder: (name: string) => void
  pullFolder: (name: string) => void
  getFolderStatus: (name: string) => void
  removeFolder: (name: string) => void
}

defineProps<Props>()
</script>
