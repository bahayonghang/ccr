<template>
  <div class="checkin-accounts-tab">
    <div class="checkin-accounts-tab__panel">
      <div class="checkin-accounts-tab__toolbar">
        <h2 class="checkin-accounts-tab__title">
          签到账号
        </h2>
        <!-- 搜索和过滤区域 -->
        <div class="checkin-accounts-tab__filters">
          <!-- 搜索框 -->
          <div class="checkin-accounts-tab__search">
            <input
              v-model="searchQuery"
              type="text"
              placeholder="搜索账号..."
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
              全部提供商
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
            @click="openAccountModal()"
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
            <span>添加账号</span>
          </button>
          <button
            :disabled="builtinProviders.filter(p => p.oauth_config).length === 0"
            class="checkin-accounts-tab__action-button checkin-accounts-tab__action-button--secondary"
            @click="emit('show-oauth-wizard')"
          >
            <SIcon
              name="Shield"
              size="w-5 h-5"
            />
            <span>OAuth 登录</span>
          </button>
        </div>
      </div>
    </div>

    <!-- 账号列表 -->
    <div
      v-if="accounts.length === 0"
      class="checkin-accounts-tab__empty"
    >
      {{ providers.length === 0 ? '请先添加提供商' : '暂无账号，点击上方按钮添加' }}
    </div>
    <div
      v-else
      class="checkin-accounts-tab__table-shell"
    >
      <table class="checkin-accounts-tab__table">
        <thead class="checkin-accounts-tab__table-head">
          <tr>
            <th class="checkin-accounts-tab__th">
              账号名
            </th>
            <th class="checkin-accounts-tab__th checkin-accounts-tab__th--right">
              余额
            </th>
            <th class="checkin-accounts-tab__th checkin-accounts-tab__th--right">
              总额度
            </th>
            <th class="checkin-accounts-tab__th checkin-accounts-tab__th--right">
              历史消耗
            </th>
            <th class="checkin-accounts-tab__th">
              最后签到
            </th>
            <th class="checkin-accounts-tab__th checkin-accounts-tab__th--actions">
              操作
            </th>
          </tr>
        </thead>
        <tbody class="checkin-accounts-tab__table-body">
          <tr
            v-for="account in filteredAccounts"
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
                  :disabled="props.checkinLoading"
                  class="checkin-accounts-tab__mini-button"
                  @click="emit('checkin', account.id)"
                >
                  <SIcon
                    name="Calendar"
                    size="w-3 h-3"
                    class="checkin-accounts-tab__mini-button-icon"
                  /> 签到
                </button>
                <div class="checkin-accounts-tab__menu-wrap">
                  <button
                    class="checkin-accounts-tab__menu-trigger"
                    @click="toggleAccountMenu(account.id)"
                  >
                    <svg
                      class="w-4 h-4"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
                    </svg>
                  </button>
                  <!-- 下拉菜单 (向上弹出) -->
                  <div
                    v-if="openMenuAccountId === account.id"
                    class="checkin-accounts-tab__menu"
                  >
                    <button
                      class="checkin-accounts-tab__menu-item checkin-accounts-tab__menu-item--top"
                      @click="emit('refresh-balance', account.id); openMenuAccountId = null"
                    >
                      刷新余额
                    </button>
                    <button
                      class="checkin-accounts-tab__menu-item"
                      @click="openAccountModal(account); openMenuAccountId = null"
                    >
                      编辑
                    </button>
                    <button
                      class="checkin-accounts-tab__menu-item checkin-accounts-tab__menu-item--danger"
                      @click="deleteAccount(account.id); openMenuAccountId = null"
                    >
                      删除
                    </button>
                  </div>
                </div>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>

  <!-- 账号编辑弹窗 -->
  <div
    v-if="showAccountModal"
    class="checkin-accounts-tab__modal-backdrop"
    @click.self="showAccountModal = false"
  >
    <div class="checkin-accounts-tab__modal-panel">
      <!-- 标题栏 -->
      <div class="checkin-accounts-tab__modal-header">
        <h3 class="checkin-accounts-tab__modal-title">
          <SIcon
            name="Users"
            size="w-5 h-5"
            class="checkin-accounts-tab__modal-title-icon"
          />
          {{ editingAccount ? '编辑账号' : '添加账号' }}
        </h3>
      </div>

      <form
        class="checkin-accounts-tab__form"
        @submit.prevent="saveAccount"
      >
        <!-- 提供商选择 -->
        <div class="checkin-accounts-tab__field">
          <label class="checkin-accounts-tab__label">
            <span class="text-red-500">*</span> 提供商
          </label>
          <select
            v-model="accountForm.provider_id"
            required
            :disabled="!!editingAccount"
            class="checkin-accounts-tab__control"
          >
            <option value="">
              选择提供商
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

        <!-- 账号名称 -->
        <div class="checkin-accounts-tab__field">
          <label class="checkin-accounts-tab__label">
            <span class="text-red-500">*</span> 账号名称
          </label>
          <input
            v-model="accountForm.name"
            type="text"
            required
            class="checkin-accounts-tab__control"
            placeholder="例如: 主账号"
          >
        </div>

        <!-- Session 输入 -->
        <div class="checkin-accounts-tab__field">
          <label class="checkin-accounts-tab__label">
            <span
              v-if="!editingAccount"
              class="text-red-500"
            >*</span> Session / Cookies
            <span
              v-if="editingAccount"
              class="text-text-muted font-normal"
            >(留空不修改)</span>
          </label>
          <textarea
            v-model="accountForm.session"
            :required="!editingAccount"
            rows="5"
            class="checkin-accounts-tab__control checkin-accounts-tab__control--textarea checkin-accounts-tab__control--mono"
            placeholder="可直接粘贴 session 值，或完整 cookies JSON"
          />
          <p class="checkin-accounts-tab__hint checkin-accounts-tab__hint--with-icon">
            <svg
              class="checkin-accounts-tab__hint-icon"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            可直接粘贴 session 值，后台会自动包装为 cookies JSON；如果你已经拿到完整 cookies JSON，也可以直接粘贴
          </p>
        </div>

        <!-- API User -->
        <div class="checkin-accounts-tab__field">
          <label class="checkin-accounts-tab__label">
            <span class="text-red-500">*</span> API User
          </label>
          <input
            v-model="accountForm.api_user"
            type="text"
            required
            class="checkin-accounts-tab__control checkin-accounts-tab__control--mono"
            placeholder="12345"
          >
          <p class="checkin-accounts-tab__hint">
            使用 session / cookies 登录时必须填写。优先从 Local Storage 的 <code>user.id</code> 获取，也可从请求头里的 <code>new-api-user</code> 找到
          </p>
        </div>

        <div
          v-if="selectedBuiltinProvider?.requires_waf_bypass"
          class="checkin-accounts-tab__notice checkin-accounts-tab__notice--warning"
        >
          <p class="checkin-accounts-tab__notice-title checkin-accounts-tab__notice-title--warning">
            {{ selectedBuiltinProvider.name }} 需要额外的 WAF 步骤
          </p>
          <ol class="checkin-accounts-tab__notice-list checkin-accounts-tab__notice-list--warning">
            <li>先保存 <code>session / cookies</code> 和 <code>api_user</code></li>
            <li>回到“提供商”页，为 {{ selectedBuiltinProvider.name }} 点击“获取 Cookie”</li>
            <li>获取 WAF Cookie 的网页登录过程，必须与签到请求使用同一代理/出口</li>
          </ol>
        </div>

        <!-- CDK 配置区域（仅当提供商支持 CDK 时显示） -->
        <div
          v-if="selectedProviderCdkConfig"
          class="checkin-accounts-tab__notice checkin-accounts-tab__notice--amber"
        >
          <p class="checkin-accounts-tab__notice-title checkin-accounts-tab__notice-title--amber">
            🎰 CDK 充值配置
            <span class="checkin-accounts-tab__notice-title-meta">
              ({{ selectedProviderCdkConfig.cdk_type }} - 可选)
            </span>
          </p>
          <p class="checkin-accounts-tab__notice-copy">
            此提供商支持 CDK 充值码自动获取，签到后会自动尝试获取并充值。
            需要配置对应福利站的登录凭证。
          </p>

          <!-- runawaytime: fuli cookies -->
          <div
            v-if="selectedProviderCdkConfig.cdk_type === 'runawaytime'"
            class="checkin-accounts-tab__field"
          >
            <label class="checkin-accounts-tab__label checkin-accounts-tab__label--amber">
              fuli.hxi.me Cookies
            </label>
            <textarea
              v-model="accountForm.fuli_cookies"
              rows="3"
              class="checkin-accounts-tab__control checkin-accounts-tab__control--amber checkin-accounts-tab__control--textarea checkin-accounts-tab__control--mono checkin-accounts-tab__control--compact"
              placeholder="{&quot;session&quot;: &quot;xxx&quot;, &quot;token&quot;: &quot;xxx&quot;}"
            />
            <p class="checkin-accounts-tab__hint checkin-accounts-tab__hint--amber">
              输入 fuli.hxi.me 的登录 Cookies（JSON 格式）
            </p>
          </div>

          <!-- b4u: cdk cookies -->
          <div
            v-if="selectedProviderCdkConfig.cdk_type === 'b4u'"
            class="checkin-accounts-tab__field"
          >
            <label class="checkin-accounts-tab__label checkin-accounts-tab__label--amber">
              tw.b4u.qzz.io Cookies
            </label>
            <textarea
              v-model="accountForm.b4u_cdk_cookies"
              rows="3"
              class="checkin-accounts-tab__control checkin-accounts-tab__control--amber checkin-accounts-tab__control--textarea checkin-accounts-tab__control--mono checkin-accounts-tab__control--compact"
              placeholder="{&quot;session&quot;: &quot;xxx&quot;}"
            />
            <p class="checkin-accounts-tab__hint checkin-accounts-tab__hint--amber">
              输入 tw.b4u.qzz.io 的登录 Cookies（JSON 格式）
            </p>
          </div>

          <!-- x666: access_token -->
          <div
            v-if="selectedProviderCdkConfig.cdk_type === 'x666'"
            class="checkin-accounts-tab__field"
          >
            <label class="checkin-accounts-tab__label checkin-accounts-tab__label--amber">
              Access Token (JWT)
            </label>
            <input
              v-model="accountForm.x666_access_token"
              type="text"
              class="checkin-accounts-tab__control checkin-accounts-tab__control--amber checkin-accounts-tab__control--mono checkin-accounts-tab__control--compact"
              placeholder="eyJhbGciOiJIUzI1NiIs..."
            >
            <p class="checkin-accounts-tab__hint checkin-accounts-tab__hint--amber">
              输入 up.x666.me 的 JWT Access Token
            </p>
          </div>
        </div>

        <!-- 帮助提示 -->
        <div class="checkin-accounts-tab__notice checkin-accounts-tab__notice--info">
          <p class="checkin-accounts-tab__notice-title checkin-accounts-tab__notice-title--info">
            <svg
              class="checkin-accounts-tab__notice-icon"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
              />
            </svg>
            如何获取 Session
          </p>
          <ol class="checkin-accounts-tab__notice-list checkin-accounts-tab__notice-list--info">
            <li>按 <kbd class="checkin-accounts-tab__kbd">F12</kbd> 打开浏览器开发者工具</li>
            <li>转到 <span class="font-medium">Application</span> 标签页 → <span class="font-medium">Cookies</span></li>
            <li>选择目标站点，找到 <code class="checkin-accounts-tab__code">session</code> 这一行</li>
            <li>复制 session 的值，直接粘贴到上方输入框</li>
            <li>再到 <span class="font-medium">Local Storage</span> → <code class="checkin-accounts-tab__code">user</code>，取其中的 <code class="checkin-accounts-tab__code">id</code> 作为 API User</li>
          </ol>
        </div>

        <!-- 启用开关 -->
        <div class="checkin-accounts-tab__toggle">
          <input
            id="account-enabled"
            v-model="accountForm.enabled"
            type="checkbox"
            class="checkin-accounts-tab__checkbox"
          >
          <label
            for="account-enabled"
            class="checkin-accounts-tab__checkbox-label"
          >
            启用此账号
          </label>
        </div>

        <!-- 操作按钮 -->
        <div class="checkin-accounts-tab__form-actions">
          <button
            type="button"
            class="checkin-accounts-tab__form-button checkin-accounts-tab__form-button--secondary"
            @click="showAccountModal = false"
          >
            取消
          </button>
          <button
            type="submit"
            class="checkin-accounts-tab__form-button checkin-accounts-tab__form-button--primary"
          >
            保存
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  createCheckinAccount,
  updateCheckinAccount,
  deleteCheckinAccount as apiDeleteAccount,
  getCheckinAccountCookies,
} from '@/api'
import type {
  CheckinProvider,
  AccountInfo,
  BuiltinProvider,
  CdkExtraConfig,
} from '@/types/checkin'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'

