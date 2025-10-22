<template>
  <aside
    class="sticky top-6 h-fit rounded-xl p-5"
    :style="{
      background: 'var(--bg-secondary)',
      border: '1px solid var(--border-color)',
      boxShadow: 'var(--shadow-small)',
      maxHeight: 'calc(100vh - 200px)',
      overflowY: 'auto'
    }"
  >
    <!-- 标题 -->
    <div class="mb-5 pb-4" :style="{ borderBottom: '1px solid var(--border-color)' }">
      <h2 class="text-lg font-bold mb-1" :style="{ color: 'var(--text-primary)' }">
        快速导航
      </h2>
      <p class="text-xs" :style="{ color: 'var(--text-muted)' }">
        点击跳转到配置
      </p>
    </div>

    <!-- 统计信息 -->
    <div class="grid grid-cols-2 gap-3 mb-5">
      <div
        class="p-3 rounded-lg text-center"
        :style="{
          background: 'rgba(99, 102, 241, 0.1)',
          border: '1px solid rgba(99, 102, 241, 0.3)'
        }"
      >
        <div class="text-2xl font-bold mb-1" :style="{ color: '#6366f1' }">
          {{ filteredConfigs.length }}
        </div>
        <div class="text-[10px] font-medium" :style="{ color: 'var(--text-muted)' }">
          当前筛选
        </div>
      </div>
      <div
        class="p-3 rounded-lg text-center"
        :style="{
          background: 'rgba(16, 185, 129, 0.1)',
          border: '1px solid rgba(16, 185, 129, 0.3)'
        }"
      >
        <div class="text-2xl font-bold mb-1" :style="{ color: '#10b981' }">
          {{ configs.length }}
        </div>
        <div class="text-[10px] font-medium" :style="{ color: 'var(--text-muted)' }">
          总配置数
        </div>
      </div>
    </div>

    <!-- 配置列表 -->
    <div class="space-y-2">
      <div
        v-for="config in filteredConfigs"
        :key="config.name"
        class="group cursor-pointer p-3 rounded-lg transition-all duration-200 hover:scale-[1.02]"
        :style="{
          background: config.is_current 
            ? 'linear-gradient(135deg, rgba(99, 102, 241, 0.15), rgba(139, 92, 246, 0.15))'
            : 'var(--bg-tertiary)',
          border: config.is_current 
            ? '1.5px solid var(--accent-primary)'
            : '1px solid var(--border-color)',
          boxShadow: config.is_current 
            ? '0 0 20px var(--glow-primary)'
            : 'none'
        }"
        @click="$emit('configClick', config.name)"
      >
        <!-- 配置名称和徽章 -->
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <!-- 当前指示器 -->
            <div
              v-if="config.is_current"
              class="w-2 h-2 rounded-full flex-shrink-0 animate-pulse"
              :style="{ background: 'var(--accent-success)' }"
            />
            
            <span
              class="text-sm font-bold font-mono truncate"
              :style="{ color: config.is_current ? 'var(--accent-primary)' : 'var(--text-primary)' }"
            >
              {{ config.name }}
            </span>
          </div>
          
          <!-- 徽章 -->
          <div class="flex gap-1 flex-shrink-0">
            <span
              v-if="config.is_current"
              class="px-1.5 py-0.5 rounded text-[9px] font-bold uppercase"
              :style="{
                background: 'var(--accent-success)',
                color: 'white'
              }"
            >
              当前
            </span>
            <span
              v-if="config.is_default"
              class="px-1.5 py-0.5 rounded text-[9px] font-bold uppercase"
              :style="{
                background: 'var(--accent-warning)',
                color: 'white'
              }"
            >
              默认
            </span>
          </div>
        </div>

        <!-- Provider Type -->
        <div v-if="config.provider_type" class="mb-2">
          <span
            class="inline-block px-2 py-0.5 rounded text-[9px] font-bold uppercase tracking-wide"
            :style="{
              background: getProviderTypeBadgeStyle(config.provider_type).background,
              color: getProviderTypeBadgeStyle(config.provider_type).color,
              border: `1px solid ${getProviderTypeBadgeStyle(config.provider_type).border}`
            }"
          >
            {{ getProviderTypeText(config.provider_type) }}
          </span>
        </div>

        <!-- Provider 和 Account -->
        <div class="space-y-1">
          <div
            v-if="config.provider"
            class="flex items-center gap-1.5 text-[10px]"
            :style="{ color: 'var(--text-muted)' }"
          >
            <Building2 class="w-3 h-3 flex-shrink-0" />
            <span class="truncate">{{ config.provider }}</span>
          </div>
          <div
            v-if="config.account"
            class="flex items-center gap-1.5 text-[10px]"
            :style="{ color: 'var(--text-muted)' }"
          >
            <User class="w-3 h-3 flex-shrink-0" />
            <span class="truncate">{{ config.account }}</span>
          </div>
        </div>

        <!-- 悬停效果箭头 -->
        <div class="mt-2 flex items-center justify-end opacity-0 group-hover:opacity-100 transition-opacity">
          <ChevronRight
            class="w-4 h-4 transform group-hover:translate-x-1 transition-transform"
            :style="{ color: 'var(--accent-primary)' }"
          />
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-if="filteredConfigs.length === 0"
      class="text-center py-8"
    >
      <div
        class="inline-flex p-4 rounded-full mb-3"
        :style="{ background: 'var(--bg-tertiary)' }"
      >
        <Search class="w-6 h-6" :style="{ color: 'var(--text-muted)' }" />
      </div>
      <p class="text-sm font-medium mb-1" :style="{ color: 'var(--text-secondary)' }">
        未找到配置
      </p>
      <p class="text-xs" :style="{ color: 'var(--text-muted)' }">
        当前筛选条件下无配置
      </p>
    </div>

    <!-- 筛选器提示 -->
    <div
      v-if="currentFilter !== 'all'"
      class="mt-5 pt-4"
      :style="{ borderTop: '1px solid var(--border-color)' }"
    >
      <div
        class="p-3 rounded-lg text-center"
        :style="{
          background: 'rgba(99, 102, 241, 0.08)',
          border: '1px solid rgba(99, 102, 241, 0.2)'
        }"
      >
        <div class="flex items-center justify-center gap-2 text-xs font-medium mb-1">
          <Filter class="w-3.5 h-3.5" :style="{ color: 'var(--accent-primary)' }" />
          <span :style="{ color: 'var(--text-secondary)' }">
            当前筛选
          </span>
        </div>
        <div
          class="text-xs font-bold uppercase tracking-wide"
          :style="{ color: 'var(--accent-primary)' }"
        >
          {{ getFilterLabel(currentFilter) }}
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Building2, User, ChevronRight, Search, Filter } from 'lucide-vue-next'
import type { ConfigItem } from '@/types'

