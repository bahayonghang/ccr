<template>
  <div class="grok-view stage-page">
    <div class="grok-shell">
      <ModuleSubnav module="grok" />

      <section
        v-if="localOnly"
        class="grok-local-only"
        role="status"
      >
        <div class="grok-local-only__icon">
          <SIcon
            name="Monitor"
            size="w-6 h-6"
          />
        </div>
        <div>
          <p class="grok-kicker">
            {{ t('grok.dashboard.header.eyebrow') }}
          </p>
          <h1>{{ t('grok.dashboard.localOnly.title') }}</h1>
          <p>{{ t('grok.dashboard.localOnly.description') }}</p>
          <span>
            {{ t('grok.dashboard.localOnly.environment', {
              env: localOnlyEnvType || t('grok.states.unknown'),
            }) }}
          </span>
        </div>
      </section>

      <template v-else>
        <header class="grok-hero">
          <div class="grok-hero__main">
            <div class="grok-identity">
              <div class="grok-mark">
                <SIcon
                  name="Zap"
                  size="w-6 h-6"
                />
              </div>
              <div class="grok-identity__copy">
                <p class="grok-kicker">
                  {{ t('grok.dashboard.header.eyebrow') }}
                </p>
                <h1>{{ t('grok.overview.title') }}</h1>
                <p>{{ t('grok.overview.subtitle') }}</p>
              </div>
            </div>

            <div class="grok-hero__actions">
              <Button
                variant="ghost"
                size="sm"
                :disabled="loading"
                @click="refresh(true)"
              >
                <SIcon
                  name="RefreshCw"
                  size="w-4 h-4"
                  class="mr-2"
                  :class="{ 'animate-spin': loading }"
                />
                {{ t('grok.dashboard.header.refresh') }}
              </Button>

              <a
                v-if="overview && primaryAction.external"
                :href="primaryAction.to"
                target="_blank"
                rel="noreferrer"
              >
                <Button :variant="primaryButtonVariant">
                  <SIcon
                    :name="primaryAction.icon"
                    size="w-4 h-4"
                    class="mr-2"
                  />
                  {{ primaryAction.title }}
                </Button>
              </a>
              <RouterLink
                v-else-if="overview"
                :to="primaryAction.to"
              >
                <Button :variant="primaryButtonVariant">
                  <SIcon
                    :name="primaryAction.icon"
                    size="w-4 h-4"
                    class="mr-2"
                  />
                  {{ primaryAction.title }}
                </Button>
              </RouterLink>
            </div>
          </div>

          <div class="grok-meta">
            <div
              class="grok-chip"
              :class="`grok-chip--${versionTone}`"
            >
              <span>{{ t('grok.dashboard.header.version') }}</span>
              <strong>{{ versionLabel }}</strong>
            </div>
            <div class="grok-chip grok-chip--neutral">
              <span>{{ t('grok.dashboard.header.profile') }}</span>
              <strong :title="currentProfileLabel">{{ currentProfileLabel }}</strong>
            </div>
            <div class="grok-chip grok-chip--neutral">
              <span>{{ t('grok.dashboard.header.auth') }}</span>
              <strong>{{ authModeLabel }}</strong>
            </div>
            <div
              v-if="activationWarning"
              class="grok-chip"
              :class="`grok-chip--${activationWarning.tone}`"
            >
              <SIcon
                :name="activationWarning.icon"
                size="w-4 h-4"
              />
              <strong :title="activationWarning.label">{{ activationWarning.label }}</strong>
            </div>
          </div>
        </header>

        <div
          v-if="initialLoading"
          class="grok-loading"
          aria-hidden="true"
        >
          <div class="grok-skeleton grok-skeleton--wide" />
          <div class="grok-loading__grid">
            <div
              v-for="index in 3"
              :key="index"
              class="grok-skeleton"
            />
          </div>
        </div>

        <EmptyState
          v-else-if="loadError && !overview"
          icon="AlertCircle"
          :title="t('grok.dashboard.empty.title')"
          :description="loadError"
          :action-text="t('grok.dashboard.header.refresh')"
          action-icon="RefreshCw"
          :on-action="() => refresh(true)"
        />

        <template v-else-if="overview">
          <section class="grok-section">
            <div class="grok-section__heading">
              <div>
                <p class="grok-kicker">
                  {{ t('grok.dashboard.readiness.eyebrow') }}
                </p>
                <h2>{{ t('grok.dashboard.readiness.title') }}</h2>
              </div>
              <p>{{ t('grok.dashboard.readiness.subtitle') }}</p>
            </div>

            <div class="grok-readiness-grid">
              <Card
                v-for="item in readinessItems"
                :key="item.key"
                variant="elevated"
                padding="none"
                class="grok-readiness-card"
                :class="`grok-readiness-card--${item.tone}`"
              >
                <div class="grok-readiness-card__top">
                  <div
                    class="grok-tone-icon"
                    :class="toneClassMap[item.tone]"
                  >
                    <SIcon
                      :name="item.icon"
                      size="w-4 h-4"
                    />
                  </div>
                  <span>{{ item.statusLabel }}</span>
                </div>
                <p class="grok-readiness-card__label">
                  {{ item.title }}
                </p>
                <strong class="grok-readiness-card__value">{{ item.value }}</strong>
                <p class="grok-readiness-card__detail">
                  {{ item.detail }}
                </p>
              </Card>
            </div>
          </section>

          <section class="grok-workspace">
            <div class="grok-actions-panel">
              <div class="grok-section__heading grok-section__heading--stacked">
                <div>
                  <p class="grok-kicker">
                    {{ t('grok.dashboard.actions.eyebrow') }}
                  </p>
                  <h2>{{ t('grok.dashboard.actions.title') }}</h2>
                </div>
                <p>{{ t('grok.dashboard.actions.subtitle') }}</p>
              </div>

              <div class="grok-action-list">
                <template
                  v-for="(action, index) in nextActions"
                  :key="action.key"
                >
                  <a
                    v-if="action.external"
                    :href="action.to"
                    target="_blank"
                    rel="noreferrer"
                    class="grok-action-row"
                  >
                    <span class="grok-action-row__index">{{ index + 1 }}</span>
                    <div
                      class="grok-tone-icon"
                      :class="toneClassMap[action.tone]"
                    >
                      <SIcon
                        :name="action.icon"
                        size="w-4 h-4"
                      />
                    </div>
                    <div class="grok-action-row__copy">
                      <strong>{{ action.title }}</strong>
                      <p>{{ action.description }}</p>
                    </div>
                    <SIcon
                      name="ExternalLink"
                      size="w-4 h-4"
                      class="grok-action-row__arrow"
                    />
                  </a>
                  <RouterLink
                    v-else
                    :to="action.to"
                    class="grok-action-row"
                  >
                    <span class="grok-action-row__index">{{ index + 1 }}</span>
                    <div
                      class="grok-tone-icon"
                      :class="toneClassMap[action.tone]"
                    >
                      <SIcon
                        :name="action.icon"
                        size="w-4 h-4"
                      />
                    </div>
                    <div class="grok-action-row__copy">
                      <strong>{{ action.title }}</strong>
                      <p>{{ action.description }}</p>
                    </div>
                    <SIcon
                      name="ArrowRight"
                      size="w-4 h-4"
                      class="grok-action-row__arrow"
                    />
                  </RouterLink>
                </template>
              </div>
            </div>

            <div class="grok-management-panel">
              <div class="grok-section__heading grok-section__heading--stacked">
                <div>
                  <p class="grok-kicker">
                    {{ t('grok.dashboard.management.eyebrow') }}
                  </p>
                  <h2>{{ t('grok.dashboard.management.title') }}</h2>
                </div>
                <p>{{ t('grok.dashboard.management.subtitle') }}</p>
              </div>

              <div class="grok-management-list">
                <RouterLink
                  v-for="item in managementItems"
                  :key="item.key"
                  :to="item.to"
                  class="grok-management-row"
                >
                  <div
                    class="grok-tone-icon"
                    :class="toneClassMap[item.tone]"
                  >
                    <SIcon
                      :name="item.icon"
                      size="w-4 h-4"
                    />
                  </div>
                  <div class="grok-management-row__copy">
                    <strong>{{ item.title }}</strong>
                    <p>{{ item.description }}</p>
                  </div>
                  <span>{{ item.badge }}</span>
                  <SIcon
                    name="ArrowRight"
                    size="w-4 h-4"
                    class="grok-management-row__arrow"
                  />
                </RouterLink>
              </div>
            </div>
          </section>

          <section class="grok-section grok-command-section">
            <div class="grok-section__heading">
              <div>
                <p class="grok-kicker">
                  {{ t('grok.dashboard.commands.eyebrow') }}
                </p>
                <h2>{{ t('grok.dashboard.commands.title') }}</h2>
              </div>
            </div>

            <div class="grok-command-list">
              <div
                v-for="command in commands"
                :key="command"
                class="grok-command-row"
              >
                <SIcon
                  name="Terminal"
                  size="w-4 h-4"
                />
                <code>{{ command }}</code>
                <button
                  type="button"
                  :title="t(copiedCommand === command
                    ? 'grok.dashboard.commands.copied'
                    : 'grok.dashboard.commands.copy')"
                  :aria-label="t(copiedCommand === command
                    ? 'grok.dashboard.commands.copied'
                    : 'grok.dashboard.commands.copy')"
                  @click="copyCommand(command)"
                >
                  <SIcon
                    :name="copiedCommand === command ? 'Check' : 'Copy'"
                    size="w-4 h-4"
                  />
                </button>
              </div>
            </div>
          </section>
        </template>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onActivated, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import SIcon from '@/components/ui/SIcon.vue'
