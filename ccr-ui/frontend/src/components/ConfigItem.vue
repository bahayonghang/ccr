<template>
  <div
    class="group cursor-pointer px-2 py-1.5 rounded-md transition-all duration-150 hover:translate-x-0.5"
    :style="{
      background: config.is_current
        ? 'linear-gradient(135deg, rgba(var(--color-accent-primary-rgb), 0.12), rgba(var(--color-accent-secondary-rgb), 0.12))'
        : 'transparent',
      borderLeft: config.is_current
        ? '2px solid var(--accent-primary)'
        : '2px solid transparent'
    }"
    @click="$emit('click')"
  >
    <div class="flex items-center gap-1.5">
      <!-- 当前指示器 -->
      <div
        v-if="config.is_current"
        class="w-1.5 h-1.5 rounded-full flex-shrink-0 animate-pulse"
        :style="{ background: 'var(--accent-success)' }"
      />
      
      <!-- 配置名称 -->
      <span
        class="text-xs font-medium font-mono truncate flex-1"
        :style="{ 
          color: config.is_current ? 'var(--accent-primary)' : 'var(--text-primary)',
          maxWidth: '120px'
        }"
        :title="config.name"
      >
        {{ config.name }}
      </span>
      
      <!-- 徽章 -->
      <div class="flex gap-0.5 flex-shrink-0">
        <span
          v-if="config.is_current"
          class="px-1 py-0.5 rounded text-[8px] font-bold"
          :style="{
            background: 'var(--accent-success)',
            color: 'white'
          }"
        >
          当前
        </span>
        <span
          v-if="config.is_default"
          class="px-1 py-0.5 rounded text-[8px] font-bold"
          :style="{
            background: 'var(--accent-warning)',
            color: 'white'
          }"
        >
          默认
        </span>
      </div>

      <!-- 悬停箭头 -->
      <ChevronRight
        class="w-3 h-3 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
        :style="{ color: 'var(--accent-primary)' }"
      />
    </div>

    <!-- Provider 信息 (悬停显示) -->
    <div
      v-if="config.provider || config.account"
      class="mt-0.5 flex items-center gap-2 text-[9px] overflow-hidden"
      :style="{ color: 'var(--text-muted)' }"
    >
      <span
        v-if="config.provider"
        class="truncate flex items-center gap-0.5"
      >
        <Building2 class="w-2.5 h-2.5 flex-shrink-0" />
        {{ config.provider }}
      </span>
      <span
        v-if="config.account"
        class="truncate flex items-center gap-0.5"
      >
        <User class="w-2.5 h-2.5 flex-shrink-0" />
        {{ config.account }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ChevronRight, Building2, User } from 'lucide-vue-next'
import type { ConfigItem } from '@/types'

interface Props {
  config: ConfigItem
}

defineProps<Props>()

defineEmits<{
  click: []
}>()
</script>
