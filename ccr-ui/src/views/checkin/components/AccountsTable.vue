<template>
  <div class="checkin-accounts-tab__table-shell checkin-surface-card">
    <table class="checkin-accounts-tab__table">
      <thead class="checkin-accounts-tab__table-head">
        <tr>
          <th class="checkin-accounts-tab__th">
            {{ t('checkin.accounts.columns.account') }}
          </th>
          <th class="checkin-accounts-tab__th checkin-accounts-tab__th--right">
            {{ t('checkin.accounts.columns.balance') }}
          </th>
          <th class="checkin-accounts-tab__th checkin-accounts-tab__th--right">
            {{ t('checkin.accounts.columns.totalQuota') }}
          </th>
          <th class="checkin-accounts-tab__th checkin-accounts-tab__th--right">
            {{ t('checkin.accounts.columns.totalConsumed') }}
          </th>
          <th class="checkin-accounts-tab__th">
            {{ t('checkin.accounts.columns.lastCheckin') }}
          </th>
          <th class="checkin-accounts-tab__th checkin-accounts-tab__th--actions">
            {{ t('checkin.accounts.columns.actions') }}
          </th>
        </tr>
      </thead>
      <tbody class="checkin-accounts-tab__table-body">
        <tr
          v-for="account in accounts"
          :key="account.id"
          class="checkin-accounts-tab__row"
          @click="emit('navigate', account.id)"
        >
          <!-- 账号名 + 提供商 -->
          <td class="checkin-accounts-tab__cell">
            <div class="checkin-accounts-tab__account">
              <div class="checkin-accounts-tab__account-row">
                <div
                  class="checkin-accounts-tab__status-dot"
                  :class="account.enabled ? 'bg-accent-success' : 'bg-text-muted'"
                />
                <span class="checkin-accounts-tab__account-name">
                  {{ account.name }}
                </span>
              </div>
              <span class="checkin-accounts-tab__provider-chip">
                {{ account.provider_name || getProviderName(account.provider_id) }}
              </span>
            </div>
          </td>
          <!-- 余额 -->
          <td class="checkin-accounts-tab__cell checkin-accounts-tab__cell--right">
            <span
              v-if="account.latest_balance !== undefined && account.latest_balance !== null"
              class="checkin-accounts-tab__metric checkin-accounts-tab__metric--balance"
            >
              ${{ account.latest_balance.toFixed(2) }}
            </span>
            <span
              v-else
              class="checkin-accounts-tab__placeholder"
            >-</span>
          </td>
          <!-- 总额度 -->
          <td class="checkin-accounts-tab__cell checkin-accounts-tab__cell--right">
            <span
              v-if="account.total_quota !== undefined && account.total_quota !== null"
              class="checkin-accounts-tab__metric checkin-accounts-tab__metric--quota"
            >
              ${{ account.total_quota.toFixed(2) }}
            </span>
            <span
              v-else
              class="checkin-accounts-tab__placeholder"
            >-</span>
          </td>
          <!-- 历史消耗 -->
          <td class="checkin-accounts-tab__cell checkin-accounts-tab__cell--right">
            <span
              v-if="account.total_consumed !== undefined && account.total_consumed !== null"
              class="checkin-accounts-tab__metric checkin-accounts-tab__metric--consumed"
            >
              ${{ account.total_consumed.toFixed(2) }}
            </span>
            <span
              v-else
              class="checkin-accounts-tab__placeholder"
            >-</span>
          </td>
          <!-- 最后签到 -->
          <td class="checkin-accounts-tab__cell checkin-accounts-tab__cell--mono">
            {{ account.last_checkin_at ? formatDate(account.last_checkin_at) : '-' }}
          </td>
          <!-- 操作 -->
          <td
            class="checkin-accounts-tab__cell"
            @click.stop
          >
            <div class="checkin-accounts-tab__row-actions">
              <button
                :disabled="checkinLoading"
                class="checkin-accounts-tab__mini-button"
                :title="checkinLoading ? t('checkin.actions.checking') : t('checkin.actions.checkIn')"
                @click="emit('checkin', account.id)"
              >
                <SIcon
                  :name="checkinLoading ? 'Loader2' : 'Calendar'"
                  size="w-3 h-3"
                  :class="[
                    'checkin-accounts-tab__mini-button-icon',
                    { 'animate-spin': checkinLoading },
                  ]"
                />
                <span class="checkin-accounts-tab__mini-button-label">{{ checkinLoading ? t('checkin.actions.checking') : t('checkin.actions.checkIn') }}</span>
              </button>
              <div class="checkin-accounts-tab__menu-wrap">
                <button
                  class="checkin-accounts-tab__menu-trigger"
                  :class="{
                    'checkin-accounts-tab__menu-trigger--active':
                      actionsMenuRef?.openAccountId === account.id,
                  }"
                  @click="actionsMenuRef?.toggle(account.id, $event)"
                >
                  <svg
                    class="w-4 h-4"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z"
                    />
                  </svg>
                </button>
              </div>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>

  <AccountActionsMenu
    ref="actionsMenuRef"
    :accounts="accounts"
    @refresh-balance="emit('refresh-balance', $event)"
    @edit="emit('edit', $event)"
    @delete="emit('delete', $event)"
  />
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CheckinProvider, AccountInfo } from '@/types/checkin'
import AccountActionsMenu from './AccountActionsMenu.vue'

