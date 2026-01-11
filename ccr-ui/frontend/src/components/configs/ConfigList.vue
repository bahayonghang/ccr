<template>
  <div class="space-y-6">
    <!-- 加载状态 -->
    <div
      v-if="loading"
      class="flex items-center justify-center py-20"
    >
      <div
        class="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
        :style="{
          borderTopColor: 'var(--accent-primary)',
          borderRightColor: 'var(--accent-secondary)'
        }"
      />
    </div>

    <!-- 错误状态 -->
    <div
      v-else-if="error"
      class="rounded-lg p-4 flex items-center space-x-2"
      :style="{
        background: 'color-mix(in srgb, var(--accent-danger) 10%, transparent)',
        border: '1px solid var(--accent-danger)'
      }"
    >
      <AlertCircle :style="{ color: 'var(--accent-danger)' }" />
      <span :style="{ color: 'var(--accent-danger)' }">
        {{ $t('configs.operationFailed') }}: {{ error }}
      </span>
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="configs.length === 0"
      class="text-center py-10"
      :style="{ color: 'var(--text-muted)' }"
    >
      {{ $t('configs.noConfigsInCategory') }}
    </div>

    <!-- 配置卡片列表 -->
    <template v-else>
      <ConfigCard
        v-for="config in configs"
        :key="config.name"
        :config="config"
        @switch="$emit('switch', $event)"
        @edit="$emit('edit', $event)"
        @delete="$emit('delete', $event)"
        @enable="$emit('enable', $event)"
        @disable="$emit('disable', $event)"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { AlertCircle } from 'lucide-vue-next'
import type { ConfigItem } from '@/types'
import ConfigCard from '@/components/ConfigCard.vue'

interface Props {
  configs: ConfigItem[]
  loading: boolean
  error: string | null
}

defineProps<Props>()

interface Emits {
  (e: 'switch', configName: string): void
  (e: 'edit', configName: string): void
  (e: 'delete', configName: string): void
  (e: 'enable', configName: string): void
  (e: 'disable', configName: string): void
}

defineEmits<Emits>()
</script>
