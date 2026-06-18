<template>
  <BaseModal
    v-model="isVisible"
    :title="t('checkin.actions.oauthLoginTitle')"
    size="lg"
    :persistent="loading"
    @close="handleClose"
  >
    <!-- 步骤指示器 -->
    <div class="oauth-wizard__steps">
      <template
        v-for="(stepLabel, idx) in stepLabels"
        :key="idx"
      >
        <div
          :class="[
            'oauth-wizard__step',
            step > idx
              ? 'oauth-wizard__step--complete'
              : step === idx
                ? 'oauth-wizard__step--current'
                : 'oauth-wizard__step--inactive',
          ]"
        >
          <div
            :class="[
              'oauth-wizard__step-circle',
              step > idx
                ? 'oauth-wizard__step-circle--complete'
                : step === idx
                  ? 'oauth-wizard__step-circle--current'
                  : 'oauth-wizard__step-circle--inactive',
            ]"
          >
            <SIcon
              v-if="step> idx"
              name="CheckCircle"
              class="oauth-wizard__step-icon"
            />
            <span v-else>{{ idx + 1 }}</span>
          </div>
          <span class="oauth-wizard__step-label">{{ stepLabel }}</span>
        </div>
        <SIcon
          v-if="idx < stepLabels.length - 1"
          name="ChevronRight"
          size="w-4 h-4"
          class="oauth-wizard__step-divider"
        />
      </template>
    </div>

    <!-- Step 0: 选择提供商和 OAuth 方式 -->
    <div
      v-if="step === 0"
      class="oauth-wizard__section"
    >
      <!-- 提供商选择 -->
      <div class="oauth-wizard__field">
        <label class="oauth-wizard__label">{{ t('checkin.oauthWizard.providerLabel') }}</label>
        <select
          v-model="selectedProviderId"
          class="oauth-wizard__input"
        >
          <option
            value=""
            disabled
          >
            {{ chosenOptionText }}
          </option>
          <option
            v-for="provider in oauthProviders"
            :key="provider.id"
            :value="provider.id"
          >
            {{ provider.icon }} {{ provider.name }} ({{ provider.domain }})
          </option>
        </select>
      </div>

      <!-- OAuth 方式选择 -->
      <div
        v-if="selectedProvider"
        class="oauth-wizard__field"
      >
        <label class="oauth-wizard__label">{{ t('checkin.oauthWizard.loginMethodLabel') }}</label>
        <div class="oauth-wizard__choice-grid">
          <button
            v-if="selectedProvider.oauth_config?.linuxdo_client_id"
            :class="[
              'oauth-wizard__choice',
              selectedOAuthType === 'linuxdo'
                ? 'oauth-wizard__choice--active'
                : 'oauth-wizard__choice--inactive',
            ]"
            @click="selectedOAuthType = 'linuxdo'"
          >
            <SIcon
              name="Globe"
              size="w-5 h-5"
            />
            {{ t('checkin.oauthWizard.oauthTypes.linuxdo') }}
          </button>
          <button
            v-if="selectedProvider.oauth_config?.github_client_id"
            :class="[
              'oauth-wizard__choice',
              selectedOAuthType === 'github'
                ? 'oauth-wizard__choice--active'
                : 'oauth-wizard__choice--inactive',
            ]"
            @click="selectedOAuthType = 'github'"
          >
            <SIcon
              name="Github"
              size="w-5 h-5"
            />
            {{ t('checkin.oauthWizard.oauthTypes.github') }}
          </button>
        </div>
        <p
          v-if="!selectedProvider.oauth_config?.linuxdo_client_id && !selectedProvider.oauth_config?.github_client_id"
          class="oauth-wizard__warning"
        >
          {{ t('checkin.oauthWizard.oauthNotConfigured') }}
        </p>
      </div>
    </div>

    <!-- Step 1: 获取授权链接 -->
    <div
      v-else-if="step === 1"
      class="oauth-wizard__section"
    >
      <div
        v-if="loading"
        class="oauth-wizard__state"
      >
        <SIcon
          name="Loader2"
          size="w-8 h-8"
          class="oauth-wizard__state-icon oauth-wizard__state-icon--loading animate-spin"
        />
        <p class="oauth-wizard__state-text">
          {{ t('checkin.oauthWizard.loadingAuthorizeUrl') }}
        </p>
      </div>

      <div
        v-else-if="oauthError"
        class="oauth-wizard__panel oauth-wizard__panel--error"
      >
        <p class="oauth-wizard__error-text">
          {{ oauthError }}
        </p>
        <button
          class="oauth-wizard__link-button"
          @click="step = 0"
        >
          {{ t('checkin.oauthWizard.backToSelection') }}
        </button>
      </div>

      <div
        v-else-if="authorizeUrl"
        class="oauth-wizard__section"
      >
        <div class="oauth-wizard__panel oauth-wizard__panel--info">
          <p class="oauth-wizard__panel-title oauth-wizard__panel-title--info">
            <SIcon
              name="ExternalLink"
              size="w-4 h-4"
            />
            {{ t('checkin.oauthWizard.openInBrowserHint') }}
          </p>
          <div class="oauth-wizard__url-row">
            <input
              :value="authorizeUrl"
              readonly
              class="oauth-wizard__url-input"
            >
            <button
              class="oauth-wizard__button oauth-wizard__button--primary oauth-wizard__button--compact"
              @click="copyUrl"
            >
              {{ copied ? t('checkin.oauthWizard.copied') : t('checkin.oauthWizard.copy') }}
            </button>
          </div>
          <a
            :href="authorizeUrl"
            target="_blank"
            rel="noopener noreferrer"
            class="oauth-wizard__external-link"
          >
            <SIcon
              name="ExternalLink"
              size="w-3.5 h-3.5"
            />
            {{ t('checkin.oauthWizard.openInNewTab') }}
          </a>
        </div>

        <!-- 引导说明 -->
        <div class="oauth-wizard__panel oauth-wizard__panel--neutral">
          <p class="oauth-wizard__panel-title oauth-wizard__panel-title--neutral">
            {{ t('checkin.oauthWizard.guideTitle') }}
          </p>
          <ol class="oauth-wizard__guide-list">
            <li
              v-for="(guide, idx) in extractionGuide"
              :key="idx"
              class="oauth-wizard__guide-item"
              :data-index="(idx + 1) + '.'"
            >
              {{ guide }}
            </li>
          </ol>
        </div>
      </div>
    </div>

    <!-- Step 2: 粘贴 Cookies -->
    <div
      v-else-if="step === 2"
      class="oauth-wizard__section"
    >
      <div class="oauth-wizard__field">
        <label class="oauth-wizard__label">
          {{ t('checkin.oauthWizard.credentialsLabel') }}
        </label>
        <textarea
          v-model="pastedCredentials"
          rows="6"
          :placeholder="t('checkin.oauthWizard.credentialsPlaceholder')"
          class="oauth-wizard__input oauth-wizard__input--textarea oauth-wizard__input--mono"
        />
      </div>

      <div class="oauth-wizard__field">
        <label class="oauth-wizard__label">
          {{ t('checkin.oauthWizard.apiUserLabel') }}
        </label>
        <input
          v-model="pastedApiUser"
          :placeholder="t('checkin.oauthWizard.apiUserPlaceholder')"
          class="oauth-wizard__input"
        >
      </div>

      <div class="oauth-wizard__field">
        <label class="oauth-wizard__label">{{ t('checkin.oauthWizard.accountNameLabel') }}</label>
        <input
          v-model="accountName"
          :placeholder="defaultAccountName"
          class="oauth-wizard__input"
        >
      </div>

      <div
        v-if="parseError"
        class="oauth-wizard__panel oauth-wizard__panel--error"
      >
        <p class="oauth-wizard__error-text oauth-wizard__error-text--small">
          {{ parseError }}
        </p>
      </div>
    </div>

    <!-- Step 3: 确认创建 -->
    <div
      v-else-if="step === 3"
      class="oauth-wizard__section"
    >
      <div
        v-if="creatingAccount"
        class="oauth-wizard__state"
      >
        <SIcon
          name="Loader2"
          size="w-8 h-8"
          class="oauth-wizard__state-icon oauth-wizard__state-icon--loading animate-spin"
        />
        <p class="oauth-wizard__state-text">
          {{ t('checkin.oauthWizard.creatingAccount') }}
        </p>
      </div>

      <div
        v-else-if="createSuccess"
        class="oauth-wizard__state"
      >
        <SIcon
          name="CheckCircle"
          size="w-12 h-12"
          class="oauth-wizard__state-icon oauth-wizard__state-icon--success"
        />
        <p class="oauth-wizard__state-text oauth-wizard__state-text--success">
          {{ t('checkin.oauthWizard.createSuccess') }}
        </p>
        <p class="oauth-wizard__state-subtitle">
          {{ selectedProvider?.name }} - {{ accountName || t('checkin.oauthWizard.newAccount') }}
        </p>
      </div>

      <div
        v-else
        class="oauth-wizard__section oauth-wizard__section--compact"
      >
        <div class="oauth-wizard__panel oauth-wizard__panel--neutral oauth-wizard__summary">
          <div class="oauth-wizard__summary-row">
            <span class="oauth-wizard__summary-label">{{ t('checkin.providers.provider') }}</span>
            <span class="oauth-wizard__summary-value">{{ selectedProvider?.name }}</span>
          </div>
          <div class="oauth-wizard__summary-row">
            <span class="oauth-wizard__summary-label">{{ t('checkin.oauthWizard.summary.accountName') }}</span>
            <span class="oauth-wizard__summary-value">{{ accountName || defaultAccountName }}</span>
          </div>
          <div class="oauth-wizard__summary-row">
            <span class="oauth-wizard__summary-label">{{ t('checkin.oauthWizard.summary.cookieCount') }}</span>
            <span class="oauth-wizard__summary-value">{{ t('checkin.oauthWizard.summary.cookieCountValue', { count: parsedCookieCount }) }}</span>
          </div>
          <div class="oauth-wizard__summary-row">
            <span class="oauth-wizard__summary-label">{{ t('checkin.oauthWizard.summary.apiUser') }}</span>
            <span class="oauth-wizard__summary-value">{{ pastedApiUser || t('checkin.oauthWizard.unsetValue') }}</span>
          </div>
        </div>

        <div
          v-if="createError"
          class="oauth-wizard__panel oauth-wizard__panel--error"
        >
          <p class="oauth-wizard__error-text oauth-wizard__error-text--small">
            {{ createError }}
          </p>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="oauth-wizard__footer">
        <button
          v-if="step > 0 && !createSuccess"
          class="oauth-wizard__button oauth-wizard__button--ghost"
          :disabled="loading || creatingAccount"
          @click="step--"
        >
          {{ t('common.previous') }}
        </button>
        <div v-else />

        <div class="oauth-wizard__footer-actions">
          <button
            class="oauth-wizard__button oauth-wizard__button--secondary"
            @click="handleClose"
          >
            {{ createSuccess ? t('common.close') : t('common.cancel') }}
          </button>

          <button
            v-if="step === 0"
            :disabled="!canProceedStep0"
            class="oauth-wizard__button oauth-wizard__button--primary"
            @click="goToStep1"
          >
            {{ t('checkin.actions.oauthLogin') }}
          </button>

          <button
            v-else-if="step === 1 && authorizeUrl"
            class="oauth-wizard__button oauth-wizard__button--primary"
            @click="step = 2"
          >
            {{ t('common.next') }}
          </button>

          <button
            v-else-if="step === 2"
            :disabled="!pastedCredentials.trim()"
            class="oauth-wizard__button oauth-wizard__button--primary"
            @click="goToStep3"
          >
            {{ t('common.next') }}
          </button>

          <button
            v-else-if="step === 3 && !createSuccess"
            :disabled="creatingAccount"
            class="oauth-wizard__button oauth-wizard__button--success"
            @click="createAccount"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import { getOAuthAuthorizeUrl, createCheckinAccount } from '@/api'
