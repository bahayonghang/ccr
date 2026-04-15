<template>
  <div class="claude-auth-view">
    <div class="claude-auth-view__shell">
      <ModuleSubnav module="claude-code" />

      <header class="claude-auth-view__header">
        <div>
          <p class="claude-auth-view__eyebrow">
            Claude Official Subscription
          </p>
          <h1 class="claude-auth-view__title">
            官方账号管理
          </h1>
          <p class="claude-auth-view__subtitle">
            保存、切换、删除 Claude Code 官方订阅账号快照；切换只会改写
            <code>~/.claude/.credentials.json</code>。
          </p>
        </div>

        <div class="claude-auth-view__actions">
          <RouterLink
            to="/claude-code"
            class="claude-auth-view__ghost-button"
          >
            返回 Claude Code
          </RouterLink>
          <button
            type="button"
            class="claude-auth-view__ghost-button"
            :disabled="loading"
            @click="refreshAll"
          >
            刷新
          </button>
          <button
            type="button"
            class="claude-auth-view__primary-button"
            :disabled="saving"
            @click="showSaveForm = true"
          >
            保存当前登录
          </button>
        </div>
      </header>

      <div
        v-if="authActionError"
        class="claude-auth-view__banner claude-auth-view__banner--error"
      >
        {{ authActionError }}
      </div>

      <section class="claude-auth-view__stats">
        <article class="claude-auth-view__stat-card">
          <p class="claude-auth-view__stat-label">
            登录状态
          </p>
          <p class="claude-auth-view__stat-value">
            {{ loginStateLabel }}
          </p>
        </article>
        <article class="claude-auth-view__stat-card">
          <p class="claude-auth-view__stat-label">
            运行时模式
          </p>
          <p class="claude-auth-view__stat-value">
            {{ runtimeModeLabel }}
          </p>
        </article>
        <article class="claude-auth-view__stat-card">
          <p class="claude-auth-view__stat-label">
            当前 Profile
          </p>
          <p class="claude-auth-view__stat-value">
            {{ currentProfileLabel }}
          </p>
        </article>
        <article class="claude-auth-view__stat-card">
          <p class="claude-auth-view__stat-label">
            已保存账号
          </p>
          <p class="claude-auth-view__stat-value">
            {{ accounts.length }}
          </p>
        </article>
      </section>

      <section
        v-if="currentInfo"
        class="claude-auth-view__panel"
      >
        <div class="claude-auth-view__panel-header">
          <h2 class="claude-auth-view__panel-title">
            当前运行时官方登录
          </h2>
          <span
            class="claude-auth-view__freshness"
            :class="freshnessClass(currentInfo.freshness)"
          >
            {{ currentInfo.freshness_icon }} {{ currentInfo.freshness_description }}
          </span>
        </div>

        <div class="claude-auth-view__detail-grid">
          <div>
            <p class="claude-auth-view__detail-label">
              邮箱
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.email || '-' }}
            </p>
          </div>
          <div>
            <p class="claude-auth-view__detail-label">
              账号 UUID
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.account_uuid || '-' }}
            </p>
          </div>
          <div>
            <p class="claude-auth-view__detail-label">
              订阅类型
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.subscription_type || '-' }}
            </p>
          </div>
          <div>
            <p class="claude-auth-view__detail-label">
              计费类型
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.billing_type || '-' }}
            </p>
          </div>
          <div>
            <p class="claude-auth-view__detail-label">
              速率档位
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.rate_limit_tier || '-' }}
            </p>
          </div>
          <div>
            <p class="claude-auth-view__detail-label">
              Access Token 到期
            </p>
            <p
              class="claude-auth-view__detail-value"
              :class="{ 'text-red-400': currentInfo.is_expired }"
            >
              {{ currentInfo.expires_at ? formatDate(currentInfo.expires_at) : '-' }}
            </p>
          </div>
        </div>
      </section>

      <section class="claude-auth-view__panel">
        <div class="claude-auth-view__panel-header">
          <div>
            <h2 class="claude-auth-view__panel-title">
              已保存账号快照
            </h2>
            <p class="claude-auth-view__panel-subtitle">
              每个快照都保存当前 `claudeAiOauth`，切换时不会改写
              <code>~/.claude.json</code>。
            </p>
          </div>
        </div>

        <div
          v-if="loading"
          class="claude-auth-view__empty"
        >
          正在加载账号信息…
        </div>

        <div
          v-else-if="accounts.length === 0"
          class="claude-auth-view__empty"
        >
          尚未保存任何官方账号快照。
        </div>

        <div
          v-else
          class="claude-auth-view__table-wrap"
        >
          <table class="claude-auth-view__table">
            <thead>
              <tr>
                <th>名称</th>
                <th>邮箱</th>
                <th>订阅</th>
                <th>到期</th>
                <th>状态</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="account in accounts"
                :key="account.name"
              >
                <td>
                  <div class="claude-auth-view__account-name">
                    <span>{{ account.name }}</span>
                    <span
                      v-if="account.is_current"
                      class="claude-auth-view__pill"
                    >
                      当前
                    </span>
                  </div>
                  <p
                    v-if="account.description"
                    class="claude-auth-view__muted"
                  >
                    {{ account.description }}
                  </p>
                </td>
                <td>{{ account.email || '-' }}</td>
                <td>{{ account.subscription_type || '-' }}</td>
                <td :class="{ 'text-red-400': account.is_expired }">
                  {{ account.expires_at ? formatDate(account.expires_at) : '-' }}
                </td>
                <td>
                  <span
                    class="claude-auth-view__freshness"
                    :class="freshnessClass(account.freshness)"
                  >
                    {{ account.freshness_icon }} {{ account.freshness_description }}
                  </span>
                </td>
                <td>
                  <div class="claude-auth-view__row-actions">
                    <button
                      type="button"
                      class="claude-auth-view__table-button"
                      :disabled="busyName === account.name"
                      @click="handleSwitch(account.name)"
                    >
                      切换
                    </button>
                    <button
                      type="button"
                      class="claude-auth-view__table-button claude-auth-view__table-button--danger"
                      :disabled="busyName === account.name"
                      @click="handleDelete(account.name)"
                    >
                      删除
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <div
        v-if="showSaveForm"
        class="claude-auth-view__modal-backdrop"
        @click.self="showSaveForm = false"
      >
        <div class="claude-auth-view__modal">
          <h3 class="claude-auth-view__modal-title">
            保存当前官方登录
          </h3>
          <p class="claude-auth-view__modal-subtitle">
            当前必须已经通过 `claude login` 拿到官方登录，CCR 只负责保存快照和切换。
          </p>

          <label class="claude-auth-view__field">
            <span>账号名称</span>
            <input
              v-model="saveForm.name"
              type="text"
              placeholder="例如 work / personal"
            >
          </label>

          <label class="claude-auth-view__field">
            <span>描述（可选）</span>
            <input
              v-model="saveForm.description"
              type="text"
              placeholder="例如 公司订阅 / 个人订阅"
            >
          </label>

          <label class="claude-auth-view__checkbox">
            <input
              v-model="saveForm.force"
              type="checkbox"
            >
            <span>覆盖同名账号</span>
          </label>

          <div class="claude-auth-view__modal-actions">
            <button
              type="button"
              class="claude-auth-view__ghost-button"
              @click="showSaveForm = false"
            >
              取消
            </button>
            <button
              type="button"
              class="claude-auth-view__primary-button"
              :disabled="saving"
              @click="handleSave"
            >
              {{ saving ? '保存中…' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onActivated, onMounted, reactive, ref } from 'vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import {
  deleteClaudeAuth,
  getClaudeAuthCurrent,
  listClaudeAuthAccounts,
  saveClaudeAuth,
  switchClaudeAuth,
} from '@/api'
import type {
  ClaudeAuthAccountItem,
  ClaudeAuthCurrentInfo,
  ClaudeAuthCurrentResponse,
  ClaudeAuthListResponse,
  ClaudeAuthSaveRequest,
  ClaudeLoginState,
  ClaudeRuntimeSummary,
  ClaudeTokenFreshness,
} from '@/types'
import { logger } from '@/utils/logger'
import { useUIStore } from '@/stores/ui'

