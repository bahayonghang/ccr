<!-- eslint-disable no-console -->
<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between flex-wrap gap-4">
      <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
        签到账号
      </h2>
      <!-- 搜索和过滤区域 -->
      <div class="flex items-center gap-3 flex-1 justify-end">
        <!-- 搜索框 -->
        <div class="relative">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索账号..."
            class="w-48 pl-9 pr-4 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
          <svg
            class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400"
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
          class="px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
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
      <button
        :disabled="providers.length === 0"
        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg flex items-center space-x-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
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
        class="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg flex items-center space-x-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        @click="emit('show-oauth-wizard')"
      >
        <Shield class="w-5 h-5" />
        <span>OAuth 登录</span>
      </button>
    </div>

    <!-- 账号列表 -->
    <div
      v-if="accounts.length === 0"
      class="text-center py-12 text-gray-500 dark:text-gray-400"
    >
      {{ providers.length === 0 ? '请先添加提供商' : '暂无账号，点击上方按钮添加' }}
    </div>
    <div
      v-else
      class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700"
    >
      <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
        <thead class="bg-gray-50/80 dark:bg-gray-700/50 backdrop-blur-sm sticky top-0 rounded-t-xl">
          <tr>
            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-300 uppercase tracking-wider">
              账号名
            </th>
            <th class="px-4 py-3 text-right text-xs font-semibold text-gray-500 dark:text-gray-300 uppercase tracking-wider">
              余额
            </th>
            <th class="px-4 py-3 text-right text-xs font-semibold text-gray-500 dark:text-gray-300 uppercase tracking-wider">
              总额度
            </th>
            <th class="px-4 py-3 text-right text-xs font-semibold text-gray-500 dark:text-gray-300 uppercase tracking-wider">
              历史消耗
            </th>
            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-500 dark:text-gray-300 uppercase tracking-wider">
              最后签到
            </th>
            <th class="px-4 py-3 text-center text-xs font-semibold text-gray-500 dark:text-gray-300 uppercase tracking-wider w-36">
              操作
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
          <tr
            v-for="account in filteredAccounts"
            :key="account.id"
            class="hover:bg-gray-50/60 dark:hover:bg-gray-700/50 transition-colors duration-200 cursor-pointer"
            @click="emit('navigate', account.id)"
          >
            <!-- 账号名 + 提供商 -->
            <td class="px-4 py-3">
              <div class="flex flex-col gap-1">
                <div class="flex items-center gap-2">
                  <div
                    class="w-2 h-2 rounded-full flex-shrink-0"
                    :class="account.enabled ? 'bg-green-500' : 'bg-gray-400'"
                  />
                  <span class="text-sm font-semibold text-gray-900 dark:text-white">
                    {{ account.name }}
                  </span>
                </div>
                <span class="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 rounded-md w-fit">
                  {{ account.provider_name || getProviderName(account.provider_id) }}
                </span>
              </div>
            </td>
            <!-- 余额 -->
            <td class="px-4 py-3 text-right">
              <span
                v-if="account.latest_balance !== undefined && account.latest_balance !== null"
                class="font-mono text-sm font-semibold text-green-600 dark:text-green-400"
              >
                ${{ account.latest_balance.toFixed(2) }}
              </span>
              <span
                v-else
                class="text-xs text-gray-400"
              >-</span>
            </td>
            <!-- 总额度 -->
            <td class="px-4 py-3 text-right">
              <span
                v-if="account.total_quota !== undefined && account.total_quota !== null"
                class="font-mono text-sm font-semibold text-blue-600 dark:text-blue-400"
              >
                ${{ account.total_quota.toFixed(2) }}
              </span>
              <span
                v-else
                class="text-xs text-gray-400"
              >-</span>
            </td>
            <!-- 历史消耗 -->
            <td class="px-4 py-3 text-right">
              <span
                v-if="account.total_consumed !== undefined && account.total_consumed !== null"
                class="font-mono text-sm font-semibold text-orange-600 dark:text-orange-400"
              >
                ${{ account.total_consumed.toFixed(2) }}
              </span>
              <span
                v-else
                class="text-xs text-gray-400"
              >-</span>
            </td>
            <!-- 最后签到 -->
            <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400 font-mono text-xs">
              {{ account.last_checkin_at ? formatDate(account.last_checkin_at) : '-' }}
            </td>
            <!-- 操作 -->
            <td
              class="px-4 py-3"
              @click.stop
            >
              <div class="flex items-center justify-center gap-2">
                <button
                  class="inline-flex items-center px-3 py-1.5 rounded-lg text-xs font-medium bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white shadow-sm transition-colors duration-200"
                  @click="emit('checkin', account.id)"
                >
                  <Calendar class="w-3 h-3 mr-1 inline" /> 签到
                </button>
                <div class="relative">
                  <button
                    class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 dark:hover:text-gray-300 transition-colors"
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
                    class="absolute right-0 bottom-full mb-1 w-32 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-50"
                  >
                    <button
                      class="w-full px-3 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-t-lg"
                      @click="emit('refresh-balance', account.id); openMenuAccountId = null"
                    >
                      刷新余额
                    </button>
                    <button
                      class="w-full px-3 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                      @click="openAccountModal(account); openMenuAccountId = null"
                    >
                      编辑
                    </button>
                    <button
                      class="w-full px-3 py-2 text-left text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-b-lg"
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
    class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50"
    @click.self="showAccountModal = false"
  >
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-xl mx-4 overflow-hidden">
      <!-- 标题栏 -->
      <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-gray-800 dark:to-gray-800">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
          <Users class="w-5 h-5 text-blue-600 dark:text-blue-400" />
          {{ editingAccount ? '编辑账号' : '添加账号' }}
        </h3>
      </div>

      <form
        class="p-6 space-y-5"
        @submit.prevent="saveAccount"
      >
        <!-- 提供商选择 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
            <span class="text-red-500">*</span> 提供商
          </label>
          <select
            v-model="accountForm.provider_id"
            required
            :disabled="!!editingAccount"
            class="block w-full px-3 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white disabled:opacity-50 disabled:cursor-not-allowed focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
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
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
            <span class="text-red-500">*</span> 账号名称
          </label>
          <input
            v-model="accountForm.name"
            type="text"
            required
            class="block w-full px-3 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
            placeholder="例如: 主账号"
          >
        </div>

        <!-- Session 输入 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
            <span
              v-if="!editingAccount"
              class="text-red-500"
            >*</span> Session
            <span
              v-if="editingAccount"
              class="text-gray-400 dark:text-gray-500 font-normal"
            >(留空不修改)</span>
          </label>
          <textarea
            v-model="accountForm.session"
            :required="!editingAccount"
            rows="5"
            class="block w-full px-3 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 font-mono text-sm leading-relaxed resize-y min-h-[120px] placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 focus:bg-white dark:focus:bg-gray-800 transition-colors"
            placeholder="直接粘贴 session 值即可"
          />
          <p class="mt-1.5 text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1">
            <svg
              class="w-3.5 h-3.5"
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
            直接粘贴 session 值，后台会自动处理格式
          </p>
        </div>

        <!-- API User -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
            API User
            <span class="text-gray-400 dark:text-gray-500 font-normal">(可选)</span>
          </label>
          <input
            v-model="accountForm.api_user"
            type="text"
            class="block w-full px-3 py-2.5 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
            placeholder="12345"
          >
          <p class="mt-1.5 text-xs text-gray-500 dark:text-gray-400">
            通常为 5 位数字，可在 Network 标签的请求头中找到 "New-Api-User"
          </p>
        </div>

        <!-- CDK 配置区域（仅当提供商支持 CDK 时显示） -->
        <div
          v-if="selectedProviderCdkConfig"
          class="bg-gradient-to-r from-amber-50 to-orange-50 dark:from-amber-900/20 dark:to-orange-900/20 border border-amber-200 dark:border-amber-800/50 rounded-lg p-4 space-y-4"
        >
          <p class="text-sm font-medium text-amber-800 dark:text-amber-300 flex items-center gap-1.5">
            🎰 CDK 充值配置
            <span class="text-xs font-normal text-amber-600 dark:text-amber-400">
              ({{ selectedProviderCdkConfig.cdk_type }} - 可选)
            </span>
          </p>
          <p class="text-xs text-amber-700 dark:text-amber-300/80">
            此提供商支持 CDK 充值码自动获取，签到后会自动尝试获取并充值。
            需要配置对应福利站的登录凭证。
          </p>

          <!-- runawaytime: fuli cookies -->
          <div v-if="selectedProviderCdkConfig.cdk_type === 'runawaytime'">
            <label class="block text-sm font-medium text-amber-800 dark:text-amber-300 mb-1">
              fuli.hxi.me Cookies
            </label>
            <textarea
              v-model="accountForm.fuli_cookies"
              rows="3"
              class="block w-full px-3 py-2 border border-amber-300 dark:border-amber-700 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 font-mono text-xs resize-y placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-amber-500 focus:border-amber-500 transition-colors"
              placeholder="{&quot;session&quot;: &quot;xxx&quot;, &quot;token&quot;: &quot;xxx&quot;}"
            />
            <p class="mt-1 text-xs text-amber-600 dark:text-amber-400">
              输入 fuli.hxi.me 的登录 Cookies（JSON 格式）
            </p>
          </div>

          <!-- b4u: cdk cookies -->
          <div v-if="selectedProviderCdkConfig.cdk_type === 'b4u'">
            <label class="block text-sm font-medium text-amber-800 dark:text-amber-300 mb-1">
              tw.b4u.qzz.io Cookies
            </label>
            <textarea
              v-model="accountForm.b4u_cdk_cookies"
              rows="3"
              class="block w-full px-3 py-2 border border-amber-300 dark:border-amber-700 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 font-mono text-xs resize-y placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-amber-500 focus:border-amber-500 transition-colors"
              placeholder="{&quot;session&quot;: &quot;xxx&quot;}"
            />
            <p class="mt-1 text-xs text-amber-600 dark:text-amber-400">
              输入 tw.b4u.qzz.io 的登录 Cookies（JSON 格式）
            </p>
          </div>

          <!-- x666: access_token -->
          <div v-if="selectedProviderCdkConfig.cdk_type === 'x666'">
            <label class="block text-sm font-medium text-amber-800 dark:text-amber-300 mb-1">
              Access Token (JWT)
            </label>
            <input
              v-model="accountForm.x666_access_token"
              type="text"
              class="block w-full px-3 py-2 border border-amber-300 dark:border-amber-700 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 font-mono text-xs placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-amber-500 focus:border-amber-500 transition-colors"
              placeholder="eyJhbGciOiJIUzI1NiIs..."
            >
            <p class="mt-1 text-xs text-amber-600 dark:text-amber-400">
              输入 up.x666.me 的 JWT Access Token
            </p>
          </div>
        </div>

        <!-- 帮助提示 -->
        <div class="bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 border border-blue-200 dark:border-blue-800/50 rounded-lg p-4">
          <p class="text-sm font-medium text-blue-800 dark:text-blue-300 mb-2 flex items-center gap-1.5">
            <svg
              class="w-4 h-4"
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
          <ol class="text-xs text-blue-700 dark:text-blue-300/90 space-y-1.5 list-decimal list-inside ml-0.5">
            <li>按 <kbd class="px-1.5 py-0.5 bg-blue-100 dark:bg-blue-800/50 rounded text-blue-800 dark:text-blue-200 font-mono">F12</kbd> 打开浏览器开发者工具</li>
            <li>转到 <span class="font-medium">Application</span> 标签页 → <span class="font-medium">Cookies</span></li>
            <li>选择目标站点，找到 <code class="px-1 py-0.5 bg-blue-100 dark:bg-blue-800/50 rounded font-mono">session</code> 这一行</li>
            <li>复制 session 的值，直接粘贴到上方输入框</li>
          </ol>
        </div>

        <!-- 启用开关 -->
        <div class="flex items-center py-1">
          <input
            id="account-enabled"
            v-model="accountForm.enabled"
            type="checkbox"
            class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
          >
          <label
            for="account-enabled"
            class="ml-2.5 text-sm text-gray-700 dark:text-gray-300 cursor-pointer select-none"
          >
            启用此账号
          </label>
        </div>

        <!-- 操作按钮 -->
        <div class="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
          <button
            type="button"
            class="px-4 py-2 text-sm font-medium border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 focus:ring-2 focus:ring-gray-300 dark:focus:ring-gray-600 transition-colors"
            @click="showAccountModal = false"
          >
            取消
          </button>
          <button
            type="submit"
            class="px-5 py-2 text-sm font-medium bg-blue-600 hover:bg-blue-700 text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800 transition-colors"
          >
            保存
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
/* eslint-disable no-console */
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Users, Shield, Calendar } from 'lucide-vue-next'
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