import type { BuiltinProvider } from '@/types/checkin'
import { copyText } from '@/utils/clipboard'

const props = defineProps<{
  isOpen: boolean
  builtinProviders: BuiltinProvider[]
}>()

const emit = defineEmits<{
  (e: 'update:isOpen', value: boolean): void
  (e: 'close'): void
  (e: 'success'): void
}>()

const { t } = useI18n()
const isVisible = computed({
  get: () => props.isOpen,
  set: (val: boolean) => emit('update:isOpen', val),
})

// Steps
const stepLabels = computed(() => [
  t('checkin.oauthWizard.steps.selectMethod'),
  t('checkin.oauthWizard.steps.getLink'),
  t('checkin.oauthWizard.steps.pasteCredentials'),
  t('checkin.oauthWizard.steps.confirmCreate'),
])
const step = ref(0)

// Step 0 state
const selectedProviderId = ref('')
const selectedOAuthType = ref<'github' | 'linuxdo'>('linuxdo')

// Step 1 state
const loading = ref(false)
const oauthError = ref('')
const authorizeUrl = ref('')
const extractionGuide = ref<string[]>([])
const copied = ref(false)

// Step 2 state
const pastedCredentials = ref('')
const pastedApiUser = ref('')
const accountName = ref('')
const parseError = ref('')

