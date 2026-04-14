<template>
  <div class="opencode-page-shell">
    <AnimatedBackground
      contained
      variant="minimal"
    />

    <div class="opencode-page-shell__inner">
      <Card
        variant="glass"
        class="opencode-page-shell__hero"
      >
        <div class="opencode-page-shell__glow" />

        <div class="opencode-page-shell__hero-content">
          <div class="opencode-page-shell__header">
            <div class="opencode-page-shell__header-main">
              <RouterLink
                :to="backTo"
                class="inline-flex"
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

              <div class="opencode-page-shell__title-row">
                <div
                  class="opencode-page-shell__icon"
                  :class="toneClass"
                >
                  <SIcon
                    :name="icon"
                    size="w-5 h-5"
                  />
                </div>
                <div>
                  <div class="opencode-page-shell__eyebrow">
                    OpenCode operator surface
                  </div>
                  <h1 class="opencode-page-shell__title">
                    {{ title }}
                  </h1>
                  <p class="opencode-page-shell__description">
                    {{ description }}
                  </p>
                </div>
              </div>
            </div>

            <div
              v-if="$slots.actions"
              class="opencode-page-shell__actions"
            >
              <slot name="actions" />
            </div>
          </div>

          <div
            v-if="badge || $slots.meta"
            class="opencode-page-shell__meta"
          >
            <span
              v-if="badge"
              class="opencode-page-shell__badge"
              :class="toneClass"
            >
              {{ badge }}
            </span>
            <slot name="meta" />
          </div>
        </div>
      </Card>

      <div class="opencode-page-shell__body">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'

const props = withDefaults(defineProps<{
  title: string
  description: string
  icon?: string
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

const toneClass = computed(() => ({
  lime: 'opencode-page-shell__tone--lime',
  violet: 'opencode-page-shell__tone--violet',
  cyan: 'opencode-page-shell__tone--cyan',
  amber: 'opencode-page-shell__tone--amber',
  emerald: 'opencode-page-shell__tone--emerald',
}[props.tone]))
</script>

<style scoped>
.opencode-page-shell {
  @apply relative min-h-full overflow-hidden px-4 py-4 sm:px-6 sm:py-6;
}

.opencode-page-shell__inner {
  @apply relative z-10 mx-auto flex max-w-[1480px] flex-col gap-5;
}

.opencode-page-shell__hero {
  @apply relative overflow-hidden p-5 sm:p-6;
}

.opencode-page-shell__glow {
  position: absolute;
  inset: auto -3rem -3rem auto;
  height: 14rem;
  width: 14rem;
  border-radius: 999px;
  background: radial-gradient(circle, rgb(163 230 53 / 18%), transparent 70%);
  pointer-events: none;
}

.opencode-page-shell__hero-content {
  @apply relative z-10 flex flex-col gap-4;
}

.opencode-page-shell__header {
  @apply flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between;
}

.opencode-page-shell__header-main {
  @apply flex flex-col gap-4;
}

.opencode-page-shell__title-row {
  @apply flex items-start gap-4;
}

.opencode-page-shell__icon {
  @apply flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl border border-border-default/15 bg-bg-surface/70 shadow-lg backdrop-blur-md;
}

.opencode-page-shell__tone--lime {
  color: rgb(163 230 53);
  background: rgb(163 230 53 / 12%);
  border-color: rgb(163 230 53 / 25%);
}

.opencode-page-shell__tone--violet {
  color: rgb(167 139 250);
  background: rgb(167 139 250 / 12%);
  border-color: rgb(167 139 250 / 25%);
}

.opencode-page-shell__tone--cyan {
  color: rgb(103 232 249);
  background: rgb(103 232 249 / 12%);
  border-color: rgb(103 232 249 / 25%);
}

.opencode-page-shell__tone--amber {
  color: rgb(251 191 36);
  background: rgb(251 191 36 / 12%);
  border-color: rgb(251 191 36 / 25%);
}

.opencode-page-shell__tone--emerald {
  color: rgb(52 211 153);
  background: rgb(52 211 153 / 12%);
  border-color: rgb(52 211 153 / 25%);
}

.opencode-page-shell__eyebrow {
  @apply mb-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-text-muted;
}

.opencode-page-shell__title {
  @apply text-2xl font-semibold tracking-[-0.03em] text-text-primary sm:text-3xl;
}

.opencode-page-shell__description {
  @apply mt-2 max-w-3xl text-sm leading-7 text-text-secondary;
}

.opencode-page-shell__actions {
  @apply flex flex-wrap items-center gap-2;
}

.opencode-page-shell__meta {
  @apply flex flex-wrap items-center gap-2;
}

.opencode-page-shell__badge {
  @apply inline-flex items-center rounded-full border px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em];
}

.opencode-page-shell__body {
  @apply flex flex-col gap-5;
}
</style>

