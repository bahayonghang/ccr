<template>
  <PageShell class="checkin-view">
    <template #header>
      <PageHeader
        :title="t('checkin.title')"
        :description="t('checkin.description')"
      >
        <template #actions>
          <button
            :disabled="loading || checkinLoading || enabledAccounts.length === 0"
            class="checkin-view__action-button checkin-view__action-button--checkin"
            @click="showCheckinConfirm = true"
          >
            <svg
              class="w-5 h-5"
              :class="{ 'animate-spin': checkinLoading }"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <span>{{ checkinLoading ? t('checkin.actions.checkingAll') : t('checkin.actions.checkAll') }}</span>
          </button>
          <button
            :disabled="balanceRefreshing || accounts.length === 0"
            class="checkin-view__action-button checkin-view__action-button--balance"
            @click="refreshAllBalances"
          >
            <svg
              class="w-5 h-5"
              :class="{ 'animate-spin': balanceRefreshing }"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
            <span>{{ balanceRefreshing ? t('checkin.actions.refreshing') : t('checkin.actions.refreshBalances') }}</span>
          </button>
        </template>
      </PageHeader>
    </template>

    <!-- 加载状态 -->
    <div
      v-if="loading"
      class="checkin-view__loading"
    >
      <div class="checkin-view__loading-spinner" />
    </div>

    <!-- 错误提示 -->
    <div
      v-if="error"
      class="checkin-view__error"
    >
      <div class="checkin-view__error-content">
        <svg
          class="checkin-view__error-icon"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd"
          />
        </svg>
        <div class="checkin-view__error-body">
          <h3 class="checkin-view__error-title">
            {{ t('checkin.errors.loadFailed') }}
          </h3>
          <p class="checkin-view__error-message">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <!-- 签到结果汇总 -->
    <div
      v-if="checkinResult"
      ref="checkinResultRef"
      :class="[
        'checkin-view__result',
        checkinFlowPhase === 'recovering'
          ? 'checkin-view__result--recovering'
          : checkinResult.summary.failed > 0
            ? 'checkin-view__result--warning'
            : 'checkin-view__result--success',
      ]"
    >
      <div class="checkin-view__result-header">
        <div class="checkin-view__result-main">
          <div class="checkin-view__result-status">
            <svg
              :class="[
                'checkin-view__result-status-icon',
                checkinFlowPhase === 'recovering'
                  ? 'checkin-view__result-status-icon--recovering'
                  : checkinResult.summary.failed > 0
                    ? 'checkin-view__result-status-icon--warning'
                    : 'checkin-view__result-status-icon--success',
              ]"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                v-if="checkinFlowPhase === 'recovering'"
                fill-rule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm1.53-11.53a.75.75 0 011.06 1.06L10.06 10l2.53 2.47a.75.75 0 01-1.06 1.06l-3.06-3a.75.75 0 010-1.06l3.06-3z"
                clip-rule="evenodd"
              />
              <path
                v-else-if="checkinResult.summary.failed > 0"
                fill-rule="evenodd"
                d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                clip-rule="evenodd"
              />
              <path
                v-else
                fill-rule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                clip-rule="evenodd"
              />
            </svg>
            <h3
              :class="[
                'checkin-view__result-status-title',
                checkinFlowPhase === 'recovering'
                  ? 'checkin-view__result-status-title--recovering'
                  : checkinResult.summary.failed > 0
                    ? 'checkin-view__result-status-title--warning'
                    : 'checkin-view__result-status-title--success',
              ]"
            >
              {{
                checkinFlowPhase === 'recovering'
                  ? t('checkin.result.recoveringTitle')
                  : checkinResult.summary.failed > 0 ? t('checkin.result.completedWithFailures') : t('checkin.result.completed')
              }}
            </h3>
          </div>
          <!-- 结果统计 -->
          <div class="checkin-view__result-summary">
            <span class="checkin-view__result-badge checkin-view__result-badge--success">
              <SIcon
                name="CheckCircle"
                size="w-3.5 h-3.5"
              />
              {{ t('checkin.result.summarySuccess', { count: checkinResult.summary.success }) }}
            </span>
            <span class="checkin-view__result-badge checkin-view__result-badge--info">
              <SIcon
                name="Calendar"
                size="w-3.5 h-3.5"
              />
              {{ t('checkin.result.summaryAlready', { count: checkinResult.summary.already_checked_in }) }}
            </span>
            <span class="checkin-view__result-badge checkin-view__result-badge--danger">
              <SIcon
                name="XCircle"
                size="w-3.5 h-3.5"
              />
              {{ t('checkin.result.summaryFailed', { count: checkinResult.summary.failed }) }}
            </span>
            <span
              v-if="(checkinResult.summary.skipped ?? 0) > 0"
              class="checkin-view__result-badge checkin-view__result-badge--muted"
            >
              <SIcon
                name="Circle"
                size="w-3.5 h-3.5"
              />
              {{ t('checkin.result.summarySkipped', { count: checkinResult.summary.skipped }) }}
            </span>
            <span class="checkin-view__result-badge checkin-view__result-badge--neutral">
              {{ t('checkin.result.summaryTotal', { count: checkinResult.summary.total }) }}
            </span>
          </div>
          <div class="checkin-view__result-grid">
            <div
              v-if="wafRecoveryRunning && wafRecoveryMessage"
              class="checkin-view__callout checkin-view__callout--recovery"
            >
              <div class="checkin-view__callout-layout">
                <SIcon
                  name="Loader2"
                  size="h-4 w-4"
                  class="checkin-view__callout-icon checkin-view__callout-icon--recovery animate-spin"
                />
                <div>
                  <p class="checkin-view__callout-title checkin-view__callout-title--recovery">
                    {{ t('checkin.result.recoveringTitle') }}
                  </p>
                  <p class="checkin-view__callout-message checkin-view__callout-message--recovery">
                    {{ wafRecoveryMessage }}
                  </p>
                  <p
                    v-if="wafRecoveryProviderName"
                    class="checkin-view__callout-meta checkin-view__callout-meta--recovery"
                  >
                    {{ t('checkin.result.currentProvider', { provider: wafRecoveryProviderName }) }}
                  </p>
                </div>
              </div>
            </div>
            <div
              v-if="failedCheckinResults.some((item) => item.error_code === 'waf_blocked')"
              class="checkin-view__callout checkin-view__callout--waf"
            >
              <div class="checkin-view__callout-layout checkin-view__callout-layout--split">
                <div>
                  <p class="checkin-view__callout-title checkin-view__callout-title--waf">
                    {{ wafRecoveryRunning ? t('checkin.waf.runningTitle') : t('checkin.waf.detectedTitle') }}
                  </p>
                  <p class="checkin-view__callout-message checkin-view__callout-message--waf">
                    <template v-if="wafRecoveryRunning">
                      {{ t('checkin.waf.runningMessage') }}
                    </template>
                    <template v-else>
                      {{ t('checkin.waf.detectedMessage') }}
                    </template>
                  </p>
                </div>
                <button
                  class="checkin-view__callout-action"
                  @click="activeTab = 'providers'"
                >
                  {{ t('checkin.actions.openProviders') }}
                </button>
              </div>
            </div>
            <!-- 成功结果 -->
            <div
              v-if="successCheckinResults.length > 0"
              class="checkin-view__result-section"
            >
              <p class="checkin-view__result-section-title checkin-view__result-section-title--success">
                {{ t('checkin.result.successTitle', { count: successCheckinResults.length }) }}
              </p>
              <div class="checkin-view__result-list">
                <div
                  v-for="item in successCheckinResults"
                  :key="item.account_id"
                  class="checkin-view__result-item checkin-view__result-item--success"
                >
                  <SIcon
                    name="CheckCircle"
                    size="w-4 h-4"
                    class="checkin-view__result-item-icon checkin-view__result-item-icon--success"
                  />
                  <div class="checkin-view__result-item-body">
                    <div class="checkin-view__result-item-meta">
                      <span class="checkin-view__result-item-name checkin-view__result-item-name--success">
                        {{ item.account_name }}
                      </span>
                      <span class="checkin-view__result-tag checkin-view__result-tag--success">
                        {{ item.provider_name }}
                      </span>
                      <span
                        v-if="item.reward"
                        class="checkin-view__result-tag checkin-view__result-tag--reward"
                      >
                        {{ t('checkin.result.reward', { reward: item.reward }) }}
                      </span>
                      <span
                        v-if="item.waf_recovery_attempted && item.waf_recovered"
                        class="checkin-view__result-tag checkin-view__result-tag--recovery"
                      >
                        {{ t('checkin.result.recoverySuccess') }}
                      </span>
                    </div>
                    <p class="checkin-view__result-message checkin-view__result-message--success">
                      {{ getSuccessDetail(item) }}
                    </p>
                  </div>
                </div>
              </div>
            </div>
            <!-- 失败结果 -->
            <div
              v-if="failedCheckinResults.length > 0"
              class="checkin-view__result-section"
            >
              <p class="checkin-view__result-section-title checkin-view__result-section-title--danger">
                {{ t('checkin.result.failedTitle', { count: failedCheckinResults.length }) }}
              </p>
              <div class="checkin-view__result-list">
                <div
                  v-for="item in failedCheckinResults"
                  :key="item.account_id"
                  class="checkin-view__result-item checkin-view__result-item--danger"
                >
                  <SIcon
                    name="XCircle"
                    size="w-4 h-4"
                    class="checkin-view__result-item-icon checkin-view__result-item-icon--danger"
                  />
                  <div class="checkin-view__result-item-body">
                    <div class="checkin-view__result-item-meta">
                      <span class="checkin-view__result-item-name checkin-view__result-item-name--danger">
                        {{ item.account_name }}
                      </span>
                      <span class="checkin-view__result-tag checkin-view__result-tag--danger">
                        {{ item.provider_name }}
                      </span>
                      <span
                        v-if="item.error_code"
                        class="checkin-view__result-tag checkin-view__result-tag--warning"
                      >
                        {{ getErrorLabel(item.error_code) }}
                      </span>
                      <span
                        v-if="item.waf_recovery_attempted && item.waf_recovered === false"
                        class="checkin-view__result-tag checkin-view__result-tag--recovery"
                      >
                        {{ t('checkin.result.recoveryStillFailed') }}
                      </span>
                      <button
                        v-if="item.error_code === 'cookie_expired'"
                        type="button"
                        class="checkin-view__result-fix-button"
                        @click="openAccountCookieFix(item.account_id)"
                      >
                        {{ t('checkin.actions.updateCookie') }}
                      </button>
                    </div>
                    <p class="checkin-view__result-message checkin-view__result-message--danger">
                      {{ getFailedDetail(item) }}
                    </p>
                  </div>
                </div>
              </div>
            </div>
            <!-- 跳过结果（4 态：不计入失败） -->
            <div
              v-if="skippedCheckinResults.length > 0"
              class="checkin-view__result-section"
            >
              <p class="checkin-view__result-section-title checkin-view__result-section-title--muted">
                {{ t('checkin.result.skippedTitle', { count: skippedCheckinResults.length }) }}
              </p>
              <div class="checkin-view__result-list">
                <div
                  v-for="item in skippedCheckinResults"
                  :key="item.account_id"
                  class="checkin-view__result-item checkin-view__result-item--muted"
                >
                  <SIcon
                    name="Circle"
                    size="w-4 h-4"
                    class="checkin-view__result-item-icon checkin-view__result-item-icon--muted"
                  />
                  <div class="checkin-view__result-item-body">
                    <div class="checkin-view__result-item-meta">
                      <span class="checkin-view__result-item-name checkin-view__result-item-name--muted">
                        {{ item.account_name }}
                      </span>
                      <span class="checkin-view__result-tag checkin-view__result-tag--muted">
                        {{ item.provider_name }}
                      </span>
                    </div>
                    <p class="checkin-view__result-message checkin-view__result-message--muted">
                      {{ getSkippedDetail(item) }}
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <!-- 已签到结果 -->
          <div
            v-if="alreadyCheckedInResults.length > 0"
            class="checkin-view__result-section checkin-view__result-section--spaced"
          >
            <p class="checkin-view__result-section-title checkin-view__result-section-title--info">
              {{ t('checkin.result.alreadyTitle', { count: alreadyCheckedInResults.length }) }}
            </p>
            <div class="checkin-view__result-list">
              <div
                v-for="item in alreadyCheckedInResults"
                :key="item.account_id"
                class="checkin-view__result-item checkin-view__result-item--info"
              >
                <SIcon
                  name="Calendar"
                  size="w-4 h-4"
                  class="checkin-view__result-item-icon checkin-view__result-item-icon--info"
                />
                <div class="checkin-view__result-item-body">
                  <div class="checkin-view__result-item-meta">
                    <span class="checkin-view__result-item-name checkin-view__result-item-name--info">
                      {{ item.account_name }}
                    </span>
                    <span class="checkin-view__result-tag checkin-view__result-tag--info">
                      {{ item.provider_name }}
                    </span>
                    <span
                      v-if="item.waf_recovery_attempted && item.waf_recovered"
                      class="checkin-view__result-tag checkin-view__result-tag--recovery"
                    >
                      {{ t('checkin.result.recoveryCompleted') }}
                    </span>
                  </div>
                  <p class="checkin-view__result-message checkin-view__result-message--info">
                    {{ getAlreadyCheckedInDetail(item) }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
        <button
          :class="[
            'checkin-view__result-close',
            checkinResult.summary.failed > 0
              ? 'checkin-view__result-close--warning'
              : 'checkin-view__result-close--success',
          ]"
          @click="checkinResult = null"
        >
          <svg
            class="w-5 h-5"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fill-rule="evenodd"
              d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
              clip-rule="evenodd"
            />
          </svg>
        </button>
      </div>
    </div>

    <!-- 页面主体 -->
    <div
      v-if="!loading && !error"
      class="checkin-view__content"
    >
      <div class="checkin-view__stats">
        <StatTile
          :label="t('checkin.stats.currentBalance')"
          :value="`$${totalStatistics.currentBalance.toFixed(2)}`"
        />
        <StatTile
          :label="t('checkin.stats.totalQuota')"
          :value="`$${totalStatistics.totalQuota.toFixed(2)}`"
        />
        <StatTile
          :label="t('checkin.stats.totalConsumed')"
          :value="`$${totalStatistics.totalConsumed.toFixed(2)}`"
        />
      </div>

      <div class="checkin-view__tabs-shell">
        <PillToggleGroup
          :options="tabToggleOptions"
          :model-value="activeTab"
          @update:model-value="activeTab = $event"
        />
      </div>

      <!-- Tab 内容 -->
      <CheckinProvidersTab
        v-if="activeTab === 'providers'"
        :providers="providers"
        :builtin-providers="builtinProviders"
        @add-builtin="addBuiltinProvider"
        @refresh="loadAllData"
      />
      <CheckinAccountsTab
        v-if="activeTab === 'accounts'"
        :accounts="accounts"
        :checkin-loading="checkinLoading"
        :providers="providers"
        :builtin-providers="builtinProviders"
        :pending-edit-account-id="pendingEditAccountId"
        @refresh="loadAllData"
        @checkin="executeCheckinSingle"
        @refresh-balance="refreshAccountBalance"
        @navigate="openAccountDashboard"
        @show-oauth-wizard="showOAuthWizard = true"
        @pending-edit-consumed="clearPendingEditAccount"
      />
      <CheckinRecordsTab
        v-if="activeTab === 'records'"
        :records="records"
        :records-load-error="recordsLoadError"
        :providers="providers"
        :accounts="accounts"
        :today-stats="todayStats"
        @update-cookie="openAccountCookieFix"
      />
      <CheckinImportExportTab
        v-if="activeTab === 'import-export'"
        @refresh="loadAllData"
      />
    </div>

    <!-- 一键签到确认弹窗 -->
    <ConfirmModal
      :is-open="showCheckinConfirm"
      :title="t('checkin.dialog.confirmAllTitle')"
      :message="t('checkin.dialog.confirmAllMessage', { count: enabledAccounts.length })"
      :confirm-text="t('checkin.dialog.startCheckin')"
      :cancel-text="t('common.cancel')"
      type="info"
      surface="solid"
      @confirm="handleCheckinConfirm"
      @cancel="showCheckinConfirm = false"
      @update:is-open="showCheckinConfirm = $event"
    />

    <!-- 签到进度弹窗 -->
    <CheckinProgressModal
      :is-open="showProgressModal"
      :total="checkinProgress.total"
      :current="checkinProgress.completed"
      :current-account-name="checkinProgress.currentAccountName"
      :logs="checkinLogs"
      :phase="checkinFlowPhase"
      :recovery-message="wafRecoveryMessage"
      :recovery-provider-name="wafRecoveryProviderName"
      @close="closeCheckinModal"
    />

    <!-- OAuth 向导弹窗 -->
    <OAuthWizardModal
      :is-open="showOAuthWizard"
      :builtin-providers="builtinProviders"
      @update:is-open="showOAuthWizard = $event"
      @close="showOAuthWizard = false"
      @success="handleOAuthSuccess"
    />
  </PageShell>