defineOptions({ name: 'ClaudeAuthView' })

const uiStore = useUIStore()

const loading = ref(false)
const saving = ref(false)
const busyName = ref<string | null>(null)
const showSaveForm = ref(false)
const authActionError = ref<string | null>(null)

const accounts = ref<ClaudeAuthAccountItem[]>([])
const currentInfo = ref<ClaudeAuthCurrentInfo | null>(null)
const runtimeSummary = ref<ClaudeRuntimeSummary | null>(null)
const loginState = ref<ClaudeLoginState>({ type: 'NotLoggedIn' })

const saveForm = reactive({
  name: '',
  description: '',
  force: false,
})

const extractErrorMessage = (error: unknown): string => {
  if (error instanceof Error && error.message) return error.message
  if (typeof error === 'string') return error
  return '请求失败'
}

const loginStateLabel = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved':
      return `已登录（已保存为 ${loginState.value.account_name}）`
    case 'LoggedInUnsaved':
      return '已登录（未保存）'
    case 'ApiKeyActive':
      return '当前由 API key profile 控制'
    case 'NotLoggedIn':
    default:
      return '未登录'
  }
})

const runtimeModeLabel = computed(() => {
  switch (runtimeSummary.value?.mode) {
    case 'profile_with_auth':
      return 'Profile + 官方订阅'
    case 'profile_pending_auth':
      return 'Profile 等待官方订阅'
    case 'profile_only':
      return 'Profile 驱动（API key）'
    case 'runtime_only':
      return '仅官方订阅运行时'
    case 'unresolved':
    default:
      return '未解析'
  }
})

