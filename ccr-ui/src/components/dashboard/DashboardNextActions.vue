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
  gap: 0.85rem;
  height: 100%;
  padding: var(--home-card-pad);
  border: 1px solid var(--home-border-card);
  border-radius: var(--home-card-radius);
  background: var(--home-surface-card);
  box-shadow: var(--home-elevation-raised);
}

.dashboard-actions__header {
  display: grid;
  gap: 0.22rem;
}

.dashboard-actions__eyebrow {
  margin: 0;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.dashboard-actions__title {
  margin: 0;
  color: var(--color-text-primary);
  font-family: var(--font-brand);
  font-size: var(--home-text-section);
  font-weight: 640;
  letter-spacing: var(--home-tracking-display);
}

.dashboard-actions__description {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
  line-height: 1.5;
}

.dashboard-actions__queue {
  display: grid;
  gap: 0.55rem;
  flex: 1;
}

.dashboard-action {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.55rem;
  min-width: 0;
  padding: 0.74rem 0.78rem;
  border: 1px solid var(--home-border-card);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 68%);
  color: var(--color-text-primary);
  text-decoration: none;
  transition:
    border-color var(--home-motion-duration) var(--home-motion-ease),
    background-color var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-action:hover {
  border-color: var(--home-border-card-hover);
  background: var(--home-surface-card-hover);
  transform: translateY(var(--home-motion-lift));
}

.dashboard-action:focus-visible {
  outline: 0;
  box-shadow: var(--home-focus-ring);
}

.dashboard-action--primary {
  border-color: rgb(var(--color-accent-primary-rgb) / 50%);
  background: var(--color-accent-primary);
  color: var(--color-text-inverted);
}

.dashboard-action--primary:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 65%);
  background: var(--color-accent-primary-hover);
}

.dashboard-action--primary .dashboard-action__icon {
  border-color: rgb(var(--color-accent-primary-contrast-rgb) / 24%);
  background: rgb(var(--color-accent-primary-contrast-rgb) / 16%);
  color: var(--color-text-inverted);
}

.dashboard-action--primary .dashboard-action__copy strong {
  color: var(--color-text-inverted);
}

.dashboard-action--primary .dashboard-action__copy span,
.dashboard-action--primary .dashboard-action__copy em {
  color: rgb(var(--color-accent-primary-contrast-rgb) / 78%);
}

.dashboard-action--primary .dashboard-action__cta {
  color: rgb(var(--color-accent-primary-contrast-rgb) / 78%);
}

.dashboard-action--primary:hover .dashboard-action__cta,
.dashboard-action--primary:focus-visible .dashboard-action__cta {
  color: var(--color-text-inverted);
}

.dashboard-action__icon {
  display: grid;
  place-items: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 16%);
  border-radius: 8px;
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--color-text-secondary);
}

.dashboard-action__copy {
  display: grid;
  gap: 0.16rem;
  min-width: 0;
}

.dashboard-action__copy strong {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: var(--home-text-body);
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-action__copy span,
.dashboard-action__copy em {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-style: normal;
  line-height: 1.45;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-action__copy em {
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-weight: 700;
}

.dashboard-action__cta {
  color: var(--color-text-muted);
  transition:
    color var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-action:hover .dashboard-action__cta,
.dashboard-action:focus-visible .dashboard-action__cta {
  color: var(--color-accent-primary);
  transform: translateX(3px);
}

.dashboard-action--monitoring .dashboard-action__icon {
  color: var(--color-warning);
}

.dashboard-action--usage .dashboard-action__icon {
  color: var(--color-success);
}

.dashboard-action--platform .dashboard-action__icon {
  color: var(--color-accent-secondary);
}

.dashboard-action--system .dashboard-action__icon {
  color: var(--color-accent-primary);
}

.dashboard-action--primary.dashboard-action--monitoring .dashboard-action__icon,
.dashboard-action--primary.dashboard-action--usage .dashboard-action__icon,
.dashboard-action--primary.dashboard-action--platform .dashboard-action__icon,
.dashboard-action--primary.dashboard-action--system .dashboard-action__icon {
  color: var(--color-text-inverted);
}

.dashboard-actions__onboarding {
  display: grid;
  gap: 0.55rem;
  flex: 1;
  margin: 0;
  padding: 0;
  list-style: none;
}

.dashboard-onboarding-step {
  display: grid;
  grid-template-columns: auto auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.55rem;
  min-width: 0;
  padding: 0.74rem 0.78rem;
  border: 1px solid var(--home-border-card);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 68%);
  color: var(--color-text-primary);
  text-decoration: none;
  transition:
    border-color var(--home-motion-duration) var(--home-motion-ease),
    background-color var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-onboarding-step:hover {
  border-color: var(--home-border-card-hover);
  background: var(--home-surface-card-hover);
  transform: translateY(var(--home-motion-lift));
}

.dashboard-onboarding-step:focus-visible {
  outline: 0;
  box-shadow: var(--home-focus-ring);
}

.dashboard-onboarding-step__index {
  display: grid;
  place-items: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 999px;
  background: rgb(var(--color-accent-primary-rgb) / 14%);
  color: var(--color-accent-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-meta);
  font-weight: 800;
}

.dashboard-onboarding-step__icon {
  display: grid;
  place-items: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 16%);
  border-radius: 8px;
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--color-text-secondary);
}

.dashboard-onboarding-step__copy {
  display: grid;
  gap: 0.16rem;
  min-width: 0;
}

.dashboard-onboarding-step__copy strong {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: var(--home-text-body);
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-onboarding-step__copy span {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  line-height: 1.45;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (prefers-reduced-motion: reduce) {
  .dashboard-action,
  .dashboard-action__cta,
  .dashboard-onboarding-step {
    transition: none;
  }
}
</style>