</template>

<script setup lang="ts">
defineOptions({ name: 'CheckinView' })

import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import PillToggleGroup from '@/components/ui/PillToggleGroup.vue'
import StatTile from '@/components/ui/StatTile.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useCheckinState } from './checkin/composables/useCheckinState'
// Tab 组件
import CheckinProvidersTab from './checkin/tabs/CheckinProvidersTab.vue'
import CheckinAccountsTab from './checkin/tabs/CheckinAccountsTab.vue'
import CheckinRecordsTab from './checkin/tabs/CheckinRecordsTab.vue'
import CheckinImportExportTab from './checkin/tabs/CheckinImportExportTab.vue'
// 弹窗组件
import ConfirmModal from '@/components/ConfirmModal.vue'
import CheckinProgressModal from '@/components/CheckinProgressModal.vue'
import OAuthWizardModal from '@/views/checkin/components/OAuthWizardModal.vue'
// 图标：去除未使用项，仅保留当前界面需要的图标
const router = useRouter()
const { t } = useI18n()

const {
  // 状态
  loading,
  checkinLoading,
  balanceRefreshing,
  error,
  recordsLoadError,
  checkinResultRef,
  activeTab,
  showCheckinConfirm,
  showProgressModal,
  showOAuthWizard,
  checkinFlowPhase,
  checkinProgress,
  checkinLogs,
  wafRecoveryRunning,
  wafRecoveryProviderName,
  wafRecoveryMessage,
  // 数据
  providers,
  accounts,
  records,
  checkinResult,
  builtinProviders,
  todayStats,
  // 计算属性
  totalStatistics,
  enabledAccounts,
  failedCheckinResults,
  successCheckinResults,
  alreadyCheckedInResults,
  skippedCheckinResults,
  // Tab 配置
  tabs,
  // 数据加载
  loadAllData,
  // 签到操作
  executeCheckinSingle,
  handleCheckinConfirm,
  closeCheckinModal,
  handleOAuthSuccess,
  // Cookie 快捷修复
  pendingEditAccountId,
  openAccountCookieFix,
  clearPendingEditAccount,
  // 余额操作
  refreshAllBalances,
  refreshAccountBalance,
  // 内置提供商操作
  addBuiltinProvider,
  // 结果详情格式化
  getSuccessDetail,
  getAlreadyCheckedInDetail,
  getFailedDetail,
  getSkippedDetail,
  getErrorLabel,
} = useCheckinState()

