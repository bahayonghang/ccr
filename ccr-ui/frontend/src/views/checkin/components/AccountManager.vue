<template>
  <div class="account-manager">
    <div class="section-header">
      <h2>账号管理</h2>
      <button
        class="add-button"
        @click="goToCheckinManage"
      >
        前往管理
      </button>
    </div>

    <div
      v-if="loading"
      class="loading"
    >
      加载中...
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
            Dashboard
          </button>
          <button
            class="action-button delete"
            @click="deleteAccount(account.id)"
          >
            删除
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { listCheckinAccounts, deleteCheckinAccount } from '@/api/client'
import type { AccountInfo } from '@/types/checkin'

const accounts = ref<AccountInfo[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const router = useRouter()

const fetchAccounts = async () => {
  try {
    const response = await listCheckinAccounts()
    accounts.value = response.accounts
  } catch (err) {
    error.value = '加载账号列表失败'
  } finally {
    loading.value = false
  }
}

const openAccountDashboard = (accountId: string) => {
  router.push({ name: 'checkin-account-dashboard', params: { accountId } })
}

const deleteAccount = async (id: string) => {
  if (!confirm('确定要删除此账号吗？')) return
  try {
    await deleteCheckinAccount(id)
    accounts.value = accounts.value.filter(account => account.id !== id)
  } catch (err) {
    alert('删除失败')
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
  color: #1f2937;
}

.add-button {
  padding: 0.5rem 1rem;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
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
  background: #f9fafb;
  border-radius: 0.5rem;
  border: 1px solid #e5e7eb;
}

.account-name {
  font-weight: 600;
  color: #1f2937;
}

.account-link {
  background: transparent;
  border: none;
  padding: 0;
  cursor: pointer;
  text-align: left;
  color: #2563eb;
}

.account-link:hover {
  text-decoration: underline;
}
.account-provider {
  font-size: 0.875rem;
  color: #6b7280;
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
  background: #e0f2fe;
  color: #0284c7;
}

.action-button.delete {
  background: #fee2e2;
  color: #dc2626;
}

.loading,
.error {
  text-align: center;
  padding: 2rem;
}

.error {
  color: #dc2626;
}
</style>