const props = defineProps<{
  /** 已过滤的账号列表（搜索/提供商过滤由父级完成） */
  accounts: AccountInfo[]
  providers: CheckinProvider[]
  checkinLoading: boolean
}>()

const emit = defineEmits<{
  (e: 'navigate', accountId: string): void
  (e: 'checkin', accountId: string): void
  (e: 'refresh-balance', accountId: string): void
  (e: 'edit', account: AccountInfo): void
  (e: 'delete', accountId: string): void
}>()

const { t, locale } = useI18n()

const actionsMenuRef = ref<InstanceType<typeof AccountActionsMenu> | null>(null)

const getProviderName = (providerId: string) => {
  return props.providers.find((p) => p.id === providerId)?.name || providerId
}

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleString(locale.value)
}
</script>

<style scoped>
.checkin-accounts-tab__table-shell {
  overflow: hidden;
}

.checkin-accounts-tab__table {
  width: 100%;
  min-width: 100%;
  border-collapse: separate;
  border-spacing: 0;
}

.checkin-accounts-tab__table-head {
  position: sticky;
  top: 0;
  background: var(--color-bg-surface);
}

.checkin-accounts-tab__table-body {
  background: transparent;
}

.checkin-accounts-tab__table-head tr,
.checkin-accounts-tab__table-body tr + tr td {
  border-top: 1px solid var(--color-border-subtle);
}

.checkin-accounts-tab__th,
.checkin-accounts-tab__cell {
  padding: 0.75rem 1rem;
}

.checkin-accounts-tab__th {
  text-align: left;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.checkin-accounts-tab__th--right,
.checkin-accounts-tab__cell--right {
  text-align: right;
}

.checkin-accounts-tab__th--actions {
  width: 9rem;
  text-align: center;
}

.checkin-accounts-tab__row {
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.checkin-accounts-tab__row:hover {
  background: rgb(var(--color-accent-primary-rgb) / 6%);
}

.checkin-accounts-tab__account {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.checkin-accounts-tab__account-row,
.checkin-accounts-tab__row-actions {
  display: flex;
  align-items: center;
}

.checkin-accounts-tab__account-row {
  gap: 0.5rem;
}

.checkin-accounts-tab__status-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 9999px;
  flex-shrink: 0;
}

.checkin-accounts-tab__account-name {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-primary);
}

.checkin-accounts-tab__provider-chip {
  width: fit-content;
  border-radius: var(--radius-md);
  background: var(--color-bg-overlay);
  padding: 0.125rem 0.5rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.checkin-accounts-tab__metric,
.checkin-accounts-tab__cell--mono {
  font-family: var(--font-mono);
}

.checkin-accounts-tab__metric {
  font-size: 0.875rem;
  font-weight: 600;
}

.checkin-accounts-tab__metric--balance {
  color: var(--accent-success);
}

.checkin-accounts-tab__metric--quota {
  color: var(--accent-primary);
}

.checkin-accounts-tab__metric--consumed {
  color: var(--accent-warning);
}

.checkin-accounts-tab__placeholder,
.checkin-accounts-tab__cell--mono {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.checkin-accounts-tab__row-actions {
  justify-content: center;
  gap: 0.5rem;
}

.checkin-accounts-tab__mini-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.35rem;
  min-width: 5.75rem;
  min-height: 36px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  padding: 0.375rem 0.7rem;
  font-size: 0.75rem;
  font-weight: 700;
  line-height: 1rem;
  color: var(--text-primary);
  white-space: nowrap;
  background: var(--color-bg-surface);
  box-shadow: var(--shadow-sm);
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    color 0.2s ease,
    opacity 0.2s ease,
    transform 0.2s ease;
}

.checkin-accounts-tab__mini-button:hover:not(:disabled) {
  border-color: var(--color-border-accent);
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.checkin-accounts-tab__mini-button:disabled {
  cursor: not-allowed;
  opacity: 0.62;
  filter: grayscale(0.35);
  border-color: rgb(var(--color-border-default-rgb) / 44%);
  background: var(--color-bg-elevated);
  color: var(--text-muted);
  box-shadow: none;
}

.checkin-accounts-tab__mini-button-icon {
  flex-shrink: 0;
}

.checkin-accounts-tab__mini-button-label {
  white-space: nowrap;
}

.checkin-accounts-tab__menu-wrap {
  position: relative;
  z-index: 1;
}

.checkin-accounts-tab__menu-trigger {
  min-width: 40px;
  min-height: 40px;
  border-radius: var(--radius-lg);
  border: 1px solid transparent;
  color: var(--text-muted);
  background: rgb(var(--color-bg-elevated-rgb) / 20%);
  transition:
    color 0.2s ease,
    background-color 0.2s ease,
    border-color 0.2s ease,
    box-shadow 0.2s ease;
}

.checkin-accounts-tab__menu-trigger:hover,
.checkin-accounts-tab__menu-trigger--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 30%);
  background: rgb(var(--color-bg-elevated-rgb) / 52%);
  color: var(--text-primary);
  box-shadow: var(--shadow-md);
}

@media (width <= 900px) {
  .checkin-accounts-tab__table-shell {
    overflow-x: auto;
  }
}
</style>
