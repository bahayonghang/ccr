<template>
  <div
    class="config-item group cursor-pointer rounded-lg transition-all duration-200"
    :class="{
      'config-item--active': config.is_current,
      'config-item--idle': !config.is_current,
    }"
    @click="$emit('click')"
  >
    <div class="flex items-center gap-2.5 px-2.5 py-2">
      <!-- 提供商首字母头像 -->
      <div
        class="w-7 h-7 rounded-lg flex items-center justify-center text-[10px] font-bold shrink-0 transition-all duration-200"
        :class="avatarClass"
      >
        {{ config.name[0]?.toUpperCase() || '?' }}
      </div>

      <!-- 信息区 -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-1.5">
          <span
            class="text-xs font-semibold truncate transition-colors duration-200"
            :class="config.is_current ? 'text-accent-primary' : 'text-text-primary group-hover:text-text-primary'"
            :title="config.name"
          >
            {{ config.name }}
          </span>
          <!-- 状态点 -->
          <span
            v-if="config.is_current"
            class="w-1.5 h-1.5 rounded-full bg-emerald-400 shrink-0 shadow-[0_0_4px_rgba(52,211,153,0.5)]"
          />
        </div>
        <!-- Provider 信息 -->
        <p
          v-if="config.provider"
          class="text-[10px] text-text-muted truncate mt-0.5 leading-tight"
        >
          {{ config.provider }}
        </p>
      </div>

      <!-- 右侧标签 -->
      <div class="flex items-center gap-1 shrink-0">
        <span
          v-if="config.is_current"
          class="px-1.5 py-0.5 rounded text-[8px] font-bold bg-emerald-500/15 text-emerald-400 ring-1 ring-emerald-500/20"
        >
          当前
        </span>
        <span
          v-if="config.is_default"
          class="px-1.5 py-0.5 rounded text-[8px] font-bold bg-amber-500/15 text-amber-400 ring-1 ring-amber-500/20"
        >
          默认
        </span>
        <ChevronRight
          class="w-3 h-3 text-text-muted opacity-0 group-hover:opacity-70 transition-opacity shrink-0"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ChevronRight } from 'lucide-vue-next'
import type { ConfigItem } from '@/types'

interface Props {
  config: ConfigItem
}

const props = defineProps<Props>()

defineEmits<{
  click: []
}>()

const providerType = computed(() => {
  const type = props.config.provider_type?.toLowerCase() || ''
  if (type.includes('official')) return 'official'
  if (type.includes('third')) return 'third'
  return 'uncategorized'
})

const avatarClass = computed(() => {
  if (props.config.is_current) {
    return 'bg-accent-primary/20 text-accent-primary ring-1 ring-accent-primary/30'
  }
  const map = {
    official: 'bg-cyan-500/10 text-cyan-400 group-hover:bg-cyan-500/15',
    third: 'bg-violet-500/10 text-violet-400 group-hover:bg-violet-500/15',
    uncategorized: 'bg-amber-500/10 text-amber-400 group-hover:bg-amber-500/15',
  }
  return map[providerType.value]
})
</script>

<style scoped>
.config-item--idle {
  background: transparent;
}

.config-item--idle:hover {
  background: var(--color-bg-surface);
}

.config-item--active {
  background: rgb(var(--color-accent-primary-rgb) / 8%);
  backdrop-filter: blur(8px);
  backdrop-filter: blur(8px);
  box-shadow: inset 2px 0 0 var(--color-accent-primary);
}
</style>