const tabToggleOptions = computed(() =>
  tabs.map((tab) => ({
    value: tab.id,
    label: t(tab.nameKey),
  })),
)

const openAccountDashboard = (accountId: string) => {
  router.push({ name: 'checkin-account-dashboard', params: { accountId } })
}
</script>

<style scoped>
.checkin-view {
  min-width: 0;
}

.checkin-view__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.checkin-view__header-copy {
  min-width: 0;
}

.checkin-view__title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  color: var(--color-text-primary);
  font-size: 2rem;
  line-height: 1.1;
  font-weight: 600;
  letter-spacing: -0.045em;
  font-kerning: normal;
}

.checkin-view__title-icon-shell {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.5rem;
  height: 2.5rem;
  flex-shrink: 0;
  border-radius: 0.875rem;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 16%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  box-shadow: var(--shadow-inner);
}

.checkin-view__title-icon {
  color: var(--color-accent-primary);
}


.checkin-view__title-label {
  min-width: 0;
}

.checkin-view__subtitle {
  max-width: 40ch;
  margin-top: 0.625rem;
  color: var(--color-text-secondary);
  font-size: 0.9375rem;
  line-height: 1.5rem;
}

.checkin-view__actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.checkin-view__action-button {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 44px;
  border: 1px solid transparent;
  border-radius: 0.875rem;
  padding: 0.55rem 1.05rem;
  font-weight: 700;
  letter-spacing: -0.01em;
  white-space: nowrap;
  box-shadow: var(--shadow-sm);
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    opacity 0.2s ease,
    transform 0.2s ease,
    filter 0.2s ease;
}

