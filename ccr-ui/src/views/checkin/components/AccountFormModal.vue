<template>
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
          <span class="checkin-accounts-tab__modal-badge checkin-badge-pill">
            {{ modalProviderLabel }}
          </span>
          <span
            v-if="selectedBuiltinProvider?.requires_waf_bypass"
            class="checkin-accounts-tab__modal-badge checkin-badge-pill checkin-accounts-tab__modal-badge--warning"
          >
            {{ t('checkin.accounts.modal.requiresWaf') }}
          </span>
        </div>
      </div>
    </template>

    <div class="checkin-accounts-tab__modal-body">
      <div class="checkin-accounts-tab__modal-intro">
        <span class="checkin-accounts-tab__modal-intro-pill checkin-badge-pill">{{ t('checkin.accounts.modal.introSession') }}</span>
        <span class="checkin-accounts-tab__modal-intro-pill checkin-badge-pill">{{ t('checkin.accounts.modal.introApiUser') }}</span>
        <span class="checkin-accounts-tab__modal-intro-pill checkin-badge-pill">{{ t('checkin.accounts.modal.introNoOverwrite') }}</span>
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
                  <span class="text-accent-danger">*</span> {{ t('checkin.accounts.fields.provider') }}
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
                  <span class="text-accent-danger">*</span> {{ t('checkin.accounts.fields.accountName') }}
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
                  class="text-accent-danger"
                >*</span> Session / Cookies
                <span
                  v-if="editingAccount"
                  class="text-text-muted font-normal"
                >{{ t('checkin.accounts.fields.leaveBlank') }}</span>
              </label>
              <textarea
                ref="sessionTextareaRef"
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
                <span class="text-accent-danger">*</span> API User
              </label>
              <input
                v-model="accountForm.api_user"
                type="text"
                required
                class="checkin-accounts-tab__control checkin-accounts-tab__control--mono"
                placeholder="12345"
              >
              <p class="checkin-accounts-tab__hint">
                {{ t('checkin.accounts.fields.apiUserHintPrefix') }}
                <code>user.id</code>
                {{ t('checkin.accounts.fields.apiUserHintMiddle') }}
                <code>new-api-user</code>
                {{ t('checkin.accounts.fields.apiUserHintSuffix') }}
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
import { ref, computed, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  createCheckinAccount,
  updateCheckinAccount,
  getCheckinAccountCookies,
} from '@/api'
import type { CheckinProvider, AccountInfo, BuiltinProvider, CdkExtraConfig } from '@/types/checkin'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'
import { getErrorMessage } from '@/types/api'
import { resolveBuiltinProvider } from '../composables/builtinProviderLookup'

const props = defineProps<{
  providers: CheckinProvider[]
  builtinProviders: BuiltinProvider[]
}>()

const emit = defineEmits<{
  (e: 'refresh'): void
}>()

const uiStore = useUIStore()
const { t } = useI18n()

interface CheckinAccountCookiesResponse {
  cookies_json: string
  api_user?: string | null
}

const showAccountModal = ref(false)
const editingAccount = ref<AccountInfo | null>(null)
const sessionTextareaRef = ref<HTMLTextAreaElement | null>(null)

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

// 选中提供商对应的内置站（builtin_id 优先反查，name 回退兼容旧数据）
const selectedBuiltinProvider = computed(() => {
  if (!accountForm.value.provider_id) return null
  const provider = props.providers.find((p) => p.id === accountForm.value.provider_id)
  if (!provider) return null
  return resolveBuiltinProvider(props.builtinProviders, provider) || null
})

// CDK 配置：取自选中提供商对应的内置站
const selectedProviderCdkConfig = computed(() => {
  return selectedBuiltinProvider.value?.cdk_config || null
})

const modalProviderLabel = computed(() => {
  if (!accountForm.value.provider_id) return t('checkin.accounts.modal.providerPending')
  return selectedBuiltinProvider.value?.name || getProviderName(accountForm.value.provider_id)
})