// Step 3 state
const creatingAccount = ref(false)
const createSuccess = ref(false)
const createError = ref('')

const chosenOptionText = computed(() => t('common.selectOption'))
const defaultAccountName = computed(() =>
  selectedProvider.value?.name
    ? t('checkin.oauthWizard.defaultAccountName', { provider: selectedProvider.value.name })
    : ''
)

// Computed
const oauthProviders = computed(() =>
  props.builtinProviders.filter((p) => p.oauth_config != null)
)

const selectedProvider = computed(() =>
  props.builtinProviders.find((p) => p.id === selectedProviderId.value)
)

const canProceedStep0 = computed(
  () => selectedProviderId.value && selectedOAuthType.value
)

const parsedCookieCount = computed(() => {
  try {
    const parsed = parseCookies(pastedCredentials.value)
    return Object.keys(parsed).length
  } catch {
    return 0
  }
})

// Reset when modal opens/closes
watch(
  () => props.isOpen,
  (open) => {
    if (open) {
      step.value = 0
      selectedProviderId.value = ''
      selectedOAuthType.value = 'linuxdo'
      loading.value = false
      oauthError.value = ''
      authorizeUrl.value = ''
      extractionGuide.value = []
      pastedCredentials.value = ''
      pastedApiUser.value = ''
      accountName.value = ''
      parseError.value = ''
      createSuccess.value = false
      createError.value = ''
    }
  }
)