import {
  useGrokDashboard,
  type GrokDashboardTone,
} from '@/composables/useGrokDashboard'
import { useUIStore } from '@/stores/ui'
import { copyText } from '@/utils/clipboard'

defineOptions({ name: 'GrokView' })

const { t } = useI18n()
const uiStore = useUIStore()
const copiedCommand = ref<string | null>(null)
let copyResetTimer: number | undefined

const commands = [
  'ccr grok profile list',
  'ccr grok profile switch <name>',
  'ccr grok profile off',
  'ccr grok profile init',
]

const {
  overview,
  loading,
  initialLoading,
  loadError,
  refreshError,
  localOnly,
  localOnlyEnvType,
  versionLabel,
  versionTone,
  currentProfileLabel,
  authModeLabel,
  activationWarning,
  readinessItems,
  nextActions,
  primaryAction,
  managementItems,
  refresh,
} = useGrokDashboard()

const toneClassMap: Record<GrokDashboardTone, string> = {
  success: 'grok-tone-icon--success',
  warning: 'grok-tone-icon--warning',
  danger: 'grok-tone-icon--danger',
  neutral: 'grok-tone-icon--neutral',
}

const primaryButtonVariant = computed(() => {
  if (primaryAction.value.tone === 'danger') return 'danger'
  if (primaryAction.value.tone === 'success') return 'success'
  return 'primary'
})