const props = defineProps<{
  providers: CheckinProvider[]
  accounts: AccountInfo[]
  builtinProviders: BuiltinProvider[]
  checkinLoading: boolean
}>()

const emit = defineEmits<{
  (e: 'refresh'): void
  (e: 'checkin', accountId: string): void
  (e: 'refresh-balance', accountId: string): void
  (e: 'navigate', accountId: string): void
  (e: 'show-oauth-wizard'): void
}>()

const uiStore = useUIStore()

interface CheckinAccountCookiesResponse {
  cookies_json: string
  api_user?: string | null
}

// 本地状态
const showAccountModal = ref(false)
const editingAccount = ref<AccountInfo | null>(null)
const openMenuAccountId = ref<string | null>(null)
const searchQuery = ref('')
const providerFilter = ref<string>('all')

// 表单
const accountForm = ref({
  provider_id: '',
  name: '',
  session: '',
  api_user: '',
  enabled: true,
  fuli_cookies: '',
  b4u_cdk_cookies: '',
  x666_access_token: '',
})

// CDK 配置：根据选中的提供商查找对应的内置 CDK 配置
const selectedProviderCdkConfig = computed(() => {
  if (!accountForm.value.provider_id) return null
  const provider = props.providers.find(p => p.id === accountForm.value.provider_id)
  if (!provider) return null
  const builtin = props.builtinProviders.find(bp => bp.name === provider.name)
  return builtin?.cdk_config || null
})

