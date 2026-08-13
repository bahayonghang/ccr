<template>
  <div class="claude-auth-view">
    <div class="claude-auth-view__shell">
      <ModuleSubnav module="claude-code" />

      <header class="claude-auth-view__header">
        <div>
          <p class="claude-auth-view__eyebrow">
            {{ tt('Claude 官方订阅', 'Claude Official Subscription') }}
          </p>
          <h1 class="claude-auth-view__title">
            {{ tt('官方账号管理', 'Official account management') }}
          </h1>
          <p class="claude-auth-view__subtitle">
            {{ tt('保存、切换、删除 Claude Code 官方订阅账号快照；切换会更新', 'Save, switch, or delete Claude Code official subscription snapshots. Switching updates') }}
            <code>{{ credentialsFile }}</code>{{ tt('，并只清理 CCR 托管的 Profile 设置。', ' and clears only CCR-managed profile settings.') }}
          </p>
        </div>

        <div class="claude-auth-view__actions">
          <RouterLink
            to="/claude-code"
            class="claude-auth-view__ghost-button"
          >
            {{ tt('返回 Claude Code', 'Back to Claude Code') }}
          </RouterLink>
          <button
            type="button"
            class="claude-auth-view__ghost-button"
            :disabled="loading"
            @click="refreshAll"
          >
            {{ tt('刷新', 'Refresh') }}
          </button>
          <button
            type="button"
            class="claude-auth-view__primary-button"
            :disabled="saving"
            @click="showSaveForm = true"
          >
            {{ tt('保存当前登录', 'Save current login') }}
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
            {{ tt('登录状态', 'Login state') }}
          </p>
          <p class="claude-auth-view__stat-value">
            {{ loginStateLabel }}
          </p>
        </article>
        <article class="claude-auth-view__stat-card">
          <p class="claude-auth-view__stat-label">
            {{ tt('运行时模式', 'Runtime mode') }}
          </p>
          <p class="claude-auth-view__stat-value">
            {{ runtimeModeLabel }}
          </p>
        </article>
        <article class="claude-auth-view__stat-card">
          <p class="claude-auth-view__stat-label">
            {{ tt('当前 Profile', 'Current profile') }}
          </p>
          <p class="claude-auth-view__stat-value">
            {{ currentProfileLabel }}
          </p>
        </article>
        <article class="claude-auth-view__stat-card">
          <p class="claude-auth-view__stat-label">
            {{ tt('已保存账号', 'Saved accounts') }}
          </p>
          <p class="claude-auth-view__stat-value">
            {{ accounts.length }}
          </p>
        </article>
      </section>

      <section
        v-if="runtimeSummary"
        class="claude-auth-view__panel"
        data-testid="claude-auth-diagnosis"
      >
        <div class="claude-auth-view__panel-header">
          <div>
            <h2 class="claude-auth-view__panel-title">
              {{ tt('认证来源诊断', 'Auth source diagnosis') }}
            </h2>
            <p class="claude-auth-view__panel-subtitle">
              {{ tt('范围限于当前 CCR 进程和已解析的用户级文件。', 'Scope is limited to this CCR process and the resolved user-level files.') }}
            </p>
          </div>
          <div class="claude-auth-view__diagnosis-actions">
            <button
              v-if="canOff"
              type="button"
              class="claude-auth-view__ghost-button"
              data-testid="claude-auth-profile-off"
              :disabled="loading"
              @click="handleOff"
            >
              {{ tt('退出 Profile', 'Exit profile') }}
            </button>
            <span
              class="claude-auth-view__diagnosis-state"
              :class="visibleSuppressors.length > 0 ? 'claude-auth-view__diagnosis-state--warning' : 'claude-auth-view__diagnosis-state--clear'"
            >
              {{ visibleSuppressors.length > 0 ? tt(`${visibleSuppressors.length} 个可见竞争来源`, `${visibleSuppressors.length} visible competing source(s)`) : tt('未发现可见竞争来源', 'No visible competing source') }}
            </span>
          </div>
        </div>

        <dl class="claude-auth-view__diagnosis-facts">
          <div>
            <dt>{{ tt('当前推定来源', 'Presumed source') }}</dt>
            <dd data-testid="claude-auth-presumed-source">
              {{ presumedSourceLabel }}
            </dd>
          </div>
          <div>
            <dt>{{ tt('置信度', 'Confidence') }}</dt>
            <dd>{{ presumedConfidenceLabel }}</dd>
          </div>
          <div>
            <dt>{{ tt('API Key 批准记录', 'API key response state') }}</dt>
            <dd>
              {{ authDiagnosis?.custom_api_key_responses_present ? tt('存在，仅作解释', 'Present, context only') : tt('未观察到', 'Not observed') }}
            </dd>
          </div>
        </dl>

        <div
          v-if="visibleSuppressors.length > 0"
          class="claude-auth-view__source-list"
        >
          <div
            v-for="(source, index) in visibleSuppressors"
            :key="`${source.kind}-${source.location}-${index}`"
            class="claude-auth-view__source-row"
          >
            <div class="claude-auth-view__source-main">
              <strong>{{ authSourceKindLabel(source.kind) }}</strong>
              <span>{{ authSourceLocationLabel(source.location) }}</span>
            </div>
            <div class="claude-auth-view__source-meta">
              <span>{{ authConfidenceLabel(source.confidence) }}</span>
              <span>{{ authEvidenceLabel(source.evidence) }}</span>
              <span>{{ authOwnershipLabel(source.ownership) }}</span>
            </div>
          </div>
        </div>

        <details class="claude-auth-view__scope-details">
          <summary>
            {{ tt(`${unobservableLabels.length} 个不可观测层`, `${unobservableLabels.length} unobservable layer(s)`) }}
          </summary>
          <ul>
            <li
              v-for="item in unobservableLabels"
              :key="item"
            >
              {{ item }}
            </li>
          </ul>
        </details>
      </section>

      <section
        v-if="currentInfo"
        class="claude-auth-view__panel"
      >
        <div class="claude-auth-view__panel-header">
          <h2 class="claude-auth-view__panel-title">
            {{ tt('当前运行时官方登录', 'Current runtime official login') }}
          </h2>
        </div>

        <div class="claude-auth-view__detail-grid">
          <div>
            <p class="claude-auth-view__detail-label">
              {{ tt('邮箱', 'Email') }}
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.email || '-' }}
            </p>
          </div>
          <div>
            <p class="claude-auth-view__detail-label">
              {{ tt('账号 UUID', 'Account UUID') }}
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.account_uuid || '-' }}
            </p>
          </div>
          <div>
            <p class="claude-auth-view__detail-label">
              {{ tt('订阅类型', 'Subscription type') }}
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.subscription_type || '-' }}
            </p>
          </div>
          <div>
            <p class="claude-auth-view__detail-label">
              {{ tt('计费类型', 'Billing type') }}
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.billing_type || '-' }}
            </p>
          </div>
          <div>
            <p class="claude-auth-view__detail-label">
              {{ tt('速率档位', 'Rate tier') }}
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.rate_limit_tier || '-' }}
            </p>
          </div>
          <div>
            <p class="claude-auth-view__detail-label">
              {{ tt('Access Token 到期', 'Access token expiry') }}
            </p>
            <p class="claude-auth-view__detail-value">
              {{ currentInfo.expires_at ? formatDate(currentInfo.expires_at) : '-' }}
            </p>
          </div>
        </div>
      </section>

      <section class="claude-auth-view__panel">
        <div class="claude-auth-view__panel-header">
          <div>
            <h2 class="claude-auth-view__panel-title">
              {{ tt('已保存账号快照', 'Saved account snapshots') }}
            </h2>
            <p class="claude-auth-view__panel-subtitle">
              {{ tt('每个快照都保存当前 `claudeAiOauth`，切换时不会改写', 'Each snapshot keeps the current `claudeAiOauth`, and switching will not rewrite') }}
              <code>{{ claudeJsonFile }}</code>{{ tt('。', '.') }}
            </p>
          </div>
        </div>

        <div
          v-if="loading"
          class="claude-auth-view__empty"
        >
          {{ tt('正在加载账号信息…', 'Loading account details...') }}
        </div>

        <EmptyState
          v-else-if="accounts.length === 0"
          icon="User"
          :title="tt('尚未保存任何官方账号快照。', 'No official account snapshots saved yet.')"
          :action-text="tt('保存当前登录', 'Save current login')"
          action-icon="Plus"
          :on-action="() => { showSaveForm = true }"
        />

        <div
          v-else
          class="claude-auth-view__table-wrap"
        >
          <table class="claude-auth-view__table">
            <thead>
              <tr>
                <th>{{ tt('名称', 'Name') }}</th>
                <th>{{ tt('邮箱', 'Email') }}</th>
                <th>{{ tt('订阅', 'Subscription') }}</th>
                <th>{{ tt('到期', 'Expiry') }}</th>
                <th>{{ tt('状态', 'State') }}</th>
                <th>{{ tt('操作', 'Actions') }}</th>
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
                      {{ tt('当前', 'Current') }}
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
                <td>
                  {{ account.expires_at ? formatDate(account.expires_at) : '-' }}
                </td>
                <td>
                  {{ account.is_current ? tt('当前生效', 'Active now') : account.is_logged_in ? tt('已登录', 'Logged in') : tt('已保存', 'Saved') }}
                </td>
                <td>
                  <div class="claude-auth-view__row-actions">
                    <button
                      type="button"
                      class="claude-auth-view__table-button"
                      :disabled="busyName === account.name"
                      @click="handleSwitch(account.name)"
                    >
                      {{ tt('切换', 'Switch') }}
                    </button>
                    <button
                      type="button"
                      class="claude-auth-view__table-button claude-auth-view__table-button--danger"
                      :disabled="busyName === account.name"
                      @click="handleDelete(account.name)"
                    >
                      {{ tt('删除', 'Delete') }}
                    </button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <BaseModal
        v-model="showSaveForm"
        :title="tt('保存当前官方登录', 'Save current official login')"
        size="md"
        surface="solid"
      >
        <p class="claude-auth-view__modal-subtitle">
          {{ tt('当前必须已经通过 `claude login` 拿到官方登录，CCR 只负责保存快照和切换。', 'You must already have an official login from `claude login`. CCR only saves and switches snapshots.') }}
        </p>

        <label class="claude-auth-view__field">
          <span>{{ tt('账号名称', 'Account name') }}</span>
          <input
            v-model="saveForm.name"
            type="text"
            :placeholder="tt('例如 work / personal', 'e.g. work / personal')"
          >
        </label>

        <label class="claude-auth-view__field">
          <span>{{ tt('描述（可选）', 'Description (optional)') }}</span>
          <input
            v-model="saveForm.description"
            type="text"
            :placeholder="tt('例如 公司订阅 / 个人订阅', 'e.g. company plan / personal plan')"
          >
        </label>

        <label class="claude-auth-view__checkbox">
          <input
            v-model="saveForm.force"
            type="checkbox"
          >
          <span>{{ tt('覆盖同名账号', 'Overwrite same-name account') }}</span>
        </label>

        <template #footer>
          <button
            type="button"
            class="claude-auth-view__ghost-button"
            @click="showSaveForm = false"
          >
            {{ tt('取消', 'Cancel') }}
          </button>
          <button
            type="button"
            class="claude-auth-view__primary-button"
            :disabled="saving"
            @click="handleSave"
          >
            {{ saving ? tt('保存中…', 'Saving...') : tt('保存', 'Save') }}
          </button>
        </template>
      </BaseModal>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onActivated, onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import {
  deleteClaudeAuth,
  getClaudeAuthCurrent,
  listClaudeAuthAccounts,
  saveClaudeAuth,
  switchClaudeAuth,
} from '@/api'
import { claudeProfileOff, listClaudeProfiles } from '@/api/domains/claude'
import type {
  ClaudeAuthAccountItem,
  ClaudeAuthCurrentInfo,
  ClaudeAuthSaveRequest,
  ClaudeAuthSourceObservation,
  ClaudeLoginState,
  ClaudeRuntimeSummary,
} from '@/types'
import { logger } from '@/utils/logger'
import { useUIStore } from '@/stores/ui'

