<template>
  <section
    class="dashboard-actions"
    data-dashboard-actions
  >
    <header class="dashboard-actions__header">
      <p class="dashboard-actions__eyebrow">
        {{ t('dashboard.actions.eyebrow') }}
      </p>
      <h2 class="dashboard-actions__title">
        {{ t('dashboard.actions.title') }}
      </h2>
      <p class="dashboard-actions__description">
        {{ showOnboarding ? t('dashboard.actions.onboardingDescription') : t('dashboard.actions.description') }}
      </p>
    </header>

    <ol
      v-if="showOnboarding"
      class="dashboard-actions__onboarding"
    >
      <li
        v-for="(step, index) in onboardingSteps"
        :key="step.id"
      >
        <RouterLink
          :to="step.path"
          class="dashboard-onboarding-step"
          :class="{ 'dashboard-onboarding-step--primary': index === 0 }"
        >
          <span class="dashboard-onboarding-step__index">{{ index + 1 }}</span>
          <span class="dashboard-onboarding-step__icon">
            <SIcon
              :name="step.icon"
              size="w-4 h-4"
            />
          </span>
          <span class="dashboard-onboarding-step__copy">
            <strong>{{ t(step.titleKey) }}</strong>
            <span>{{ t(step.descKey) }}</span>
          </span>
          <span
            class="dashboard-action__cta"
            aria-hidden="true"
          >
            <SIcon
              name="ArrowRight"
              size="w-4 h-4"
            />
          </span>
        </RouterLink>
      </li>
    </ol>

    <div
      v-else
      class="dashboard-actions__queue"
    >
      <RouterLink
        v-for="(action, index) in actions"
        :key="action.id"
        :to="action.path"
        :class="[
          'dashboard-action',
          `dashboard-action--${action.tone}`,
          { 'dashboard-action--primary': index === 0 },
        ]"
      >
        <span class="dashboard-action__icon">
          <SIcon
            :name="action.icon"
            size="w-4 h-4"
          />
        </span>
        <span class="dashboard-action__copy">
          <strong>{{ t(action.titleKey) }}</strong>
          <span>{{ t(action.descKey) }}</span>
          <em v-if="action.detail">{{ action.detail }}</em>
        </span>
        <span
          class="dashboard-action__cta"
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
import type { IconName } from '@/config/icons'
import type { DashboardAction } from '@/views/dashboard/dashboardPresentation'

withDefaults(defineProps<{
  actions: DashboardAction[]
  showOnboarding?: boolean
}>(), {
  showOnboarding: false,
})

const { t } = useI18n()

const onboardingSteps: Array<{ id: string; path: string; icon: IconName; titleKey: string; descKey: string }> = [
  {
    id: 'create-profile',
    path: '/claude-code',
    icon: 'UserCheck',
    titleKey: 'dashboard.actions.onboardingStep1Title',
    descKey: 'dashboard.actions.onboardingStep1Desc',
  },
  {
    id: 'configure-mcp',
    path: '/mcp-manager',
    icon: 'Plug',
    titleKey: 'dashboard.actions.onboardingStep2Title',
    descKey: 'dashboard.actions.onboardingStep2Desc',
  },
  {
    id: 'import-usage',
    path: '/usage',
    icon: 'Download',
    titleKey: 'dashboard.actions.onboardingStep3Title',
    descKey: 'dashboard.actions.onboardingStep3Desc',
  },
]
</script>

<style scoped>
.dashboard-actions {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  height: 100%;
  padding: var(--home-card-pad);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--home-card-radius);
  background: var(--color-bg-surface);
}

.dashboard-actions__header {
  display: grid;
  gap: 0.2rem;
}

.dashboard-actions__eyebrow {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 0.8125rem;
  font-weight: 500;
  line-height: 1.24;
  letter-spacing: 0;
}

.dashboard-actions__title {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1.0625rem;
  font-weight: 600;
  line-height: 1.3;
  letter-spacing: 0;
}

.dashboard-actions__description {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 0.8125rem;
  line-height: 1.5;
}

.dashboard-actions__queue,
.dashboard-actions__onboarding {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.35rem;
  flex: 1;
  align-content: start;
  margin: 0;
  padding: 0;
  list-style: none;
}

.dashboard-action,
.dashboard-onboarding-step {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
  padding: 0.5rem 0.65rem;
  border: 1px solid transparent;
  border-radius: 8px;
  background: transparent;
  color: var(--color-text-primary);
  text-decoration: none;
  transition:
    border-color var(--home-motion-duration) var(--home-motion-ease),
    background-color var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-onboarding-step {
  grid-template-columns: auto auto minmax(0, 1fr) auto;
}

.dashboard-action:hover,
.dashboard-onboarding-step:hover {
  background: rgb(var(--color-bg-elevated-rgb) / 88%);
  border-color: var(--color-border-subtle);
}

.dashboard-action:focus-visible,
.dashboard-onboarding-step:focus-visible {
  outline: 2px solid var(--color-accent-primary);
  outline-offset: 2px;
}

.dashboard-action--primary,
.dashboard-onboarding-step--primary {
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
}

.dashboard-action--primary:hover,
.dashboard-onboarding-step--primary:hover {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
}

.dashboard-action__icon,
.dashboard-onboarding-step__icon {
  display: grid;
  place-items: center;
  width: 1.5rem;
  height: 1.5rem;
  color: var(--color-text-secondary);
}

.dashboard-action--monitoring .dashboard-action__icon {
  color: var(--color-warning);
}

.dashboard-action__copy,
.dashboard-onboarding-step__copy {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
}

.dashboard-action__copy strong,
.dashboard-onboarding-step__copy strong {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-action__copy span,
.dashboard-action__copy em,
.dashboard-onboarding-step__copy span {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-style: normal;
  line-height: 1.4;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-action__copy em {
  color: var(--color-text-secondary);
  font-variant-numeric: tabular-nums;
  font-weight: 500;
}

.dashboard-action__cta {
  color: var(--color-text-muted);
}

.dashboard-action:hover .dashboard-action__cta,
.dashboard-action:focus-visible .dashboard-action__cta,
.dashboard-onboarding-step:hover .dashboard-action__cta,
.dashboard-onboarding-step:focus-visible .dashboard-action__cta {
  color: var(--color-text-primary);
}

.dashboard-onboarding-step__index {
  display: grid;
  place-items: center;
  width: 1.25rem;
  height: 1.25rem;
  border-radius: 999px;
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

@media (width <= 720px) {
  .dashboard-actions__queue,
  .dashboard-actions__onboarding {
    grid-template-columns: 1fr;
  }
}

@media (prefers-reduced-motion: reduce) {
  .dashboard-action,
  .dashboard-onboarding-step {
    transition: none;
  }
}
</style>
