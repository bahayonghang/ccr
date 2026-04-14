<template>
  <section class="page-header-card">
    <div
      class="page-header-card__glow"
      :class="toneClasses.glow"
    />

    <div class="page-header-card__content">
      <div class="page-header-card__top">
        <div class="page-header-card__intro">
          <div
            class="page-header-card__icon"
            :class="toneClasses.iconBox"
          >
            <SIcon
              :name="icon"
              size="w-6 h-6"
              :class="toneClasses.icon"
            />
          </div>

          <div class="min-w-0 flex-1">
            <div class="page-header-card__title-row">
              <h1 class="page-header-card__title">
                {{ title }}
              </h1>
              <span
                v-if="badge"
                class="page-header-card__badge"
                :class="toneClasses.badge"
              >
                {{ badge }}
              </span>
            </div>

            <p
              v-if="description"
              class="page-header-card__description"
            >
              {{ description }}
            </p>

            <div
              v-if="$slots.meta"
              class="page-header-card__meta"
            >
              <slot name="meta" />
            </div>
          </div>
        </div>

        <div
          v-if="$slots.actions"
          class="page-header-card__actions"
        >
          <slot name="actions" />
        </div>
      </div>

      <div
        v-if="$slots.default"
        class="page-header-card__body"
      >
        <slot />
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'

type HeaderTone = 'primary' | 'secondary' | 'success' | 'danger' | 'info'

interface Props {
  title: string
  icon: string
  description?: string
  badge?: string
  tone?: HeaderTone
}

const props = withDefaults(defineProps<Props>(), {
  description: undefined,
  badge: undefined,
  tone: 'primary',
})

const toneClasses = computed(() => {
  const map: Record<HeaderTone, { glow: string; iconBox: string; icon: string; badge: string }> = {
    primary: {
      glow: 'page-header-card__glow--primary',
      iconBox: 'page-header-card__icon--primary',
      icon: 'text-accent-primary',
      badge: 'page-header-card__badge--primary',
    },
    secondary: {
      glow: 'page-header-card__glow--secondary',
      iconBox: 'page-header-card__icon--secondary',
      icon: 'text-accent-primary',
      badge: 'page-header-card__badge--secondary',
    },
    success: {
      glow: 'page-header-card__glow--success',
      iconBox: 'page-header-card__icon--success',
      icon: 'text-success',
      badge: 'page-header-card__badge--success',
    },
    danger: {
      glow: 'page-header-card__glow--danger',
      iconBox: 'page-header-card__icon--danger',
      icon: 'text-danger',
      badge: 'page-header-card__badge--danger',
    },
    info: {
      glow: 'page-header-card__glow--info',
      iconBox: 'page-header-card__icon--info',
      icon: 'text-info',
      badge: 'page-header-card__badge--info',
    },
  }

  return map[props.tone]
})
</script>

<style scoped>
.page-header-card {
  position: relative;
  overflow: hidden;
  border-radius: var(--radius-2xl);
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  box-shadow: var(--surface-card-shadow), var(--glass-inner-glow);
  backdrop-filter: var(--surface-card-blur);
}

.page-header-card__glow {
  position: absolute;
  inset: auto auto 0 0;
  width: 16rem;
  height: 16rem;
  border-radius: 9999px;
  filter: blur(92px);
  opacity: 0.32;
  pointer-events: none;
  transform: translate(-28%, 38%);
}

.page-header-card__glow--primary {
  background: rgb(var(--color-accent-primary-rgb) / 16%);
}

.page-header-card__glow--secondary {
  background: rgb(var(--color-premium-blue-rgb) / 56%);
}

.page-header-card__glow--success {
  background: rgb(var(--color-success-rgb) / 14%);
}

.page-header-card__glow--danger {
  background: rgb(var(--color-danger-rgb) / 14%);
}

.page-header-card__glow--info {
  background: rgb(var(--color-info-rgb) / 14%);
}

.page-header-card__content {
  position: relative;
  z-index: 1;
  padding: 1.45rem 1.6rem;
}

.page-header-card__top {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.page-header-card__intro {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  min-width: 0;
}

.page-header-card__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 3.25rem;
  height: 3.25rem;
  border-radius: 1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 10%);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 64%);
}

.page-header-card__icon--primary {
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 100%), rgb(var(--color-bg-surface-rgb) / 82%));
}

.page-header-card__icon--secondary {
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 100%), rgb(var(--color-bg-surface-rgb) / 82%));
}

.page-header-card__icon--success {
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 100%), rgb(var(--color-bg-surface-rgb) / 82%));
}

.page-header-card__icon--danger {
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 100%), rgb(var(--color-bg-surface-rgb) / 82%));
}

.page-header-card__icon--info {
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 100%), rgb(var(--color-bg-surface-rgb) / 82%));
}

.page-header-card__title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
}

.page-header-card__title {
  font-size: 1.75rem;
  line-height: 1.08;
  font-weight: 600;
  letter-spacing: -0.03em;
  color: var(--color-text-primary);
}

.page-header-card__description {
  margin-top: 0.5rem;
  max-width: 52rem;
  color: var(--color-text-secondary);
  line-height: 1.47;
}

.page-header-card__badge {
  display: inline-flex;
  align-items: center;
  min-height: 1.9rem;
  padding: 0.25rem 0.82rem;
  border-radius: 9999px;
  border: 1px solid transparent;
  font-size: 0.75rem;
  font-weight: 600;
}

.page-header-card__badge--primary {
  color: var(--color-accent-primary);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border-color: rgb(var(--color-accent-primary-rgb) / 16%);
}

.page-header-card__badge--secondary {
  color: var(--color-accent-primary);
  background: rgb(var(--color-bg-overlay-rgb) / 72%);
  border-color: rgb(var(--color-border-default-rgb) / 14%);
}

.page-header-card__badge--success {
  color: var(--color-success);
  background: rgb(var(--color-success-rgb) / 10%);
  border-color: rgb(var(--color-success-rgb) / 16%);
}

.page-header-card__badge--danger {
  color: var(--color-danger);
  background: rgb(var(--color-danger-rgb) / 10%);
  border-color: rgb(var(--color-danger-rgb) / 16%);
}

.page-header-card__badge--info {
  color: var(--color-info);
  background: rgb(var(--color-info-rgb) / 10%);
  border-color: rgb(var(--color-info-rgb) / 16%);
}

.page-header-card__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.625rem;
  margin-top: 0.875rem;
}

.page-header-card__actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
}

.page-header-card__body {
  margin-top: 1.25rem;
}

@media (width >= 768px) {
  .page-header-card__content {
    padding: 1.75rem;
  }
}

@media (width >= 1024px) {
  .page-header-card__top {
    flex-direction: row;
    align-items: flex-start;
    justify-content: space-between;
  }
}
</style>
