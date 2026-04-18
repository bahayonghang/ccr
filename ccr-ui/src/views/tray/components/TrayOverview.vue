<template>
  <section
    class="tray-overview"
    data-testid="tray-overview"
  >
    <article class="tray-overview__hero">
      <div class="tray-overview__hero-main">
        <div class="tray-overview__hero-lead">
          <div class="tray-overview__hero-icon">
            <SIcon
              name="KeyRound"
              size="w-5 h-5"
            />
          </div>
          <div class="min-w-0">
            <p class="tray-overview__eyebrow">
              {{ profileLine }}
            </p>
            <div class="tray-overview__title-row">
              <p class="tray-overview__headline">
                {{ accountHeadline }}
              </p>
              <span
                v-if="currentAccount?.quota?.plan_type"
                class="tray-overview__plan-badge"
              >
                {{ currentAccount.quota.plan_type }}
              </span>
            </div>
            <p class="tray-overview__support">
              {{ currentAccount?.last_refresh ? `最近刷新 ${currentAccount.last_refresh}` : snapshot.auth_label }}
            </p>
          </div>
        </div>

        <div class="tray-overview__route-grid">
          <div class="tray-overview__route-item">
            <span class="tray-overview__route-label">{{ t('codex.auth.tray.runtimeLabel') }}</span>
            <strong class="tray-overview__route-value">{{ snapshot.runtime_description }}</strong>
          </div>
          <div class="tray-overview__route-item">
            <span class="tray-overview__route-label">{{ t('codex.auth.tray.authRouteLabel') }}</span>
            <strong class="tray-overview__route-value">{{ snapshot.auth_label }}</strong>
          </div>
        </div>
      </div>
    </article>

    <section
      v-if="currentAccount?.quota"
      class="tray-overview__quota-grid"
      data-testid="tray-overview-quotas"
    >
      <article
        class="tray-overview__quota-card"
        :class="quotaToneClass(currentAccount.quota.hourly_percentage)"
      >
        <div class="tray-overview__quota-head">
          <div>
            <p class="tray-overview__quota-label">
              {{ t('codex.auth.hourlyQuota') }}
            </p>
            <p
              v-if="currentAccount.quota.hourly_reset_time"
              class="tray-overview__quota-note"
            >
              {{ t('codex.auth.tray.resetIn') }} {{ formatReset(currentAccount.quota.hourly_reset_time) }}
            </p>
          </div>
          <strong class="tray-overview__quota-value">{{ currentAccount.quota.hourly_percentage }}%</strong>
        </div>
        <div class="tray-overview__progress">
          <span
            class="tray-overview__progress-fill"
            :style="{ transform: `scaleX(${quotaScale(currentAccount.quota.hourly_percentage)})` }"
          />
        </div>
      </article>

      <article
        class="tray-overview__quota-card"
        :class="quotaToneClass(currentAccount.quota.weekly_percentage)"
      >
        <div class="tray-overview__quota-head">
          <div>
            <p class="tray-overview__quota-label">
              {{ t('codex.auth.weeklyQuota') }}
            </p>
            <p
              v-if="currentAccount.quota.weekly_reset_time"
              class="tray-overview__quota-note"
            >
              {{ t('codex.auth.tray.resetIn') }} {{ formatResetDetailed(currentAccount.quota.weekly_reset_time) }}
            </p>
          </div>
          <strong class="tray-overview__quota-value">{{ currentAccount.quota.weekly_percentage }}%</strong>
        </div>
        <div class="tray-overview__progress">
          <span
            class="tray-overview__progress-fill"
            :style="{ transform: `scaleX(${quotaScale(currentAccount.quota.weekly_percentage)})` }"
          />
        </div>
      </article>
    </section>

    <div
      v-else
      class="tray-overview__quota-status"
    >
      <SIcon
        :name="currentAccount?.quota_error ? 'AlertCircle' : 'Clock3'"
        size="w-4 h-4"
      />
      <p>{{ currentAccount?.quota_error || t('codex.auth.quotaNotQueried') }}</p>
    </div>

    <section class="tray-overview__actions">
      <button
        type="button"
        class="tray-overview__action tray-overview__action--primary"
        data-testid="tray-action-switch"
        :disabled="!canManageAccounts"
        @click="$emit('open-switch')"
      >
        <SIcon
          name="ArrowLeftRight"
          size="w-4 h-4"
        />
        <span>{{ t('codex.auth.tray.switchAccount') }}</span>
      </button>

      <button
        type="button"
        class="tray-overview__action"
        data-testid="tray-action-open-usage"
        @click="$emit('open-usage')"
      >
        <SIcon
          name="BarChart3"
          size="w-4 h-4"
        />
        <span>{{ t('codex.auth.tray.openUsage') }}</span>
      </button>

      <button
        type="button"
        class="tray-overview__action"
        data-testid="tray-action-open-main"
        @click="$emit('open-main')"
      >
        <SIcon
          name="PanelLeftOpen"
          size="w-4 h-4"
        />
        <span>{{ t('codex.auth.tray.openMain') }}</span>
      </button>
    </section>

    <div
      v-if="!canManageAccounts"
      class="tray-overview__hint"
    >
      <span>{{ t('codex.auth.tray.switchUnavailable') }}</span>
      <button
        type="button"
        class="tray-overview__link"
        data-testid="tray-action-open-auth"
        @click="$emit('open-auth')"
      >
        {{ t('codex.auth.tray.openAuth') }}
      </button>
    </div>

    <footer class="tray-overview__footer">
      <span class="tray-overview__footer-note">
        <span class="tray-overview__footer-dot" />
        {{ currentAccount?.last_refresh ? `最近刷新 ${currentAccount.last_refresh}` : snapshot.auth_label }}
      </span>
      <button
        type="button"
        class="tray-overview__secondary"
        @click="$emit('quit')"
      >
        <SIcon
          name="Power"
          size="w-4 h-4"
        />
        <span>{{ t('codex.auth.tray.quit') }}</span>
      </button>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { CodexTrayAccountRow, CodexTraySnapshot } from '@/types'

