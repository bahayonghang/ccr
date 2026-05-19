<template>
  <div class="checkin-accounts-tab">
    <div class="checkin-accounts-tab__panel">
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
      class="checkin-accounts-tab__empty"
    >
      {{ providers.length === 0 ? t('checkin.accounts.emptyNoProviders') : t('checkin.accounts.emptyNoAccounts') }}
    </div>
    <div
      v-else
      class="checkin-accounts-tab__table-shell"
    >
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
                  :title="props.checkinLoading ? t('checkin.actions.checking') : t('checkin.actions.checkIn')"
                  @click="emit('checkin', account.id)"
                >
                  <SIcon
                    :name="props.checkinLoading ? 'Loader2' : 'Calendar'"
                    size="w-3 h-3"
                    :class="[
                      'checkin-accounts-tab__mini-button-icon',
                      { 'animate-spin': props.checkinLoading },
                    ]"
                  />
                  <span class="checkin-accounts-tab__mini-button-label">{{ props.checkinLoading ? t('checkin.actions.checking') : t('checkin.actions.checkIn') }}</span>
                </button>
                <div class="checkin-accounts-tab__menu-wrap">
                  <button
                    class="checkin-accounts-tab__menu-trigger"
                    :class="{
                      'checkin-accounts-tab__menu-trigger--active':
                        openMenuAccountId === account.id,
                    }"
                    @click="toggleAccountMenu(account.id, $event)"
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
  </div>

  <Teleport to="body">
    <div
      v-if="activeMenuAccount"
      class="checkin-accounts-tab__menu checkin-accounts-tab__menu--floating"
      :class="`checkin-accounts-tab__menu--${accountMenuPosition.placement}`"
      :style="accountMenuStyle"
      @click.stop
    >
      <button
        class="checkin-accounts-tab__menu-item checkin-accounts-tab__menu-item--top"
        @click="emit('refresh-balance', activeMenuAccount.id); closeAccountMenu()"
      >
        {{ t('checkin.actions.refreshBalance') }}
      </button>
      <button
        class="checkin-accounts-tab__menu-item"
        @click="openAccountModal(activeMenuAccount); closeAccountMenu()"
      >
        {{ t('checkin.accounts.edit') }}
      </button>
      <button
        class="checkin-accounts-tab__menu-item checkin-accounts-tab__menu-item--danger"
        @click="deleteAccount(activeMenuAccount.id); closeAccountMenu()"
      >
        {{ t('checkin.accounts.delete') }}
      </button>
    </div>
  </Teleport>

  <!-- 账号编辑弹窗 -->
  <BaseModal
    v-model="showAccountModal"
    size="xl"
    surface="solid"
    content-class="checkin-accounts-tab__account-modal"
  >
    <template #header="{ titleId }">
      <div class="checkin-accounts-tab__modal-header">
        <div class="checkin-accounts-tab__modal-header-copy">
          <p class="checkin-accounts-tab__modal-eyebrow">
            {{ editingAccount ? t('checkin.accounts.modal.editEyebrow') : t('checkin.accounts.modal.createEyebrow') }}
          </p>
          <h3
            :id="titleId"
            class="checkin-accounts-tab__modal-title"
          >
            <SIcon
              name="Users"
              size="w-5 h-5"
              class="checkin-accounts-tab__modal-title-icon"
            />
            {{ editingAccount ? t('checkin.accounts.editAccount') : t('checkin.accounts.addAccount') }}
          </h3>
          <p class="checkin-accounts-tab__modal-subtitle">
            {{
              editingAccount
                ? t('checkin.accounts.modal.editSubtitle')
                : t('checkin.accounts.modal.createSubtitle')
            }}
          </p>
        </div>
        <div class="checkin-accounts-tab__modal-badge-row">
          <span class="checkin-accounts-tab__modal-badge">
            {{ modalProviderLabel }}
          </span>
          <span
            v-if="selectedBuiltinProvider?.requires_waf_bypass"
            class="checkin-accounts-tab__modal-badge checkin-accounts-tab__modal-badge--warning"
          >
            {{ t('checkin.accounts.modal.requiresWaf') }}
          </span>
        </div>
      </div>
    </template>

    <div class="checkin-accounts-tab__modal-body">
      <div class="checkin-accounts-tab__modal-intro">
        <span class="checkin-accounts-tab__modal-intro-pill">{{ t('checkin.accounts.modal.introSession') }}</span>
        <span class="checkin-accounts-tab__modal-intro-pill">{{ t('checkin.accounts.modal.introApiUser') }}</span>
        <span class="checkin-accounts-tab__modal-intro-pill">{{ t('checkin.accounts.modal.introNoOverwrite') }}</span>
      </div>

      <div class="checkin-accounts-tab__modal-scroll">
        <form
          id="checkin-account-form"
          class="checkin-accounts-tab__form"
          @submit.prevent="saveAccount"
        >
          <section class="checkin-accounts-tab__form-section checkin-accounts-tab__form-section--identity">
            <div class="checkin-accounts-tab__form-grid">
              <!-- 提供商选择 -->
              <div class="checkin-accounts-tab__field">
                <label class="checkin-accounts-tab__label">
                  <span class="text-red-500">*</span> {{ t('checkin.accounts.fields.provider') }}
                </label>
                <select
                  v-model="accountForm.provider_id"
                  required
                  :disabled="!!editingAccount"
                  class="checkin-accounts-tab__control"
                >
                  <option value="">
                    {{ t('checkin.accounts.fields.selectProvider') }}
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
                  <span class="text-red-500">*</span> {{ t('checkin.accounts.fields.accountName') }}
                </label>
                <input
                  v-model="accountForm.name"
                  type="text"
                  required
                  class="checkin-accounts-tab__control"
                  :placeholder="t('checkin.accounts.fields.accountNamePlaceholder')"
                >
              </div>
            </div>
          </section>

          <section class="checkin-accounts-tab__form-section checkin-accounts-tab__form-section--credentials">
            <!-- Session 输入 -->
            <div class="checkin-accounts-tab__field checkin-accounts-tab__field--credential">
              <label class="checkin-accounts-tab__label">
                <span
                  v-if="!editingAccount"
                  class="text-red-500"
                >*</span> Session / Cookies
                <span
                  v-if="editingAccount"
                  class="text-text-muted font-normal"
                >{{ t('checkin.accounts.fields.leaveBlank') }}</span>
              </label>
              <textarea
                v-model="accountForm.session"
                :required="!editingAccount"
                rows="7"
                class="checkin-accounts-tab__control checkin-accounts-tab__control--textarea checkin-accounts-tab__control--mono checkin-accounts-tab__control--credential"
                :placeholder="t('checkin.accounts.fields.sessionPlaceholder')"
              />
              <p class="checkin-accounts-tab__hint checkin-accounts-tab__hint--with-icon checkin-accounts-tab__hint--credential">
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
                {{ t('checkin.accounts.fields.sessionHint') }}
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
                <i18n-t
                  keypath="checkin.accounts.fields.apiUserHint"
                  scope="global"
                  tag="span"
                >
                  <template #userId>
                    <code>user.id</code>
                  </template>
                  <template #header>
                    <code>new-api-user</code>
                  </template>
                </i18n-t>
              </p>
            </div>
          </section>

          <div
            v-if="selectedBuiltinProvider?.requires_waf_bypass"
            class="checkin-accounts-tab__notice checkin-accounts-tab__notice--warning"
          >
            <p class="checkin-accounts-tab__notice-title checkin-accounts-tab__notice-title--warning">
              {{ t('checkin.accounts.waf.title', { provider: selectedBuiltinProvider.name }) }}
            </p>
            <ol class="checkin-accounts-tab__notice-list checkin-accounts-tab__notice-list--warning">
              <li>{{ t('checkin.accounts.waf.stepSave') }}</li>
              <li>{{ t('checkin.accounts.waf.stepProviders', { provider: selectedBuiltinProvider.name }) }}</li>
              <li>{{ t('checkin.accounts.waf.stepProxy') }}</li>
            </ol>
          </div>

          <!-- CDK 配置区域（仅当提供商支持 CDK 时显示） -->
          <div
            v-if="selectedProviderCdkConfig"
            class="checkin-accounts-tab__notice checkin-accounts-tab__notice--amber"
          >
            <p class="checkin-accounts-tab__notice-title checkin-accounts-tab__notice-title--amber">
              {{ t('checkin.accounts.cdk.title') }}
              <span class="checkin-accounts-tab__notice-title-meta">
                {{ t('checkin.accounts.cdk.typeOptional', { type: selectedProviderCdkConfig.cdk_type }) }}
              </span>
            </p>
            <p class="checkin-accounts-tab__notice-copy">
              {{ t('checkin.accounts.cdk.description') }}
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
                {{ t('checkin.accounts.cdk.cookiesHint', { site: 'fuli.hxi.me' }) }}
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
                {{ t('checkin.accounts.cdk.cookiesHint', { site: 'tw.b4u.qzz.io' }) }}
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
                {{ t('checkin.accounts.cdk.accessTokenHint', { site: 'up.x666.me' }) }}
              </p>
            </div>
          </div>

          <!-- 帮助提示 -->
          <div class="checkin-accounts-tab__notice checkin-accounts-tab__notice--info checkin-accounts-tab__notice--help">
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
              {{ t('checkin.accounts.help.title') }}
            </p>
            <ol class="checkin-accounts-tab__notice-list checkin-accounts-tab__notice-list--info">
              <li>{{ t('checkin.accounts.help.stepOpenDevtools') }}</li>
              <li>
                {{ t('checkin.accounts.help.stepApplicationCookies') }}
              </li>
              <li>
                {{ t('checkin.accounts.help.stepFindSession') }}
              </li>
              <li>{{ t('checkin.accounts.help.stepCopySession') }}</li>
              <li>
                {{ t('checkin.accounts.help.stepApiUser') }}
              </li>
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
              {{ t('checkin.accounts.fields.enabled') }}
            </label>
          </div>
        </form>
      </div>
    </div>

    <template #footer>
      <div class="checkin-accounts-tab__modal-footer">
        <button
          type="button"
          class="checkin-accounts-tab__form-button checkin-accounts-tab__form-button--secondary"
          @click="showAccountModal = false"
        >
          {{ t('common.cancel') }}
        </button>
        <button
          type="submit"
          form="checkin-account-form"
          class="checkin-accounts-tab__form-button checkin-accounts-tab__form-button--primary"
        >
          {{ editingAccount ? t('checkin.accounts.modal.saveChanges') : t('checkin.accounts.modal.createAccount') }}
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import BaseModal from '@/components/common/BaseModal.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  createCheckinAccount,
  updateCheckinAccount,
  deleteCheckinAccount as apiDeleteAccount,
  getCheckinAccountCookies,
} from '@/api'
import type { CheckinProvider, AccountInfo, BuiltinProvider, CdkExtraConfig } from '@/types/checkin'
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
const { t, locale } = useI18n()

