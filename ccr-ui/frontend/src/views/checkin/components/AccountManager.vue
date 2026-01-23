<template>
  <div class="account-manager">
    <div class="section-header">
      <h2>{{ t('checkin.account_manager.title') }}</h2>
      <button
        class="add-button"
        @click="goToCheckinManage"
      >
        {{ t('checkin.account_manager.go_manage') }}
      </button>
    </div>

    <div
      v-if="loading"
      class="loading"
    >
      {{ t('common.loading') }}
    </div>
    <div
      v-else-if="error"
      class="error"
    >
      {{ error }}
    </div>
    <div
      v-else
      class="account-list"
    >
      <div
        v-for="account in accounts"
        :key="account.id"
        class="account-item"
      >
        <div class="account-info">
          <button
            class="account-name account-link"
            @click="openAccountDashboard(account.id)"
          >
            {{ account.name }}
          </button>
          <div class="account-provider">
            {{ account.provider_name || account.provider_id }}
          </div>
        </div>
        <div class="account-actions">
          <button
            class="action-button view"
            @click="openAccountDashboard(account.id)"
          >
            {{ t('checkin.account_manager.dashboard') }}
          </button>
          <button
            class="action-button delete"
            @click="deleteAccount(account.id)"
          >
            {{ t('checkin.account_list.delete') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { listCheckinAccounts, deleteCheckinAccount } from '@/api/client'
import type { AccountInfo } from '@/types/checkin'

const accounts = ref<AccountInfo[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const router = useRouter()
const { t } = useI18n()

const fetchAccounts = async () => {
  try {
    const response = await listCheckinAccounts()
    accounts.value = response.accounts
  } catch (err) {
    error.value = t('checkin.account_manager.load_error')
  } finally {
    loading.value = false
  }
}

const openAccountDashboard = (accountId: string) => {
  router.push({ name: 'checkin-account-dashboard', params: { accountId } })
}

const deleteAccount = async (id: string) => {
  if (!confirm(t('checkin.account_manager.delete_confirm'))) return
  try {
    await deleteCheckinAccount(id)
    accounts.value = accounts.value.filter(account => account.id !== id)
  } catch (err) {
    alert(t('checkin.account_manager.delete_fail'))
  }
}

const goToCheckinManage = () => {
  router.push({ name: 'checkin' })
}

onMounted(() => {
  fetchAccounts()
})
</script>

<style scoped>
.account-manager {
  width: 100%;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.section-header h2 {
  margin: 0;
  font-size: 1.25rem;
  color: var(--text-primary);
}

.add-button {
  padding: 0.5rem 1rem;
  background: var(--gradient-secondary);
  color: white;
  border: none;
  border-radius: 0.5rem;
  cursor: pointer;
  font-weight: 500;
  transition: opacity 0.3s ease;
}

.add-button:hover {
  opacity: 0.9;
}

.account-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.account-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background: var(--bg-secondary);
  border-radius: 0.5rem;
  border: 1px solid var(--glass-border-medium);
}

.account-name {
  font-weight: 600;
  color: var(--text-primary);
}

.account-link {
  background: transparent;
  border: none;
  padding: 0;
  cursor: pointer;
  text-align: left;
  color: var(--accent-info);
}

.account-link:hover {
  text-decoration: underline;
}

.account-provider {
  font-size: 0.875rem;
  color: var(--text-muted);
  margin-top: 0.25rem;
}

.account-actions {
  display: flex;
  gap: 0.5rem;
}

.action-button {
  padding: 0.375rem 0.75rem;
  border: none;
  border-radius: 0.375rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
}

.action-button.view {
  background: rgb(var(--color-info-rgb), 0.15);
  color: var(--accent-info);
}

.action-button.delete {
  background: rgb(var(--color-danger-rgb), 0.15);
  color: var(--accent-danger);
}

.loading,
.error {
  text-align: center;
  padding: 2rem;
}

.error {
  color: var(--accent-danger);
}

:global(.dark) .account-item {
  background: rgb(var(--color-slate-rgb), 0.4);
  border-color: rgb(var(--color-slate-rgb), 0.6);
}
</style>