const currentProfileLabel = computed(() => {
  const summary = runtimeSummary.value
  if (!summary?.current_profile_name) {
    return '未绑定'
  }

  const authMode = summary.current_profile_auth_mode
  return authMode
    ? `${summary.current_profile_name} · ${authMode}`
    : summary.current_profile_name
})

const freshnessClass = (freshness: ClaudeTokenFreshness) => {
  switch (freshness) {
    case 'Fresh':
      return 'claude-auth-view__freshness--fresh'
    case 'Stale':
      return 'claude-auth-view__freshness--stale'
    case 'Old':
      return 'claude-auth-view__freshness--old'
    default:
      return ''
  }
}

const formatDate = (date: string) => {
  try {
    return new Date(date).toLocaleString()
  } catch {
    return date
  }
}

const refreshAll = async () => {
  try {
    loading.value = true
    authActionError.value = null

    const [accountsData, currentData] = await Promise.all([
      listClaudeAuthAccounts<ClaudeAuthListResponse>(),
      getClaudeAuthCurrent<ClaudeAuthCurrentResponse>(),
    ])

    accounts.value = accountsData.accounts || []
    runtimeSummary.value = accountsData.runtime_summary || currentData.runtime_summary
    loginState.value = accountsData.login_state || currentData.login_state
    currentInfo.value = currentData.info || null
  } catch (error) {
    logger.error('Failed to load Claude auth data:', error)
    authActionError.value = extractErrorMessage(error)
    uiStore.showError(authActionError.value)
  } finally {
    loading.value = false
  }
}