interface CheckinAccountCookiesResponse {
  cookies_json: string
  api_user?: string | null
}

type AccountMenuPlacement = 'top' | 'bottom'

interface AccountMenuPosition {
  top: number
  left: number
  width: number
  maxHeight: number
  placement: AccountMenuPlacement
}

const ACCOUNT_MENU_WIDTH = 168
const ACCOUNT_MENU_ESTIMATED_HEIGHT = 144
const ACCOUNT_MENU_MARGIN = 12
const ACCOUNT_MENU_GAP = 8

// 本地状态
const showAccountModal = ref(false)
const editingAccount = ref<AccountInfo | null>(null)
const openMenuAccountId = ref<string | null>(null)
const accountMenuPosition = ref<AccountMenuPosition>({
  top: ACCOUNT_MENU_MARGIN,
  left: ACCOUNT_MENU_MARGIN,
  width: ACCOUNT_MENU_WIDTH,
  maxHeight: ACCOUNT_MENU_ESTIMATED_HEIGHT,
  placement: 'bottom',
})
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
  const provider = props.providers.find((p) => p.id === accountForm.value.provider_id)
  if (!provider) return null
  const builtin = props.builtinProviders.find((bp) => bp.name === provider.name)
  return builtin?.cdk_config || null
})

const selectedBuiltinProvider = computed(() => {
  if (!accountForm.value.provider_id) return null
  const provider = props.providers.find((p) => p.id === accountForm.value.provider_id)
  if (!provider) return null
  return props.builtinProviders.find((bp) => bp.name === provider.name) || null
})

