<!--
  SyncAccountDialog — WebDAV 账号添加/编辑弹窗
  - Provider 预设：坚果云（默认锁定）/ Nextcloud / ownCloud / 自定义
  - 测试连接走 test_webdav_config，保存走 set_webdav_config
  - 编辑模式默认不重写密码；勾选「修改密码」才能输入
-->
<template>
  <BaseModal
    :model-value="modelValue"
    :title="title"
    size="lg"
    surface="glass"
    :close-on-backdrop="!saving"
    :close-on-escape="!saving"
    @update:model-value="handleVisibilityChange"
    @close="handleClose"
  >
    <div class="dialog-body">
      <!-- Provider 下拉 -->
      <div class="field">
        <label class="field-label">{{ t('sync.account.provider') }}</label>
        <select
          v-model="form.provider"
          class="field-select"
          :disabled="saving"
        >
          <option value="nutstore">
            {{ t('sync.account.providerNutstore') }}
          </option>
          <option value="nextcloud">
            {{ t('sync.account.providerNextcloud') }}
          </option>
          <option value="owncloud">
            {{ t('sync.account.providerOwncloud') }}
          </option>
          <option value="custom">
            {{ t('sync.account.providerCustom') }}
          </option>
        </select>
        <p
          v-if="form.provider === 'nutstore'"
          class="field-hint"
        >
          <SIcon
            name="AlertCircle"
            size="w-3.5 h-3.5"
          />
          {{ t('sync.account.nutstoreHint') }}
        </p>
      </div>

      <!-- WebDAV URL -->
      <Input
        v-model="form.webdavUrl"
        :label="t('sync.account.webdavUrlLabel')"
        :placeholder="t('sync.account.webdavUrlPlaceholder')"
        :disabled="saving || form.provider === 'nutstore'"
        surface="modal"
        type="url"
      >
        <template
          v-if="form.provider === 'nutstore'"
          #trailing
        >
          <SIcon
            name="Lock"
            size="w-4 h-4"
          />
        </template>
      </Input>

      <!-- 用户名 -->
      <Input
        v-model="form.username"
        :label="t('sync.account.usernameLabel')"
        :placeholder="t('sync.account.usernamePlaceholder')"
        :disabled="saving"
        surface="modal"
        type="text"
      />

      <!-- 密码区 -->
      <div class="field">
        <div class="password-header">
          <label class="field-label mb-0">{{ t('sync.account.passwordLabel') }}</label>
          <label
            v-if="mode === 'edit'"
            class="password-change-toggle"
          >
            <input
              v-model="form.changePassword"
              type="checkbox"
              :disabled="saving"
            >
            <span>{{ t('sync.account.passwordChangeBtn') }}</span>
          </label>
        </div>
        <Input
          v-model="form.password"
          :placeholder="passwordPlaceholder"
          :disabled="saving || (mode === 'edit' && !form.changePassword)"
          surface="modal"
          type="password"
        />
        <p
          v-if="mode === 'edit'"
          class="field-hint subtle"
        >
          {{ t('sync.account.passwordKeep') }}
        </p>
      </div>

      <!-- 远程路径 -->
      <Input
        v-model="form.remotePath"
        :label="t('sync.account.remotePathLabel')"
        :hint="t('sync.account.remotePathHint')"
        :disabled="saving"
        surface="modal"
        type="text"
      />

      <!-- 自动同步开关 -->
      <label class="auto-sync-row">
        <div class="auto-sync-text">
          <span class="auto-sync-label">{{ t('sync.account.autoSyncLabel') }}</span>
          <span class="auto-sync-hint">{{ t('sync.account.autoSyncHint') }}</span>
        </div>
        <span
          class="toggle"
          :class="{ 'toggle--on': form.autoSync }"
        >
          <input
            v-model="form.autoSync"
            type="checkbox"
            class="sr-only"
            :disabled="saving"
          >
          <span class="toggle-thumb" />
        </span>
      </label>

      <!-- 测试结果 inline banner -->
      <div
        v-if="testBanner"
        class="test-banner"
        :class="testBanner.ok ? 'test-banner--ok' : 'test-banner--fail'"
      >
        <SIcon
          :name="testBanner.ok ? 'CheckCircle' : 'AlertCircle'"
          size="w-4 h-4"
        />
        <div class="test-banner-text">
          <strong>{{ testBanner.ok ? t('sync.account.testOk') : t('sync.account.testFail') }}</strong>
          <span
            v-if="!testBanner.ok && testBanner.message"
            class="test-banner-detail"
          >{{ testBanner.message }}</span>
        </div>
      </div>

      <!-- CLI 二级折叠 -->
      <details class="cli-hint">
        <summary>{{ t('sync.account.cliHint') }}</summary>
        <code class="cli-command">{{ t('sync.webdav.configureCommand') }}</code>
      </details>
    </div>

    <template #footer>
      <Button
        variant="secondary"
        :loading="testing"
        :disabled="saving || !canSubmit"
        @click="onTest"
      >
        {{ testing ? t('sync.account.testing') : t('sync.account.testBtn') }}
      </Button>
      <span class="footer-spacer" />
      <Button
        variant="ghost"
        :disabled="saving"
        @click="handleClose"
      >
        {{ t('sync.account.cancelBtn') }}
      </Button>
      <Button
        variant="primary"
        :loading="saving"
        :disabled="saving || !canSubmit"
        @click="onSave"
      >
        {{ saving ? t('sync.account.saving') : t('sync.account.saveBtn') }}
      </Button>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { getErrorMessage } from '@/utils/errorHandler'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { setWebdavConfig, testWebdavConfig } from '@/api'