defineOptions({ name: 'ClaudeAuthView' })

const { locale } = useI18n()
const uiStore = useUIStore()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)
const credentialsFile = '~/.claude/.credentials.json'
const claudeJsonFile = '~/.claude.json'

const loading = ref(false)
const saving = ref(false)
const busyName = ref<string | null>(null)
const showSaveForm = ref(false)
const authActionError = ref<string | null>(null)

const accounts = ref<ClaudeAuthAccountItem[]>([])
const currentInfo = ref<ClaudeAuthCurrentInfo | null>(null)
const runtimeSummary = ref<ClaudeRuntimeSummary | null>(null)
const loginState = ref<ClaudeLoginState>({ type: 'NotLoggedIn' })
const canOff = ref(false)

const authDiagnosis = computed(() => runtimeSummary.value?.auth_diagnosis ?? null)
const visibleSuppressors = computed(() =>
  authDiagnosis.value?.observations.filter(source => source.suppresses_subscription) ?? []
)

const authSourceKindLabel = (kind: ClaudeAuthSourceObservation['kind']) => {
  const labels: Record<ClaudeAuthSourceObservation['kind'], [string, string]> = {
    bedrock: ['Amazon Bedrock', 'Amazon Bedrock'],
    vertex: ['Google Vertex AI', 'Google Vertex AI'],
    foundry: ['Microsoft Foundry', 'Microsoft Foundry'],
    anthropic_auth_token: ['ANTHROPIC_AUTH_TOKEN', 'ANTHROPIC_AUTH_TOKEN'],
    anthropic_api_key: ['ANTHROPIC_API_KEY', 'ANTHROPIC_API_KEY'],
    api_key_helper: ['apiKeyHelper', 'apiKeyHelper'],
    claude_code_oauth_token: ['CLAUDE_CODE_OAUTH_TOKEN', 'CLAUDE_CODE_OAUTH_TOKEN'],
    subscription_oauth: ['官方订阅 OAuth', 'Official subscription OAuth'],
    primary_api_key: ['primaryApiKey', 'primaryApiKey'],
  }
  return tt(...labels[kind])
}

