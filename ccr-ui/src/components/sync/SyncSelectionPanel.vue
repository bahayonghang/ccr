<template>
  <div class="glass-effect rounded-2xl border border-white/5 p-6 transition-[transform,box-shadow] duration-300 hover:scale-[1.01] hover:border-cyan-500/30">
    <div class="mb-6 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="rounded-2xl border border-success/30 bg-success/15 p-3">
          <SIcon
            name="CheckSquare"
            size="w-6 h-6"
            class="text-success"
          />
        </div>
        <h2 class="text-2xl font-bold text-white">
          {{ $t('sync.platformSelection.title') }}
        </h2>
      </div>
      <button
        :disabled="applying || !hasChanges"
        class="flex items-center gap-2 rounded-xl border px-4 py-2.5 font-medium transition-[color,background-color,border-color,transform] duration-300 hover:scale-105 disabled:cursor-not-allowed disabled:opacity-50"
        :class="applying || !hasChanges
          ? 'glass-surface border-white/5 text-white/50'
          : 'border-success/30 bg-success/15 text-success hover:bg-success/20'"
        @click="applySelection"
      >
        <SIcon
          name="Save"
          size="w-4 h-4"
        />
        <span>{{ applying ? $t('sync.platformSelection.applying') : $t('sync.platformSelection.applyButton') }}</span>
      </button>
    </div>

    <div class="mb-6 rounded-xl border border-warning/30 bg-warning/5 p-5 glass-effect">
      <div class="flex items-center gap-4">
        <div class="rounded-xl bg-warning/15 p-2">
          <SIcon
            name="CheckCircle"
            size="w-6 h-6"
            class="text-warning"
          />
        </div>
        <div class="flex-1">
          <div class="mb-2 flex items-center gap-3">
            <h3 class="text-lg font-bold text-white">
              {{ $t('sync.platformSelection.configRequired') }}
            </h3>
            <span class="rounded-full border border-warning/30 bg-warning/20 px-2.5 py-1 text-xs font-bold text-warning">
              {{ $t('sync.platformSelection.configRequiredBadge') }}
            </span>
          </div>
          <p class="mb-3 text-sm text-white/80">
            {{ $t('sync.platformSelection.configDescription') }}
          </p>
          <div class="flex items-center gap-2">
            <SIcon
              name="Folder"
              size="w-4 h-4"
              class="text-white/50"
            />
            <input
              :value="presetConfig.localPath"
              type="text"
              class="glass-surface flex-1 rounded-lg border border-white/5 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-cyan-500/30"
              :placeholder="$t('sync.customFolder.localPathPlaceholder')"
              @input="updatePresetLocalPath(($event.target as HTMLInputElement).value)"
            >
          </div>
        </div>
      </div>
    </div>

    <div class="space-y-4">
      <div
        v-for="(item, index) in optionalItems"
        :key="item.key"
        class="glass-card cursor-pointer rounded-xl p-5 transition-[transform,box-shadow] duration-300 hover:scale-[1.02]"
        :style="{
          background: item.selected ? 'rgba(var(--color-accent-primary-rgb), 0.05)' : 'transparent',
          animationDelay: `${index * 0.05}s`
        }"
        @click="toggleItem(item.key)"
      >
        <div class="flex items-start gap-4">
          <div class="flex-shrink-0">
            <div
              class="flex h-7 w-7 items-center justify-center rounded-lg transition-colors duration-300"
              :style="{
                background: item.selected ? 'rgba(var(--color-accent-primary-rgb), 0.15)' : 'rgba(var(--color-gray-rgb), 0.1)',
                border: item.selected ? '2px solid var(--color-accent-primary)' : '2px solid var(--border-color)'
              }"
            >
              <SIcon
                v-if="item.selected"
                name="Check"
                size="w-4 h-4"
                :style="{ color: 'var(--color-accent-primary)' }"
              />
            </div>
          </div>
          <div class="flex-1">
            <div class="mb-2 flex items-center gap-3">
              <div
                class="rounded-lg p-2"
                :style="{ background: 'rgba(var(--color-accent-primary-rgb), 0.1)' }"
              >
                <SIcon
                  :name="item.icon || 'Cloud'"
                  size="w-5 h-5"
                  :style="{ color: 'var(--color-accent-primary)' }"
                />
              </div>
              <h3
                class="text-lg font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ item.name }}
              </h3>
            </div>
            <p
              class="mb-3 text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              {{ item.description }}
            </p>
            <div
              v-if="item.selected"
              class="space-y-2"
              @click.stop
            >
              <div class="flex items-center gap-2">
                <SIcon
                  name="Folder"
                  size="w-4 h-4"
                  :style="{ color: 'var(--text-muted)' }"
                />
                <input
                  :value="item.localPath"
                  type="text"
                  class="glass-card flex-1 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2"
                  :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
                  :placeholder="$t('sync.customFolder.localPathPlaceholder')"
                  @input="updateOptionalLocalPath(item.key, ($event.target as HTMLInputElement).value)"
                >
              </div>
              <div class="flex items-center gap-2">
                <SIcon
                  name="Cloud"
                  size="w-4 h-4"
                  :style="{ color: 'var(--text-muted)' }"
                />
                <input
                  :value="item.remotePath"
                  type="text"
                  class="glass-card flex-1 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2"
                  :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
                  :placeholder="$t('sync.customFolder.remotePathPlaceholder')"
                  @input="updateOptionalRemotePath(item.key, ($event.target as HTMLInputElement).value)"
                >
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div
      class="glass-card mt-6 rounded-xl p-5"
      :style="{ background: 'rgba(var(--color-accent-secondary-rgb), 0.05)' }"
    >
      <div class="mb-4 flex items-center gap-3">
        <div
          class="rounded-xl p-2"
          :style="{ background: 'rgba(var(--color-accent-secondary-rgb), 0.15)' }"
        >
          <SIcon
            name="Plus"
            size="w-5 h-5"
            :style="{ color: 'var(--accent-secondary)' }"
          />
        </div>
        <h3
          class="text-lg font-bold"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ $t('sync.customFolder.title') }}
        </h3>
      </div>
      <div class="mb-4 grid grid-cols-1 gap-4 md:grid-cols-2">
        <input
          :value="customFolder.name"
          type="text"
          :placeholder="$t('sync.customFolder.namePlaceholder')"
          class="glass-card rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2"
          :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
          @input="updateCustomField('name', ($event.target as HTMLInputElement).value)"
        >
        <input
          :value="customFolder.localPath"
          type="text"
          :placeholder="$t('sync.customFolder.localPathPlaceholder')"
          class="glass-card rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2"
          :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
          @input="updateCustomField('localPath', ($event.target as HTMLInputElement).value)"
        >
        <input
          :value="customFolder.remotePath"
          type="text"
          :placeholder="$t('sync.customFolder.remotePathPlaceholder')"
          class="glass-card rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2"
          :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
          @input="updateCustomField('remotePath', ($event.target as HTMLInputElement).value)"
        >
        <input
          :value="customFolder.description"
          type="text"
          :placeholder="$t('sync.customFolder.descriptionPlaceholder')"
          class="glass-card rounded-lg px-4 py-2 text-sm focus:outline-none focus:ring-2"
          :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
          @input="updateCustomField('description', ($event.target as HTMLInputElement).value)"
        >
      </div>
      <button
        :disabled="!customFolder.name || !customFolder.localPath || addingCustom"
        class="glass-card flex w-full items-center justify-center gap-2 rounded-lg px-4 py-2.5 font-medium transition-[color,background-color,border-color,transform] duration-300 hover:scale-105 disabled:cursor-not-allowed disabled:opacity-50"
        :style="{ background: 'rgba(var(--color-accent-secondary-rgb), 0.1)', color: 'var(--accent-secondary)' }"
        @click="addCustomFolder"
      >
        <SIcon
          name="Plus"
          size="w-5 h-5"
        />
        {{ addingCustom ? $t('sync.customFolder.adding') : $t('sync.customFolder.addButton') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { CustomSyncFolderForm, SyncSelectableItem } from '@/types/syncSelection'

type CustomFolderField = keyof CustomSyncFolderForm

interface Props {
  applying: boolean
  addingCustom: boolean
  hasChanges: boolean
  presetConfig: SyncSelectableItem
  optionalItems: SyncSelectableItem[]
  customFolder: CustomSyncFolderForm
  toggleItem: (key: string) => void
  applySelection: () => void
  addCustomFolder: () => void
  updatePresetLocalPath: (value: string) => void
  updateOptionalLocalPath: (key: string, value: string) => void
  updateOptionalRemotePath: (key: string, value: string) => void
  updateCustomField: (field: CustomFolderField, value: string) => void
}

defineProps<Props>()
</script>