const selectedBuiltinProvider = computed(() => {
  if (!accountForm.value.provider_id) return null
  const provider = props.providers.find(p => p.id === accountForm.value.provider_id)
  if (!provider) return null
  return props.builtinProviders.find(bp => bp.name === provider.name) || null
})

// 过滤后的账号列表
const filteredAccounts = computed(() => {
  let result = props.accounts

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(account =>
      account.name.toLowerCase().includes(query) ||
      (account.provider_name && account.provider_name.toLowerCase().includes(query))
    )
  }

  if (providerFilter.value !== 'all') {
    result = result.filter(account => account.provider_id === providerFilter.value)
  }

  return result
})

// 辅助函数
const getProviderName = (providerId: string) => {
  return props.providers.find(p => p.id === providerId)?.name || providerId
}

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleString('zh-CN')
}

const getErrorMessage = (error: unknown, fallback: string) =>
  error instanceof Error ? error.message : fallback

// 切换账号菜单
const toggleAccountMenu = (accountId: string) => {
  if (openMenuAccountId.value === accountId) {
    openMenuAccountId.value = null
  } else {
    openMenuAccountId.value = accountId
  }
}

// 从 cookies JSON 中提取 session 值
const extractSessionFromJson = (json: string): string => {
  try {
    const parsed: unknown = JSON.parse(json)
    if (typeof parsed === 'object' && parsed !== null && 'session' in parsed) {
      const session = (parsed as Record<string, unknown>).session
      return typeof session === 'string' ? session : ''
    }
    return ''
  } catch {
    return ''
  }
}