const authSourceLocationLabel = (location: ClaudeAuthSourceObservation['location']) => {
  const labels: Record<ClaudeAuthSourceObservation['location'], [string, string]> = {
    process_env: ['当前进程环境', 'Current process environment'],
    settings_env: ['settings.json env', 'settings.json env'],
    settings_root: ['settings.json 顶层', 'settings.json root'],
    state_file: ['Claude state file', 'Claude state file'],
    credentials_file: ['credentials file', 'credentials file'],
  }
  return tt(...labels[location])
}

const authConfidenceLabel = (confidence: ClaudeAuthSourceObservation['confidence']) => {
  const labels: Record<ClaudeAuthSourceObservation['confidence'], [string, string]> = {
    confirmed: ['已确认', 'Confirmed'],
    potential: ['潜在', 'Potential'],
    unobservable: ['不可观测', 'Unobservable'],
  }
  return tt(...labels[confidence])
}

const authEvidenceLabel = (evidence: ClaudeAuthSourceObservation['evidence']) =>
  evidence === 'issue_report'
    ? tt('Issue 报告行为', 'Issue-reported behavior')
    : tt('官方契约', 'Official contract')

const authOwnershipLabel = (ownership: ClaudeAuthSourceObservation['ownership']) => {
  const labels: Record<ClaudeAuthSourceObservation['ownership'], [string, string]> = {
    ccr_managed: ['CCR 托管', 'CCR-managed'],
    user_owned: ['用户自有', 'User-owned'],
    external_runtime: ['外部运行时', 'External runtime'],
  }
  return tt(...labels[ownership])
}