const modalProviderLabel = computed(() => {
  if (!accountForm.value.provider_id) return t('checkin.accounts.modal.providerPending')
  return selectedBuiltinProvider.value?.name || getProviderName(accountForm.value.provider_id)
})

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

const activeMenuAccount = computed(
  () => props.accounts.find((account) => account.id === openMenuAccountId.value) || null
)

const accountMenuStyle = computed(() => ({
  top: `${accountMenuPosition.value.top}px`,
  left: `${accountMenuPosition.value.left}px`,
  width: `${accountMenuPosition.value.width}px`,
  maxHeight: `${accountMenuPosition.value.maxHeight}px`,
}))

// 辅助函数
const getProviderName = (providerId: string) => {
  return props.providers.find((p) => p.id === providerId)?.name || providerId
}

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleString(locale.value)
}

const getErrorMessage = (error: unknown, fallback: string) =>
  error instanceof Error ? error.message : fallback

// 切换账号菜单
const closeAccountMenu = () => {
  openMenuAccountId.value = null
}

const updateAccountMenuPosition = (trigger: HTMLElement) => {
  const rect = trigger.getBoundingClientRect()
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight
  const menuWidth = Math.min(ACCOUNT_MENU_WIDTH, viewportWidth - ACCOUNT_MENU_MARGIN * 2)
  const availableBelow = viewportHeight - rect.bottom - ACCOUNT_MENU_MARGIN
  const availableAbove = rect.top - ACCOUNT_MENU_MARGIN
  const placement: AccountMenuPlacement =
    availableBelow >= ACCOUNT_MENU_ESTIMATED_HEIGHT || availableBelow >= availableAbove
      ? 'bottom'
      : 'top'

  const left = Math.min(
    Math.max(ACCOUNT_MENU_MARGIN, rect.right - menuWidth),
    viewportWidth - menuWidth - ACCOUNT_MENU_MARGIN
  )

  const minimumVisibleHeight = 108
  const top =
    placement === 'bottom'
      ? Math.min(
          Math.max(ACCOUNT_MENU_MARGIN, rect.bottom + ACCOUNT_MENU_GAP),
          viewportHeight - ACCOUNT_MENU_MARGIN - minimumVisibleHeight
        )
      : Math.max(ACCOUNT_MENU_MARGIN, rect.top - ACCOUNT_MENU_ESTIMATED_HEIGHT - ACCOUNT_MENU_GAP)

  const maxHeight = Math.max(
    minimumVisibleHeight,
    placement === 'bottom'
      ? viewportHeight - top - ACCOUNT_MENU_MARGIN
      : rect.top - ACCOUNT_MENU_GAP - ACCOUNT_MENU_MARGIN
  )

  accountMenuPosition.value = {
    top,
    left,
    width: menuWidth,
    maxHeight,
    placement,
  }
}

