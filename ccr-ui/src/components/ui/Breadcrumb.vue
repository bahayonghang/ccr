<template>
  <nav
    class="flex flex-wrap items-center gap-2 mb-6"
    aria-label="Breadcrumb"
  >
    <template
      v-for="(item, index) in items"
      :key="index"
    >
      <!-- 非最后一项：可点击链接 -->
      <template v-if="index < items.length - 1">
        <RouterLink
          :to="item.path || '/'"
          class="flex items-center gap-1.5 rounded-xl border border-white/10 px-3 py-1.5 text-sm font-medium text-text-secondary transition-[color,background-color,border-color,transform] hover:-translate-y-px hover:border-accent-secondary/30 hover:bg-white/10 hover:text-text-primary"
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
          class="w-4 h-4 text-text-muted/70"
        />
      </template>
      
      <!-- 最后一项：当前页面，不可点击 -->
      <template v-else>
        <div
          class="flex items-center gap-1.5 rounded-xl px-3 py-1.5 text-sm font-semibold text-text-primary border glass-surface"
          :class="moduleColor ? '' : 'border-white/20'"
          :style="moduleColor ? {
            background: `color-mix(in srgb, ${moduleColor} 14%, transparent)`,
            borderColor: `color-mix(in srgb, ${moduleColor} 26%, transparent)`
          } : {}"
        >
          <component
            :is="item.icon"
            v-if="item.icon"
            class="w-4 h-4"
            :style="{ color: moduleColor || 'var(--color-accent-primary)' }"
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
  path?: string
  icon?: Component
}

interface Props {
  items: BreadcrumbItem[]
  moduleColor?: string // 模块主题色
}

defineProps<Props>()
</script>