import {
  WEBDAV_PROVIDER_PRESETS,
  detectProvider,
  type WebDavConfigInput,
  type WebDavProvider,
  type WebDavTestResult,
} from '@/types/sync'
import type { SyncStatusView } from '@/types/syncSelection'
import { logger } from '@/utils/logger'

interface Props {
  modelValue: boolean
  mode: 'add' | 'edit'
  initial: SyncStatusView | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'update:modelValue': [v: boolean]
  saved: []
}>()

const { t } = useI18n()

interface FormState {
  provider: WebDavProvider
  webdavUrl: string
  username: string
  password: string
  remotePath: string
  autoSync: boolean
  changePassword: boolean
}

const buildInitialForm = (): FormState => {
  const initial = props.initial
  const provider = detectProvider(initial?.webdav_url)
  const url = initial?.webdav_url
    ?? (provider === 'custom' ? '' : WEBDAV_PROVIDER_PRESETS[provider])
  return {
    provider,
    webdavUrl: url ?? WEBDAV_PROVIDER_PRESETS.nutstore,
    username: initial?.username ?? '',
    password: '',
    remotePath: initial?.remote_path?.trim() ? initial.remote_path : '/ccr/',
    autoSync: initial?.auto_sync ?? false,
    changePassword: false,
  }
}

const form = reactive<FormState>(buildInitialForm())
const testing = ref(false)
const saving = ref(false)
const testBanner = ref<WebDavTestResult | null>(null)

const title = computed(() =>
  props.mode === 'edit' ? t('sync.account.editTitle') : t('sync.account.addTitle'),
)

const passwordPlaceholder = computed(() => {
  if (props.mode === 'edit' && !form.changePassword) {
    return t('sync.account.passwordMaskPlaceholder')
  }
  return t('sync.account.passwordPlaceholder')
})

const passwordRequired = computed(() =>
  props.mode === 'add' || form.changePassword,
)

const canSubmit = computed(() => {
  if (!form.webdavUrl.trim()) return false
  if (!form.username.trim()) return false
  if (passwordRequired.value && !form.password) return false
  return true
})

// Provider 切换：覆写 URL（custom 保留当前值）
watch(
  () => form.provider,
  (next, prev) => {
    if (next === prev) return
    if (next === 'custom') return
    form.webdavUrl = WEBDAV_PROVIDER_PRESETS[next]
  },
)

// 弹窗每次打开重置表单与状态
watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      const fresh = buildInitialForm()
      Object.assign(form, fresh)
      testBanner.value = null
      testing.value = false
      saving.value = false
    }
  },
  { immediate: true },
)

const buildPayload = (overridePassword?: string): WebDavConfigInput => ({
  webdavUrl: form.webdavUrl.trim(),
  username: form.username.trim(),
  password: overridePassword ?? form.password,
  remotePath: form.remotePath.trim() || '/ccr/',
  autoSync: form.autoSync,
})