// 将 session 值转换为 cookies JSON 格式
const sessionToCookiesJson = (session: string): string => {
  const trimmed = session.trim()
  if (!trimmed) return ''

  // 如果用户输入的已经是 JSON 格式，直接返回
  if (trimmed.startsWith('{')) {
    try {
      JSON.parse(trimmed)
      return trimmed
    } catch {
      // 不是有效 JSON，当作 session 值处理
    }
  }

  // 否则包装成 {"session": "xxx"} 格式
  return JSON.stringify({ session: trimmed })
}

// 账号操作
const openAccountModal = async (account?: AccountInfo) => {
  editingAccount.value = account || null

  if (account) {
    // 编辑已有账号：从后端获取解密后的 cookies
    let existingExtra: CdkExtraConfig = {}
    try {
      existingExtra = account.extra_config ? JSON.parse(account.extra_config) : {}
    } catch { /* ignore */ }

    try {
      const cookiesData = await getCheckinAccountCookies<CheckinAccountCookiesResponse>(account.id)
      accountForm.value = {
        provider_id: account.provider_id,
        name: account.name,
        session: extractSessionFromJson(cookiesData.cookies_json),
        api_user: typeof cookiesData.api_user === 'string' ? cookiesData.api_user : '',
        enabled: account.enabled,
        fuli_cookies: existingExtra.fuli_cookies ? JSON.stringify(existingExtra.fuli_cookies) : '',
        b4u_cdk_cookies: existingExtra.b4u_cdk_cookies ? JSON.stringify(existingExtra.b4u_cdk_cookies) : '',
        x666_access_token: existingExtra.x666_access_token || '',
      }
    } catch (e: unknown) {
      logger.error('Failed to get cookies:', e)
      accountForm.value = {
        provider_id: account.provider_id,
        name: account.name,
        session: '',
        api_user: account.api_user || '',
        enabled: account.enabled,
        fuli_cookies: existingExtra.fuli_cookies ? JSON.stringify(existingExtra.fuli_cookies) : '',
        b4u_cdk_cookies: existingExtra.b4u_cdk_cookies ? JSON.stringify(existingExtra.b4u_cdk_cookies) : '',
        x666_access_token: existingExtra.x666_access_token || '',
      }
    }
  } else {
    accountForm.value = {
      provider_id: props.providers[0]?.id || '',
      name: '',
      session: '',
      api_user: '',
      enabled: true,
      fuli_cookies: '',
      b4u_cdk_cookies: '',
      x666_access_token: '',
    }
  }
  showAccountModal.value = true
}

