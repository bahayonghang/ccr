<template>
  <section
    class="tray-switch"
    data-testid="tray-switch-screen"
  >
    <header class="tray-switch__header">
      <button
        type="button"
        class="tray-switch__back"
        data-testid="tray-switch-back"
        @click="$emit('back')"
      >
        <SIcon
          name="ArrowLeft"
          size="w-4 h-4"
        />
        <span>{{ t('common.back') }}</span>
      </button>
      <div>
        <p class="tray-switch__eyebrow">
          {{ t('codex.auth.tray.currentSession') }}
        </p>
        <h2 class="tray-switch__title">
          {{ t('codex.auth.tray.switchAccount') }}
        </h2>
      </div>
    </header>

    <article class="tray-switch__current">
      <div class="min-w-0">
        <p class="tray-switch__current-title">
          {{ currentAccount?.email || currentAccount?.name || snapshot.auth_label }}
        </p>
        <p class="tray-switch__current-subtitle">
          {{ snapshot.current_profile_name || snapshot.profile_label }}
        </p>
      </div>
      <span class="tray-switch__badge">
        {{ snapshot.runtime_description }}
      </span>
    </article>

    <div
      v-if="accounts.length === 0"
      class="tray-switch__empty"
    >
      <SIcon
        name="Users"
        size="w-4 h-4"
      />
      <div>
        <p>{{ t('codex.auth.tray.noAccountsTitle') }}</p>
        <p>{{ t('codex.auth.tray.noAccountsHint') }}</p>
      </div>
    </div>

    <section
      v-else
      class="tray-switch__list"
      data-testid="tray-switch-list"
    >
      <article
        v-for="account in accounts"
        :key="account.name"
        class="tray-switch__row"
        :class="{ 'tray-switch__row--current': account.is_current }"
        :data-testid="`tray-switch-row-${account.name}`"
      >
        <div class="min-w-0">
          <p class="tray-switch__row-title">
            {{ account.email || account.name }}
          </p>
          <p class="tray-switch__row-subtitle">
            {{ account.name }} · {{ account.freshness_description }}
          </p>
        </div>

        <div class="tray-switch__row-actions">
          <span
            class="tray-switch__status"
            :class="statusClass(account)"
          >
            {{ statusLabel(account) }}
          </span>

          <button
            v-if="account.can_switch"
            type="button"
            class="tray-switch__action"
            :disabled="busyAccount === account.name"
            @click="$emit('switch', account.name)"
          >
            <SIcon
              :name="busyAccount === account.name ? 'RefreshCw' : 'ArrowLeftRight'"
              size="w-4 h-4"
              :class="{ 'animate-spin': busyAccount === account.name }"
            />
            <span>{{ t('codex.auth.switch') }}</span>
          </button>
        </div>
      </article>
    </section>

    <footer class="tray-switch__footer">
      <button
        type="button"
        class="tray-switch__footer-action"
        @click="$emit('open-auth')"
      >
        <SIcon
          name="Users"
          size="w-4 h-4"
        />
        <span>{{ t('codex.auth.tray.openAuth') }}</span>
      </button>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { CodexTrayAccountRow, CodexTraySnapshot } from '@/types'

const props = defineProps<{
  snapshot: CodexTraySnapshot
  currentAccount: CodexTrayAccountRow | null
  accounts: CodexTrayAccountRow[]
  busyAccount: string | null
  canManageAccounts: boolean
}>()

defineEmits<{
  (event: 'back'): void
  (event: 'switch', accountName: string): void
  (event: 'open-auth'): void
}>()

const { t } = useI18n()

const statusLabel = (account: CodexTrayAccountRow) => {
  if (account.is_current) {
    return t('codex.auth.currentBadge')
  }
  if (account.is_expired) {
    return t('codex.auth.expiredBadge')
  }
  if (!props.canManageAccounts) {
    return t('codex.auth.tray.unavailableInCurrentProfile')
  }
  if (!account.can_switch) {
    return t('settings.disabled')
  }
  return t('codex.auth.tray.available')
}

const statusClass = (account: CodexTrayAccountRow) => {
  if (account.is_current) return 'tray-switch__status--current'
  if (account.is_expired) return 'tray-switch__status--danger'
  if (!account.can_switch || !props.canManageAccounts) return 'tray-switch__status--muted'
  return 'tray-switch__status--available'
}
</script>

<style scoped>
.tray-switch {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
}

.tray-switch__header,
.tray-switch__current,
.tray-switch__row,
.tray-switch__row-actions,
.tray-switch__footer {
  display: flex;
  align-items: center;
  gap: 10px;
}

.tray-switch__header,
.tray-switch__current,
.tray-switch__row,
.tray-switch__footer {
  justify-content: space-between;
}

.tray-switch__back,
.tray-switch__action,
.tray-switch__footer-action {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 52%);
  border-radius: 14px;
  background: rgb(var(--color-bg-base-rgb) / 78%);
  color: var(--color-text-secondary);
  padding: 10px 12px;
  font-size: 12px;
  font-weight: 600;
  transition: border-color 0.18s ease, background-color 0.18s ease, color 0.18s ease, transform 0.18s ease;
}

.tray-switch__back:hover,
.tray-switch__action:hover:not(:disabled),
.tray-switch__footer-action:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--color-text-primary);
}

.tray-switch__eyebrow {
  color: var(--color-text-muted);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.tray-switch__title {
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 700;
  letter-spacing: -0.03em;
}

.tray-switch__current,
.tray-switch__row,
.tray-switch__empty {
  border: 1px solid rgb(var(--color-border-default-rgb) / 42%);
  border-radius: 18px;
  background: rgb(var(--color-bg-base-rgb) / 50%);
  padding: 12px;
}

.tray-switch__current-title,
.tray-switch__row-title,
.tray-switch__empty p:first-child {
  color: var(--color-text-primary);
  font-size: 13px;
  font-weight: 600;
}

.tray-switch__current-subtitle,
.tray-switch__row-subtitle,
.tray-switch__empty p:last-child {
  margin-top: 2px;
  color: var(--color-text-secondary);
  font-size: 11px;
  line-height: 1.45;
}

.tray-switch__badge,
.tray-switch__status {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  padding: 4px 8px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.tray-switch__badge {
  border: 1px solid rgb(var(--color-border-default-rgb) / 52%);
  background: rgb(var(--color-bg-base-rgb) / 84%);
  color: var(--color-text-muted);
}

.tray-switch__list {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
  overflow: auto;
  padding-right: 2px;
}

.tray-switch__row {
  align-items: center;
}

.tray-switch__row--current {
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: rgb(var(--color-accent-primary-rgb) / 7%);
}

.tray-switch__row-actions {
  flex-shrink: 0;
}

.tray-switch__action:disabled {
  opacity: 0.48;
  cursor: not-allowed;
}

.tray-switch__status--current {
  background: rgb(16 185 129 / 12%);
  color: rgb(16 185 129 / 100%);
}

.tray-switch__status--danger {
  background: rgb(var(--color-danger-rgb) / 10%);
  color: var(--color-danger);
}

.tray-switch__status--muted {
  background: rgb(var(--color-border-default-rgb) / 20%);
  color: var(--color-text-muted);
}

.tray-switch__status--available {
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);
}

.tray-switch__empty {
  align-items: flex-start;
}

.tray-switch__footer {
  margin-top: auto;
}

.tray-switch__footer-action {
  margin-left: auto;
}
</style>