const copyCommand = async (command: string) => {
  if (!(await copyText(command))) {
    uiStore.showError(t('grok.dashboard.commands.copyFailed'))
    return
  }

  copiedCommand.value = command
  if (copyResetTimer) window.clearTimeout(copyResetTimer)
  copyResetTimer = window.setTimeout(() => {
    if (copiedCommand.value === command) copiedCommand.value = null
  }, 1_600)
}

watch(refreshError, (message) => {
  if (message) {
    uiStore.showError(`${t('grok.dashboard.error.refreshFailed')} ${message}`)
  }
})

onMounted(() => {
  void refresh(false)
})

onActivated(() => {
  void refresh(false)
})

onBeforeUnmount(() => {
  if (copyResetTimer) window.clearTimeout(copyResetTimer)
})
</script>

<style scoped>
.grok-view {
  min-height: 100%;
  padding: 1.5rem;
  overflow: hidden;
}

.grok-shell {
  width: min(100%, 80rem);
  margin: 0 auto;
}

.grok-hero {
  padding: 2rem 0 1.5rem;
  border-bottom: 1px solid var(--stage-border-soft);
}

.grok-hero__main,
.grok-identity,
.grok-hero__actions,
.grok-readiness-card__top,
.grok-action-row,
.grok-management-row,
.grok-command-row {
  display: flex;
  align-items: center;
}

.grok-hero__main {
  justify-content: space-between;
  gap: 1.5rem;
}

.grok-identity {
  min-width: 0;
  align-items: flex-start;
  gap: 1rem;
}

.grok-mark,
.grok-local-only__icon {
  display: flex;
  width: 3.25rem;
  height: 3.25rem;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  color: var(--color-platform-grok);
  background: rgb(var(--color-platform-grok-rgb) / 12%);
  border: 1px solid rgb(var(--color-platform-grok-rgb) / 24%);
  border-radius: var(--radius-lg);
}

.grok-identity__copy {
  min-width: 0;
}

.grok-kicker {
  margin: 0;
  color: var(--color-platform-grok);
  font-size: 0.8125rem;
  font-weight: 600;
  letter-spacing: 0;
}

.grok-identity h1,
.grok-local-only h1 {
  margin: 0.375rem 0 0;
  color: var(--stage-text-primary);
  font-family: var(--font-brand);
  font-size: 2rem;
  font-weight: 600;
  line-height: 1.1;
  letter-spacing: 0;
}