const props = defineProps<{
  providers: CheckinProvider[]
  accounts: AccountInfo[]
  builtinProviders: BuiltinProvider[]
}>()

const emit = defineEmits<{
  (e: 'refresh'): void
  (e: 'checkin', accountId: string): void
  (e: 'refresh-balance', accountId: string): void
  (e: 'navigate', accountId: string): void
  (e: 'show-oauth-wizard'): void
}>()

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
      console.error('Failed to get cookies:', e)
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

    // 构建 extra_config JSON
    const extraConfig: CdkExtraConfig = {}
    if (accountForm.value.fuli_cookies) {
      try {
        extraConfig.fuli_cookies = JSON.parse(accountForm.value.fuli_cookies)
      } catch {
        alert('fuli cookies JSON 格式错误')
        return
      }
    }
    if (accountForm.value.b4u_cdk_cookies) {
      try {
        extraConfig.b4u_cdk_cookies = JSON.parse(accountForm.value.b4u_cdk_cookies)
      } catch {
        alert('b4u cookies JSON 格式错误')
        return
      }
    }
    if (accountForm.value.x666_access_token) {
      extraConfig.x666_access_token = accountForm.value.x666_access_token
    }
    const extraConfigJson = Object.keys(extraConfig).length > 0 ? JSON.stringify(extraConfig) : '{}'

    if (editingAccount.value) {
      const updateData: { name?: string; cookies_json?: string; api_user?: string; enabled?: boolean; extra_config?: string } = {
        name: accountForm.value.name,
        enabled: accountForm.value.enabled,
        extra_config: extraConfigJson,
      }
      if (cookiesJson) {
        updateData.cookies_json = cookiesJson
      }
      if (accountForm.value.api_user) {
        updateData.api_user = accountForm.value.api_user
      }
      await updateCheckinAccount(editingAccount.value.id, updateData)
    } else {
      if (!cookiesJson) {
        alert('请输入 Session 值')
        return
      }
      await createCheckinAccount({
        provider_id: accountForm.value.provider_id,
        name: accountForm.value.name,
        cookies_json: cookiesJson,
        api_user: accountForm.value.api_user || '',
        extra_config: extraConfigJson,
      })
    }
    showAccountModal.value = false
    emit('refresh')
  } catch (e: unknown) {
    alert('保存失败: ' + getErrorMessage(e, '未知错误'))
  }
}

const deleteAccount = async (id: string) => {
  if (!confirm('确定要删除此账号吗？')) return
  try {
    await apiDeleteAccount(id)
    emit('refresh')
  } catch (e: unknown) {
    alert('删除失败: ' + getErrorMessage(e, '未知错误'))
  }
}

// 点击页面其他地方关闭菜单
const closeMenuOnClickOutside = (e: MouseEvent) => {
  if (openMenuAccountId.value && !(e.target as HTMLElement).closest('.relative')) {
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