// Auto-select first available OAuth type when provider changes
watch(selectedProviderId, () => {
  const provider = selectedProvider.value
  if (!provider?.oauth_config) return
  if (provider.oauth_config.linuxdo_client_id) {
    selectedOAuthType.value = 'linuxdo'
  } else if (provider.oauth_config.github_client_id) {
    selectedOAuthType.value = 'github'
  }
})

// Methods
function handleClose() {
  emit('close')
  emit('update:isOpen', false)
}

async function goToStep1() {
  step.value = 1
  loading.value = true
  oauthError.value = ''
  authorizeUrl.value = ''

  try {
    const response = await getOAuthAuthorizeUrl({
      provider_id: selectedProviderId.value,
      oauth_type: selectedOAuthType.value,
    })

    if (response.success && response.authorize_url) {
      authorizeUrl.value = response.authorize_url
      extractionGuide.value = response.extraction_guide || []
    } else {
      oauthError.value = response.message || t('checkin.oauthWizard.errors.fetchAuthorizeUrlFailed')
    }
  } catch (err: unknown) {
    oauthError.value = err instanceof Error ? err.message : t('checkin.oauthWizard.errors.networkRequestFailed')
  } finally {
    loading.value = false
  }
}

async function copyUrl() {
  try {
    if (!(await copyText(authorizeUrl.value))) throw new Error('clipboard copy failed')
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch {
    // fallback
  }
}

function parseCookies(input: string): Record<string, string> {
  const trimmed = input.trim()

  // 尝试 JSON 解析
  try {
    const json = JSON.parse(trimmed)

    // 格式1: { cookies: "a=b; c=d", api_user: "123" }
    if (json.cookies && typeof json.cookies === 'string') {
      if (json.api_user) pastedApiUser.value = String(json.api_user)
      return parseCookieString(json.cookies)
    }

    // 格式2: { "session": "abc", "token": "xyz" }
    if (typeof json === 'object' && !Array.isArray(json)) {
      return json as Record<string, string>
    }
  } catch {
    // not JSON
  }

  // 格式3: cookie string "a=b; c=d"
  if (trimmed.includes('=')) {
    return parseCookieString(trimmed)
  }

  throw new Error(t('checkin.oauthWizard.errors.unrecognizedCredentialsFormat'))
}

function parseCookieString(str: string): Record<string, string> {
  const cookies: Record<string, string> = {}
  for (const part of str.split(';')) {
    const eqIdx = part.indexOf('=')
    if (eqIdx > 0) {
      const key = part.substring(0, eqIdx).trim()
      const value = part.substring(eqIdx + 1).trim()
      cookies[key] = value
    }
  }
  return cookies
}

function goToStep3() {
  parseError.value = ''
  try {
    const cookies = parseCookies(pastedCredentials.value)
    if (Object.keys(cookies).length === 0) {
      parseError.value = t('checkin.oauthWizard.errors.emptyCookies')
      return
    }
    step.value = 3
  } catch (err: unknown) {
    parseError.value = err instanceof Error ? err.message : t('checkin.oauthWizard.errors.parseFailed')
  }
}

async function createAccount() {
  creatingAccount.value = true
  createError.value = ''

  try {
    const provider = selectedProvider.value
    if (!provider) throw new Error(t('checkin.oauthWizard.errors.providerRequired'))

    const cookies = parseCookies(pastedCredentials.value)
    const cookiesJson = JSON.stringify(cookies)

    await createCheckinAccount({
      provider_id: provider.id.replace('builtin-', ''),
      name: accountName.value || t('checkin.oauthWizard.defaultAccountName', { provider: provider.name }),
      cookies_json: cookiesJson,
      api_user: pastedApiUser.value || '',
    })

    createSuccess.value = true
    emit('success')
  } catch (err: unknown) {
    createError.value = err instanceof Error ? err.message : t('checkin.oauthWizard.errors.createFailed')
  } finally {
    creatingAccount.value = false
  }
}
</script>

<style scoped>
.oauth-wizard__steps,
.oauth-wizard__step,
.oauth-wizard__step-circle,
.oauth-wizard__choice,
.oauth-wizard__state,
.oauth-wizard__url-row,
.oauth-wizard__external-link,
.oauth-wizard__summary-row,
.oauth-wizard__footer,
.oauth-wizard__footer-actions,
.oauth-wizard__button {
  display: flex;
  align-items: center;
}

.oauth-wizard__steps {
  justify-content: center;
  margin-bottom: 1.5rem;
  gap: 0.5rem;
}

.oauth-wizard__step {
  gap: 0.375rem;
}

.oauth-wizard__step--complete {
  color: var(--color-success);
}

.oauth-wizard__step--current {
  color: var(--color-info);
}

.oauth-wizard__step--inactive {
  color: var(--text-muted);
}

.oauth-wizard__step-circle {
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 2px solid;
  border-radius: 9999px;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  transition: background-color 0.2s ease, border-color 0.2s ease, color 0.2s ease;
}

.oauth-wizard__step-circle--complete {
  border-color: var(--color-success);
  background: rgb(var(--color-success-rgb) / 20%);
  color: var(--color-success);
}

.oauth-wizard__step-circle--current {
  border-color: var(--color-info);
  background: rgb(var(--color-info-rgb) / 20%);
  color: var(--color-info);
}

.oauth-wizard__step-circle--inactive {
  border-color: var(--color-border-strong);
  background: var(--color-bg-surface);
  color: var(--text-ghost);
}

.oauth-wizard__step-icon {
  width: 1rem;
  height: 1rem;
}

.oauth-wizard__step-label,
.oauth-wizard__warning,
.oauth-wizard__error-text--small {
  font-size: 0.75rem;
  line-height: 1rem;
}

.oauth-wizard__step-label {
  display: none;
}

.oauth-wizard__step-divider {
  color: var(--text-disabled);
}

.oauth-wizard__section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.oauth-wizard__section--compact {
  gap: 0.75rem;
}

.oauth-wizard__field {
  display: flex;
  flex-direction: column;
}

.oauth-wizard__label {
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.oauth-wizard__input {
  width: 100%;
  border: 1px solid var(--color-border-default);
  border-radius: 0.5rem;
  background: var(--color-bg-surface);
  padding: 0.5rem 0.75rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--text-primary);
}

.oauth-wizard__input:focus {
  outline: 2px solid var(--color-info);
  outline-offset: 0;
}

.oauth-wizard__input--mono {
  font-family: var(--font-mono);
}

.oauth-wizard__input--textarea {
  resize: none;
}

.oauth-wizard__choice-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
}