const saveAccount = async () => {
  try {
    const cookiesJson = sessionToCookiesJson(accountForm.value.session)
    const apiUser = accountForm.value.api_user.trim()

    // 构建 extra_config JSON
    const extraConfig: CdkExtraConfig = {}
    if (accountForm.value.fuli_cookies) {
      try {
        extraConfig.fuli_cookies = JSON.parse(accountForm.value.fuli_cookies)
      } catch {
        uiStore.showError('fuli cookies JSON 格式错误')
        return
      }
    }
    if (accountForm.value.b4u_cdk_cookies) {
      try {
        extraConfig.b4u_cdk_cookies = JSON.parse(accountForm.value.b4u_cdk_cookies)
      } catch {
        uiStore.showError('b4u cookies JSON 格式错误')
        return
      }
    }
    if (accountForm.value.x666_access_token) {
      extraConfig.x666_access_token = accountForm.value.x666_access_token
    }
    const extraConfigJson = Object.keys(extraConfig).length > 0 ? JSON.stringify(extraConfig) : '{}'

    if (!apiUser) {
      uiStore.showError('请输入 API User。使用 session / cookies 登录的站点必须提供该值。')
      return
    }

    if (editingAccount.value) {
      const updateData: { name?: string; cookies_json?: string; api_user?: string; enabled?: boolean; extra_config?: string } = {
        name: accountForm.value.name,
        api_user: apiUser,
        enabled: accountForm.value.enabled,
        extra_config: extraConfigJson,
      }
      if (cookiesJson) {
        updateData.cookies_json = cookiesJson
      }
      await updateCheckinAccount(editingAccount.value.id, updateData)
    } else {
      if (!cookiesJson) {
        uiStore.showError('请输入 Session 值')
        return
      }
      await createCheckinAccount({
        provider_id: accountForm.value.provider_id,
        name: accountForm.value.name,
        cookies_json: cookiesJson,
        api_user: apiUser,
        extra_config: extraConfigJson,
      })
    }
    showAccountModal.value = false
    emit('refresh')
  } catch (e: unknown) {
    uiStore.showError('保存失败: ' + getErrorMessage(e, '未知错误'))
  }
}

const deleteAccount = async (id: string) => {
  const confirmed = await uiStore.requestConfirm({
    title: '删除账号',
    message: '确定要删除此账号吗？',
    confirmText: '删除',
    cancelText: '取消',
    type: 'danger',
    surface: 'solid',
  })
  if (!confirmed) return
  try {
    await apiDeleteAccount(id)
    emit('refresh')
  } catch (e: unknown) {
    uiStore.showError('删除失败: ' + getErrorMessage(e, '未知错误'))
  }
}

// 点击页面其他地方关闭菜单
const closeMenuOnClickOutside = (e: MouseEvent) => {
  if (openMenuAccountId.value && !(e.target as HTMLElement).closest('.checkin-accounts-tab__menu-wrap')) {
    openMenuAccountId.value = null
  }
}

