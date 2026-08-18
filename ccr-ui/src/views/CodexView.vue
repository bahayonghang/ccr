<template>
  <PageShell class="codex-view">
    <template #header>
      <PageHeader
        :title="t('codex.overview.title')"
        :eyebrow="$t('codex.dashboard.header.eyebrow')"
        :description="$t('codex.dashboard.header.subtitle')"
      >
        <template #actions>
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
            {{ $t('codex.dashboard.header.refresh') }}
          </Button>
          <RouterLink :to="primaryAction.to">
            <Button :variant="primaryButtonVariant">
              <SIcon
                :name="primaryAction.icon"
                size="w-4 h-4"
                class="mr-2"
              />
              {{ primaryAction.title }}
            </Button>
          </RouterLink>
          <RouterLink to="/codex/auth">
            <Button variant="secondary">
              <SIcon
                name="KeyRound"
                size="w-4 h-4"
                class="mr-2"
              />
              {{ $t('codex.dashboard.header.authConfig') }}
            </Button>
          </RouterLink>
          <RouterLink to="/codex/profiles">
            <Button variant="secondary">
              <SIcon
                name="Folders"
                size="w-4 h-4"
                class="mr-2"
              />
              {{ $t('codex.dashboard.header.profileConfig') }}
            </Button>
          </RouterLink>
        </template>
      </PageHeader>
    </template>

    <div class="codex-stats">
      <StatTile
        :label="$t('codex.dashboard.header.version')"
        :value="versionLabel"
      />
      <StatTile
        :label="$t('codex.dashboard.header.profile')"
        :value="currentProfileLabel"
      />
      <StatTile
        :label="$t('codex.dashboard.header.account')"
        :value="currentAccountLabel"
      />
    </div>

      <PlatformUsageInsightPanel
        :spec="codexUsageSpec"
        :state="codexUsagePresentation"
        :loading="codexUsage.loading.value"
        :error="codexUsage.error.value"
        @refresh="codexUsage.refresh()"
      />

      <section class="codex-readiness-board">
        <div class="codex-section-heading">
          <div>
            <p class="codex-section-kicker">
              {{ $t('codex.dashboard.readiness.eyebrow') }}
            </p>
            <h2 class="codex-section-title">
              {{ $t('codex.dashboard.readiness.title') }}
            </h2>
          </div>
          <p class="codex-section-description">
            {{ $t('codex.dashboard.readiness.subtitle') }}
          </p>
        </div>

        <div
          v-if="readinessItems.length"
          class="codex-readiness-grid"
        >
          <RouterLink
            v-for="item in readinessItems"
            :key="item.key"
            :to="item.to"
            class="codex-readiness-item"
            :class="`codex-readiness-item--${item.tone}`"
          >
            <div class="codex-readiness-item__axis" />
            <div class="codex-readiness-item__body">
              <div class="codex-readiness-item__topline">
                <div
                  class="codex-tone-icon"
                  :class="toneClassMap[item.tone]"
                >
                  <SIcon
                    :name="item.icon"
                    size="w-4 h-4"
                  />
                </div>
                <span class="codex-readiness-status">{{ item.statusLabel }}</span>
              </div>
              <p class="codex-readiness-label">
                {{ item.title }}
              </p>
              <p class="codex-readiness-value">
                {{ item.value }}
              </p>
              <p class="codex-readiness-detail">
                {{ item.detail }}
              </p>
            </div>
            <SIcon
              name="ArrowUpRight"
              size="w-4 h-4"
              class="codex-readiness-arrow"
            />
          </RouterLink>
        </div>

        <div
          v-else-if="overviewLoading"
          class="codex-readiness-grid"
        >
          <div
            v-for="n in 4"
            :key="`readiness-skeleton-${n}`"
            class="codex-skeleton codex-skeleton--readiness"
          />
        </div>

        <EmptyState
          v-else
          icon="ShieldCheck"
          :title="$t('codex.dashboard.empty.readinessTitle')"
          :description="$t('codex.dashboard.empty.readinessDescription')"
          :action-text="$t('codex.dashboard.header.refresh')"
          action-icon="RefreshCw"
          :on-action="() => refresh(true)"
        />
      </section>

      <section class="codex-action-console">
        <Card
          variant="glass"
          class="codex-console-card codex-console-card--actions"
        >
          <div class="codex-console-header">
            <div>
              <p class="codex-section-kicker">
                {{ $t('codex.dashboard.actionConsole.eyebrow') }}
              </p>
              <h2 class="codex-section-title">
                {{ $t('codex.dashboard.actionConsole.title') }}
              </h2>
              <p class="codex-console-subtitle">
                {{ $t('codex.dashboard.actionConsole.subtitle') }}
              </p>
            </div>
          </div>

          <div
            v-if="error && !overview"
            class="codex-alert codex-alert--danger"
          >
            <div>
              <p class="codex-alert-title">
                {{ $t('codex.dashboard.error.title') }}
              </p>
              <p class="codex-alert-message">
                {{ error }}
              </p>
            </div>
            <Button
              variant="glass"
              size="sm"
              @click="refresh(true)"
            >
              <SIcon
                name="RefreshCw"
                size="w-4 h-4"
                class="mr-2"
              />
              {{ $t('codex.dashboard.header.refresh') }}
            </Button>
          </div>

          <div
            v-if="visibleNextActions.length"
            class="codex-next-list"
          >
            <RouterLink
              v-for="(action, index) in visibleNextActions"
              :key="action.title"
              :to="action.to"
              class="codex-next-item"
              :class="`codex-next-item--${action.tone}`"
            >
              <span class="codex-next-index">{{ index + 1 }}</span>
              <div
                class="codex-tone-icon codex-tone-icon--large"
                :class="toneClassMap[action.tone]"
              >
                <SIcon
                  :name="action.icon"
                  size="w-5 h-5"
                />
              </div>
              <div class="codex-next-copy">
                <h3>{{ action.title }}</h3>
                <p>{{ action.description }}</p>
              </div>
              <SIcon
                name="ArrowRight"
                size="w-4 h-4"
                class="codex-next-arrow"
              />
            </RouterLink>
          </div>

          <div
            v-else-if="overviewLoading"
            class="codex-next-list"
          >
            <div class="codex-skeleton codex-skeleton--next" />
            <div class="codex-skeleton codex-skeleton--next" />
          </div>

          <EmptyState
            v-else-if="!overview"
            icon="Route"
            :title="$t('codex.dashboard.empty.actionsTitle')"
            :description="$t('codex.dashboard.empty.actionsDescription')"
            :action-text="$t('codex.dashboard.header.refresh')"
            action-icon="RefreshCw"
            :on-action="() => refresh(true)"
          />

          <div class="codex-usage-strip">
            <div class="codex-usage-strip__item">
              <span>{{ $t('codex.dashboard.usage.requests') }}</span>
              <strong>{{ usageTotalRequests }}</strong>
            </div>
            <div class="codex-usage-strip__item">
              <span>{{ $t('codex.dashboard.usage.tokens') }}</span>
              <strong>{{ usageTotalTokens }}</strong>
            </div>
            <div class="codex-usage-strip__item codex-usage-strip__item--wide">
              <span>{{ $t('codex.dashboard.usage.model') }}</span>
              <strong>
                {{ usageSummary?.top_model?.model || overview?.config.model || (usageLoading ? $t('codex.dashboard.usage.loading') : $t('codex.dashboard.usage.unknownModel')) }}
              </strong>
            </div>
            <div class="codex-usage-strip__item codex-usage-strip__item--wide">
              <span>{{ $t('codex.dashboard.usage.lastActivity') }}</span>
              <strong>
                {{ usageSummary?.last_activity_at ? formatDateTime(usageSummary.last_activity_at) : (usageLoading ? $t('codex.dashboard.usage.loading') : $t('codex.dashboard.usage.noActivity')) }}
              </strong>
            </div>
          </div>

          <div
            v-if="usageError && !usageSummary"
            class="codex-alert codex-alert--warning"
          >
            <p class="codex-alert-title">
              {{ $t('codex.dashboard.error.usageTitle') }}
            </p>
            <p class="codex-alert-message">
              {{ usageError }}
            </p>
          </div>
        </Card>

        <Card
          variant="glass"
          class="codex-console-card codex-console-card--manage"
        >
          <div class="codex-console-header codex-console-header--compact">
            <div>
              <p class="codex-section-kicker">
                {{ $t('codex.dashboard.management.eyebrow') }}
              </p>
              <h2 class="codex-section-title">
                {{ $t('codex.dashboard.management.title') }}
              </h2>
              <p class="codex-console-subtitle">
                {{ $t('codex.dashboard.management.subtitle') }}
              </p>
            </div>
          </div>

          <div
            v-if="compactInventory.length"
            class="codex-manage-list"
          >
            <RouterLink
              v-for="item in compactInventory"
              :key="item.key"
              :to="item.to"
              class="codex-manage-row"
            >
              <div
                class="codex-tone-icon"
                :class="toneClassMap[item.tone]"
              >
                <SIcon
                  :name="item.icon"
                  size="w-4 h-4"
                />
              </div>
              <div class="codex-manage-copy">
                <span>{{ item.title }}</span>
                <small>{{ item.detail }}</small>
              </div>
              <strong>{{ item.value }}</strong>
            </RouterLink>
          </div>

          <div
            v-else-if="overviewLoading"
            class="codex-manage-list"
          >
            <div
              v-for="n in 6"
              :key="`manage-skeleton-${n}`"
              class="codex-skeleton codex-skeleton--manage"
            />
          </div>

          <EmptyState
            v-else
            icon="Folders"
            :title="$t('codex.dashboard.empty.managementTitle')"
            :description="$t('codex.dashboard.empty.managementDescription')"
            :action-text="$t('codex.dashboard.header.refresh')"
            action-icon="RefreshCw"
            :on-action="() => refresh(true)"
          />
        </Card>
      </section>
  </PageShell>