const toggleAccountMenu = (accountId: string, event: MouseEvent) => {
  if (openMenuAccountId.value === accountId) {
    closeAccountMenu()
  } else {
    const trigger = event.currentTarget
    if (!(trigger instanceof HTMLElement)) return
    updateAccountMenuPosition(trigger)
    openMenuAccountId.value = accountId
  }
}

// 从 cookies JSON 中提取表单展示值
const extractCookiesFieldValue = (json: string): string => {
  const trimmed = json.trim()
  if (!trimmed) return ''

  try {
    const parsed: unknown = JSON.parse(trimmed)
    if (typeof parsed === 'object' && parsed !== null && !Array.isArray(parsed)) {
      const record = parsed as Record<string, unknown>
      const keys = Object.keys(record)
      if (keys.length === 1 && 'session' in record) {
        const session = record.session
        return typeof session === 'string' ? session : ''
      }
    }
    return trimmed
  } catch {
    return trimmed
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
    } catch {
      /* ignore */
    }

    try {
      const cookiesData = await getCheckinAccountCookies<CheckinAccountCookiesResponse>(account.id)
      accountForm.value = {
        provider_id: account.provider_id,
        name: account.name,
        session: extractCookiesFieldValue(cookiesData.cookies_json),
        api_user:
          typeof cookiesData.api_user === 'string' && cookiesData.api_user.trim()
            ? cookiesData.api_user
            : account.api_user || '',
        enabled: account.enabled,
        fuli_cookies: existingExtra.fuli_cookies ? JSON.stringify(existingExtra.fuli_cookies) : '',
        b4u_cdk_cookies: existingExtra.b4u_cdk_cookies
          ? JSON.stringify(existingExtra.b4u_cdk_cookies)
          : '',
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
        b4u_cdk_cookies: existingExtra.b4u_cdk_cookies
          ? JSON.stringify(existingExtra.b4u_cdk_cookies)
          : '',
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
        uiStore.showError(t('checkin.accounts.errors.invalidFuliCookies'))
        return
      }
    }
    if (accountForm.value.b4u_cdk_cookies) {
      try {
        extraConfig.b4u_cdk_cookies = JSON.parse(accountForm.value.b4u_cdk_cookies)
      } catch {
        uiStore.showError(t('checkin.accounts.errors.invalidB4uCookies'))
        return
      }
    }
    if (accountForm.value.x666_access_token) {
      extraConfig.x666_access_token = accountForm.value.x666_access_token
    }
    const extraConfigJson = Object.keys(extraConfig).length > 0 ? JSON.stringify(extraConfig) : '{}'

    if (!apiUser) {
      uiStore.showError(t('checkin.accounts.errors.apiUserRequired'))
      return
    }

    if (editingAccount.value) {
      const updateData: {
        name?: string
        cookies_json?: string
        api_user?: string
        enabled?: boolean
        extra_config?: string
      } = {
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
        uiStore.showError(t('checkin.accounts.errors.sessionRequired'))
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
    uiStore.showError(t('checkin.accounts.errors.saveFailed', { error: getErrorMessage(e, t('checkin.errors.unknown')) }))
  }
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

// 点击页面其他地方关闭菜单
const closeMenuOnClickOutside = (e: MouseEvent) => {
  if (
    openMenuAccountId.value &&
    !(e.target as HTMLElement).closest(
      '.checkin-accounts-tab__menu-wrap, .checkin-accounts-tab__menu--floating'
    )
  ) {
    closeAccountMenu()
  }
}

onMounted(() => {
  document.addEventListener('click', closeMenuOnClickOutside)
  window.addEventListener('resize', closeAccountMenu)
  window.addEventListener('scroll', closeAccountMenu, true)
})

onUnmounted(() => {
  document.removeEventListener('click', closeMenuOnClickOutside)
  window.removeEventListener('resize', closeAccountMenu)
  window.removeEventListener('scroll', closeAccountMenu, true)
})
</script>

<style scoped>
.checkin-accounts-tab,
.checkin-accounts-tab__form,
.checkin-accounts-tab__modal-body,
.checkin-accounts-tab__form-section,
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
.checkin-accounts-tab__modal-header,
.checkin-accounts-tab__modal-badge-row,
.checkin-accounts-tab__modal-title,
.checkin-accounts-tab__modal-footer,
.checkin-accounts-tab__modal-intro,
.checkin-accounts-tab__hint--with-icon,
.checkin-accounts-tab__notice-title,
.checkin-accounts-tab__toggle {
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
  border: 1px solid rgb(var(--color-border-default-rgb) / 72%);
  color: var(--text-primary);
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    background-color 0.2s ease,
    transform 0.2s ease;
}

.checkin-accounts-tab__input {
  background: rgb(var(--color-bg-elevated-rgb) / 36%);
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

.checkin-accounts-tab__actions {
  flex-wrap: wrap;
  gap: 0.75rem;
}

.checkin-accounts-tab__action-button,
.checkin-accounts-tab__mini-button,
.checkin-accounts-tab__form-button {
  border-radius: 0.75rem;
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    color 0.2s ease,
    opacity 0.2s ease,
    transform 0.2s ease;
}

.checkin-accounts-tab__action-button {
  min-height: 44px;
  gap: 0.5rem;
  border: 1px solid transparent;
  padding: 0.5rem 1rem;
  color: white;
  font-weight: 700;
  white-space: nowrap;
}

.checkin-accounts-tab__action-button:disabled,
.checkin-accounts-tab__mini-button:disabled {
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
  border-color: rgb(125 211 252 / 32%);
  background:
    radial-gradient(circle at 16% 0%, rgb(56 189 248 / 22%), transparent 34%),
    linear-gradient(135deg, rgb(15 23 42 / 94%), rgb(17 24 39 / 86%));
  color: rgb(236 253 245 / 96%);
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 12%),
    0 0 0 1px rgb(20 184 166 / 10%),
    0 14px 28px rgb(15 23 42 / 18%),
    0 0 22px rgb(20 184 166 / 10%);
}

.checkin-accounts-tab__action-button--secondary:hover:not(:disabled) {
  border-color: rgb(94 234 212 / 48%);
  color: white;
  transform: translateY(-1px);
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 16%),
    0 0 0 1px rgb(20 184 166 / 18%),
    0 18px 36px rgb(15 23 42 / 24%),
    0 0 28px rgb(20 184 166 / 18%);
}

.checkin-accounts-tab__action-button--secondary:disabled {
  border-color: rgb(var(--color-border-default-rgb) / 54%);
  background:
    linear-gradient(135deg, rgb(var(--color-bg-elevated-rgb) / 54%), rgb(var(--color-bg-surface-rgb) / 48%));
  color: var(--text-muted);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 8%);
}