const onTest = async () => {
  if (!canSubmit.value) {
    testBanner.value = { ok: false, message: t('sync.account.validationError') }
    return
  }
  testing.value = true
  testBanner.value = null
  try {
    const result = await testWebdavConfig(buildPayload())
    testBanner.value = result
  } catch (err: unknown) {
    logger.error('test_webdav_config failed:', err)
    testBanner.value = {
      ok: false,
      message: getErrorMessage(err),
    }
  } finally {
    testing.value = false
  }
}

const onSave = async () => {
  if (!canSubmit.value) {
    testBanner.value = { ok: false, message: t('sync.account.validationError') }
    return
  }
  saving.value = true
  try {
    await setWebdavConfig(buildPayload())
    emit('saved')
    emit('update:modelValue', false)
  } catch (err: unknown) {
    logger.error('set_webdav_config failed:', err)
    testBanner.value = {
      ok: false,
      message: getErrorMessage(err),
    }
  } finally {
    saving.value = false
  }
}

const handleVisibilityChange = (v: boolean) => {
  if (saving.value) return
  emit('update:modelValue', v)
}

const handleClose = () => {
  if (saving.value) return
  emit('update:modelValue', false)
}
</script>

<style scoped>
.dialog-body {
  @apply space-y-4 py-2;
}

.field {
  @apply flex flex-col;
}

.field-label {
  @apply mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted;
}

.field-select {
  @apply rounded-2xl border border-border-default/70 px-4 py-2.5 text-sm text-text-primary;
  @apply focus:outline-none focus:ring-2 focus:ring-accent-primary/20 focus:border-accent-primary/30;
  @apply disabled:cursor-not-allowed disabled:opacity-50;

  background: var(--surface-modal-bg);
  backdrop-filter: var(--surface-modal-blur);
}

.field-hint {
  @apply mt-1.5 ml-1 flex items-center gap-1.5 text-xs text-text-muted;
}

.field-hint.subtle {
  @apply text-text-ghost;
}

.password-header {
  @apply mb-1.5 flex items-center justify-between;
}

.password-change-toggle {
  @apply flex cursor-pointer items-center gap-1.5 text-xs text-text-secondary;
}

.password-change-toggle input {
  @apply h-3.5 w-3.5 rounded border-border-default text-accent-primary;
}

.auto-sync-row {
  @apply flex cursor-pointer items-center justify-between rounded-2xl border border-border-default/60 px-4 py-3;

  background: rgb(var(--color-bg-elevated-rgb) / 50%);
}

.auto-sync-text {
  @apply flex flex-col gap-0.5;
}

.auto-sync-label {
  @apply text-sm font-medium text-text-primary;
}

.auto-sync-hint {
  @apply text-xs text-text-muted;
}

.toggle {
  @apply relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full transition-colors;

  background: rgb(var(--color-border-default-rgb) / 60%);
}

.toggle--on {
  background: var(--color-accent-primary);
}

.toggle-thumb {
  @apply inline-block h-5 w-5 transform rounded-full bg-white shadow transition-transform;

  transform: translateX(2px);
}

.toggle--on .toggle-thumb {
  transform: translateX(22px);
}

.sr-only {
  @apply absolute h-0 w-0 overflow-hidden opacity-0;
}

.test-banner {
  @apply flex items-start gap-2 rounded-xl border px-4 py-3;
}

.test-banner--ok {
  border-color: rgb(var(--color-success-rgb) / 30%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--accent-success);
}

.test-banner--fail {
  border-color: rgb(var(--color-danger-rgb) / 30%);
  background: rgb(var(--color-danger-rgb) / 10%);
  color: var(--accent-danger);
}

.test-banner-text {
  @apply flex flex-col gap-0.5 text-sm;
}

.test-banner-detail {
  @apply text-xs opacity-80;

  color: var(--text-secondary);
  overflow-wrap: anywhere;
}

.cli-hint {
  @apply mt-2 rounded-xl border border-border-default/50 px-4 py-2 text-xs text-text-muted;

  background: rgb(var(--color-bg-elevated-rgb) / 40%);
}

.cli-hint summary {
  @apply cursor-pointer select-none py-1 font-medium text-text-secondary;
}

.cli-command {
  @apply mt-2 block rounded-lg px-3 py-2 font-mono text-xs text-text-primary;

  background: rgb(var(--color-bg-elevated-rgb) / 70%);
}

.footer-spacer {
  @apply flex-1;
}
</style>