</template>

<script setup lang="ts">
import { computed, onActivated, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import StatTile from '@/components/ui/StatTile.vue'
import SIcon from '@/components/ui/SIcon.vue'
import PlatformUsageInsightPanel from '@/components/platform-usage/PlatformUsageInsightPanel.vue'
import { useCodexDashboard, type CodexDashboardTone } from '@/composables/useCodexDashboard'
import { usePlatformUsageInsight } from '@/composables/usePlatformUsageInsight'
import {
  buildPlatformUsageI18nLabels,
  buildPlatformUsageSpec,
} from '@/views/platform-usage/platformUsageSpecs'

defineOptions({ name: 'CodexView' })

const { t } = useI18n()

const {
  overview,
  usageSummary,
  loading,
  error,
  overviewLoading,
  usageLoading,
  usageError,
  versionLabel,
  currentAccountLabel,
  currentProfileLabel,
  usageTotalRequests,
  usageTotalTokens,
  readinessItems,
  nextActions,
  primaryAction,
  compactInventory,
  formatDateTime,
  refresh,
} = useCodexDashboard()

const codexUsageLabels = computed(() => buildPlatformUsageI18nLabels(t))
const codexUsage = usePlatformUsageInsight({
  platform: 'codex',
  labels: codexUsageLabels,
  tone: 'codex',
})
const codexUsageSpec = computed(() => buildPlatformUsageSpec(t, 'codex'))
const codexUsagePresentation = computed(() => codexUsage.presentation.value)

const toneClassMap: Record<CodexDashboardTone, string> = {
  success: 'codex-tone-icon--success',
  warning: 'codex-tone-icon--warning',
  danger: 'codex-tone-icon--danger',
  neutral: 'codex-tone-icon--neutral',
}

const visibleNextActions = computed(() => nextActions.value.slice(0, 2))

const primaryButtonVariant = computed(() => {
  if (primaryAction.value.tone === 'danger') return 'danger'
  if (primaryAction.value.tone === 'success') return 'success'
  return 'primary'
})

onMounted(() => {
  void refresh(false)
})

onActivated(() => {
  void refresh(false)
})
</script>

<style scoped>
.codex-view {
  background: var(--color-bg-elevated);
}

.codex-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
  gap: 1rem;
  padding: 1rem 1.25rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: 12px;
  background: var(--color-bg-surface);
}

