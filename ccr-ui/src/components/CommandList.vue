<template>
  <div
    v-if="loading"
    class="text-center py-8"
    :style="{ color: 'var(--text-secondary)' }"
  >
    <div class="inline-flex items-center gap-2">
      <SIcon
        name="RefreshCw"
        size="w-4 h-4"
        class="animate-spin"
      />
      <span>{{ $t('common.loading') }}</span>
    </div>
  </div>

  <div
    v-else
    class="grid gap-4 grid-cols-1 lg:grid-cols-2"
  >
    <div
      v-for="command in commands"
      :key="command.name"
      class="rounded-xl p-4 transition-[box-shadow] duration-200 hover:shadow-md flex"
      :style="{
        background: 'var(--bg-secondary)',
        border: '1px solid var(--border-color)'
      }"
    >
      <div class="flex-1 min-w-0">
        <!-- 命令名称和状态 -->
        <div class="flex items-center justify-between mb-2">
          <h3
            class="text-base font-semibold"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ command.name }}
          </h3>
          <span
            class="px-2 py-1 rounded-full text-xs font-medium"
            :style="statusStyle(command.enabled)"
          >
            {{ command.enabled ? $t('common.enabled') : $t('common.disabled') }}
          </span>
        </div>

        <!-- 命令内容 -->
        <div class="mb-2">
          <code
            class="block text-sm px-2 py-1 rounded font-mono"
            :style="{
              background: 'var(--bg-tertiary)',
              color: 'var(--text-primary)'
            }"
          >
            {{ command.command }}
          </code>
        </div>

        <!-- 描述 -->
        <p
          class="text-sm"
          :style="{ color: 'var(--text-secondary)' }"
        >
          {{ command.description }}
        </p>

        <!-- 文件夹标签 -->
        <div class="flex items-center gap-2 mt-3">
          <span
            class="px-2 py-1 rounded-md text-xs"
            :style="{
              background: 'color-mix(in srgb, var(--accent-primary) 10%, transparent)',
              color: 'var(--accent-primary)'
            }"
          >
            {{ command.folder }}
          </span>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center gap-2 ml-4">
        <button
          class="p-2 rounded-lg transition-colors hover:opacity-80"
          :style="{
            color: 'var(--text-muted)',
            background: 'transparent'
          }"
          :title="command.enabled ? $t('common.disable') : $t('common.enable')"
          :aria-label="command.enabled ? $t('common.disable') : $t('common.enable')"
          @click="$emit('toggle', command.name)"
        >
          <SIcon
            name="Power"
            size="w-4 h-4"
          />
        </button>
        <button
          class="p-2 rounded-lg transition-colors hover:opacity-80"
          :style="{
            color: 'var(--text-muted)',
            background: 'transparent'
          }"
          :title="$t('common.edit')"
          :aria-label="$t('common.edit')"
          @click="$emit('edit', command)"
        >
          <SIcon
            name="Edit"
            size="w-4 h-4"
          />
        </button>
        <button
          class="p-2 rounded-lg transition-colors hover:text-red-500"
          :style="{
            color: 'var(--text-muted)',
            background: 'transparent'
          }"
          :title="$t('common.delete')"
          :aria-label="$t('common.delete')"
          @click="$emit('delete', command.name)"
        >
          <SIcon
            name="Trash2"
            size="w-4 h-4"
          />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { SlashCommand } from '@/types/platform'
// Props - 简化后只保留必要的属性
interface Props {
  commands: SlashCommand[]
  loading: boolean
}

defineProps<Props>()

// Emits
interface Emits {
  (e: 'edit', command: SlashCommand): void
  (e: 'delete', name: string): void
  (e: 'toggle', name: string): void
}

defineEmits<Emits>()

// 状态样式 - 使用 CSS 变量
const statusStyle = (enabled: boolean) => ({
  background: enabled
    ? 'rgb(var(--color-success-rgb) / 15%)'
    : 'rgb(var(--color-danger-rgb) / 15%)',
  color: enabled
    ? 'var(--color-success)'
    : 'var(--color-danger)',
})
</script>
