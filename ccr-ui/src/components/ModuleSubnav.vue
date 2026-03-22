<template>
  <nav
    v-if="items.length > 0"
    class="module-subnav"
    aria-label="Module navigation"
  >
    <RouterLink
      v-for="item in items"
      :key="item.href"
      :to="item.href"
      class="module-subnav__item"
      :class="{ 'module-subnav__item--active': isActive(item.href) }"
      :aria-current="isActive(item.href) ? 'page' : undefined"
    >
      <SIcon
        :name="item.icon"
        size="w-4 h-4"
      />
      <span>{{ item.label }}</span>
    </RouterLink>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import SIcon from '@/components/ui/SIcon.vue'
import { getModuleSubnavItems } from '@/config/moduleSubnav'

interface Props {
  module: string
}

const props = defineProps<Props>()
const route = useRoute()

const items = computed(() => getModuleSubnavItems(props.module))
const activeHref = computed(() => {
  return items.value
    .map(item => item.href)
    .filter(href => route.path === href || route.path.startsWith(`${href}/`))
    .sort((left, right) => right.length - left.length)[0]
})

const isActive = (href: string) => activeHref.value === href
</script>

<style scoped>
.module-subnav {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  padding: 0.75rem;
  border-radius: 1rem;
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 55%);
  backdrop-filter: blur(16px);
}

.module-subnav__item {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 2.5rem;
  padding: 0.5rem 0.875rem;
  border-radius: 0.75rem;
  color: var(--color-text-secondary);
  border: 1px solid transparent;
  transition: color 150ms ease, background-color 150ms ease, border-color 150ms ease;
}

.module-subnav__item:hover {
  color: var(--color-text-primary);
  background: rgb(var(--color-bg-surface-rgb) / 70%);
  border-color: rgb(var(--color-border-default-rgb) / 70%);
}

.module-subnav__item--active {
  color: var(--color-accent-primary);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border-color: rgb(var(--color-accent-primary-rgb) / 25%);
  box-shadow: 0 8px 24px rgb(var(--color-accent-primary-rgb) / 10%);
}
</style>