.codex-readiness-board,
.codex-console-card {
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-surface);
  border-radius: 12px;
}

.codex-command-header__main {
  @apply relative z-10 flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between;
}

.codex-command-header__copy {
  @apply min-w-0 space-y-4;
}

.codex-eyebrow-row {
  @apply flex flex-wrap items-center gap-2;
}

.codex-eyebrow {
  color: var(--color-accent-primary);

  @apply text-xs font-semibold uppercase tracking-[0.2em];
}

.codex-eyebrow--muted {
  color: var(--stage-text-quiet);
}

.codex-status-dot {
  background: var(--color-accent-primary);
  box-shadow: 0 0 0 4px rgb(var(--color-accent-primary-rgb) / 10%);

  @apply h-2 w-2 rounded-full;
}

.codex-title-row {
  @apply flex items-center gap-4;
}

.codex-mark {
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 20%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);

  @apply flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl;
}

.codex-title {
  color: var(--stage-text-primary);
  font-family: var(--font-brand);

  @apply text-[2.65rem] font-semibold leading-none tracking-[-0.055em];
}

.codex-subtitle {
  color: var(--stage-text-secondary);

  @apply mt-2 max-w-2xl text-sm leading-6;
}

.codex-command-actions {
  @apply flex flex-wrap gap-2 lg:justify-end;
}

