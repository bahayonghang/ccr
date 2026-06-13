<!-- -->
<template>
  <div class="checkin-providers">
    <!-- 内置中转站区域 -->
    <div v-if="availableBuiltinProviders.length > 0">
      <div class="checkin-providers__section-header">
        <SIcon
          name="Store"
          size="w-5 h-5"
          class="checkin-providers__section-icon checkin-providers__section-icon--primary"
        />
        <h2 class="checkin-providers__section-title">
          内置中转站
        </h2>
        <span class="checkin-providers__section-count">
          ({{ availableBuiltinProviders.length }})
        </span>
      </div>
      <div class="checkin-providers__builtin-grid">
        <div
          v-for="bp in availableBuiltinProviders"
          :key="bp.id"
          class="checkin-providers__builtin-card"
        >
          <div class="checkin-providers__builtin-card-header">
            <div class="checkin-providers__builtin-card-main">
              <span class="checkin-providers__builtin-card-emoji">{{ bp.icon }}</span>
              <div>
                <div class="checkin-providers__builtin-card-title-row">
                  <h3 class="checkin-providers__builtin-card-title">
                    {{ bp.name }}
                  </h3>
                  <span class="checkin-providers__builtin-badge checkin-badge-pill">
                    内置
                  </span>
                </div>
                <p class="checkin-providers__builtin-domain">
                  {{ bp.domain }}
                </p>
              </div>
            </div>
            <button
              class="checkin-providers__primary-button checkin-providers__primary-button--compact"
              @click="emit('add-builtin', bp.id)"
            >
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
                  d="M12 4v16m8-8H4"
                />
              </svg>
              <span>添加</span>
            </button>
          </div>
          <p class="checkin-providers__builtin-description">
            {{ bp.description }}
          </p>
          <div class="checkin-providers__tag-list">
            <span
              v-if="bp.supports_checkin"
              class="checkin-providers__tag checkin-badge-pill"
              :class="bp.checkin_bugged
                ? 'checkin-providers__tag--warning'
                : 'checkin-providers__tag--success'"
            >
              <SIcon
                :name="bp.checkin_bugged ? 'AlertTriangle' : 'CheckCircle'"
                size="w-3 h-3"
                class="mr-1 inline"
              />
              {{ bp.checkin_bugged ? '自动签到' : '支持签到' }}
            </span>
            <span
              v-else
              class="checkin-providers__tag checkin-badge-pill checkin-providers__tag--muted"
            >
              <SIcon
                name="XCircle"
                size="w-3 h-3"
                class="mr-1"
              /> 无签到
            </span>
            <span
              v-if="bp.requires_waf_bypass"
              class="checkin-providers__tag checkin-badge-pill checkin-providers__tag--warning"
            >
              <SIcon
                name="Shield"
                size="w-3 h-3"
                class="mr-1"
              /> 需要 WAF 绕过
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 已添加的提供商 -->
    <div>
      <div class="checkin-providers__section-header checkin-providers__section-header--split">
        <div class="checkin-providers__section-heading">
          <SIcon
            name="Building2"
            size="w-5 h-5"
            class="checkin-providers__section-icon checkin-providers__section-icon--secondary"
          />
          <h2 class="checkin-providers__section-title">
            已添加的提供商
          </h2>
          <span class="checkin-providers__section-count">
            ({{ providers.length }})
          </span>
        </div>
        <button
          class="checkin-providers__primary-button"
          @click="openProviderModal()"
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
          <span>自定义添加</span>
        </button>
      </div>

      <!-- 提供商列表 -->
      <div
        v-if="providers.length === 0"
        class="checkin-providers__empty-state"
      >
        <p class="checkin-providers__empty-icon">
          <SIcon
            name="Package"
            size="w-12 h-12"
            class="checkin-providers__empty-icon-symbol"
          />
        </p>
        <p>暂无提供商配置</p>
        <p class="checkin-providers__empty-subtitle">
          点击上方内置中转站快速添加，或自定义添加
        </p>
      </div>
      <div
        v-else
        class="checkin-providers__provider-grid"
      >
        <div
          v-for="provider in providers"
          :key="provider.id"
          :class="[
            'checkin-providers__provider-card',
            provider.enabled
              ? 'checkin-providers__provider-card--enabled'
              : 'checkin-providers__provider-card--disabled',
          ]"
        >
          <div class="checkin-providers__provider-card-header">
            <div>
              <h3 class="checkin-providers__provider-title">
                {{ provider.name }}
              </h3>
              <p class="checkin-providers__provider-url">
                {{ provider.base_url }}
              </p>
            </div>
            <div class="checkin-providers__provider-actions">
              <button
                class="checkin-providers__icon-button checkin-providers__icon-button--edit"
                title="编辑"
                @click="openProviderModal(provider)"
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
                    d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                  />
                </svg>
              </button>
              <button
                class="checkin-providers__icon-button checkin-providers__icon-button--delete"
                title="删除"
                @click="deleteProvider(provider.id)"
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
                    d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                  />
                </svg>
              </button>
            </div>
          </div>
          <div class="checkin-providers__provider-meta">
            <span>签到路径: {{ provider.checkin_path }}</span>
          </div>
          <div
            v-if="requiresWafBypass(provider)"
            class="checkin-providers__waf-card"
          >
            <div class="checkin-providers__waf-card-layout">
              <div class="checkin-providers__waf-card-body">
                <div class="checkin-providers__waf-card-header">
                  <SIcon
                    name="ShieldCheck"
                    size="w-4 h-4"
                    class="checkin-providers__waf-icon"
                  />
                  <p class="checkin-providers__waf-title">
                    WAF 验证
                  </p>
                  <span
                    class="checkin-providers__tag checkin-badge-pill"
                    :class="hasCachedWafCookie(provider.id)
                      ? 'checkin-providers__tag--success'
                      : 'checkin-providers__tag--warning'"
                  >
                    {{ hasCachedWafCookie(provider.id) ? '已缓存 Cookie' : '未缓存 Cookie' }}
                  </span>
                </div>
                <p class="checkin-providers__waf-message">
                  AnyRouter 这类站点签到前需要先获取 WAF Cookie，且网页登录与签到请求必须使用同一代理/出口。
                </p>
                <p class="checkin-providers__waf-hint">
                  参考流程：先保存 <code>session</code> 和 <code>api_user</code>，再打开登录页完成挑战，最后回到签到页重试。
                </p>
              </div>
              <button
                class="checkin-providers__waf-action"
                :disabled="wafLoadingMap[provider.id] === true"
                @click="startWafLogin(provider)"
              >
                <SIcon
                  name="RefreshCw"
                  size="w-3.5 h-3.5"
                  :class="{ 'animate-spin': wafLoadingMap[provider.id] === true }"
                />
                <span>
                  {{
                    wafLoadingMap[provider.id] === true
                      ? '获取中...'
                      : hasCachedWafCookie(provider.id)
                        ? '重新获取'
                        : '获取 Cookie'
                  }}
                </span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- 提供商编辑弹窗 -->
  <div
    v-if="showProviderModal"
    class="checkin-providers__modal-overlay"
    @click.self="showProviderModal = false"
  >
    <div class="checkin-providers__modal-panel">
      <h3 class="checkin-providers__modal-title">
        {{ editingProvider ? '编辑提供商' : '添加提供商' }}
      </h3>
      <form
        class="checkin-providers__modal-form"
        @submit.prevent="saveProvider"
      >
        <div>
          <label class="checkin-providers__field-label">
            名称 *
          </label>
          <input
            v-model="providerForm.name"
            type="text"
            required
            class="checkin-providers__field-input"
            placeholder="例如: OpenRouter"
          >
        </div>
        <div>
          <label class="checkin-providers__field-label">
            Base URL *
          </label>
          <input
            v-model="providerForm.base_url"
            type="url"
            required
            class="checkin-providers__field-input"
            placeholder="https://api.example.com"
          >
        </div>
        <div class="checkin-providers__field-grid">
          <div>
            <label class="checkin-providers__field-label">
              签到路径
            </label>
            <input
              v-model="providerForm.checkin_path"
              type="text"
              class="checkin-providers__field-input"
              placeholder="/api/user/checkin"
            >
          </div>
          <div>
            <label class="checkin-providers__field-label">
              余额路径
            </label>
            <input
              v-model="providerForm.balance_path"
              type="text"
              class="checkin-providers__field-input"
              placeholder="/api/user/dashboard"
            >
          </div>
        </div>
        <div class="checkin-providers__field-grid">
          <div>
            <label class="checkin-providers__field-label">
              认证 Header
            </label>
            <input
              v-model="providerForm.auth_header"
              type="text"
              class="checkin-providers__field-input"
              placeholder="Authorization"
            >
          </div>
          <div>
            <label class="checkin-providers__field-label">
              认证前缀
            </label>
            <input
              v-model="providerForm.auth_prefix"
              type="text"
              class="checkin-providers__field-input"
              placeholder="Bearer "
            >
          </div>
        </div>
        <div class="checkin-providers__modal-actions">
          <button
            type="button"
            class="checkin-providers__secondary-button"
            @click="showProviderModal = false"
          >
            取消
          </button>
          <button
            type="submit"
            class="checkin-providers__primary-button"
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
import { ref, computed, watch } from 'vue'
import { useUIStore } from '@/stores/ui'
import {
  createCheckinProvider,
  updateCheckinProvider,
  deleteCheckinProvider as apiDeleteProvider,
  openWafLogin,
  getWafCookieStatus,
} from '@/api'
import type {
  CheckinProvider,
  BuiltinProvider,
  WafCookieRecoveryResult,
  WafCookieStatus,
} from '@/types/checkin'
import { logger } from '@/utils/logger'
import { getErrorMessage } from '@/types/api'
import {
  filterAvailableBuiltinProviders,
  resolveBuiltinProvider,
} from '../composables/builtinProviderLookup'