.oauth-wizard__choice {
  gap: 0.5rem;
  border: 1px solid;
  border-radius: 0.5rem;
  padding: 0.75rem 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
  transition: border-color 0.2s ease, background-color 0.2s ease, color 0.2s ease;
}

.oauth-wizard__choice--active {
  border-color: var(--color-info);
  background: rgb(var(--color-info-rgb) / 10%);
  color: var(--color-info);
}

.oauth-wizard__choice--inactive {
  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
  color: var(--text-secondary);
}

.oauth-wizard__choice--inactive:hover {
  border-color: var(--text-ghost);
}

.oauth-wizard__warning {
  margin-top: 0.5rem;
  color: var(--color-warning);
}

.oauth-wizard__state {
  flex-direction: column;
  justify-content: center;
  gap: 0.75rem;
  padding: 2rem 0;
}

.oauth-wizard__state-icon--loading {
  color: var(--color-info);
}

.oauth-wizard__state-icon--success {
  color: var(--color-success);
}

.oauth-wizard__state-text,
.oauth-wizard__panel-title,
.oauth-wizard__summary-row,
.oauth-wizard__button {
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.oauth-wizard__state-text {
  color: var(--text-muted);
}

.oauth-wizard__state-text--success {
  font-weight: 500;
  color: var(--color-success);
}

.oauth-wizard__state-subtitle {
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--text-ghost);
}