const formatAuthSource = (source: ClaudeAuthSourceObservation) =>
  `${authSourceKindLabel(source.kind)} · ${authSourceLocationLabel(source.location)}`

const presumedSourceLabel = computed(() => {
  const source = authDiagnosis.value?.presumed_effective_source
  return source
    ? formatAuthSource(source)
    : tt('未解析或存在同级歧义', 'Unresolved or same-priority ambiguity')
})

const presumedConfidenceLabel = computed(() => {
  const source = authDiagnosis.value?.presumed_effective_source
  return source ? authConfidenceLabel(source.confidence) : '-'
})

const unobservableLabels = computed(() => {
  const labels: Record<string, [string, string]> = {
    other_shell_environment: ['其他 shell 的环境变量', 'Environment variables in other shells'],
    project_settings_for_unknown_working_directories: ['未知工作目录下的项目级 settings', 'Project settings under unknown working directories'],
    external_process_cli_arguments: ['外部 Claude Code 进程的 CLI 参数', 'CLI arguments of external Claude Code processes'],
    managed_settings_dynamic_policy: ['组织级 managed settings 动态策略', 'Dynamic organization-managed settings policy'],
    api_key_helper_result_and_external_secret_store: ['apiKeyHelper 返回值与外部 secret store', 'apiKeyHelper output and external secret stores'],
    macos_keychain_contents: ['macOS Keychain 内容', 'macOS Keychain contents'],
  }
  return authDiagnosis.value?.unobservable.map(item => labels[item] ? tt(...labels[item]) : item) ?? []
})