const props = defineProps<{
  providers: CheckinProvider[]
  builtinProviders: BuiltinProvider[]
}>()

const emit = defineEmits<{
  (e: 'add-builtin', builtinId: string): void
  (e: 'refresh'): void
}>()
const uiStore = useUIStore()

const formatWafRecoveryResult = (result: WafCookieRecoveryResult) => {
  if (result.missing_cookie_names.length > 0) {
    return `缺少 WAF Cookie: ${result.missing_cookie_names.join(', ')}`
  }
  return result.message || 'WAF Cookie 未获取完整'
}

// 计算属性：过滤出尚未添加的内置提供商（builtin_id 优先判定，name 回退兼容旧数据）
const availableBuiltinProviders = computed(() =>
  filterAvailableBuiltinProviders(props.builtinProviders, props.providers)
)

const wafStatusMap = ref<Record<string, WafCookieStatus | undefined>>({})
const wafLoadingMap = ref<Record<string, boolean>>({})

// builtin_id 优先反查内置站（改名安全），旧数据无 builtin_id 时回退 name 匹配
const getBuiltinProvider = (provider: CheckinProvider): BuiltinProvider | undefined => {
  return resolveBuiltinProvider(props.builtinProviders, provider)
}

const requiresWafBypass = (provider: CheckinProvider) => {
  return getBuiltinProvider(provider)?.requires_waf_bypass === true
}