const props = defineProps<{
  snapshot: CodexTraySnapshot
  currentAccount: CodexTrayAccountRow | null
  canManageAccounts: boolean
}>()

defineEmits<{
  (event: 'open-main'): void
  (event: 'open-switch'): void
  (event: 'open-usage'): void
  (event: 'open-auth'): void
  (event: 'quit'): void
}>()

const { t } = useI18n()

const accountHeadline = computed(() => {
  return props.currentAccount?.email || props.currentAccount?.name || props.snapshot.auth_label
})

const profileLine = computed(() => {
  return props.snapshot.current_profile_name || props.snapshot.profile_label
})

const quotaScale = (value: number) => Math.min(Math.max(value, 0), 100) / 100

const quotaToneClass = (value: number) => {
  if (value >= 85) return 'tray-overview__quota-card--critical'
  if (value >= 60) return 'tray-overview__quota-card--warning'
  return 'tray-overview__quota-card--healthy'
}

const formatReset = (timestamp: number) => {
  const now = Math.floor(Date.now() / 1000)
  const remaining = timestamp - now
  if (remaining <= 0) return t('codex.auth.resetDone')

  const hours = Math.floor(remaining / 3600)
  const minutes = Math.floor((remaining % 3600) / 60)
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

const formatResetDetailed = (timestamp: number) => {
  const now = Math.floor(Date.now() / 1000)
  const remaining = timestamp - now
  if (remaining <= 0) return t('codex.auth.resetDone')

  const hours = Math.floor(remaining / 3600)
  const minutes = Math.floor((remaining % 3600) / 60)
  if (hours >= 24) {
    const days = Math.floor(hours / 24)
    return `${days}d ${hours % 24}h ${minutes}m`
  }
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}
</script>

<style scoped>
.tray-overview {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 16px;
  min-height: 0;
}

.tray-overview__hero,
.tray-overview__quota-card,
.tray-overview__quota-status {
  border: 1px solid rgb(var(--color-border-default-rgb) / 42%);
  border-radius: 22px;
  background: linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 66%), rgb(var(--color-bg-base-rgb) / 48%));
}

.tray-overview__hero {
  padding: 16px;
}

.tray-overview__hero-main {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.tray-overview__hero-lead {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 14px;
}

.tray-overview__hero-icon {
  display: inline-flex;
  width: 48px;
  height: 48px;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 18%);
  border-radius: 16px;
  background:
    radial-gradient(circle at top, rgb(var(--color-accent-primary-rgb) / 16%), transparent 72%),
    rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--color-accent-primary);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 12%);
}

.tray-overview__eyebrow {
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.35;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.tray-overview__title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  flex-wrap: wrap;
}

.tray-overview__headline {
  color: var(--color-text-primary);
  font-size: 1.28rem;
  font-weight: 700;
  line-height: 1.08;
  letter-spacing: -0.05em;
}

