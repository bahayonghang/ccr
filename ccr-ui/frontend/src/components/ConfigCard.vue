<template>
  <article
    :id="`config-${config.name}`"
    class="glass-card rounded-xl p-5 transition-all duration-300 hover:scale-[1.01]"
    :style="{
      background: config.is_current
        ? 'linear-gradient(135deg, rgba(139, 92, 246, 0.15), rgba(168, 85, 247, 0.15))'
        : 'rgba(255, 255, 255, 0.03)',
      border: `1.5px solid ${config.is_current ? 'var(--accent-primary)' : 'rgba(139, 92, 246, 0.2)'}`,
      boxShadow: config.is_current
        ? '0 0 30px var(--glow-primary), inset 0 1px 0 0 rgba(255, 255, 255, 0.1)'
        : '0 4px 16px rgba(0, 0, 0, 0.2), inset 0 1px 0 0 rgba(255, 255, 255, 0.05)',
      backdropFilter: 'blur(16px) saturate(180%)',
      WebkitBackdropFilter: 'blur(16px) saturate(180%)'
    }"
  >
    <!-- 头部 -->
    <header class="mb-3">
      <h3 class="flex items-center flex-wrap gap-2 mb-2">
        <!-- Provider Type 徽章 -->
        <span
          v-if="providerTypeBadge"
          class="inline-block px-2.5 py-0.5 rounded-lg text-xs font-semibold uppercase tracking-wide"
          :style="{
            background: providerTypeBadge.background,
            color: providerTypeBadge.color,
            border: `1px solid ${providerTypeBadge.border}`
          }"
        >
          {{ providerTypeBadge.text }}
        </span>

        <!-- 配置名称 -->
        <span
          class="text-base font-bold font-mono tracking-wide"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ config.name }}
        </span>

        <!-- 当前徽章 -->
        <span
          v-if="config.is_current"
          class="px-2 py-0.5 rounded-lg text-xs font-semibold uppercase"
          :style="{
            background: 'var(--accent-success)',
            color: 'white'
          }"
        >
          当前
        </span>

        <!-- 默认徽章 -->
        <span
          v-if="config.is_default"
          class="px-2 py-0.5 rounded-lg text-xs font-semibold uppercase"
          :style="{
            background: 'var(--accent-warning)',
            color: 'white'
          }"
        >
          默认
        </span>
      </h3>

      <!-- 描述 -->
      <div
        class="flex items-center gap-1.5 p-2 px-3 rounded-md mb-2.5 transition-all hover:translate-x-0.5"
        :style="{
          background: 'rgba(139, 92, 246, 0.08)',
          borderLeft: '3px solid var(--accent-primary)'
        }"
      >
        <FileText class="w-3.5 h-3.5 flex-shrink-0" :style="{ opacity: 0.8 }" />
        <span
          class="text-xs font-medium leading-relaxed"
          :style="{ color: 'var(--text-secondary)' }"
        >
          {{ config.description || '无描述' }}
        </span>
      </div>

      <!-- Provider 信息 -->
      <div v-if="config.provider" class="flex flex-wrap gap-3 py-2">
        <div
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs transition-all"
          :style="{
            background: 'var(--bg-secondary)',
            border: '1px solid var(--border-color)'
          }"
        >
          <Building2 class="w-3 h-3" />
          <span :style="{ color: 'var(--text-muted)' }">提供商:</span>
          <span
            class="font-semibold font-mono"
            :style="{ color: 'var(--accent-secondary)' }"
          >
            {{ config.provider }}
          </span>
        </div>

        <div
          v-if="config.account"
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs transition-all"
          :style="{
            background: 'var(--bg-secondary)',
            border: '1px solid var(--border-color)'
          }"
        >
          <User class="w-3 h-3" />
          <span :style="{ color: 'var(--text-muted)' }">账号:</span>
          <span
            class="font-semibold font-mono"
            :style="{ color: 'var(--accent-success)' }"
          >
            {{ config.account }}
          </span>
        </div>
      </div>

      <!-- 标签 -->
      <div v-if="config.tags && config.tags.length > 0" class="flex flex-wrap gap-1 mt-2">
        <span
          v-for="tag in config.tags"
          :key="tag"
          class="px-2 py-0.5 rounded-md text-xs transition-all"
          :style="{
            background: 'var(--bg-secondary)',
            border: '1px solid var(--border-color)',
            color: 'var(--text-muted)'
          }"
        >
          {{ tag }}
        </span>
      </div>
    </header>

    <!-- 详细信息 -->
    <div class="grid grid-cols-2 gap-2.5 mb-3">
      <DetailField label="Base URL" :value="config.base_url" />
      <DetailField label="Auth Token" :value="maskToken(config.auth_token)" />
      <DetailField v-if="config.model" label="Model" :value="config.model" />
      <DetailField
        v-if="config.small_fast_model"
        label="Small Fast Model"
        :value="config.small_fast_model"
      />
    </div>

    <!-- 操作按钮 -->
    <div class="flex gap-2 flex-wrap">
      <button
        v-if="!config.is_current"
        class="btn-primary px-3 py-1.5 rounded-lg text-xs font-semibold text-white hover:scale-105 transition-transform"
        :style="{
          background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
          boxShadow: '0 0 20px var(--glow-primary)'
        }"
        @click="$emit('switch', config.name)"
      >
        切换
      </button>

      <button
        class="px-3 py-1.5 rounded-lg text-xs font-semibold transition-all hover:scale-105"
        :style="{
          background: 'var(--bg-tertiary)',
          color: 'var(--text-primary)',
          border: '1px solid var(--border-color)'
        }"
        @click="$emit('edit', config.name)"
      >
        编辑
      </button>

      <button
        v-if="!config.is_current && !config.is_default"
        class="px-3 py-1.5 rounded-lg text-xs font-semibold text-white transition-all hover:scale-105"
        :style="{
          background: 'var(--accent-danger)',
          boxShadow: '0 0 20px var(--glow-danger)'
        }"
        @click="$emit('delete', config.name)"
      >
        删除
      </button>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { FileText, Building2, User } from 'lucide-vue-next'