const hasCachedWafCookie = (providerId: string) => {
  return wafStatusMap.value[providerId]?.has_cookie === true
}

const getProviderLoginUrl = (provider: CheckinProvider) => {
  return `${provider.base_url.replace(/\/+$/, '')}/login`
}

const loadWafStatus = async (providerId: string) => {
  try {
    const status = await getWafCookieStatus<WafCookieStatus>(providerId)
    wafStatusMap.value = {
      ...wafStatusMap.value,
      [providerId]: status,
    }
  } catch (error: unknown) {
    logger.warn('Failed to load WAF status', error)
  }
}

const refreshWafStatuses = async () => {
  const wafProviders = props.providers.filter((provider) => requiresWafBypass(provider))
  if (wafProviders.length === 0) {
    wafStatusMap.value = {}
    return
  }

  await Promise.all(wafProviders.map((provider) => loadWafStatus(provider.id)))
}

const startWafLogin = async (provider: CheckinProvider) => {
  wafLoadingMap.value = {
    ...wafLoadingMap.value,
    [provider.id]: true,
  }

  try {
    const result = await openWafLogin<WafCookieRecoveryResult>(
      getProviderLoginUrl(provider),
      provider.id
    )
    await loadWafStatus(provider.id)
    if (result.persisted) {
      uiStore.showSuccess(`${provider.name} 的 WAF Cookie 已更新，现在可以回到签到页重试。`)
    } else {
      uiStore.showError(`获取 WAF Cookie 失败: ${formatWafRecoveryResult(result)}`)
    }
  } catch (error: unknown) {
    uiStore.showError('获取 WAF Cookie 失败: ' + getErrorMessage(error, '未知错误'))
  } finally {
    wafLoadingMap.value = {
      ...wafLoadingMap.value,
      [provider.id]: false,
    }
  }
}

