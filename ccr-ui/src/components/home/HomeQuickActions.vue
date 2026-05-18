<template>
  <section
    class="home-actions"
    data-home-actions
  >
    <header class="home-actions__header">
      <p class="home-actions__eyebrow">
        {{ t('home.actionsEyebrow') }}
      </p>
      <h2 class="home-actions__title">
        {{ t('home.actionsTitle') }}
      </h2>
    </header>

    <div class="home-actions__grid">
      <RouterLink
        v-for="(action, index) in actions"
        :key="action.path"
        :to="action.path"
        class="home-action"
        :class="`home-action--${action.tone}`"
      >
        <span class="home-action__index">{{ formatIndex(index) }}</span>
        <span class="home-action__icon">
          <SIcon
            :name="action.icon"
            size="w-4 h-4"
          />
        </span>
        <span class="home-action__copy">
          <strong class="home-action__title-text">{{ action.title }}</strong>
          <span class="home-action__desc">{{ action.desc }}</span>
        </span>
        <span
          class="home-action__cta"
          aria-hidden="true"
        >
          <SIcon
            name="ArrowRight"
            size="w-4 h-4"
          />
        </span>
      </RouterLink>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { HomeQuickAction } from './types'

defineProps<{
  actions: HomeQuickAction[]
}>()

const { t } = useI18n()

const formatIndex = (index: number) => String(index + 1).padStart(2, '0')
</script>

<style scoped>
.home-actions {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
  height: 100%;
  padding: var(--home-card-pad);
  border: 1px solid var(--home-border-card);
  border-radius: var(--home-card-radius);
  background: var(--home-surface-card);
  box-shadow: var(--home-elevation-raised);
}

.home-actions__header {
  display: grid;
  gap: 0.2rem;
}

.home-actions__eyebrow {
  margin: 0;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.home-actions__title {
  margin: 0;
  color: var(--color-text-primary);
  font-size: var(--home-text-section);
  font-weight: 600;
  letter-spacing: var(--home-tracking-display);
}

.home-actions__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.6rem;
  flex: 1;
  align-content: stretch;
}

.home-action {
  position: relative;
  display: grid;
  grid-template:
    'index icon copy cta' auto
    'index .    copy cta' auto
    / auto auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.55rem 0.7rem;
  padding: 0.8rem 0.85rem;
  border: 1px solid var(--home-border-card);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 72%);
  color: var(--color-text-primary);
  text-decoration: none;
  transition:
    border-color var(--home-motion-duration) var(--home-motion-ease),
    background-color var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.home-action:hover {
  border-color: var(--home-border-card-hover);
  background: var(--home-surface-card-hover);
  transform: translateY(var(--home-motion-lift));
}

.home-action:focus-visible {
  outline: 0;
  border-color: var(--home-border-card-hover);
  box-shadow: var(--home-focus-ring);
}

.home-action:active {
  transform: translateY(0);
  box-shadow: var(--home-elevation-sunk);
}

.home-action__index {
  grid-area: index;
  align-self: start;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: 1.35rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1;
  opacity: 0.62;
}

.home-action--command .home-action__index {
  color: var(--color-accent-primary);
  opacity: 0.86;
}

.home-action--config .home-action__index {
  color: var(--color-accent-secondary);
  opacity: 0.86;
}

.home-action--sync .home-action__index {
  color: var(--color-info);
  opacity: 0.86;
}

.home-action--usage .home-action__index {
  color: var(--color-success);
  opacity: 0.86;
}

.home-action__icon {
  grid-area: icon;
  display: grid;
  place-items: center;
  width: 1.6rem;
  height: 1.6rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  border-radius: 7px;
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--color-text-secondary);
}

.home-action__copy {
  grid-area: copy;
  display: grid;
  gap: 0.18rem;
  min-width: 0;
}

.home-action__title-text {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: var(--home-text-body);
  font-weight: 600;
  letter-spacing: var(--home-tracking-body);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.home-action__desc {
  display: -webkit-box;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  line-height: 1.5;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.home-action__cta {
  grid-area: cta;
  display: grid;
  place-items: center;
  color: var(--color-text-muted);
  transition: transform var(--home-motion-duration) var(--home-motion-ease),
    color var(--home-motion-duration) var(--home-motion-ease);
}

.home-action:hover .home-action__cta,
.home-action:focus-visible .home-action__cta {
  color: var(--color-accent-primary);
  transform: translateX(3px);
}

@media (width <= 720px) {
  .home-actions__grid {
    grid-template-columns: 1fr;
  }
}

@media (prefers-reduced-motion: reduce) {
  .home-action,
  .home-action__cta {
    transition: none;
  }
}
</style>