.tray-overview__plan-badge {
  display: inline-flex;
  align-items: center;
  padding: 4px 8px;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 20%);
  border-radius: 999px;
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.tray-overview__support {
  margin-top: 6px;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.45;
}

.tray-overview__route-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  padding-top: 12px;
  border-top: 1px solid rgb(var(--color-border-default-rgb) / 28%);
}

.tray-overview__route-item {
  min-width: 0;
}

.tray-overview__route-label {
  display: block;
  color: var(--color-text-muted);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.tray-overview__route-value {
  display: block;
  margin-top: 5px;
  color: var(--color-text-primary);
  font-size: 12px;
  font-weight: 600;
  line-height: 1.45;
  overflow-wrap: anywhere;
}

.tray-overview__quota-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.tray-overview__quota-card {
  padding: 14px 14px 13px;
}

.tray-overview__quota-card--healthy {
  border-color: rgb(89 123 83 / 26%);
}

.tray-overview__quota-card--healthy .tray-overview__progress-fill {
  background: linear-gradient(90deg, rgb(96 143 88 / 100%), rgb(151 182 105 / 100%));
}

.tray-overview__quota-card--warning {
  border-color: rgb(181 132 63 / 30%);
}

.tray-overview__quota-card--warning .tray-overview__progress-fill {
  background: linear-gradient(90deg, rgb(202 140 58 / 100%), rgb(222 170 88 / 100%));
}

.tray-overview__quota-card--critical {
  border-color: rgb(185 101 70 / 32%);
}

.tray-overview__quota-card--critical .tray-overview__progress-fill {
  background: linear-gradient(90deg, rgb(193 103 73 / 100%), rgb(221 137 84 / 100%));
}

.tray-overview__quota-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.tray-overview__quota-label {
  color: var(--color-text-primary);
  font-size: 12px;
  font-weight: 700;
  line-height: 1.35;
}

.tray-overview__quota-value {
  color: var(--color-text-primary);
  font-size: 1.15rem;
  font-weight: 700;
  letter-spacing: -0.05em;
  line-height: 1;
}

.tray-overview__progress {
  margin-top: 12px;
  height: 6px;
  overflow: hidden;
  border-radius: 999px;
  background: rgb(var(--color-border-default-rgb) / 34%);
}

.tray-overview__progress-fill {
  display: block;
  width: 100%;
  height: 100%;
  transform-origin: left center;
}

.tray-overview__quota-note {
  margin-top: 4px;
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.4;
}

.tray-overview__quota-status {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.45;
}

.tray-overview__actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.tray-overview__action,
.tray-overview__secondary,
.tray-overview__link {
  transition: border-color 0.18s ease, background-color 0.18s ease, color 0.18s ease, transform 0.18s ease;
}

.tray-overview__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 52%);
  border-radius: 18px;
  background: rgb(var(--color-bg-base-rgb) / 78%);
  color: var(--color-text-secondary);
  padding: 12px 13px;
  font-size: 12px;
  font-weight: 600;
}

.tray-overview__action--primary {
  grid-column: 1 / -1;
  justify-content: space-between;
  border-color: rgb(var(--color-accent-primary-rgb) / 22%);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 98%), rgb(var(--color-bg-surface-rgb) / 88%));
  color: var(--color-text-primary);
  box-shadow: 0 14px 30px rgb(var(--color-accent-primary-rgb) / 8%);
}

.tray-overview__action:hover:not(:disabled),
.tray-overview__secondary:hover,
.tray-overview__link:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--color-text-primary);
}

.tray-overview__action:disabled {
  opacity: 0.48;
  cursor: not-allowed;
}

.tray-overview__hint {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 1.45;
}

.tray-overview__link,
.tray-overview__secondary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid transparent;
  border-radius: 999px;
  background: transparent;
  color: var(--color-text-secondary);
  padding: 6px 10px;
  font-size: 12px;
  font-weight: 600;
}

.tray-overview__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-top: auto;
}

.tray-overview__footer-note {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.4;
}

.tray-overview__footer-dot {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  background: rgb(var(--color-accent-primary-rgb) / 72%);
  box-shadow: 0 0 0 4px rgb(var(--color-accent-primary-rgb) / 10%);
  flex-shrink: 0;
}

@media (prefers-reduced-motion: reduce) {
  .tray-overview__action,
  .tray-overview__secondary,
  .tray-overview__link {
    transition: none;
  }
}
</style>