const handleSave = async () => {
  if (!saveForm.name.trim()) {
    uiStore.showError('账号名称不能为空')
    return
  }

  try {
    saving.value = true
    authActionError.value = null

    const payload: ClaudeAuthSaveRequest = {
      name: saveForm.name.trim(),
      description: saveForm.description.trim() || null,
      force: saveForm.force,
    }

    await saveClaudeAuth(payload)
    uiStore.showSuccess('Claude 官方账号已保存')
    showSaveForm.value = false
    saveForm.name = ''
    saveForm.description = ''
    saveForm.force = false
    await refreshAll()
  } catch (error) {
    logger.error('Failed to save Claude auth:', error)
    authActionError.value = extractErrorMessage(error)
    uiStore.showError(authActionError.value)
  } finally {
    saving.value = false
  }
}

const handleSwitch = async (name: string) => {
  if (!window.confirm(`确定切换到官方账号 "${name}" 吗？`)) return

  try {
    busyName.value = name
    authActionError.value = null
    await switchClaudeAuth(name)
    uiStore.showSuccess(`已切换到 ${name}`)
    await refreshAll()
  } catch (error) {
    logger.error('Failed to switch Claude auth:', error)
    authActionError.value = extractErrorMessage(error)
    uiStore.showError(authActionError.value)
  } finally {
    busyName.value = null
  }
}

const handleDelete = async (name: string) => {
  if (!window.confirm(`确定删除官方账号 "${name}" 吗？`)) return

  try {
    busyName.value = name
    authActionError.value = null
    await deleteClaudeAuth(name)
    uiStore.showSuccess(`已删除 ${name}`)
    await refreshAll()
  } catch (error) {
    logger.error('Failed to delete Claude auth:', error)
    authActionError.value = extractErrorMessage(error)
    uiStore.showError(authActionError.value)
  } finally {
    busyName.value = null
  }
}

onMounted(async () => {
  await refreshAll()
})

onActivated(() => {
  void refreshAll()
})
</script>

<style scoped>
.claude-auth-view {
  min-height: 100%;
  padding: 1.5rem;
}

.claude-auth-view__shell {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  max-width: 1440px;
  margin: 0 auto;
}

.claude-auth-view__header,
.claude-auth-view__actions,
.claude-auth-view__panel-header,
.claude-auth-view__row-actions,
.claude-auth-view__account-name,
.claude-auth-view__modal-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.claude-auth-view__header,
.claude-auth-view__panel-header {
  justify-content: space-between;
}

.claude-auth-view__eyebrow {
  color: var(--stage-text-quiet);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.claude-auth-view__title {
  color: var(--stage-text-primary);
  font-size: 1.875rem;
  line-height: 2.25rem;
  font-weight: 700;
}

.claude-auth-view__subtitle,
.claude-auth-view__panel-subtitle,
.claude-auth-view__muted,
.claude-auth-view__detail-label {
  color: var(--stage-text-secondary);
  font-size: 0.875rem;
  line-height: 1.4;
}

.claude-auth-view__ghost-button,
.claude-auth-view__primary-button,
.claude-auth-view__table-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 9999px;
  padding: 0.75rem 1rem;
  font-size: 0.875rem;
  line-height: 1;
  font-weight: 600;
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.claude-auth-view__ghost-button,
.claude-auth-view__table-button {
  border: 1px solid var(--stage-border-soft);
  background: var(--stage-surface-soft);
  color: var(--stage-text-primary);
}

.claude-auth-view__primary-button {
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 25%);
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: var(--color-accent-primary);
}

.claude-auth-view__table-button--danger {
  color: var(--color-danger);
}

.claude-auth-view__ghost-button:disabled,
.claude-auth-view__primary-button:disabled,
.claude-auth-view__table-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.claude-auth-view__stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 1rem;
}

.claude-auth-view__stat-card,
.claude-auth-view__panel,
.claude-auth-view__banner,
.claude-auth-view__modal {
  border: 1px solid var(--stage-border-soft);
  border-radius: 1rem;
  background: var(--stage-surface-elevated);
}