import type { ConfigItem } from '@/types'
import DetailField from './DetailField.vue'

interface Props {
  config: ConfigItem
}

const props = defineProps<Props>()

defineEmits<{
  switch: [configName: string]
  edit: [configName: string]
  delete: [configName: string]
}>()

// Provider Type 徽章
const providerTypeBadge = computed(() => {
  if (!props.config.provider_type) return null

  const typeMap: Record<string, { text: string; background: string; color: string; border: string }> = {
    'OfficialRelay': {
      text: '🔄 官方中转',
      background: 'rgba(59, 130, 246, 0.2)',
      color: '#3b82f6',
      border: 'rgba(59, 130, 246, 0.4)'
    },
    'official_relay': {
      text: '🔄 官方中转',
      background: 'rgba(59, 130, 246, 0.2)',
      color: '#3b82f6',
      border: 'rgba(59, 130, 246, 0.4)'
    },
    'ThirdPartyModel': {
      text: '🤖 第三方模型',
      background: 'rgba(168, 85, 247, 0.2)',
      color: '#a855f7',
      border: 'rgba(168, 85, 247, 0.4)'
    },
    'third_party_model': {
      text: '🤖 第三方模型',
      background: 'rgba(168, 85, 247, 0.2)',
      color: '#a855f7',
      border: 'rgba(168, 85, 247, 0.4)'
    }
  }

  return typeMap[props.config.provider_type]
})

// 遮蔽 Token
const maskToken = (token: string): string => {
  if (!token) return ''
  if (token.length <= 8) return '***'
  return `${token.substring(0, 4)}...${token.substring(token.length - 4)}`
}
</script>