.grok-identity__copy > p:last-child,
.grok-local-only p,
.grok-section__heading > p,
.grok-action-row__copy p,
.grok-management-row__copy p {
  color: var(--stage-text-secondary);
}

.grok-identity__copy > p:last-child {
  max-width: 42rem;
  margin: 0.625rem 0 0;
  font-size: 0.9375rem;
  line-height: 1.6;
}

.grok-hero__actions {
  flex: 0 0 auto;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.5rem;
}

.grok-meta {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.5rem;
  margin-top: 1.5rem;
}

.grok-chip {
  display: flex;
  min-width: 0;
  min-height: 2.75rem;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem 0.75rem;
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);
  border-radius: var(--radius-lg);
}

.grok-chip span {
  flex: 0 0 auto;
  color: var(--stage-text-quiet);
  font-size: 0.75rem;
  font-weight: 600;
}

.grok-chip strong {
  min-width: 0;
  overflow: hidden;
  color: var(--stage-text-primary);
  font-size: 0.8125rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.grok-chip--success {
  color: var(--color-success);
  border-color: rgb(var(--color-success-rgb) / 26%);
}

.grok-chip--warning {
  color: var(--color-warning);
  border-color: rgb(var(--color-warning-rgb) / 30%);
}

.grok-chip--danger {
  color: var(--color-danger);
  border-color: rgb(var(--color-danger-rgb) / 30%);
}

.grok-local-only {
  display: flex;
  min-height: 24rem;
  align-items: center;
  gap: 1.25rem;
  margin-top: 1rem;
  padding: 2rem;
  background: var(--stage-surface-medium);
  border-left: 3px solid var(--color-warning);
}

.grok-local-only p {
  max-width: 42rem;
  margin: 0.75rem 0;
  font-size: 0.9375rem;
  line-height: 1.6;
}

.grok-local-only span {
  color: var(--stage-text-muted);
  font-size: 0.8125rem;
}

.grok-section,
.grok-workspace {
  padding: 1.5rem 0;
  border-bottom: 1px solid var(--stage-border-soft);
}

.grok-section__heading {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1.5rem;
  margin-bottom: 1rem;
}

.grok-section__heading--stacked {
  align-items: flex-start;
  flex-direction: column;
  gap: 0.375rem;
}

.grok-section__heading h2 {
  margin: 0.25rem 0 0;
  color: var(--stage-text-primary);
  font-size: 1.125rem;
  font-weight: 600;
  letter-spacing: 0;
}

.grok-section__heading > p {
  max-width: 36rem;
  margin: 0;
  font-size: 0.875rem;
  line-height: 1.55;
}

.grok-readiness-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.75rem;
}

.grok-readiness-card {
  min-height: 12rem;
  padding: 1rem;
  border-top: 2px solid var(--stage-border-medium);
}

.grok-readiness-card--success {
  border-top-color: var(--color-success);
}

.grok-readiness-card--warning {
  border-top-color: var(--color-warning);
}

.grok-readiness-card--danger {
  border-top-color: var(--color-danger);
}

.grok-readiness-card__top {
  justify-content: space-between;
  gap: 0.75rem;
}

.grok-readiness-card__top > span {
  color: var(--stage-text-muted);
  font-size: 0.75rem;
  font-weight: 600;
}

.grok-readiness-card__label {
  margin: 1.25rem 0 0;
  color: var(--stage-text-muted);
  font-size: 0.8125rem;
  font-weight: 600;
}

.grok-readiness-card__value {
  display: block;
  margin-top: 0.375rem;
  overflow-wrap: anywhere;
  color: var(--stage-text-primary);
  font-size: 1.125rem;
  font-weight: 600;
}

.grok-readiness-card__detail {
  margin: 0.75rem 0 0;
  overflow-wrap: anywhere;
  color: var(--stage-text-secondary);
  font-size: 0.8125rem;
  line-height: 1.55;
}

.grok-workspace {
  display: grid;
  grid-template-columns: minmax(0, 3fr) minmax(18rem, 2fr);
  gap: 2rem;
}

.grok-actions-panel {
  min-width: 0;
}

.grok-management-panel {
  min-width: 0;
  padding-left: 2rem;
  border-left: 1px solid var(--stage-border-soft);
}

.grok-action-list,
.grok-management-list {
  border-top: 1px solid var(--stage-border-soft);
}

.grok-action-row,
.grok-management-row {
  min-width: 0;
  gap: 0.75rem;
  padding: 1rem 0.25rem;
  border-bottom: 1px solid var(--stage-border-soft);
  transition: color 150ms ease, background-color 150ms ease;
}

