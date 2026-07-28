<template>
  <div class="checkin-accounts-tab">
    <div class="checkin-accounts-tab__panel checkin-surface-card">
      <div class="checkin-accounts-tab__toolbar">
        <h2 class="checkin-accounts-tab__title">
          {{ t('checkin.accounts.title') }}
        </h2>
        <!-- 搜索和过滤区域 -->
        <div class="checkin-accounts-tab__filters">
          <!-- 搜索框 -->
          <div class="checkin-accounts-tab__search">
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="t('checkin.accounts.searchPlaceholder')"
              class="checkin-accounts-tab__input checkin-accounts-tab__input--search"
            >
            <svg
              class="checkin-accounts-tab__search-icon"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
          </div>
          <!-- 提供商过滤 -->
          <select
            v-model="providerFilter"
            class="checkin-accounts-tab__input checkin-accounts-tab__select"
          >
            <option value="all">
              {{ t('checkin.accounts.allProviders') }}
            </option>
            <option
              v-for="p in providers"
              :key="p.id"
              :value="p.id"
            >
              {{ p.name }}
            </option>
          </select>
        </div>
        <div class="checkin-accounts-tab__actions">
          <button
            :disabled="providers.length === 0"
            class="checkin-accounts-tab__action-button checkin-accounts-tab__action-button--primary"
            @click="accountFormModalRef?.open()"
          >
            <svg
              class="w-5 h-5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 4v16m8-8H4"
              />
            </svg>
            <span>{{ t('checkin.accounts.addAccount') }}</span>
          </button>
          <button
            :disabled="builtinProviders.filter((p) => p.oauth_config).length === 0"
            class="checkin-accounts-tab__action-button checkin-accounts-tab__action-button--secondary"
            :title="
              builtinProviders.filter((p) => p.oauth_config).length === 0
                ? t('checkin.actions.oauthLoginUnavailable')
                : t('checkin.actions.oauthLoginTitle')
            "
            @click="emit('show-oauth-wizard')"
          >
            <SIcon
              name="Shield"
              size="w-5 h-5"
              class="checkin-accounts-tab__action-button-icon"
            />
            <span>{{ t('checkin.actions.oauthLogin') }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- 账号列表 -->
    <div
      v-if="accounts.length === 0"
      class="checkin-accounts-tab__empty checkin-surface-card"
    >
      {{ providers.length === 0 ? t('checkin.accounts.emptyNoProviders') : t('checkin.accounts.emptyNoAccounts') }}
    </div>
    <AccountsTable
      v-else
      :accounts="filteredAccounts"
      :providers="providers"
      :checkin-loading="checkinLoading"
      @navigate="emit('navigate', $event)"
      @checkin="emit('checkin', $event)"
      @refresh-balance="emit('refresh-balance', $event)"
      @edit="openAccountEditor"
      @delete="deleteAccount"
    />
  </div>

  <!-- 账号编辑弹窗 -->
  <AccountFormModal
    ref="accountFormModalRef"
    :providers="providers"
    :builtin-providers="builtinProviders"
    @refresh="emit('refresh')"
  />
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { deleteCheckinAccount as apiDeleteAccount } from '@/api'
import type { CheckinProvider, AccountInfo, BuiltinProvider } from '@/types/checkin'
import { useUIStore } from '@/stores/ui'
import { getErrorMessage } from '@/types/api'
import AccountsTable from '../components/AccountsTable.vue'
import AccountFormModal from '../components/AccountFormModal.vue'

const props = defineProps<{
  providers: CheckinProvider[]
  accounts: AccountInfo[]
  builtinProviders: BuiltinProvider[]
  checkinLoading: boolean
  /** 待打开编辑弹窗的账号 ID（cookie_expired 快捷修复入口），消费后 emit pending-edit-consumed */
  pendingEditAccountId?: string | null
}>()

const emit = defineEmits<{
  (e: 'refresh'): void
  (e: 'checkin', accountId: string): void
  (e: 'refresh-balance', accountId: string): void
  (e: 'navigate', accountId: string): void
  (e: 'show-oauth-wizard'): void
  (e: 'pending-edit-consumed'): void
}>()

const uiStore = useUIStore()
const { t } = useI18n()

const accountFormModalRef = ref<InstanceType<typeof AccountFormModal> | null>(null)
const searchQuery = ref('')
const providerFilter = ref<string>('all')

// 过滤后的账号列表
const filteredAccounts = computed(() => {
  let result = props.accounts

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(
      (account) =>
        account.name.toLowerCase().includes(query) ||
        (account.provider_name && account.provider_name.toLowerCase().includes(query))
    )
  }

  if (providerFilter.value !== 'all') {
    result = result.filter((account) => account.provider_id === providerFilter.value)
  }

  return result
})

const openAccountEditor = (account: AccountInfo) => {
  void accountFormModalRef.value?.open(account)
}