.checkin-accounts-tab__action-button-icon {
  filter: drop-shadow(0 0 8px rgb(20 184 166 / 36%));
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
  font-family: var(--font-mono);
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
  justify-content: center;
  gap: 0.35rem;
  min-width: 5.75rem;
  min-height: 36px;
  border: 1px solid rgb(45 212 191 / 32%);
  padding: 0.375rem 0.7rem;
  font-size: 0.75rem;
  font-weight: 700;
  line-height: 1rem;
  color: rgb(236 253 245);
  white-space: nowrap;
  background:
    radial-gradient(circle at 20% 0%, rgb(94 234 212 / 24%), transparent 36%),
    linear-gradient(135deg, rgb(15 118 110 / 94%), rgb(22 163 74 / 88%));
  box-shadow:
    inset 0 1px 0 rgb(236 253 245 / 14%),
    0 10px 22px rgb(15 118 110 / 18%);
}

.checkin-accounts-tab__mini-button:hover:not(:disabled) {
  border-color: rgb(94 234 212 / 54%);
  transform: translateY(-1px);
  box-shadow:
    inset 0 1px 0 rgb(236 253 245 / 18%),
    0 14px 26px rgb(15 118 110 / 26%);
}

.checkin-accounts-tab__mini-button:disabled {
  border-color: rgb(var(--color-border-default-rgb) / 44%);
  background: rgb(var(--color-bg-elevated-rgb) / 42%);
  color: var(--text-muted);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 8%);
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
  border-radius: 0.85rem;
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
  box-shadow: 0 12px 28px rgb(15 23 42 / 14%);
}

