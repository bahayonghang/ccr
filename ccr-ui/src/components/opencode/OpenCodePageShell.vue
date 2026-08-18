<template>
  <PageShell>
    <template #header>
      <div
        class="opencode-page-shell__header"
        :data-tone="tone"
      >
        <RouterLink
          :to="backTo"
          class="opencode-page-shell__back"
        >
          <Button
            variant="ghost"
            surface="status"
            density="compact"
            motion="subtle"
          >
            <template #leading>
              <SIcon
                name="ChevronLeft"
                size="w-4 h-4"
              />
            </template>
            {{ backLabel }}
          </Button>
        </RouterLink>

        <PageHeader
          :title="title"
          :description="description"
          eyebrow="OpenCode operator surface"
          eyebrow-lang="en"
        >
          <template #leading>
            <div class="opencode-page-shell__icon">
              <SIcon
                :name="icon"
                size="w-5 h-5"
              />
            </div>
          </template>

          <template
            v-if="$slots.actions"
            #actions
          >
            <slot name="actions" />
          </template>

          <template
            v-if="badge || $slots.meta"
            #status
          >
            <Badge
              v-if="badge"
              size="sm"
              shape="square"
            >
              {{ badge }}
            </Badge>
            <slot name="meta" />
          </template>
        </PageHeader>
      </div>
    </template>

    <slot />
  </PageShell>
</template>

<script setup lang="ts">
import { RouterLink } from 'vue-router'
import Badge from '@/components/ui/Badge.vue'
import Button from '@/components/ui/Button.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import SIcon from '@/components/ui/SIcon.vue'

withDefaults(defineProps<{
  title: string
  description: string
  icon?: string
  /** 保留调用方 tone 契约；不再用字面色粉刷图标。 */
  tone?: 'lime' | 'violet' | 'cyan' | 'amber' | 'emerald'
  backTo?: string
  backLabel?: string
  badge?: string
}>(), {
  icon: 'TerminalSquare',
  tone: 'lime',
  backTo: '/opencode',
  backLabel: 'OpenCode',
  badge: '',
})
</script>

<style scoped>
.opencode-page-shell__header {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.opencode-page-shell__back {
  display: inline-flex;
  width: fit-content;
}

.opencode-page-shell__icon {
  display: flex;
  width: 2.75rem;
  height: 2.75rem;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-2xl);
  background: var(--color-bg-surface);
  color: var(--color-text-secondary);
}
</style>