const deleteAccount = async (id: string) => {
  const confirmed = await uiStore.requestConfirm({
    title: t('checkin.accounts.deleteAccount'),
    message: t('checkin.accounts.deleteConfirm'),
    confirmText: t('checkin.accounts.delete'),
    cancelText: t('common.cancel'),
    type: 'danger',
    surface: 'solid',
  })
  if (!confirmed) return
  try {
    await apiDeleteAccount(id)
    emit('refresh')
  } catch (e: unknown) {
    uiStore.showError(t('checkin.accounts.errors.deleteFailed', { error: getErrorMessage(e, t('checkin.errors.unknown')) }))
  }
}

// cookie_expired 快捷修复：按账号 ID 直达编辑弹窗并聚焦 cookies 输入
const consumePendingEditAccount = async (accountId: string | null | undefined) => {
  if (!accountId) return
  const account = props.accounts.find((a) => a.id === accountId)
  emit('pending-edit-consumed')
  if (!account) return
  await accountFormModalRef.value?.open(account, { focusSession: true })
}

watch(
  () => props.pendingEditAccountId,
  (accountId) => {
    void consumePendingEditAccount(accountId)
  }
)

onMounted(() => {
  void consumePendingEditAccount(props.pendingEditAccountId)
})
</script>

<style scoped>
.checkin-accounts-tab {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.checkin-accounts-tab__panel {
  padding: 1rem;
}

.checkin-accounts-tab__toolbar,
.checkin-accounts-tab__filters,
.checkin-accounts-tab__actions,
.checkin-accounts-tab__action-button {
  display: flex;
  align-items: center;
}

.checkin-accounts-tab__toolbar {
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 1rem;
}

.checkin-accounts-tab__title {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
}

.checkin-accounts-tab__filters {
  flex: 1 1 16rem;
  justify-content: flex-end;
  gap: 0.75rem;
}

.checkin-accounts-tab__search {
  position: relative;
}

.checkin-accounts-tab__input {
  border-radius: 0.75rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 72%);
  color: var(--text-primary);
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    background-color 0.2s ease,
    transform 0.2s ease;
  background: rgb(var(--color-bg-elevated-rgb) / 36%);
  padding: 0.625rem 0.75rem;
  font-size: 0.875rem;
}

.checkin-accounts-tab__input::placeholder {
  color: var(--text-muted);
}

.checkin-accounts-tab__input:focus {
  outline: none;
  border-color: rgb(var(--color-accent-primary-rgb) / 88%);
  box-shadow:
    0 0 0 3px rgb(var(--color-accent-primary-rgb) / 18%),
    0 14px 28px rgb(var(--color-accent-primary-rgb) / 12%);
}

.checkin-accounts-tab__input--search {
  width: 12rem;
  padding-left: 2.25rem;
}

.checkin-accounts-tab__search-icon {
  position: absolute;
  top: 50%;
  left: 0.75rem;
  width: 1rem;
  height: 1rem;
  color: var(--text-muted);
  transform: translateY(-50%);
}

.checkin-accounts-tab__select {
  padding-inline: 0.75rem;
}

.checkin-accounts-tab__select option {
  background: rgb(var(--color-bg-elevated-rgb) / 100%);
  color: var(--text-primary);
}

.checkin-accounts-tab__select option:disabled {
  color: var(--text-muted);
}

.checkin-accounts-tab__actions {
  flex-wrap: wrap;
  gap: 0.75rem;
}

.checkin-accounts-tab__action-button {
  border-radius: 0.75rem;
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    color 0.2s ease,
    opacity 0.2s ease,
    transform 0.2s ease;
  min-height: 44px;
  gap: 0.5rem;
  border: 1px solid transparent;
  padding: 0.5rem 1rem;
  color: var(--color-accent-primary-contrast);
  font-weight: 700;
  white-space: nowrap;
}

.checkin-accounts-tab__action-button:disabled {
  cursor: not-allowed;
  opacity: 0.62;
  filter: grayscale(0.35);
}

.checkin-accounts-tab__action-button--primary {
  background: var(--accent-primary);
}

.checkin-accounts-tab__action-button--primary:hover:not(:disabled) {
  background: rgb(var(--color-accent-primary-rgb) / 88%);
}

.checkin-accounts-tab__action-button--secondary {
  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
  color: var(--text-primary);
  box-shadow: var(--shadow-sm);
}

.checkin-accounts-tab__action-button--secondary:hover:not(:disabled) {
  border-color: var(--color-border-accent);
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.checkin-accounts-tab__action-button--secondary:disabled {
  border-color: rgb(var(--color-border-default-rgb) / 54%);
  background: var(--color-bg-elevated);
  color: var(--text-muted);
  box-shadow: none;
}

.checkin-accounts-tab__action-button-icon {
  filter: none;
}

.checkin-accounts-tab__empty {
  padding: 3rem 1rem;
  text-align: center;
  color: var(--text-muted);
}

@media (width <= 900px) {
  .checkin-accounts-tab__toolbar,
  .checkin-accounts-tab__filters {
    align-items: stretch;
  }

  .checkin-accounts-tab__filters {
    justify-content: stretch;
  }

  .checkin-accounts-tab__search,
  .checkin-accounts-tab__input--search {
    width: 100%;
  }
}
</style>