const getProviderName = (providerId: string) => {
  return props.providers.find((p) => p.id === providerId)?.name || providerId
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

// 打开弹窗（编辑已有账号或新建）；focusSession 用于 cookie_expired 快捷修复直达 cookies 输入
const open = async (account?: AccountInfo, options?: { focusSession?: boolean }) => {
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
      logger.error('Failed to get cookies for check-in account', {
        account: {
          id: account.id,
          provider_id: account.provider_id,
          provider_name: getProviderName(account.provider_id),
          name: account.name,
        },
        err: e,
      })
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

  if (options?.focusSession) {
    await nextTick()
    sessionTextareaRef.value?.focus()
  }
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

defineExpose({ open })
</script>

<style scoped>
.checkin-accounts-tab__form,
.checkin-accounts-tab__modal-body,
.checkin-accounts-tab__form-section,
.checkin-accounts-tab__field,
.checkin-accounts-tab__notice {
  display: flex;
  flex-direction: column;
}

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

.checkin-accounts-tab__modal-header {
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 1rem;
  border-bottom: 1px solid var(--color-border-default);
  background: var(--color-bg-elevated);
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
  gap: 0.35rem;
  border: 1px solid var(--color-border-default);
  background: var(--color-bg-elevated);
  padding: 0.4rem 0.75rem;
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
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background: var(--color-bg-surface);
  padding: 0.75rem;
}

.checkin-accounts-tab__modal-intro-pill {
  background: var(--color-bg-surface);
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
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background: var(--color-bg-elevated);
  padding: 0.65rem;
  box-shadow: var(--shadow-sm);
}

.checkin-accounts-tab__form {
  gap: 0.95rem;
  padding: 0.05rem 0 0.25rem;
}

.checkin-accounts-tab__form-section {
  gap: 0.9rem;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background: var(--color-bg-elevated);
  padding: 1rem;
  box-shadow: var(--shadow-sm);
}

.checkin-accounts-tab__form-section--credentials {
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: var(--color-bg-elevated);
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
  border-color: var(--color-border-strong);
  box-shadow: var(--shadow-xl);
}

:deep(.checkin-accounts-tab__account-modal > div:nth-child(2)) {
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
}

:deep(.checkin-accounts-tab__account-modal > div:last-child) {
  border-top-color: var(--color-border-default);
  background: var(--color-bg-elevated);
}

.checkin-accounts-tab__label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.checkin-accounts-tab__label--amber {
  color: var(--color-warning);
}

.checkin-accounts-tab__control {
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-default);
  color: var(--text-primary);
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    background-color 0.2s ease,
    transform 0.2s ease;
  display: block;
  width: 100%;
  background: var(--color-bg-surface);
  padding: 0.72rem 0.85rem;
  box-shadow: var(--shadow-xs);
}

.checkin-accounts-tab__control::placeholder {
  color: var(--text-muted);
}

.checkin-accounts-tab__control:focus {
  outline: none;
  border-color: rgb(var(--color-accent-primary-rgb) / 88%);
  box-shadow:
    0 0 0 3px rgb(var(--color-accent-primary-rgb) / 18%),
    0 14px 28px rgb(var(--color-accent-primary-rgb) / 12%);
}

.checkin-accounts-tab__control:disabled {
  cursor: not-allowed;
  opacity: 0.6;
  background: rgb(var(--color-bg-surface-rgb) / 72%);
}

.checkin-accounts-tab__control option {
  background: rgb(var(--color-bg-elevated-rgb) / 100%);
  color: var(--text-primary);
}

.checkin-accounts-tab__control option:disabled {
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

.checkin-accounts-tab__control--mono {
  font-family: var(--font-mono);
  background: var(--color-bg-base);
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
  border-radius: var(--radius-lg);
  background: rgb(var(--color-bg-base-rgb) / 38%);
  padding: 0.6rem 0.7rem;
}

.checkin-accounts-tab__hint-icon,
.checkin-accounts-tab__notice-icon {
  width: 0.875rem;
  height: 0.875rem;
}

.checkin-accounts-tab__hint--amber {
  color: var(--color-warning);
}

.checkin-accounts-tab__notice {
  gap: 1rem;
  border-radius: var(--radius-lg);
  border: 1px solid;
  padding: 1rem;
}

.checkin-accounts-tab__notice--warning,
.checkin-accounts-tab__notice--amber {
  border-color: rgb(var(--color-warning-rgb) / 40%);
  background: rgb(var(--color-warning-rgb) / 12%);
}

.checkin-accounts-tab__notice--info {
  border-color: rgb(var(--color-info-rgb) / 40%);
  background: rgb(var(--color-info-rgb) / 12%);
}

.checkin-accounts-tab__notice--help {
  gap: 0.7rem;
  border-color: var(--color-border-default);
  background: var(--color-bg-elevated);
}

.checkin-accounts-tab__notice-title {
  gap: 0.375rem;
  font-size: 0.875rem;
  font-weight: 500;
}

.checkin-accounts-tab__notice-title--warning,
.checkin-accounts-tab__notice-title--amber {
  color: var(--color-warning);
}

.checkin-accounts-tab__notice-title--info {
  color: var(--color-info);
}

.checkin-accounts-tab__notice-title-meta {
  font-size: 0.75rem;
  font-weight: 400;
  color: rgb(var(--color-warning-rgb) / 86%);
}

.checkin-accounts-tab__notice-copy,
.checkin-accounts-tab__notice-list--warning {
  color: rgb(var(--color-warning-rgb) / 92%);
}

.checkin-accounts-tab__notice-list {
  list-style-position: inside;
  list-style-type: decimal;
}

.checkin-accounts-tab__notice-list--info {
  margin-left: 0.125rem;
  color: rgb(var(--color-info-rgb) / 92%);
}

.checkin-accounts-tab__toggle {
  padding-block: 0.25rem;
}

.checkin-accounts-tab__checkbox {
  width: 1rem;
  height: 1rem;
  cursor: pointer;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-border-default);
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
  border-radius: var(--radius-md);
  padding: 0.6rem 1.05rem;
  font-size: 0.875rem;
  font-weight: 650;
  line-height: 1.2;
  white-space: nowrap;
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    color 0.2s ease,
    opacity 0.2s ease,
    transform 0.2s ease;
}

.checkin-accounts-tab__form-button--secondary {
  border: 1px solid var(--color-border-default);
  background: var(--color-bg-surface);
  color: var(--text-secondary);
  box-shadow: var(--shadow-xs);
}

.checkin-accounts-tab__form-button--secondary:hover {
  background: var(--color-bg-elevated);
  color: var(--text-primary);
}

.checkin-accounts-tab__form-button--primary {
  min-width: 9.5rem;
  color: white;
  background: var(--color-accent-primary);
  box-shadow: var(--shadow-sm);
}

.checkin-accounts-tab__form-button--primary:hover {
  background: var(--color-accent-primary-hover);
  transform: translateY(-1px);
}

@media (width <= 900px) {
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