.checkin-view__action-button:disabled {
  cursor: not-allowed;
  opacity: 0.58;
  filter: grayscale(0.32);
  box-shadow: none;
}

.checkin-view__action-button:hover:not(:disabled) {
  transform: translateY(-1px);
}

.checkin-view__action-button--checkin {
  background: var(--color-success);
  color: var(--color-success-contrast);
}

.checkin-view__action-button--checkin:hover:not(:disabled) {
  background: var(--color-success-hover);
  box-shadow: var(--shadow-md);
}

.checkin-view__action-button--balance {
  background: var(--color-accent-primary);
  color: var(--color-accent-primary-contrast);
}

.checkin-view__action-button--balance:hover:not(:disabled) {
  background: var(--color-accent-primary-hover);
  box-shadow: var(--shadow-md);
}

.checkin-view__loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 3rem 0;
}

.checkin-view__loading-spinner {
  width: 3rem;
  height: 3rem;
  border-radius: 9999px;
  border-bottom: 2px solid var(--color-info);
  animation: spin 1s linear infinite;
}

.checkin-view__error {
  border-radius: 0.5rem;
  border: 1px solid rgb(var(--color-danger-rgb) / 30%);
  background: rgb(var(--color-danger-rgb) / 8%);
  padding: 1rem;
}