const saveForm = reactive({
  name: '',
  description: '',
  force: false,
})

const extractErrorMessage = (error: unknown): string => {
  if (error instanceof Error && error.message) return error.message
  if (typeof error === 'string') return error
  return tt('请求失败', 'Request failed')
}

const loginStateLabel = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved':
      return tt(`已登录（已保存为 ${loginState.value.account_name}）`, `Logged in (saved as ${loginState.value.account_name})`)
    case 'LoggedInUnsaved':
      return tt('已登录（未保存）', 'Logged in (unsaved)')
    case 'ApiKeyActive':
      return tt('当前由 API key profile 控制', 'Currently controlled by the API key profile')
    case 'NotLoggedIn':
    default:
      return tt('未登录', 'Not logged in')
  }
})

const runtimeModeLabel = computed(() => {
  switch (runtimeSummary.value?.mode) {
    case 'profile_with_auth':
      return tt('Profile + 官方订阅', 'Profile + official subscription')
    case 'profile_pending_auth':
      return tt('Profile 等待官方订阅', 'Profile waiting for official subscription')
    case 'profile_only':
      return tt('Profile 驱动（API key）', 'Profile-driven (API key)')
    case 'runtime_only':
      return tt('仅官方订阅运行时', 'Official subscription runtime only')
    case 'unresolved':
    default:
      return tt('未解析', 'Unresolved')
  }
})

const currentProfileLabel = computed(() => {
  const summary = runtimeSummary.value
  if (!summary?.current_profile_name) {
    return tt('未绑定', 'Unbound')
  }

  const authMode = summary.current_profile_auth_mode
  return authMode
    ? `${summary.current_profile_name} · ${authMode}`
    : summary.current_profile_name
})

const formatDate = (date: string) => {
  try {
    return new Date(date).toLocaleString(isZh.value ? 'zh-CN' : 'en-US')
  } catch {
    return date
  }
}