.claude-auth-view__stat-card,
.claude-auth-view__panel,
.claude-auth-view__banner {
  padding: 1rem 1.125rem;
}

.claude-auth-view__stat-label {
  color: var(--stage-text-quiet);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.claude-auth-view__stat-value,
.claude-auth-view__panel-title,
.claude-auth-view__modal-title {
  color: var(--stage-text-primary);
  font-size: 1.125rem;
  line-height: 1.5rem;
  font-weight: 700;
  margin-top: 0.35rem;
}

.claude-auth-view__banner--error {
  border-color: rgb(var(--color-danger-rgb) / 25%);
  background: rgb(var(--color-danger-rgb) / 10%);
  color: var(--color-danger);
}

.claude-auth-view__detail-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1rem;
  margin-top: 1rem;
}

.claude-auth-view__detail-value {
  color: var(--stage-text-primary);
  font-size: 0.95rem;
  line-height: 1.4;
  margin-top: 0.25rem;
  overflow-wrap: anywhere;
}

.claude-auth-view__freshness {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.8rem;
  line-height: 1rem;
  font-weight: 600;
}

.claude-auth-view__freshness--fresh {
  color: rgb(16 185 129);
}

.claude-auth-view__freshness--stale {
  color: rgb(245 158 11);
}

.claude-auth-view__freshness--old {
  color: rgb(239 68 68);
}

.claude-auth-view__table-wrap {
  overflow-x: auto;
  margin-top: 1rem;
}

.claude-auth-view__table {
  width: 100%;
  border-collapse: collapse;
}

.claude-auth-view__table th,
.claude-auth-view__table td {
  border-bottom: 1px solid var(--stage-border-soft);
  padding: 0.875rem 0.75rem;
  text-align: left;
  vertical-align: top;
}

.claude-auth-view__table th {
  color: var(--stage-text-quiet);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.claude-auth-view__table td {
  color: var(--stage-text-primary);
  font-size: 0.9rem;
  line-height: 1.4;
}

.claude-auth-view__pill {
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 25%);
  border-radius: 9999px;
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  padding: 0.1rem 0.5rem;
  color: var(--color-accent-primary);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
}

.claude-auth-view__empty {
  padding: 2rem 1rem;
  color: var(--stage-text-secondary);
  text-align: center;
}

.claude-auth-view__modal-backdrop {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(15 23 42 / 55%);
  padding: 1rem;
  z-index: 50;
}

.claude-auth-view__modal {
  width: min(100%, 480px);
  padding: 1.25rem;
  box-shadow: 0 24px 48px rgb(15 23 42 / 25%);
}

.claude-auth-view__modal-subtitle {
  margin-top: 0.5rem;
  color: var(--stage-text-secondary);
  font-size: 0.875rem;
  line-height: 1.5;
}

.claude-auth-view__field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-top: 1rem;
  color: var(--stage-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
}

.claude-auth-view__field input {
  border: 1px solid var(--stage-border-soft);
  border-radius: 0.75rem;
  background: var(--stage-surface-soft);
  color: var(--stage-text-primary);
  padding: 0.8rem 0.9rem;
}

.claude-auth-view__checkbox {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 1rem;
  color: var(--stage-text-primary);
  font-size: 0.875rem;
}

.claude-auth-view__modal-actions {
  justify-content: flex-end;
  margin-top: 1.25rem;
}

@media (width <= 1024px) {
  .claude-auth-view__stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .claude-auth-view__detail-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width <= 768px) {
  .claude-auth-view__header,
  .claude-auth-view__actions,
  .claude-auth-view__panel-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .claude-auth-view__stats,
  .claude-auth-view__detail-grid {
    grid-template-columns: 1fr;
  }

  .claude-auth-view__row-actions,
  .claude-auth-view__modal-actions {
    flex-wrap: wrap;
  }
}
</style>