.checkin-view__error-content {
  display: flex;
}

.checkin-view__error-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--color-danger);
}

.checkin-view__error-body {
  margin-left: 0.75rem;
}

.checkin-view__error-title {
  color: var(--color-danger);
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}


.checkin-view__error-message {
  margin-top: 0.5rem;
  color: var(--color-danger);
  font-size: 0.875rem;
  line-height: 1.25rem;
}


.checkin-view__result {
  border-radius: 0.5rem;
  border-width: 1px;
  padding: 1rem;
  box-shadow: var(--shadow-xs);
}

.checkin-view__result--recovering {
  border-color: rgb(var(--color-info-rgb) / 30%);
  background: rgb(var(--color-info-rgb) / 8%);
}


.checkin-view__result--warning {
  border-color: rgb(var(--color-warning-rgb) / 30%);
  background: rgb(var(--color-warning-rgb) / 8%);
}


.checkin-view__result--success {
  border-color: rgb(var(--color-success-rgb) / 30%);
  background: rgb(var(--color-success-rgb) / 8%);
}


.checkin-view__result-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.checkin-view__result-main {
  flex: 1 1 auto;
}

.checkin-view__result-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.checkin-view__result-status-icon {
  width: 1.25rem;
  height: 1.25rem;
}

.checkin-view__result-status-icon--recovering {
  color: var(--color-info);
}