const refreshAll = async () => {
  try {
    loading.value = true
    authActionError.value = null

    const [accountsData, currentData, profilesData] = await Promise.all([
      listClaudeAuthAccounts(),
      getClaudeAuthCurrent(),
      listClaudeProfiles().catch(() => ({ can_off: false })),
    ])

    accounts.value = accountsData.accounts || []
    runtimeSummary.value = accountsData.runtime_summary || currentData.runtime_summary
    loginState.value = accountsData.login_state || currentData.login_state
    currentInfo.value = currentData.info || null
    canOff.value = profilesData.can_off === true
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
    uiStore.showError(tt('账号名称不能为空', 'Account name is required'))
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
    uiStore.showSuccess(tt('Claude 官方账号已保存', 'Claude official account saved'))
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

const handleOff = async () => {
  if (!canOff.value) return
  const confirmed = await uiStore.requestConfirm({
    title: tt('退出 Profile', 'Exit profile'),
    message: tt(
      '退出当前 Profile 并清理会压制官方登录的 CCR 运行时残留？已保存的账号不会删除。',
      'Exit the current profile and clear CCR leftovers that can suppress official login? Saved accounts stay.',
    ),
    confirmText: tt('退出 Profile', 'Exit profile'),
    cancelText: tt('取消', 'Cancel'),
    type: 'warning',
  })
  if (!confirmed) return

  try {
    loading.value = true
    authActionError.value = null
    const result = await claudeProfileOff()
    uiStore.showSuccess(tt('已退出 Profile 并清理登录残留', 'Exited profile mode and cleared login leftovers'))
    const suppressorWarnings = result.remaining_suppressors.map(source =>
      tt(
        `退出 Profile 后仍存在${authOwnershipLabel(source.ownership)}认证来源：${formatAuthSource(source)}（${authConfidenceLabel(source.confidence)}）`,
        `${authOwnershipLabel(source.ownership)} auth source remains after exit profile: ${formatAuthSource(source)} (${authConfidenceLabel(source.confidence)})`,
      ),
    )
    for (const warning of suppressorWarnings.length > 0 ? suppressorWarnings : result.warnings) {
      uiStore.showWarning(warning, 6000)
    }
    await refreshAll()
  } catch (error) {
    logger.error('Failed to exit Claude profile mode:', error)
    authActionError.value = extractErrorMessage(error)
    uiStore.showError(authActionError.value)
  } finally {
    loading.value = false
  }
}

const handleSwitch = async (name: string) => {
  const confirmed = await uiStore.requestConfirm({
    title: tt('切换官方账号', 'Switch official account'),
    message: tt(`确定切换到官方账号 "${name}" 吗？`, `Switch to official account "${name}"?`),
    confirmText: tt('切换', 'Switch'),
    cancelText: tt('取消', 'Cancel'),
    type: 'warning',
  })
  if (!confirmed) return

  try {
    busyName.value = name
    authActionError.value = null
    const result = await switchClaudeAuth(name)
    const clearedCount = result.cleared_managed_sources.length
    uiStore.showSuccess(clearedCount > 0
      ? tt(
        `已切换到 ${name}，并清理 ${clearedCount} 个 CCR 托管设置`,
        `Switched to ${name} and cleared ${clearedCount} CCR-managed setting(s)`,
      )
      : tt(`已切换到 ${name}`, `Switched to ${name}`))
    const structuredWarnings = result.remaining_suppressors.map(source =>
      tt(
        `切换后仍存在${authOwnershipLabel(source.ownership)}认证来源：${formatAuthSource(source)}（${authConfidenceLabel(source.confidence)}）`,
        `${authOwnershipLabel(source.ownership)} auth source remains after switching: ${formatAuthSource(source)} (${authConfidenceLabel(source.confidence)})`,
      )
    )
    const warningMessages = structuredWarnings.length > 0 ? structuredWarnings : result.warnings
    for (const warning of warningMessages) {
      uiStore.showWarning(warning, 6000)
    }
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
  const confirmed = await uiStore.requestConfirm({
    title: tt('删除官方账号', 'Delete official account'),
    message: tt(`确定删除官方账号 "${name}" 吗？`, `Delete official account "${name}"?`),
    confirmText: tt('删除', 'Delete'),
    cancelText: tt('取消', 'Cancel'),
    type: 'danger',
  })
  if (!confirmed) return

  try {
    busyName.value = name
    authActionError.value = null
    await deleteClaudeAuth(name)
    uiStore.showSuccess(tt(`已删除 ${name}`, `Deleted ${name}`))
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
.claude-auth-view__account-name {
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
.claude-auth-view__banner {
  border: 1px solid var(--stage-border-soft);
  border-radius: 1rem;
  background: var(--stage-surface-elevated);
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
.claude-auth-view__panel-title {
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

.claude-auth-view__diagnosis-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex: none;
}

.claude-auth-view__diagnosis-state {
  flex: none;
  border: 1px solid var(--stage-border-medium);
  border-radius: 9999px;
  padding: 0.35rem 0.65rem;
  color: var(--stage-text-primary);
  font-size: 0.8125rem;
  line-height: 1.2;
  font-weight: 600;
}

.claude-auth-view__diagnosis-state--warning {
  border-color: rgb(var(--color-warning-rgb) / 42%);
  background: var(--color-warning-glow);
}

.claude-auth-view__diagnosis-state--clear {
  border-color: rgb(var(--color-success-rgb) / 42%);
  background: var(--color-success-glow);
}

.claude-auth-view__diagnosis-facts {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1rem;
  margin-top: 1rem;
  padding-block: 1rem;
  border-block: 1px solid var(--stage-border-soft);
}

.claude-auth-view__diagnosis-facts > div,
.claude-auth-view__source-main,
.claude-auth-view__source-meta {
  min-width: 0;
}

.claude-auth-view__diagnosis-facts dt {
  color: var(--stage-text-quiet);
  font-size: 0.8125rem;
  line-height: 1.3;
  font-weight: 600;
}

.claude-auth-view__diagnosis-facts dd {
  margin-top: 0.3rem;
  color: var(--stage-text-primary);
  font-size: 1rem;
  line-height: 1.45;
  overflow-wrap: anywhere;
}

.claude-auth-view__source-list {
  margin-top: 0.5rem;
}

.claude-auth-view__source-row {
  display: grid;
  grid-template-columns: minmax(13rem, 1fr) minmax(18rem, 1.3fr);
  gap: 1rem;
  align-items: center;
  padding-block: 0.75rem;
  border-bottom: 1px solid var(--stage-border-soft);
}

.claude-auth-view__source-main,
.claude-auth-view__source-meta {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  flex-wrap: wrap;
}

.claude-auth-view__source-main strong {
  color: var(--stage-text-primary);
  overflow-wrap: anywhere;
}

.claude-auth-view__source-main span,
.claude-auth-view__source-meta span {
  color: var(--stage-text-secondary);
  font-size: 0.8125rem;
  line-height: 1.35;
  overflow-wrap: anywhere;
}

.claude-auth-view__source-meta span {
  border: 1px solid var(--stage-border-soft);
  border-radius: 9999px;
  padding: 0.2rem 0.5rem;
  background: var(--stage-surface-soft);
}

.claude-auth-view__scope-details {
  margin-top: 1rem;
  color: var(--stage-text-secondary);
  font-size: 0.8125rem;
  line-height: 1.5;
}

.claude-auth-view__scope-details summary {
  width: fit-content;
  cursor: pointer;
  color: var(--stage-text-primary);
  font-weight: 600;
}

.claude-auth-view__scope-details summary:focus-visible {
  outline: 2px solid var(--color-accent-primary);
  outline-offset: 3px;
}

.claude-auth-view__scope-details ul {
  margin-top: 0.6rem;
  padding-inline-start: 1.25rem;
}

.claude-auth-view__scope-details li {
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
  color: var(--color-success);
}

.claude-auth-view__freshness--stale {
  color: var(--color-warning);
}

.claude-auth-view__freshness--old {
  color: var(--color-danger);
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

@media (width <= 1024px) {
  .claude-auth-view__stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .claude-auth-view__detail-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .claude-auth-view__source-row {
    grid-template-columns: 1fr;
    gap: 0.5rem;
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
  .claude-auth-view__detail-grid,
  .claude-auth-view__diagnosis-facts {
    grid-template-columns: 1fr;
  }

  .claude-auth-view__row-actions {
    flex-wrap: wrap;
  }
}
</style>