.checkin-accounts-tab__menu {
  z-index: 60;
  padding: 0.35rem;
  border-radius: 1rem;
  border: 1px solid rgb(var(--color-border-strong-rgb) / 48%);
  background: linear-gradient(
    180deg,
    rgb(var(--color-bg-elevated-rgb) / 94%),
    rgb(var(--color-bg-surface-rgb) / 92%)
  );
  box-shadow:
    0 20px 40px rgb(15 23 42 / 18%),
    inset 0 1px 0 rgb(255 255 255 / 16%);
  backdrop-filter: blur(18px) saturate(165%);
  overflow-y: auto;
}

.checkin-accounts-tab__menu--floating {
  position: fixed;
  inset: auto auto auto 0;
}

.checkin-accounts-tab__menu--top {
  transform-origin: bottom right;
}

.checkin-accounts-tab__menu--bottom {
  transform-origin: top right;
}

.checkin-accounts-tab__menu-item {
  width: 100%;
  border-radius: 0.8rem;
  padding: 0.625rem 0.75rem;
  text-align: left;
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--text-secondary);
  transition:
    background-color 0.2s ease,
    color 0.2s ease;
}

.checkin-accounts-tab__menu-item:hover {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: var(--text-primary);
}

.checkin-accounts-tab__menu-item--top {
  margin-bottom: 0.15rem;
}

.checkin-accounts-tab__menu-item--danger {
  color: rgb(var(--color-danger-rgb) / 92%);
}

.checkin-accounts-tab__menu-item--danger:hover {
  background: rgb(var(--color-danger-rgb) / 14%);
  color: rgb(var(--color-danger-rgb) / 100%);
}

.checkin-accounts-tab__modal-header {
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 1rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 72%);
  background:
    radial-gradient(
      circle at top right,
      rgb(var(--color-accent-secondary-rgb) / 18%),
      transparent 42%
    ),
    radial-gradient(
      circle at left center,
      rgb(var(--color-accent-primary-rgb) / 20%),
      transparent 36%
    ),
    linear-gradient(
      135deg,
      rgb(var(--color-bg-surface-rgb) / 98%),
      rgb(var(--color-bg-elevated-rgb) / 98%)
    );
  padding: 1.1rem 1.5rem 1rem;
}

.checkin-accounts-tab__modal-header-copy {
  display: flex;
  min-width: 0;
  flex: 1 1 20rem;
  flex-direction: column;
  gap: 0.4rem;
}

.checkin-accounts-tab__modal-eyebrow {
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: rgb(var(--color-accent-secondary-rgb) / 86%);
}

.checkin-accounts-tab__modal-title {
  gap: 0.5rem;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--text-primary);
}

.checkin-accounts-tab__modal-title-icon {
  color: rgb(var(--color-accent-primary-rgb) / 94%);
}

.checkin-accounts-tab__modal-subtitle {
  max-width: 36rem;
  font-size: 0.8125rem;
  line-height: 1.45;
  color: var(--text-secondary);
}