.checkin-view__result-status-icon--warning {
  color: var(--color-warning);
}

.checkin-view__result-status-icon--success {
  color: var(--color-success);
}

.checkin-view__result-status-title {
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}

.checkin-view__result-status-title--recovering {
  color: var(--color-info);
}


.checkin-view__result-status-title--warning {
  color: var(--color-warning);
}


.checkin-view__result-status-title--success {
  color: var(--color-success);
}


.checkin-view__result-summary {
  margin-top: 0.75rem;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  line-height: 1rem;
}

.checkin-view__result-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border-radius: 9999px;
  padding: 0.25rem 0.5rem;
}

.checkin-view__result-badge--success {
  background: rgb(var(--color-success-rgb) / 15%);
  color: var(--color-success);
}


.checkin-view__result-badge--info {
  background: rgb(var(--color-info-rgb) / 15%);
  color: var(--color-info);
}


.checkin-view__result-badge--danger {
  background: rgb(var(--color-danger-rgb) / 15%);
  color: var(--color-danger);
}


.checkin-view__result-badge--neutral {
  background: var(--color-bg-overlay);
  color: var(--color-text-secondary);
}


.checkin-view__result-badge--muted {
  background: var(--color-bg-overlay);
  color: var(--color-text-muted);
}


.checkin-view__result-grid {
  margin-top: 1rem;
  display: grid;
  gap: 1rem;
}