type FilterType = 'all' | 'official_relay' | 'third_party_model' | 'uncategorized'

interface Props {
  configs: ConfigItem[]
  currentFilter: FilterType
}

const props = defineProps<Props>()

defineEmits<{
  configClick: [configName: string]
}>()

// 根据当前筛选器过滤配置
const filteredConfigs = computed(() => {
  if (props.currentFilter === 'all') {
    return props.configs
  } else if (props.currentFilter === 'official_relay') {
    return props.configs.filter(
      c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay'
    )
  } else if (props.currentFilter === 'third_party_model') {
    return props.configs.filter(
      c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model'
    )
  } else if (props.currentFilter === 'uncategorized') {
    return props.configs.filter(c => !c.provider_type)
  }
  return props.configs
})

// Provider Type 徽章样式
const getProviderTypeBadgeStyle = (providerType: string) => {
  if (providerType === 'OfficialRelay' || providerType === 'official_relay') {
    return {
      background: 'rgba(59, 130, 246, 0.15)',
      color: '#3b82f6',
      border: 'rgba(59, 130, 246, 0.3)'
    }
  } else if (providerType === 'ThirdPartyModel' || providerType === 'third_party_model') {
    return {
      background: 'rgba(168, 85, 247, 0.15)',
      color: '#a855f7',
      border: 'rgba(168, 85, 247, 0.3)'
    }
  }
  return {
    background: 'var(--bg-tertiary)',
    color: 'var(--text-muted)',
    border: 'var(--border-color)'
  }
}

// Provider Type 文本
const getProviderTypeText = (providerType: string): string => {
  if (providerType === 'OfficialRelay' || providerType === 'official_relay') {
    return '官方中转'
  } else if (providerType === 'ThirdPartyModel' || providerType === 'third_party_model') {
    return '第三方模型'
  }
  return providerType
}

// 筛选器标签
const getFilterLabel = (filter: FilterType): string => {
  const labels: Record<FilterType, string> = {
    'all': '全部配置',
    'official_relay': '官方中转',
    'third_party_model': '第三方模型',
    'uncategorized': '未分类'
  }
  return labels[filter]
}
</script>

<style scoped>
/* 自定义滚动条 */
aside::-webkit-scrollbar {
  width: 6px;
}

aside::-webkit-scrollbar-track {
  background: var(--bg-tertiary);
  border-radius: 3px;
}

aside::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

aside::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}
</style>