.codex-command-actions__config {
  color: var(--stage-text-secondary);
}

.codex-command-meta {
  @apply relative z-10 mt-5 grid grid-cols-1 gap-2 md:grid-cols-3;
}

.codex-meta-chip {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply flex min-w-0 items-center justify-between gap-3 rounded-2xl px-4 py-3;
}

.codex-meta-chip span {
  color: var(--stage-text-quiet);

  @apply shrink-0 text-xs font-medium uppercase tracking-[0.14em];
}

.codex-meta-chip strong {
  color: var(--stage-text-primary);

  @apply min-w-0 truncate text-sm font-semibold;
}

.codex-section-heading {
  @apply mb-4 flex flex-col gap-2 lg:flex-row lg:items-end lg:justify-between;
}

.codex-section-kicker {
  color: var(--stage-text-quiet);

  @apply text-xs font-semibold uppercase tracking-[0.18em];
}

.codex-section-title {
  color: var(--stage-text-primary);

  @apply mt-1 text-lg font-semibold tracking-[-0.02em];
}

.codex-section-description,
.codex-console-subtitle {
  color: var(--stage-text-secondary);

  @apply max-w-xl text-sm leading-6;
}

.codex-readiness-board {
  @apply p-4 lg:p-5;
}

.codex-readiness-grid {
  @apply grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4;
}

.codex-readiness-item {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply relative flex min-h-[13rem] overflow-hidden rounded-3xl p-4 transition-all duration-200;
}

.codex-readiness-item:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 22%);
  transform: translateY(-1px);
}

.codex-readiness-item--danger,
.codex-readiness-item--warning {
  background: var(--color-bg-surface);
}

.codex-readiness-item__axis {
  display: none;
}

.codex-readiness-item__body {
  @apply flex min-w-0 flex-1 flex-col;
}

.codex-readiness-item__topline {
  @apply mb-4 flex items-center justify-between gap-3;
}

.codex-readiness-status {
  background: var(--stage-chip-neutral-bg);
  border: 1px solid var(--stage-chip-neutral-border);
  color: var(--stage-chip-neutral-text);

  @apply rounded-md px-2.5 py-1 text-[0.68rem] font-medium;
}

.codex-readiness-label {
  color: var(--stage-text-quiet);

  @apply text-xs font-medium;
}

.codex-readiness-value {
  color: var(--stage-text-primary);

  @apply mt-2 break-words text-lg font-semibold leading-snug;
}

.codex-readiness-detail {
  color: var(--stage-text-secondary);

  @apply mt-auto pt-3 text-sm leading-6;
}

.codex-readiness-arrow {
  color: var(--stage-text-quiet);

  @apply absolute right-4 top-4 opacity-0 transition-opacity duration-200;
}

.codex-readiness-item:hover .codex-readiness-arrow {
  @apply opacity-100;
}

.codex-action-console {
  @apply grid grid-cols-1 gap-4 xl:grid-cols-5;
}

.codex-console-card {
  @apply p-5;
}

.codex-console-card--actions {
  @apply xl:col-span-3;
}

.codex-console-card--manage {
  @apply xl:col-span-2;
}

.codex-console-header {
  @apply mb-4 flex items-start justify-between gap-3;
}

.codex-console-header--compact {
  @apply mb-3;
}

.codex-next-list {
  @apply space-y-3;
}

.codex-next-item {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply relative grid grid-cols-[auto_auto_1fr_auto] items-start gap-3 rounded-3xl p-4 transition-all duration-200;
}

.codex-next-item:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 22%);
  transform: translateY(-1px);
}

.codex-next-item--danger {
  border-color: rgb(var(--color-danger-rgb) / 20%);
}

.codex-next-item--warning {
  border-color: rgb(var(--color-warning-rgb) / 18%);
}