@media (width >= 768px) {
  .checkin-view__result-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

.checkin-view__callout {
  border-radius: 0.5rem;
  border: 1px solid;
  padding: 0.75rem;
}

.checkin-view__callout--recovery,
.checkin-view__callout--waf {
  grid-column: 1 / -1;
}

.checkin-view__callout--recovery {
  border-color: rgb(var(--color-info-rgb) / 30%);
  background: rgb(var(--color-info-rgb) / 8%);
}


.checkin-view__callout--waf {
  border-color: rgb(var(--color-warning-rgb) / 30%);
  background: rgb(var(--color-warning-rgb) / 8%);
}


.checkin-view__callout-layout {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.checkin-view__callout-layout--split {
  justify-content: space-between;
}

.checkin-view__callout-icon {
  margin-top: 0.125rem;
}

.checkin-view__callout-icon--recovery {
  color: var(--color-info);
}


.checkin-view__callout-title {
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}

.checkin-view__callout-title--recovery {
  color: var(--color-info);
}


.checkin-view__callout-title--waf {
  color: var(--color-warning);
}


.checkin-view__callout-message,
.checkin-view__callout-meta,
.checkin-view__result-message {
  font-size: 0.75rem;
  line-height: 1.25rem;
}

.checkin-view__callout-message {
  margin-top: 0.25rem;
}

.checkin-view__callout-message--recovery,
.checkin-view__callout-meta--recovery {
  color: var(--color-info);
}


.checkin-view__callout-message--waf {
  color: var(--color-warning);
}


.checkin-view__callout-action {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  border-radius: 0.5rem;
  background: var(--color-warning);
  padding: 0.5rem 0.75rem;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 500;
  color: var(--color-warning-contrast);
  transition: background-color 0.2s ease;
}

.checkin-view__callout-action:hover {
  background: var(--color-warning-hover);
}

.checkin-view__result-section,
.checkin-view__result-list {
  display: flex;
  flex-direction: column;
}

.checkin-view__result-section {
  gap: 0.5rem;
}

.checkin-view__result-section--spaced {
  margin-top: 1rem;
}

.checkin-view__result-section-title {
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 500;
}

.checkin-view__result-section-title--success {
  color: var(--color-success);
}


.checkin-view__result-section-title--danger {
  color: var(--color-danger);
}


.checkin-view__result-section-title--info {
  color: var(--color-info);
}


.checkin-view__result-section-title--muted {
  color: var(--color-text-muted);
}


.checkin-view__result-list {
  gap: 0.375rem;
}

.checkin-view__result-item {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  border-radius: 0.375rem;
  border: 1px solid;
  padding: 0.5rem;
}

.checkin-view__result-item--success {
  border-color: rgb(var(--color-success-rgb) / 30%);
  background: rgb(var(--color-success-rgb) / 8%);
}


.checkin-view__result-item--danger {
  border-color: rgb(var(--color-danger-rgb) / 30%);
  background: rgb(var(--color-danger-rgb) / 8%);
}


.checkin-view__result-item--info {
  border-color: rgb(var(--color-info-rgb) / 30%);
  background: rgb(var(--color-info-rgb) / 8%);
}


.checkin-view__result-item--muted {
  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
}


.checkin-view__result-item-icon {
  margin-top: 0.125rem;
  flex-shrink: 0;
}

.checkin-view__result-item-icon--success {
  color: var(--color-success);
}

.checkin-view__result-item-icon--danger {
  color: var(--color-danger);
}

.checkin-view__result-item-icon--info {
  color: var(--color-info);
}

.checkin-view__result-item-icon--muted {
  color: var(--color-text-muted);
}

.checkin-view__result-item-body {
  min-width: 0;
  flex: 1 1 auto;
}

.checkin-view__result-item-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.checkin-view__result-item-name {
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}

.checkin-view__result-item-name--success {
  color: var(--color-success);
}


.checkin-view__result-item-name--danger {
  color: var(--color-danger);
}


.checkin-view__result-item-name--info {
  color: var(--color-info);
}


.checkin-view__result-item-name--muted {
  color: var(--color-text-secondary);
}


.checkin-view__result-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border-radius: 0.25rem;
  padding: 0.125rem 0.375rem;
  font-size: 0.75rem;
  line-height: 1rem;
}

.checkin-view__result-tag--success {
  background: rgb(var(--color-success-rgb) / 15%);
  color: var(--color-success);
}


.checkin-view__result-tag--reward {
  background: rgb(var(--color-success-rgb) / 15%);
  color: var(--color-success);
}


.checkin-view__result-tag--danger {
  background: rgb(var(--color-danger-rgb) / 15%);
  color: var(--color-danger);
}


.checkin-view__result-tag--warning {
  background: rgb(var(--color-warning-rgb) / 15%);
  color: var(--color-warning);
}


.checkin-view__result-tag--info {
  background: rgb(var(--color-info-rgb) / 15%);
  color: var(--color-info);
}


.checkin-view__result-tag--recovery {
  background: rgb(var(--color-info-rgb) / 15%);
  color: var(--color-info);
}


.checkin-view__result-tag--muted {
  background: var(--color-bg-overlay);
  color: var(--color-text-muted);
}


/* cookie_expired 快捷修复入口：直达账号编辑弹窗 */
.checkin-view__result-fix-button {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border-radius: 0.375rem;
  background: var(--color-danger);
  padding: 0.2rem 0.55rem;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 600;
  color: var(--color-danger-contrast);
  transition: background-color 0.2s ease;
}

.checkin-view__result-fix-button:hover {
  background: var(--color-danger-hover);
}

.checkin-view__result-message {
  margin-top: 0.125rem;
  word-break: break-all;
}

.checkin-view__result-message--success {
  color: var(--color-success);
}


.checkin-view__result-message--danger {
  color: var(--color-danger);
}


.checkin-view__result-message--info {
  color: var(--color-info);
}


.checkin-view__result-message--muted {
  color: var(--color-text-muted);
}


.checkin-view__result-close {
  margin-left: 0.75rem;
  flex-shrink: 0;
  transition: color 0.2s ease;
}

.checkin-view__result-close--warning {
  color: var(--color-warning);
}

.checkin-view__result-close--warning:hover {
  color: var(--color-warning);
}

.checkin-view__result-close--success {
  color: var(--color-success);
}

.checkin-view__result-close--success:hover {
  color: var(--color-success);
}

.checkin-view__content {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.checkin-view__stats {
  display: grid;
  gap: 1rem;
}

@media (width >= 768px) {
  .checkin-view__stats {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

.checkin-view__stat-card {
  display: flex;
  cursor: pointer;
  align-items: center;
  justify-content: space-between;
  border-radius: 0.75rem;
  border: 1px solid var(--color-border-default);
  background: var(--color-bg-surface);
  padding: 1.5rem;
  box-shadow: var(--shadow-xs);
  transition: box-shadow 0.2s ease, transform 0.2s ease;
}

.checkin-view__stat-card:hover {
  transform: scale(1.02);
  box-shadow: var(--shadow-md);
}


.checkin-view__stat-label {
  color: var(--color-text-muted);
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}

.checkin-view__stat-value {
  margin-top: 0.25rem;
  font-family: var(--font-mono);
  font-size: 1.5rem;
  line-height: 2rem;
  font-weight: 700;
}

.checkin-view__stat-value--success {
  color: var(--color-success);
}


.checkin-view__stat-value--info {
  color: var(--color-info);
}


.checkin-view__stat-value--warning {
  color: var(--color-warning);
}


.checkin-view__stat-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 9999px;
  padding: 0.75rem;
}

.checkin-view__stat-icon--success {
  background: rgb(var(--color-success-rgb) / 8%);
  color: var(--color-success);
}


.checkin-view__stat-icon--info {
  background: rgb(var(--color-info-rgb) / 8%);
  color: var(--color-info);
}


.checkin-view__stat-icon--warning {
  background: rgb(var(--color-warning-rgb) / 8%);
  color: var(--color-warning);
}


.checkin-view__tabs-shell {
  border-bottom: 1px solid var(--color-border-default);
}


.checkin-view__tabs {
  margin-bottom: -1px;
  display: flex;
  gap: 2rem;
}

.checkin-view__tab-button {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  border-bottom-width: 2px;
  padding: 1rem 0.25rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
  transition: color 0.2s ease, border-color 0.2s ease;
}

.checkin-view__tab-button--active {
  border-color: var(--color-accent-primary);
  color: var(--color-accent-primary);
}

.checkin-view__tab-button--inactive {
  border-color: transparent;
  color: var(--color-text-muted);
}

.checkin-view__tab-button--inactive:hover {
  border-color: rgb(var(--color-border-default-rgb) / 42%);
  color: var(--color-text-primary);
}

@media (width <= 900px) {
  .checkin-view__header {
    flex-direction: column;
    align-items: flex-start;
  }

  .checkin-view__actions {
    flex-wrap: wrap;
  }

  .checkin-view__tabs {
    gap: 1rem;
    overflow-x: auto;
  }
}
</style>
