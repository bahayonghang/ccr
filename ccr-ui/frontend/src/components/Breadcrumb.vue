<template>
  <nav class="flex items-center gap-2 mb-6" aria-label="Breadcrumb">
    <template v-for="(item, index) in items" :key="index">
      <!-- 非最后一项：可点击链接 -->
      <template v-if="index < items.length - 1">
        <RouterLink
          :to="item.path"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg transition-all hover:scale-105"
          :style="{
            color: 'var(--text-secondary)',
            background: 'transparent',
            fontSize: '14px',
            fontWeight: '500'
          }"
          @mouseenter="(e) => onHover(e.currentTarget, true)"
          @mouseleave="(e) => onHover(e.currentTarget, false)"
        >
          <component
            v-if="item.icon"
            :is="item.icon"
            class="w-4 h-4"
          />
          <span>{{ item.label }}</span>
        </RouterLink>
        
        <!-- 分隔符 -->
        <ChevronRight
          class="w-4 h-4"
          :style="{ color: 'var(--text-muted)', opacity: 0.5 }"
        />
      </template>
      
      <!-- 最后一项：当前页面，不可点击 -->
      <template v-else>
        <div
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg"
          :style="{
            color: 'var(--text-primary)',
            background: moduleColor ? `${moduleColor}15` : 'var(--bg-tertiary)',
            fontSize: '14px',
            fontWeight: '600',
            border: moduleColor ? `1px solid ${moduleColor}30` : '1px solid var(--border-color)'
          }"
        >
          <component
            v-if="item.icon"
            :is="item.icon"
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

const onHover = (element: HTMLElement, isHover: boolean) => {
  if (isHover) {
    element.style.background = 'var(--bg-secondary)'
    element.style.color = 'var(--text-primary)'
  } else {
    element.style.background = 'transparent'
    element.style.color = 'var(--text-secondary)'
  }
}
</script>
