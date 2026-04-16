<template>
  <section
    class="tray-overview"
    data-testid="tray-overview"
  >
    <article class="tray-overview__hero">
      <div class="tray-overview__hero-main">
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
          <p class="tray-overview__headline">
            {{ accountHeadline }}
          </p>
        </div>
      </div>

      <div class="tray-overview__meta">
        <span class="tray-overview__pill">
          {{ snapshot.runtime_description }}
        </span>
        <span class="tray-overview__pill">
          {{ snapshot.auth_label }}
        </span>
        <span
          v-if="currentAccount?.quota?.plan_type"
          class="tray-overview__pill tray-overview__pill--accent"
        >
          {{ currentAccount.quota.plan_type }}
        </span>
      </div>
    </article>

    <section
      v-if="currentAccount?.quota"
      class="tray-overview__quota-grid"
      data-testid="tray-overview-quotas"
    >
      <article class="tray-overview__quota-card">
        <div class="tray-overview__quota-row">
          <span>{{ t('codex.auth.hourlyQuota') }}</span>
          <strong>{{ currentAccount.quota.hourly_percentage }}%</strong>
        </div>
        <div class="tray-overview__progress">
          <span
            class="tray-overview__progress-fill"
            :style="{ transform: `scaleX(${quotaScale(currentAccount.quota.hourly_percentage)})` }"
          />
        </div>
        <p
          v-if="currentAccount.quota.hourly_reset_time"
          class="tray-overview__quota-note"
        >
          {{ formatReset(currentAccount.quota.hourly_reset_time) }}
        </p>
      </article>

      <article class="tray-overview__quota-card">
        <div class="tray-overview__quota-row">
          <span>{{ t('codex.auth.weeklyQuota') }}</span>
          <strong>{{ currentAccount.quota.weekly_percentage }}%</strong>
        </div>
        <div class="tray-overview__progress">
          <span
            class="tray-overview__progress-fill tray-overview__progress-fill--secondary"
            :style="{ transform: `scaleX(${quotaScale(currentAccount.quota.weekly_percentage)})` }"
          />
        </div>
        <p
          v-if="currentAccount.quota.weekly_reset_time"
          class="tray-overview__quota-note"
        >
          {{ formatResetDetailed(currentAccount.quota.weekly_reset_time) }}
        </p>
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
        @click="$emit('open-auth')"
      >
        {{ t('codex.auth.tray.openAuth') }}
      </button>
    </div>

    <footer class="tray-overview__footer">
      <span class="tray-overview__footer-note">
        {{ currentAccount?.freshness_description || snapshot.auth_label }}
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
  gap: 14px;
  min-height: 0;
}

.tray-overview__hero,
.tray-overview__quota-card,
.tray-overview__quota-status {
  border: 1px solid rgb(var(--color-border-default-rgb) / 42%);
  border-radius: 18px;
  background: rgb(var(--color-bg-base-rgb) / 50%);
}

.tray-overview__hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 14px;
}

.tray-overview__hero-main {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 12px;
}

.tray-overview__hero-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 14px;
  background: rgb(var(--color-accent-primary-rgb) / 14%);
  color: var(--color-accent-primary);
  flex-shrink: 0;
}

.tray-overview__eyebrow {
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 1.4;
}

.tray-overview__headline {
  color: var(--color-text-primary);
  font-size: 16px;
  font-weight: 700;
  line-height: 1.4;
}

.tray-overview__meta {
  display: flex;
  max-width: 42%;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 6px;
}

.tray-overview__pill {
  display: inline-flex;
  align-items: center;
  border: 1px solid rgb(var(--color-border-default-rgb) / 52%);
  border-radius: 999px;
  background: rgb(var(--color-bg-base-rgb) / 84%);
  color: var(--color-text-muted);
  padding: 5px 8px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.tray-overview__pill--accent {
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);
}

.tray-overview__quota-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.tray-overview__quota-card {
  padding: 12px;
}

.tray-overview__quota-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  color: var(--color-text-secondary);
  font-size: 12px;
}

.tray-overview__quota-row strong {
  color: var(--color-text-primary);
  font-size: 13px;
}

.tray-overview__progress {
  margin-top: 8px;
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
  background: linear-gradient(90deg, rgb(16 185 129 / 100%), rgb(59 130 246 / 100%));
}

.tray-overview__progress-fill--secondary {
  background: linear-gradient(90deg, rgb(245 158 11 / 100%), rgb(236 72 153 / 100%));
}

.tray-overview__quota-note {
  margin-top: 8px;
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.4;
}

.tray-overview__quota-status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.45;
}

.tray-overview__actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
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
  border-radius: 16px;
  background: rgb(var(--color-bg-base-rgb) / 80%);
  color: var(--color-text-secondary);
  padding: 11px 12px;
  font-size: 12px;
  font-weight: 600;
}

.tray-overview__action--primary {
  grid-column: 1 / -1;
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
  gap: 10px;
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
  gap: 10px;
  margin-top: auto;
}

.tray-overview__footer-note {
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.4;
}
</style>
