<template>
  <nav
    class="flex items-center gap-2 mb-6"
    aria-label="Breadcrumb"
  >
    <template
      v-for="(item, index) in items"
      :key="index"
    >
      <!-- 非最后一项：可点击链接 -->
      <template v-if="index < items.length - 1">
        <RouterLink
          :to="item.path"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg transition-all hover:scale-105 text-text-secondary bg-transparent text-sm font-medium hover:bg-bg-secondary hover:text-text-primary"
        >
          <component
            :is="item.icon"
            v-if="item.icon"
            class="w-4 h-4"
          />
          <span>{{ item.label }}</span>
        </RouterLink>
        
        <!-- 分隔符 -->
        <ChevronRight
          class="w-4 h-4 text-text-muted opacity-50"
        />
      </template>
      
      <!-- 最后一项：当前页面，不可点击 -->
      <template v-else>
        <div
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-text-primary text-sm font-semibold border"
          :class="moduleColor ? '' : 'bg-bg-tertiary border-border-color'"
          :style="moduleColor ? {
            background: `${moduleColor}15`,
            borderColor: `${moduleColor}30`
          } : {}"
        >
          <component
            :is="item.icon"
            v-if="item.icon"
            class="w-4 h-4"
            :style="{ color: moduleColor || 'var(--accent-primary)' }"
          />
          <span>{{ item.label }}</span>
        </div>
      </template>
    </template>
  </nav>
</template>

<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { ChevronRight } from 'lucide-vue-next'
import type { Component } from 'vue'

interface BreadcrumbItem {
  label: string
  path: string
  icon?: Component
}

interface Props {
  items: BreadcrumbItem[]
  moduleColor?: string // 模块主题色
}

defineProps<Props>()
</script>