.codex-next-index {
  color: var(--stage-text-quiet);

  @apply pt-2 text-xs font-semibold tracking-[0.14em];
}

.codex-next-copy {
  @apply min-w-0;
}

.codex-next-copy h3 {
  color: var(--stage-text-primary);

  @apply text-base font-semibold;
}

.codex-next-copy p {
  color: var(--stage-text-secondary);

  @apply mt-1 text-sm leading-6;
}

.codex-next-arrow {
  color: var(--stage-text-quiet);

  @apply mt-3;
}

.codex-usage-strip {
  border: 1px solid var(--stage-border-soft);
  background: var(--stage-surface-soft);

  @apply mt-4 grid grid-cols-2 gap-2 rounded-3xl p-2 lg:grid-cols-4;
}

.codex-usage-strip__item {
  @apply min-w-0 rounded-2xl px-3 py-2;
}

.codex-usage-strip__item span {
  color: var(--stage-text-quiet);

  @apply block text-[0.68rem] font-semibold uppercase tracking-[0.12em];
}

.codex-usage-strip__item strong {
  color: var(--stage-text-primary);

  @apply mt-1 block truncate text-sm font-semibold;
}

.codex-manage-list {
  @apply space-y-2;
}

.codex-manage-row {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply flex items-center gap-3 rounded-2xl px-3 py-3 transition-all duration-200;
}

.codex-manage-row:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  transform: translateX(1px);
}

.codex-manage-copy {
  @apply min-w-0 flex-1;
}

.codex-manage-copy span {
  color: var(--stage-text-primary);

  @apply block truncate text-sm font-semibold;
}

.codex-manage-copy small {
  color: var(--stage-text-muted);

  @apply mt-0.5 block truncate text-xs;
}

.codex-manage-row strong {
  color: var(--stage-text-secondary);

  @apply max-w-[7rem] truncate text-sm font-semibold;
}

.codex-tone-icon {
  @apply flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border;
}

.codex-tone-icon--large {
  @apply h-11 w-11;
}

.codex-tone-icon--success {
  border-color: rgb(var(--color-success-rgb) / 18%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--color-success);
}

.codex-tone-icon--warning {
  border-color: rgb(var(--color-warning-rgb) / 20%);
  background: rgb(var(--color-warning-rgb) / 11%);
  color: var(--color-warning);
}

.codex-tone-icon--danger {
  border-color: rgb(var(--color-danger-rgb) / 20%);
  background: rgb(var(--color-danger-rgb) / 11%);
  color: var(--color-danger);
}

.codex-tone-icon--neutral {
  border-color: var(--stage-chip-neutral-border);
  background: var(--stage-chip-neutral-bg);
  color: var(--stage-chip-neutral-text);
}

.codex-alert {
  @apply mb-4 flex flex-col gap-3 rounded-3xl p-4 text-sm lg:flex-row lg:items-center lg:justify-between;
}

.codex-alert--danger {
  color: rgb(136 33 57 / 96%);
  background: rgb(255 231 239 / 76%);
  border: 1px solid rgb(212 111 136 / 28%);
}

.codex-alert--warning {
  color: rgb(126 78 22 / 96%);
  background: rgb(255 243 218 / 78%);
  border: 1px solid rgb(214 161 67 / 30%);
}

.codex-alert-title {
  @apply font-semibold;
}

.codex-alert-message {
  @apply mt-1 break-words;
}

[data-theme='dark'] .codex-alert--danger,
:root[class~='dark'] .codex-alert--danger {
  color: rgb(255 225 233 / 92%);
  background: rgb(120 27 56 / 22%);
  border-color: rgb(234 143 170 / 28%);
}

[data-theme='dark'] .codex-alert--warning,
:root[class~='dark'] .codex-alert--warning {
  color: rgb(255 242 213 / 90%);
  background: rgb(131 90 19 / 22%);
  border-color: rgb(225 180 91 / 28%);
}

.codex-skeleton {
  background: var(--stage-surface-soft);

  @apply animate-pulse rounded-3xl;
}

.codex-skeleton--readiness {
  @apply h-52;
}

.codex-skeleton--next {
  @apply h-28;
}

.codex-skeleton--manage {
  @apply h-16;
}

@media (width <= 640px) {
  .codex-next-item {
    @apply grid-cols-[auto_1fr_auto];
  }

  .codex-next-index {
    @apply hidden;
  }
}
</style>