watch(
  () => props.providers.map((provider) => `${provider.id}:${provider.name}`).join('|'),
  () => {
    void refreshWafStatuses()
  },
  { immediate: true }
)

// 弹窗状态
const showProviderModal = ref(false)
const editingProvider = ref<CheckinProvider | null>(null)

// 表单
const providerForm = ref({
  name: '',
  base_url: '',
  checkin_path: '/api/user/checkin',
  balance_path: '/api/user/self',
  user_info_path: '/api/user/self',
  auth_header: 'Authorization',
  auth_prefix: 'Bearer ',
})

// 提供商操作
const openProviderModal = (provider?: CheckinProvider) => {
  editingProvider.value = provider || null
  if (provider) {
    providerForm.value = {
      name: provider.name,
      base_url: provider.base_url,
      checkin_path: provider.checkin_path,
      balance_path: provider.balance_path,
      user_info_path: provider.user_info_path,
      auth_header: provider.auth_header,
      auth_prefix: provider.auth_prefix,
    }
  } else {
    providerForm.value = {
      name: '',
      base_url: '',
      checkin_path: '/api/user/checkin',
      balance_path: '/api/user/self',
      user_info_path: '/api/user/self',
      auth_header: 'Authorization',
      auth_prefix: 'Bearer ',
    }
  }
  showProviderModal.value = true
}

const saveProvider = async () => {
  try {
    if (editingProvider.value) {
      await updateCheckinProvider(editingProvider.value.id, providerForm.value)
    } else {
      await createCheckinProvider(providerForm.value)
    }
    showProviderModal.value = false
    uiStore.showSuccess(editingProvider.value ? '提供商已更新' : '提供商已添加')
    emit('refresh')
  } catch (e: unknown) {
    uiStore.showError('保存失败: ' + getErrorMessage(e, '未知错误'))
  }
}

const deleteProvider = async (id: string) => {
  const confirmed = await uiStore.requestConfirm({
    title: '删除提供商',
    message: '确定要删除此提供商吗？相关账号也会被删除。',
    confirmText: '删除',
    cancelText: '取消',
    type: 'danger',
    surface: 'solid',
  })
  if (!confirmed) return
  try {
    await apiDeleteProvider(id)
    uiStore.showSuccess('提供商已删除')
    emit('refresh')
  } catch (e: unknown) {
    uiStore.showError('删除失败: ' + getErrorMessage(e, '未知错误'))
  }
}
</script>

<style scoped>
.checkin-providers,
.checkin-providers__modal-form {
  display: flex;
  flex-direction: column;
}

.checkin-providers {
  gap: 1.5rem;
}

.checkin-providers__section-header,
.checkin-providers__section-heading,
.checkin-providers__builtin-card-header,
.checkin-providers__builtin-card-main,
.checkin-providers__builtin-card-title-row,
.checkin-providers__provider-card-header,
.checkin-providers__provider-actions,
.checkin-providers__provider-meta,
.checkin-providers__waf-card-layout,
.checkin-providers__waf-card-header,
.checkin-providers__primary-button,
.checkin-providers__secondary-button,
.checkin-providers__modal-overlay,
.checkin-providers__modal-actions {
  display: flex;
  align-items: center;
}

.checkin-providers__section-header,
.checkin-providers__provider-card-header,
.checkin-providers__waf-card-layout,
.checkin-providers__modal-actions {
  justify-content: space-between;
}

