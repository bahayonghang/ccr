<template>
  <div
    class="rounded-2xl p-4 shadow-sm sticky top-6 self-start hidden lg:block w-64"
    :style="{
      background: 'var(--bg-secondary)',
      border: '1px solid var(--border-color)'
    }"
  >
    <h3
      class="text-xs font-bold uppercase tracking-wider mb-3 px-2 flex items-center justify-between"
      :style="{ color: 'var(--text-muted)' }"
    >
      {{ $t('slashCommands.folders.title') }}
      <span
        class="px-1.5 py-0.5 rounded text-[10px]"
        :style="{ background: 'var(--bg-tertiary)' }"
      >{{ stats.total }}</span>
    </h3>

    <div
      class="space-y-1"
      role="listbox"
      :aria-label="$t('slashCommands.folders.title')"
    >
      <button
        v-for="folder in folders"
        :key="folder.value"
        type="button"
        class="w-full flex items-center gap-2 px-3 py-2 rounded-xl cursor-pointer text-sm transition-all duration-200 group"
        :style="folderStyle(folder.value)"
        :aria-selected="selectedFolder === folder.value"
        role="option"
        @click="handleFolderClick(folder.value)"
      >
        <component
          :is="folder.icon"
          class="w-4 h-4 transition-transform group-hover:scale-110"
          :style="iconStyle(folder.value)"
          aria-hidden="true"
        />
        <span class="flex-1 truncate text-left">{{ folder.label }}</span>
        <span
          class="text-xs px-1.5 py-0.5 rounded-md transition-colors"
          :style="{ color: 'var(--text-muted)' }"
          aria-hidden="true"
        >
          {{ folder.count }}
        </span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
interface FolderOption {
  label: string
  value: string
  icon: any
  count: number
}

interface Stats {
  total: number
  enabled: number
  disabled: number
  byFolder: Record<string, number>
}

// Props
interface Props {
  folders: FolderOption[]
  selectedFolder: string
  stats: Stats
}

const props = defineProps<Props>()

// Emits
interface Emits {
  (e: 'folder-selected', folder: string): void
}

const emit = defineEmits<Emits>()

// 方法
const folderStyle = (folderValue: string) => {
  const isSelected = props.selectedFolder === folderValue
  return {
    background: isSelected ? 'color-mix(in srgb, var(--accent-primary) 10%, transparent)' : 'transparent',
    color: isSelected ? 'var(--accent-primary)' : 'var(--text-secondary)',
    fontWeight: isSelected ? '500' : '400',
    border: isSelected ? '1px solid color-mix(in srgb, var(--accent-primary) 20%, transparent)' : '1px solid transparent'
  }
}

const iconStyle = (folderValue: string) => {
  const isSelected = props.selectedFolder === folderValue
  return {
    color: isSelected ? 'var(--accent-primary)' : 'var(--text-muted)'
  }
}

const handleFolderClick = (folderValue: string) => {
  emit('folder-selected', folderValue)
}
</script>