.checkin-accounts-tab__modal-badge-row {
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.5rem;
}

.checkin-accounts-tab__modal-badge,
.checkin-accounts-tab__modal-intro-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border-radius: 999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 72%);
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
  padding: 0.4rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.checkin-accounts-tab__modal-badge--warning {
  border-color: rgb(var(--color-warning-rgb) / 42%);
  background: rgb(var(--color-warning-rgb) / 14%);
  color: rgb(var(--color-warning-rgb) / 96%);
}

.checkin-accounts-tab__modal-body {
  gap: 0.85rem;
}

.checkin-accounts-tab__modal-intro {
  flex-wrap: wrap;
  gap: 0.55rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 58%);
  border-radius: 1rem;
  background:
    linear-gradient(135deg, rgb(var(--color-bg-surface-rgb) / 72%), transparent),
    rgb(var(--color-bg-elevated-rgb) / 54%);
  padding: 0.75rem;
}

.checkin-accounts-tab__modal-intro-pill {
  background: rgb(var(--color-bg-surface-rgb) / 82%);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 9%);
}

.checkin-accounts-tab__modal-scroll {
  max-height: min(60vh, 620px);
  overflow-y: auto;
  padding: 0.15rem 0.35rem 0.35rem 0.05rem;
  scrollbar-gutter: stable;
}

.checkin-accounts-tab__modal-footer {
  width: 100%;
  justify-content: flex-end;
  gap: 0.75rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 58%);
  border-radius: 1rem;
  background:
    linear-gradient(
      180deg,
      rgb(var(--color-bg-elevated-rgb) / 88%),
      rgb(var(--color-bg-surface-rgb) / 72%)
    );
  padding: 0.65rem;
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 9%),
    0 14px 30px rgb(15 23 42 / 10%);
}

.checkin-accounts-tab__form {
  gap: 0.95rem;
  padding: 0.05rem 0 0.25rem;
}

.checkin-accounts-tab__form-section {
  gap: 0.9rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 54%);
  border-radius: 1.1rem;
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 78%), rgb(var(--color-bg-surface-rgb) / 58%));
  padding: 1rem;
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 8%),
    0 12px 28px rgb(15 23 42 / 7%);
}

.checkin-accounts-tab__form-section--credentials {
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background:
    radial-gradient(circle at 100% 0%, rgb(var(--color-accent-primary-rgb) / 10%), transparent 32%),
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 84%), rgb(var(--color-bg-surface-rgb) / 60%));
}

.checkin-accounts-tab__form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.9rem;
}

.checkin-accounts-tab__field {
  gap: 0.5rem;
}

.checkin-accounts-tab__field--credential {
  gap: 0.65rem;
}

.checkin-accounts-tab__account-modal {
  width: min(calc(100vw - 2rem), 54rem);
  max-width: min(calc(100vw - 2rem), 54rem);
  max-height: min(92vh, 920px);
  border-color: rgb(var(--color-border-strong-rgb) / 72%);
  box-shadow:
    0 28px 80px rgb(15 23 42 / 28%),
    0 0 0 1px rgb(var(--color-accent-primary-rgb) / 10%);
}

:deep(.checkin-accounts-tab__account-modal > div:nth-child(2)) {
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
}

:deep(.checkin-accounts-tab__account-modal > div:last-child) {
  border-top-color: rgb(var(--color-border-strong-rgb) / 58%);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 96%), rgb(var(--color-bg-surface-rgb) / 92%));
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
  background: linear-gradient(
    180deg,
    rgb(var(--color-bg-elevated-rgb) / 96%),
    rgb(var(--color-bg-surface-rgb) / 92%)
  );
  padding: 0.72rem 0.85rem;
  color: var(--text-primary);
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 10%),
    0 10px 24px rgb(15 23 42 / 8%);
}

.checkin-accounts-tab__control:disabled {
  cursor: not-allowed;
  opacity: 0.6;
  background: rgb(var(--color-bg-surface-rgb) / 72%);
}

.checkin-accounts-tab__control option,
.checkin-accounts-tab__select option {
  background: rgb(var(--color-bg-elevated-rgb) / 100%);
  color: var(--text-primary);
}

.checkin-accounts-tab__control option:disabled,
.checkin-accounts-tab__select option:disabled {
  color: var(--text-muted);
}

.checkin-accounts-tab__control--textarea {
  resize: vertical;
  min-height: 120px;
}

