<template>
  <div class="text-center py-12">
    <div
      class="mx-auto w-16 h-16 flex items-center justify-center rounded-full mb-4"
      :style="{
        background: 'var(--bg-tertiary)',
        color: 'var(--text-muted)'
      }"
    >
      <Command class="w-12 h-12" />
    </div>

    <h3
      class="text-lg font-medium mb-2"
      :style="{ color: 'var(--text-primary)' }"
    >
      {{ titleText }}
    </h3>

    <p
      class="max-w-md mx-auto"
      :style="{ color: 'var(--text-secondary)' }"
    >
      {{ descriptionText }}
    </p>

    <!-- 操作建议 -->
    <div
      v-if="hasSearchQuery"
      class="mt-6"
    >
      <button
        class="inline-flex items-center px-4 py-2 rounded-lg border transition-colors hover:opacity-80"
        :style="{
          background: 'var(--bg-secondary)',
          color: 'var(--text-secondary)',
          border: '1px solid var(--border-color)'
        }"
        @click="$emit('clear-search')"
      >
        {{ $t('common.clearSearch') }}
      </button>
    </div>

    <div
      v-else-if="hasFolderFilter"
      class="mt-6"
    >
      <button
        class="inline-flex items-center px-4 py-2 rounded-lg border transition-colors hover:opacity-80"
        :style="{
          background: 'var(--bg-secondary)',
          color: 'var(--text-secondary)',
          border: '1px solid var(--border-color)'
        }"
        @click="$emit('clear-filter')"
      >
        {{ $t('common.showAll') }}
      </button>
    </div>

    <div
      v-else
      class="mt-6"
    >
      <button
        class="inline-flex items-center px-4 py-2 rounded-lg border transition-colors hover:opacity-80"
        :style="{
          background: 'var(--accent-primary)',
          color: '#fff',
          borderColor: 'var(--accent-primary)'
        }"
        @click="$emit('add-first')"
      >
        <Plus class="w-4 h-4 mr-2" />
        {{ $t('slashCommands.addFirst') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Command, Plus } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// Props
interface Props {
  searchQuery: string
  selectedFolder: string
}

const props = defineProps<Props>()

// Emits
interface Emits {
  (e: 'clear-search'): void
  (e: 'clear-filter'): void
  (e: 'add-first'): void
}

defineEmits<Emits>()

// 计算属性
const hasSearchQuery = computed(() => props.searchQuery.trim() !== '')
const hasFolderFilter = computed(() => props.selectedFolder !== 'all')

const titleText = computed(() => {
  if (hasSearchQuery.value) {
    return t('slashCommands.noSearchResults')
  } else if (hasFolderFilter.value) {
    return t('slashCommands.noCommandsInFolder')
  } else {
    return t('slashCommands.noCommands')
  }
})

const descriptionText = computed(() => {
  if (hasSearchQuery.value) {
    return t('slashCommands.tryDifferentSearch')
  } else if (hasFolderFilter.value) {
    return t('slashCommands.tryDifferentFolder')
  } else {
    return t('slashCommands.addFirstCommand')
  }
})
</script>