.checkin-providers__section-header {
  margin-bottom: 1rem;
  gap: 0.5rem;
}

.checkin-providers__section-header--split {
  align-items: center;
}

.checkin-providers__section-heading {
  gap: 0.5rem;
}

.checkin-providers__section-title {
  font-size: 1.125rem;
  line-height: 1.75rem;
  font-weight: 600;
  color: var(--text-primary);
}

.checkin-providers__section-count {
  font-size: 0.875rem;
  color: var(--text-muted);
}

.checkin-providers__section-icon--primary {
  color: var(--accent-primary);
}

.checkin-providers__section-icon--secondary {
  color: var(--accent-secondary);
}

.checkin-providers__builtin-grid,
.checkin-providers__provider-grid,
.checkin-providers__field-grid {
  display: grid;
  gap: 1rem;
}

.checkin-providers__builtin-grid,
.checkin-providers__provider-grid {
  grid-template-columns: repeat(1, minmax(0, 1fr));
}

.checkin-providers__builtin-card,
.checkin-providers__provider-card {
  border-radius: 0.75rem;
  padding: 1rem;
}

.checkin-providers__builtin-card {
  border: 1px solid var(--color-border-default);
  background: var(--color-bg-surface);
  box-shadow: var(--shadow-xs);
  transition: box-shadow 0.2s ease;
}

.checkin-providers__builtin-card:hover {
  box-shadow: var(--shadow-md);
}

.checkin-providers__builtin-card-main {
  align-items: flex-start;
  gap: 0.75rem;
}

.checkin-providers__builtin-card-emoji {
  font-size: 1.5rem;
  line-height: 2rem;
}

.checkin-providers__builtin-card-title-row {
  gap: 0.5rem;
}

.checkin-providers__builtin-card-title,
.checkin-providers__provider-title {
  font-weight: 600;
  color: var(--text-primary);
}

/* 形状配方由全局 .checkin-badge-pill 提供，这里保留尺寸差异 */
.checkin-providers__builtin-badge,
.checkin-providers__tag {
  padding: 0.125rem 0.5rem;
  line-height: 1rem;
}

.checkin-providers__builtin-badge {
  background: rgb(var(--color-info-rgb) / 15%);
  color: var(--color-info);
}

.checkin-providers__builtin-domain,
.checkin-providers__builtin-description,
.checkin-providers__provider-url,
.checkin-providers__provider-meta,
.checkin-providers__empty-subtitle {
  font-size: 0.875rem;
  color: var(--text-muted);
}

.checkin-providers__builtin-domain {
  margin-top: 0.125rem;
}

.checkin-providers__builtin-description {
  margin-top: 0.75rem;
  color: var(--text-secondary);
}

.checkin-providers__tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-top: 0.75rem;
}

.checkin-providers__tag {
  gap: 0.25rem;
}

.checkin-providers__tag--success {
  background: rgb(var(--color-success-rgb) / 15%);
  color: var(--color-success);
}

.checkin-providers__tag--warning {
  background: rgb(var(--color-warning-rgb) / 15%);
  color: var(--color-warning);
}

.checkin-providers__tag--muted {
  background: var(--color-bg-overlay);
  color: var(--text-secondary);
}

.checkin-providers__primary-button,
.checkin-providers__secondary-button,
.checkin-providers__waf-action {
  gap: 0.5rem;
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
  transition: background-color 0.2s ease, color 0.2s ease, opacity 0.2s ease;
}

.checkin-providers__primary-button {
  background: var(--color-accent-primary);
  color: white;
}

.checkin-providers__primary-button:hover {
  background: var(--color-accent-primary-hover);
}

.checkin-providers__primary-button--compact {
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
}

.checkin-providers__empty-state {
  border-radius: 0.5rem;
  background: var(--color-bg-surface);
  padding: 3rem 1rem;
  text-align: center;
  color: var(--text-muted);
}

.checkin-providers__empty-icon {
  margin-bottom: 0.75rem;
  font-size: 2.25rem;
}

.checkin-providers__empty-icon-symbol {
  margin-inline: auto;
  color: var(--text-disabled);
}