.oauth-wizard__panel {
  border: 1px solid;
  border-radius: 0.5rem;
  padding: 1rem;
}

.oauth-wizard__panel--error {
  border-color: rgb(var(--color-danger-rgb) / 20%);
  background: rgb(var(--color-danger-rgb) / 10%);
}

.oauth-wizard__panel--info {
  border-color: rgb(var(--color-info-rgb) / 20%);
  background: rgb(var(--color-info-rgb) / 10%);
}

.oauth-wizard__panel--neutral {
  background: rgb(var(--color-bg-surface-rgb) / 50%);
}

.oauth-wizard__panel-title {
  margin-bottom: 0.5rem;
  font-weight: 500;
}

.oauth-wizard__panel-title--info {
  color: var(--color-info);
}

.oauth-wizard__panel-title--neutral {
  color: var(--text-secondary);
}

.oauth-wizard__error-text {
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--color-danger);
}

.oauth-wizard__link-button,
.oauth-wizard__external-link {
  color: var(--color-info);
}

.oauth-wizard__link-button {
  margin-top: 0.5rem;
  font-size: 0.75rem;
  line-height: 1rem;
}

.oauth-wizard__link-button:hover,
.oauth-wizard__external-link:hover {
  text-decoration: underline;
}

.oauth-wizard__url-row,
.oauth-wizard__footer-actions {
  gap: 0.5rem;
}

.oauth-wizard__url-input {
  flex: 1 1 auto;
  border: 1px solid var(--color-border-default);
  border-radius: 0.25rem;
  background: var(--color-bg-base);
  padding: 0.375rem 0.75rem;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--text-secondary);
}

.oauth-wizard__button {
  justify-content: center;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  font-weight: 500;
  transition: background-color 0.2s ease, color 0.2s ease, opacity 0.2s ease, border-color 0.2s ease;
}

.oauth-wizard__button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.oauth-wizard__button--compact {
  flex-shrink: 0;
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
  line-height: 1rem;
}

.oauth-wizard__button--ghost {
  color: var(--text-muted);
}

.oauth-wizard__button--ghost:hover {
  color: var(--text-primary);
}

.oauth-wizard__button--secondary {
  border: 1px solid var(--color-border-default);
  color: var(--text-muted);
}

.oauth-wizard__button--secondary:hover {
  color: var(--text-primary);
}

.oauth-wizard__button--primary {
  background: var(--color-accent-primary);
  color: white;
}

.oauth-wizard__button--primary:hover:not(:disabled) {
  background: var(--color-accent-primary-hover);
}

.oauth-wizard__button--success {
  background: var(--color-success);
  color: white;
}

.oauth-wizard__button--success:hover:not(:disabled) {
  background: var(--color-success-hover);
}

.oauth-wizard__external-link {
  display: inline-flex;
  margin-top: 0.5rem;
  gap: 0.25rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.oauth-wizard__guide-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.oauth-wizard__guide-item {
  position: relative;
  padding-left: 1rem;
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--text-muted);
}

.oauth-wizard__guide-item::before {
  position: absolute;
  left: 0;
  color: var(--text-ghost);
  content: attr(data-index);
}

.oauth-wizard__summary {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.oauth-wizard__summary-row,
.oauth-wizard__footer {
  justify-content: space-between;
}

.oauth-wizard__summary-label {
  color: var(--text-muted);
}

.oauth-wizard__summary-value {
  color: var(--text-primary);
}

@media (width >= 640px) {
  .oauth-wizard__step-label {
    display: inline;
  }
}

@media (width <= 640px) {
  .oauth-wizard__footer,
  .oauth-wizard__footer-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .oauth-wizard__button {
    width: 100%;
  }
}
</style>