.grok-action-row:hover,
.grok-management-row:hover {
  background: rgb(var(--color-platform-grok-rgb) / 6%);
}

.grok-action-row__index {
  width: 1.25rem;
  flex: 0 0 auto;
  color: var(--stage-text-quiet);
  font-size: 0.75rem;
  font-weight: 600;
  text-align: center;
}

.grok-action-row__copy,
.grok-management-row__copy {
  min-width: 0;
  flex: 1 1 auto;
}

.grok-action-row__copy strong,
.grok-management-row__copy strong {
  display: block;
  color: var(--stage-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
}

.grok-action-row__copy p,
.grok-management-row__copy p {
  margin: 0.25rem 0 0;
  font-size: 0.8125rem;
  line-height: 1.5;
}

.grok-action-row__arrow,
.grok-management-row__arrow {
  flex: 0 0 auto;
  color: var(--stage-text-quiet);
}

.grok-management-row > span {
  max-width: 8rem;
  overflow: hidden;
  color: var(--stage-text-muted);
  font-size: 0.75rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.grok-tone-icon {
  display: flex;
  width: 2.25rem;
  height: 2.25rem;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  background: var(--stage-chip-neutral-bg);
  border: 1px solid var(--stage-chip-neutral-border);
  border-radius: var(--radius-lg);
}

.grok-tone-icon--success {
  color: var(--color-success);
  background: rgb(var(--color-success-rgb) / 10%);
  border-color: rgb(var(--color-success-rgb) / 22%);
}

.grok-tone-icon--warning {
  color: var(--color-warning);
  background: rgb(var(--color-warning-rgb) / 10%);
  border-color: rgb(var(--color-warning-rgb) / 24%);
}

.grok-tone-icon--danger {
  color: var(--color-danger);
  background: rgb(var(--color-danger-rgb) / 10%);
  border-color: rgb(var(--color-danger-rgb) / 24%);
}

.grok-tone-icon--neutral {
  color: var(--stage-chip-neutral-text);
}

.grok-command-section {
  border-bottom: 0;
}

.grok-command-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.5rem 1rem;
}

.grok-command-row {
  position: relative;
  min-width: 0;
  min-height: 3rem;
  gap: 0.75rem;
  padding: 0.625rem 0.75rem;
  color: var(--stage-text-muted);
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);
  border-radius: var(--radius-lg);
}

.grok-command-row code {
  min-width: 0;
  flex: 1 1 auto;
  overflow: hidden;
  color: var(--stage-text-primary);
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.grok-command-row button {
  display: flex;
  width: 2rem;
  height: 2rem;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  color: var(--stage-text-muted);
  background: transparent;
  border: 0;
  border-radius: var(--radius-md);
  cursor: pointer;
}

.grok-command-row button:hover {
  color: var(--color-platform-grok);
  background: rgb(var(--color-platform-grok-rgb) / 10%);
}

.grok-command-row button:focus-visible {
  outline: 2px solid var(--color-platform-grok);
  outline-offset: 2px;
}

.grok-loading {
  padding: 1.5rem 0;
}

.grok-loading__grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.75rem;
  margin-top: 0.75rem;
}

.grok-skeleton {
  height: 12rem;
  background: var(--stage-surface-soft);
  border-radius: var(--radius-lg);
  animation: pulse 1.5s ease-in-out infinite;
}

.grok-skeleton--wide {
  height: 4rem;
}

@keyframes pulse {
  0%, 100% {
    opacity: 0.56;
  }

  50% {
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .grok-skeleton {
    animation: none;
  }
}

@media (width <= 960px) {
  .grok-meta,
  .grok-readiness-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .grok-workspace {
    grid-template-columns: 1fr;
    gap: 1.5rem;
  }

  .grok-management-panel {
    padding-left: 0;
    border-top: 1px solid var(--stage-border-soft);
    border-left: 0;
  }
}

@media (width <= 640px) {
  .grok-view {
    padding: 1rem;
  }

  .grok-hero {
    padding-top: 1.25rem;
  }

  .grok-hero__main,
  .grok-section__heading,
  .grok-local-only {
    align-items: flex-start;
    flex-direction: column;
  }

  .grok-hero__actions {
    width: 100%;
    justify-content: flex-start;
  }

  .grok-meta,
  .grok-readiness-grid,
  .grok-command-list,
  .grok-loading__grid {
    grid-template-columns: 1fr;
  }

  .grok-action-row__index,
  .grok-management-row > span {
    display: none;
  }

  .grok-command-row code {
    white-space: normal;
    overflow-wrap: anywhere;
  }
}
</style>