.checkin-providers__provider-card {
  border-left: 4px solid;
  background: var(--color-bg-surface);
  box-shadow: var(--shadow-xs);
}

.checkin-providers__provider-card--enabled {
  border-left-color: var(--color-success);
}

.checkin-providers__provider-card--disabled {
  border-left-color: var(--text-disabled);
}

.checkin-providers__provider-url {
  margin-top: 0.25rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.checkin-providers__provider-actions {
  gap: 0.5rem;
}

.checkin-providers__icon-button--edit {
  color: var(--color-info);
}

.checkin-providers__icon-button--delete {
  color: var(--color-danger);
}

.checkin-providers__provider-meta {
  margin-top: 0.75rem;
  gap: 1rem;
  font-size: 0.75rem;
}

.checkin-providers__waf-card {
  margin-top: 1rem;
  border-radius: 0.5rem;
  border: 1px solid rgb(var(--color-warning-rgb) / 40%);
  background: rgb(var(--color-warning-rgb) / 12%);
  padding: 0.75rem;
}

.checkin-providers__waf-card-layout {
  align-items: flex-start;
  gap: 0.75rem;
}

.checkin-providers__waf-card-body {
  min-width: 0;
}

.checkin-providers__waf-card-header {
  gap: 0.5rem;
  flex-wrap: wrap;
}

.checkin-providers__waf-icon {
  color: var(--color-warning);
}

.checkin-providers__waf-title {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-warning);
}

.checkin-providers__waf-message,
.checkin-providers__waf-hint {
  font-size: 0.75rem;
  line-height: 1.25rem;
}

.checkin-providers__waf-message {
  margin-top: 0.5rem;
  color: rgb(var(--color-warning-rgb) / 92%);
}

.checkin-providers__waf-hint {
  margin-top: 0.25rem;
  color: rgb(var(--color-warning-rgb) / 82%);
}

.checkin-providers__waf-action {
  background: var(--color-warning);
  color: white;
}

.checkin-providers__waf-action:hover:not(:disabled) {
  background: var(--color-warning-hover);
}

.checkin-providers__waf-action:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.checkin-providers__modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  justify-content: center;
  background: rgb(0 0 0 / 50%);
  padding: 1rem;
}

.checkin-providers__modal-panel {
  width: 100%;
  max-width: 32rem;
  border-radius: 0.5rem;
  background: var(--color-bg-elevated);
  padding: 1.5rem;
  box-shadow: var(--shadow-xl);
}

.checkin-providers__modal-title {
  margin-bottom: 1rem;
  font-size: 1.25rem;
  line-height: 1.75rem;
  font-weight: 600;
  color: var(--text-primary);
}

.checkin-providers__modal-form {
  gap: 1rem;
}

.checkin-providers__field-label {
  display: block;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.checkin-providers__field-input {
  display: block;
  width: 100%;
  margin-top: 0.25rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.5rem;
  background: var(--color-bg-surface);
  padding: 0.5rem 0.75rem;
  color: var(--text-primary);
}

.checkin-providers__modal-actions {
  gap: 0.75rem;
  padding-top: 1rem;
}

.checkin-providers__secondary-button {
  border: 1px solid var(--color-border-default);
  color: var(--text-secondary);
}

.checkin-providers__secondary-button:hover {
  background: var(--color-bg-elevated);
}

@media (width >= 768px) {
  .checkin-providers__builtin-grid,
  .checkin-providers__provider-grid,
  .checkin-providers__field-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width >= 1024px) {
  .checkin-providers__builtin-grid,
  .checkin-providers__provider-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

@media (width <= 767px) {
  .checkin-providers__section-header--split,
  .checkin-providers__builtin-card-header,
  .checkin-providers__provider-card-header,
  .checkin-providers__waf-card-layout {
    flex-direction: column;
    align-items: flex-start;
  }

  .checkin-providers__primary-button,
  .checkin-providers__secondary-button,
  .checkin-providers__waf-action {
    width: 100%;
    justify-content: center;
  }

  .checkin-providers__provider-actions {
    width: 100%;
    justify-content: flex-end;
  }
}
</style>