onMounted(() => {
  document.addEventListener('click', closeMenuOnClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', closeMenuOnClickOutside)
})
</script>

<style scoped>
.checkin-accounts-tab,
.checkin-accounts-tab__form,
.checkin-accounts-tab__field,
.checkin-accounts-tab__result,
.checkin-accounts-tab__notice,
.checkin-accounts-tab__account {
  display: flex;
  flex-direction: column;
}

.checkin-accounts-tab {
  gap: 1rem;
}

.checkin-accounts-tab__panel,
.checkin-accounts-tab__empty,
.checkin-accounts-tab__table-shell {
  border: 1px solid rgb(255 255 255 / 20%);
  border-radius: 1.5rem;
  background: var(--glass-bg, rgb(255 255 255 / 8%));
  box-shadow: 0 10px 30px rgb(15 23 42 / 10%);
  backdrop-filter: blur(20px);
}

.checkin-accounts-tab__panel {
  padding: 1rem;
}

.checkin-accounts-tab__toolbar,
.checkin-accounts-tab__filters,
.checkin-accounts-tab__actions,
.checkin-accounts-tab__action-button,
.checkin-accounts-tab__account-row,
.checkin-accounts-tab__row-actions,
.checkin-accounts-tab__modal-title,
.checkin-accounts-tab__hint--with-icon,
.checkin-accounts-tab__notice-title,
.checkin-accounts-tab__toggle,
.checkin-accounts-tab__form-actions {
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

.checkin-accounts-tab__input,
.checkin-accounts-tab__control {
  border-radius: 0.75rem;
  border: 1px solid var(--border-default);
  color: var(--text-primary);
  transition: border-color 0.2s ease, box-shadow 0.2s ease, background-color 0.2s ease;
}

.checkin-accounts-tab__input {
  background: rgb(255 255 255 / 8%);
  padding: 0.625rem 0.75rem;
  font-size: 0.875rem;
}

.checkin-accounts-tab__input::placeholder,
.checkin-accounts-tab__control::placeholder {
  color: var(--text-muted);
}

.checkin-accounts-tab__input:focus,
.checkin-accounts-tab__control:focus {
  outline: none;
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 2px rgb(59 130 246 / 20%);
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

.checkin-accounts-tab__actions {
  flex-wrap: wrap;
  gap: 0.75rem;
}

.checkin-accounts-tab__action-button,
.checkin-accounts-tab__mini-button,
.checkin-accounts-tab__form-button {
  border-radius: 0.75rem;
  color: white;
  transition: background-color 0.2s ease, opacity 0.2s ease;
}

.checkin-accounts-tab__action-button {
  min-height: 44px;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
}

.checkin-accounts-tab__action-button:disabled,
.checkin-accounts-tab__mini-button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.checkin-accounts-tab__action-button--primary,
.checkin-accounts-tab__mini-button {
  background: var(--accent-primary);
}

.checkin-accounts-tab__action-button--primary:hover:not(:disabled),
.checkin-accounts-tab__mini-button:hover:not(:disabled) {
  background: rgb(59 130 246 / 90%);
}

.checkin-accounts-tab__action-button--secondary {
  background: var(--accent-secondary);
}

.checkin-accounts-tab__action-button--secondary:hover:not(:disabled) {
  background: rgb(139 92 246 / 90%);
}

.checkin-accounts-tab__empty {
  padding: 3rem 1rem;
  text-align: center;
  color: var(--text-muted);
}

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
  background: rgb(255 255 255 / 10%);
  backdrop-filter: blur(16px);
}

.checkin-accounts-tab__table-body {
  background: transparent;
}

.checkin-accounts-tab__table-head tr,
.checkin-accounts-tab__table-body tr + tr td {
  border-top: 1px solid rgb(148 163 184 / 20%);
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
  background: rgb(255 255 255 / 4%);
}

.checkin-accounts-tab__account {
  gap: 0.25rem;
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
  border-radius: 0.375rem;
  background: rgb(255 255 255 / 8%);
  padding: 0.125rem 0.5rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.checkin-accounts-tab__metric,
.checkin-accounts-tab__cell--mono {
  font-family: var(--font-mono, 'Maple Mono', monospace);
}

.checkin-accounts-tab__metric {
  font-size: 0.875rem;
  font-weight: 600;
}

.checkin-accounts-tab__metric--balance {
  color: rgb(22 163 74);
}

.dark .checkin-accounts-tab__metric--balance {
  color: rgb(74 222 128);
}

.checkin-accounts-tab__metric--quota {
  color: var(--accent-primary);
}

.checkin-accounts-tab__metric--consumed {
  color: rgb(234 88 12);
}

.dark .checkin-accounts-tab__metric--consumed {
  color: rgb(251 146 60);
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
  min-height: 40px;
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 500;
}

.checkin-accounts-tab__mini-button-icon {
  margin-right: 0.25rem;
}

.checkin-accounts-tab__menu-wrap {
  position: relative;
}

.checkin-accounts-tab__menu-trigger {
  min-width: 40px;
  min-height: 40px;
  border-radius: 0.5rem;
  color: var(--text-muted);
  transition: color 0.2s ease, background-color 0.2s ease;
}

.checkin-accounts-tab__menu-trigger:hover {
  background: rgb(255 255 255 / 8%);
  color: var(--text-primary);
}

.checkin-accounts-tab__menu {
  position: absolute;
  right: 0;
  bottom: calc(100% + 0.25rem);
  z-index: 50;
  width: 8rem;
  overflow: hidden;
  border-radius: 0.5rem;
  border: 1px solid var(--border-default);
  background: var(--bg-elevated);
  box-shadow: 0 12px 24px rgb(15 23 42 / 18%);
}

.checkin-accounts-tab__menu-item {
  width: 100%;
  padding: 0.5rem 0.75rem;
  text-align: left;
  font-size: 0.875rem;
  color: var(--text-secondary);
  transition: background-color 0.2s ease;
}

.checkin-accounts-tab__menu-item:hover {
  background: rgb(255 255 255 / 8%);
}

.checkin-accounts-tab__menu-item--top {
  border-top-left-radius: 0.5rem;
  border-top-right-radius: 0.5rem;
}

.checkin-accounts-tab__menu-item--danger {
  color: rgb(220 38 38);
}

.dark .checkin-accounts-tab__menu-item--danger {
  color: rgb(248 113 113);
}

.checkin-accounts-tab__menu-item--danger:hover {
  background: rgb(254 242 242 / 60%);
}

.dark .checkin-accounts-tab__menu-item--danger:hover {
  background: rgb(127 29 29 / 20%);
}

.checkin-accounts-tab__modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(0 0 0 / 50%);
  backdrop-filter: blur(12px);
}

.checkin-accounts-tab__modal-panel {
  width: 100%;
  max-width: 36rem;
  margin: 0 1rem;
  overflow: hidden;
  border-radius: 0.75rem;
  background: white;
  box-shadow: 0 24px 48px rgb(15 23 42 / 30%);
}

.dark .checkin-accounts-tab__modal-panel {
  background: rgb(31 41 55);
}

.checkin-accounts-tab__modal-header {
  border-bottom: 1px solid rgb(229 231 235);
  background: linear-gradient(to right, rgb(239 246 255), rgb(224 231 255));
  padding: 1rem 1.5rem;
}

.dark .checkin-accounts-tab__modal-header {
  border-color: rgb(55 65 81);
  background: linear-gradient(to right, rgb(31 41 55), rgb(31 41 55));
}

.checkin-accounts-tab__modal-title {
  gap: 0.5rem;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
}

.checkin-accounts-tab__modal-title-icon {
  color: rgb(37 99 235);
}

.dark .checkin-accounts-tab__modal-title-icon {
  color: rgb(96 165 250);
}

.checkin-accounts-tab__form {
  gap: 1.25rem;
  padding: 1.5rem;
}

.checkin-accounts-tab__field {
  gap: 0.375rem;
}

.checkin-accounts-tab__label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.checkin-accounts-tab__label--amber {
  color: rgb(146 64 14);
}

.dark .checkin-accounts-tab__label--amber {
  color: rgb(252 211 77);
}

.checkin-accounts-tab__control {
  display: block;
  width: 100%;
  background: white;
  padding: 0.625rem 0.75rem;
}

.dark .checkin-accounts-tab__control {
  border-color: rgb(75 85 99);
  background: rgb(55 65 81);
  color: white;
}

.checkin-accounts-tab__control:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.checkin-accounts-tab__control--textarea {
  resize: vertical;
  min-height: 120px;
}

.checkin-accounts-tab__control--compact {
  padding-block: 0.5rem;
  font-size: 0.75rem;
}

.checkin-accounts-tab__control--mono,
.checkin-accounts-tab__kbd,
.checkin-accounts-tab__code {
  font-family: var(--font-mono, 'Maple Mono', monospace);
}

.checkin-accounts-tab__control--mono {
  background: rgb(249 250 251);
}

.dark .checkin-accounts-tab__control--mono {
  background: rgb(17 24 39);
}

.checkin-accounts-tab__control--amber {
  border-color: rgb(252 211 77);
}

.dark .checkin-accounts-tab__control--amber {
  border-color: rgb(180 83 9);
}

.checkin-accounts-tab__control--amber:focus {
  border-color: rgb(245 158 11);
  box-shadow: 0 0 0 2px rgb(245 158 11 / 20%);
}

.checkin-accounts-tab__hint,
.checkin-accounts-tab__notice-copy,
.checkin-accounts-tab__notice-list {
  font-size: 0.75rem;
  line-height: 1.25rem;
}

.checkin-accounts-tab__hint {
  color: var(--text-muted);
}

.checkin-accounts-tab__hint--with-icon {
  gap: 0.25rem;
}

.checkin-accounts-tab__hint-icon,
.checkin-accounts-tab__notice-icon {
  width: 0.875rem;
  height: 0.875rem;
}

.checkin-accounts-tab__hint--amber {
  color: rgb(202 138 4);
}

.dark .checkin-accounts-tab__hint--amber {
  color: rgb(252 211 77);
}

.checkin-accounts-tab__notice {
  gap: 1rem;
  border-radius: 0.5rem;
  border: 1px solid;
  padding: 1rem;
}

.checkin-accounts-tab__notice--warning {
  border-color: rgb(253 186 116);
  background: linear-gradient(to right, rgb(255 247 237), rgb(255 251 235));
}

.dark .checkin-accounts-tab__notice--warning {
  border-color: rgb(154 52 18 / 50%);
  background: linear-gradient(to right, rgb(154 52 18 / 20%), rgb(120 53 15 / 20%));
}

.checkin-accounts-tab__notice--amber {
  border-color: rgb(252 211 77);
  background: linear-gradient(to right, rgb(255 251 235), rgb(255 247 237));
}

.dark .checkin-accounts-tab__notice--amber {
  border-color: rgb(146 64 14 / 50%);
  background: linear-gradient(to right, rgb(146 64 14 / 20%), rgb(120 53 15 / 20%));
}

.checkin-accounts-tab__notice--info {
  border-color: rgb(191 219 254);
  background: linear-gradient(to right, rgb(239 246 255), rgb(224 231 255));
}

.dark .checkin-accounts-tab__notice--info {
  border-color: rgb(30 64 175 / 50%);
  background: linear-gradient(to right, rgb(30 64 175 / 20%), rgb(55 48 163 / 20%));
}

.checkin-accounts-tab__notice-title {
  gap: 0.375rem;
  font-size: 0.875rem;
  font-weight: 500;
}

.checkin-accounts-tab__notice-title--warning {
  color: rgb(154 52 18);
}

.dark .checkin-accounts-tab__notice-title--warning {
  color: rgb(253 186 116);
}

.checkin-accounts-tab__notice-title--amber {
  color: rgb(146 64 14);
}

.dark .checkin-accounts-tab__notice-title--amber {
  color: rgb(252 211 77);
}

.checkin-accounts-tab__notice-title--info {
  color: rgb(30 64 175);
}

.dark .checkin-accounts-tab__notice-title--info {
  color: rgb(147 197 253);
}

.checkin-accounts-tab__notice-title-meta {
  font-size: 0.75rem;
  font-weight: 400;
  color: rgb(217 119 6);
}

.dark .checkin-accounts-tab__notice-title-meta {
  color: rgb(251 191 36);
}

.checkin-accounts-tab__notice-copy,
.checkin-accounts-tab__notice-list--warning {
  color: rgb(180 83 9);
}

.dark .checkin-accounts-tab__notice-copy,
.dark .checkin-accounts-tab__notice-list--warning {
  color: rgb(253 230 138);
}

.checkin-accounts-tab__notice-list {
  list-style-position: inside;
  list-style-type: decimal;
}

.checkin-accounts-tab__notice-list--info {
  margin-left: 0.125rem;
  color: rgb(29 78 216);
}

.dark .checkin-accounts-tab__notice-list--info {
  color: rgb(191 219 254);
}

.checkin-accounts-tab__kbd,
.checkin-accounts-tab__code {
  border-radius: 0.25rem;
  background: rgb(219 234 254);
  color: rgb(30 64 175);
}

.checkin-accounts-tab__kbd {
  padding: 0.125rem 0.375rem;
}

.checkin-accounts-tab__code {
  padding: 0.125rem 0.25rem;
}

.dark .checkin-accounts-tab__kbd,
.dark .checkin-accounts-tab__code {
  background: rgb(30 64 175 / 50%);
  color: rgb(191 219 254);
}

.checkin-accounts-tab__toggle {
  padding-block: 0.25rem;
}

.checkin-accounts-tab__checkbox {
  width: 1rem;
  height: 1rem;
  cursor: pointer;
  border-radius: 0.25rem;
  border: 1px solid rgb(209 213 219);
  accent-color: rgb(37 99 235);
}

.checkin-accounts-tab__checkbox-label {
  margin-left: 0.625rem;
  cursor: pointer;
  user-select: none;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.checkin-accounts-tab__form-actions {
  justify-content: flex-end;
  gap: 0.75rem;
  border-top: 1px solid rgb(229 231 235);
  padding-top: 1rem;
}

.dark .checkin-accounts-tab__form-actions {
  border-color: rgb(55 65 81);
}

.checkin-accounts-tab__form-button {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  font-weight: 500;
}

.checkin-accounts-tab__form-button--secondary {
  border: 1px solid rgb(209 213 219);
  background: transparent;
  color: var(--text-secondary);
}

.dark .checkin-accounts-tab__form-button--secondary {
  border-color: rgb(75 85 99);
}

.checkin-accounts-tab__form-button--secondary:hover {
  background: rgb(249 250 251);
}

.dark .checkin-accounts-tab__form-button--secondary:hover {
  background: rgb(55 65 81);
}

.checkin-accounts-tab__form-button--primary {
  background: rgb(37 99 235);
}

.checkin-accounts-tab__form-button--primary:hover {
  background: rgb(29 78 216);
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

  .checkin-accounts-tab__table-shell {
    overflow-x: auto;
  }
}
</style>
