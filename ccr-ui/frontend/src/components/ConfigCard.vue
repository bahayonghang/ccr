<template>
  <article
    :id="`config-${config.name}`"
    class="relative p-6 transition-all duration-500 hover:scale-[1.02] group"
    :class="config.is_current ? 'config-card-active' : 'glass-elevated'"
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
          background: 'rgba(var(--color-accent-secondary-rgb), 0.08)',
          borderLeft: '3px solid var(--accent-primary)'
        }"
      >
        <FileText
          class="w-3.5 h-3.5 flex-shrink-0"
          :style="{ opacity: 0.8 }"
          aria-hidden="true"
        />
        <span
          class="text-xs font-medium leading-relaxed"
          :style="{ color: 'var(--text-secondary)' }"
        >
          {{ config.description || '无描述' }}
        </span>
      </div>

      <!-- Provider 信息 -->
      <div
        v-if="config.provider"
        class="flex flex-wrap gap-3 py-2"
      >
        <div
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs transition-all"
          :style="{
            background: 'var(--bg-secondary)',
            border: '1px solid var(--border-color)'
          }"
        >
          <Building2
            class="w-3 h-3"
            aria-hidden="true"
          />
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
          <User
            class="w-3 h-3"
            aria-hidden="true"
          />
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
      <div
        v-if="config.tags && config.tags.length > 0"
        class="flex flex-wrap gap-1 mt-2"
      >
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

      <!-- 📊 使用次数显示 -->
      <div
        v-if="config.usage_count !== undefined"
        class="flex items-center gap-2 mt-3"
      >
        <div
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs transition-all"
          :style="{
            background: 'rgba(var(--color-accent-primary-rgb), 0.08)',
            border: '1px solid rgba(var(--color-accent-primary-rgb), 0.2)'
          }"
        >
          <span :style="{ color: 'var(--text-muted)' }">使用次数:</span>
          <span
            class="font-bold font-mono"
            :style="{ color: 'var(--accent-primary)' }"
          >
            {{ config.usage_count }}
          </span>
        </div>

        <!-- 禁用状态徽章 -->
        <div
          v-if="config.enabled === false"
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-semibold"
          :style="{
            background: 'rgba(var(--color-danger-rgb), 0.15)',
            border: '1px solid rgba(var(--color-danger-rgb), 0.3)',
            color: 'var(--color-danger)'
          }"
        >
          ❌ 已禁用
        </div>
      </div>
    </header>

    <!-- 详细信息 -->
    <div class="grid grid-cols-2 gap-2.5 mb-3">
      <DetailField
        label="Base URL"
        :value="config.base_url"
      />
      <DetailField
        label="Auth Token"
        :value="maskToken(config.auth_token)"
      />
      <DetailField
        v-if="config.model"
        label="Model"
        :value="config.model"
      />
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
        :aria-label="`切换到配置 ${config.name}`"
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
        :aria-label="`编辑配置 ${config.name}`"
        @click="$emit('edit', config.name)"
      >
        编辑
      </button>

      <!-- 📊 启用/禁用按钮 -->
      <button
        v-if="config.enabled !== false"
        class="px-3 py-1.5 rounded-lg text-xs font-semibold transition-all hover:scale-105"
        :style="{
          background: 'var(--accent-warning)',
          color: 'white',
          boxShadow: '0 0 15px rgba(var(--color-warning-rgb), 0.3)'
        }"
        :aria-label="`禁用配置 ${config.name}`"
        @click="$emit('disable', config.name)"
      >
        禁用
      </button>

      <button
        v-else
        class="px-3 py-1.5 rounded-lg text-xs font-semibold text-white transition-all hover:scale-105"
        :style="{
          background: 'var(--accent-success)',
          boxShadow: '0 0 15px rgba(var(--color-success-rgb), 0.3)'
        }"
        :aria-label="`启用配置 ${config.name}`"
        @click="$emit('enable', config.name)"
      >
        启用
      </button>

      <button
        v-if="!config.is_current && !config.is_default"
        class="px-3 py-1.5 rounded-lg text-xs font-semibold text-white transition-all hover:scale-105"
        :style="{
          background: 'var(--accent-danger)',
          boxShadow: '0 0 20px var(--glow-danger)'
        }"
        :aria-label="`删除配置 ${config.name}`"
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
import { maskToken } from '@/utils/codexHelpers'

interface Props {
  config: ConfigItem
}

const props = defineProps<Props>()

defineEmits<{
  switch: [configName: string]
  edit: [configName: string]
  delete: [configName: string]
  enable: [configName: string]
  disable: [configName: string]
}>()

// Provider Type 徽章
const providerTypeBadge = computed(() => {
  if (!props.config.provider_type) return null

  const typeMap: Record<string, { text: string; background: string; color: string; border: string }> = {
    'OfficialRelay': {
      text: '🔄 官方中转',
      background: 'rgba(var(--color-info-rgb), 0.2)',
      color: 'var(--accent-info)',
      border: 'rgba(var(--color-info-rgb), 0.4)'
    },
    'official_relay': {
      text: '🔄 官方中转',
      background: 'rgba(var(--color-info-rgb), 0.2)',
      color: 'var(--accent-info)',
      border: 'rgba(var(--color-info-rgb), 0.4)'
    },
    'ThirdPartyModel': {
      text: '🤖 第三方模型',
      background: 'rgba(var(--color-purple-rgb), 0.2)',
      color: 'var(--color-purple)',
      border: 'rgba(var(--color-purple-rgb), 0.4)'
    },
    'third_party_model': {
      text: '🤖 第三方模型',
      background: 'rgba(var(--color-purple-rgb), 0.2)',
      color: 'var(--color-purple)',
      border: 'rgba(var(--color-purple-rgb), 0.4)'
    }
  }

  return typeMap[props.config.provider_type]
})

</script>
