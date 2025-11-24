<template>
  <nav class="flex items-center text-sm text-guofeng-text-secondary mb-6 animate-fade-in">
    <template v-for="(item, index) in items" :key="index">
      <RouterLink
        v-if="item.to"
        :to="item.to"
        class="hover:text-guofeng-jade transition-colors flex items-center gap-1"
      >
        <component :is="item.icon" v-if="item.icon" class="w-3.5 h-3.5" />
        {{ item.label }}
      </RouterLink>
      <span v-else class="font-medium text-guofeng-text-primary bg-guofeng-bg-secondary px-2 py-0.5 rounded-md border border-guofeng-border/50">
        {{ item.label }}
      </span>
      <ChevronRight v-if="index < items.length - 1" class="w-4 h-4 mx-2 text-guofeng-text-muted" />
    </template>
  </nav>
</template>

<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { ChevronRight } from 'lucide-vue-next'
import type { Component } from 'vue'

interface BreadcrumbItem {
  label: string
  to?: string
  icon?: Component
}

defineProps<{
  items: BreadcrumbItem[]
}>()
</script>