.checkin-accounts-tab__control--credential {
  min-height: 11.5rem;
  max-height: 19rem;
  padding: 0.95rem 1rem;
  overflow: auto;
  font-size: 0.8125rem;
  line-height: 1.6;
  tab-size: 2;
}

.checkin-accounts-tab__control--compact {
  padding-block: 0.5rem;
  font-size: 0.75rem;
}

.checkin-accounts-tab__control--mono,
.checkin-accounts-tab__kbd,
.checkin-accounts-tab__code {
  font-family: var(--font-mono);
}

.checkin-accounts-tab__control--mono {
  background: linear-gradient(
    180deg,
    rgb(var(--color-bg-base-rgb) / 94%),
    rgb(var(--color-bg-elevated-rgb) / 92%)
  );
  letter-spacing: 0.01em;
}

.checkin-accounts-tab__control--amber {
  border-color: rgb(var(--color-warning-rgb) / 46%);
}

.checkin-accounts-tab__control--amber:focus {
  border-color: rgb(var(--color-warning-rgb) / 86%);
  box-shadow:
    0 0 0 3px rgb(var(--color-warning-rgb) / 16%),
    0 14px 28px rgb(var(--color-warning-rgb) / 12%);
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

.checkin-accounts-tab__hint--credential {
  align-items: flex-start;
  border-radius: 0.85rem;
  background: rgb(var(--color-bg-base-rgb) / 38%);
  padding: 0.6rem 0.7rem;
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
  border-radius: 0.9rem;
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

.checkin-accounts-tab__notice--help {
  gap: 0.7rem;
  border-color: rgb(var(--color-border-default-rgb) / 54%);
  background:
    linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 8%), transparent),
    rgb(var(--color-bg-elevated-rgb) / 44%);
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
  accent-color: rgb(var(--color-accent-primary-rgb) / 100%);
}

.checkin-accounts-tab__checkbox-label {
  margin-left: 0.625rem;
  cursor: pointer;
  user-select: none;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.checkin-accounts-tab__form-button {
  min-height: 42px;
  border-radius: 0.9rem;
  padding: 0.6rem 1.05rem;
  font-size: 0.875rem;
  font-weight: 650;
  line-height: 1.2;
  white-space: nowrap;
}

.checkin-accounts-tab__form-button--secondary {
  border: 1px solid rgb(var(--color-border-default-rgb) / 82%);
  background: rgb(var(--color-bg-surface-rgb) / 78%);
  color: var(--text-secondary);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 10%);
}

.checkin-accounts-tab__form-button--secondary:hover {
  background: rgb(var(--color-bg-elevated-rgb) / 90%);
  color: var(--text-primary);
}

.checkin-accounts-tab__form-button--primary {
  min-width: 9.5rem;
  color: white;
  background: linear-gradient(
    135deg,
    rgb(var(--color-accent-primary-rgb) / 96%),
    rgb(var(--color-accent-secondary-rgb) / 92%)
  );
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 16%),
    0 16px 28px rgb(var(--color-accent-primary-rgb) / 24%);
}

.checkin-accounts-tab__form-button--primary:hover {
  background: linear-gradient(
    135deg,
    rgb(var(--color-accent-primary-rgb) / 100%),
    rgb(var(--color-accent-secondary-rgb) / 100%)
  );
  transform: translateY(-1px);
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

  .checkin-accounts-tab__modal-header,
  .checkin-accounts-tab__modal-footer {
    align-items: stretch;
  }

  .checkin-accounts-tab__modal-badge-row,
  .checkin-accounts-tab__modal-footer {
    justify-content: stretch;
  }

  .checkin-accounts-tab__modal-badge-row,
  .checkin-accounts-tab__modal-footer,
  .checkin-accounts-tab__modal-intro {
    flex-direction: column;
  }

  .checkin-accounts-tab__modal-intro,
  .checkin-accounts-tab__modal-intro-pill,
  .checkin-accounts-tab__modal-footer {
    width: 100%;
  }

  .checkin-accounts-tab__modal-intro-pill {
    justify-content: center;
  }

  .checkin-accounts-tab__modal-scroll {
    max-height: min(58vh, 560px);
  }

  .checkin-accounts-tab__form-grid {
    grid-template-columns: 1fr;
  }

  .checkin-accounts-tab__form-section {
    padding: 0.85rem;
  }

  .checkin-accounts-tab__control--credential {
    min-height: 10rem;
    max-height: 15rem;
  }

  .checkin-accounts-tab__form-button {
    width: 100%;
  }
}
</style>
