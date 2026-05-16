<template>
  <div class="checkin-view">
    <!-- 页面标题与操作 -->
    <div class="checkin-view__header">
      <div class="checkin-view__header-copy">
        <h1 class="checkin-view__title">
          <span class="checkin-view__title-icon-shell">
            <SIcon
              name="ClipboardList"
              size="w-6 h-6"
              class="checkin-view__title-icon"
            />
          </span>
          <span class="checkin-view__title-label">{{ t('checkin.title') }}</span>
        </h1>
        <p class="checkin-view__subtitle">
          {{ t('checkin.description') }}
        </p>
      </div>
      <div class="checkin-view__actions">
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
      </div>
    </div>

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
                    </div>
                    <p class="checkin-view__result-message checkin-view__result-message--danger">
                      {{ getFailedDetail(item) }}
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
      <!-- 顶部统计卡片 -->
      <div class="checkin-view__stats">
        <!-- 当前余额 -->
        <div class="checkin-view__stat-card">
          <div>
            <p class="checkin-view__stat-label">
              {{ t('checkin.stats.currentBalance') }}
            </p>
            <p class="checkin-view__stat-value checkin-view__stat-value--success">
              ${{ totalStatistics.currentBalance.toFixed(2) }}
            </p>
          </div>
          <div class="checkin-view__stat-icon checkin-view__stat-icon--success">
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
                d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z"
              />
            </svg>
          </div>
        </div>
        <!-- 总额度-->
        <div class="checkin-view__stat-card">
          <div>
            <p class="checkin-view__stat-label">
              {{ t('checkin.stats.totalQuota') }}
            </p>
            <p class="checkin-view__stat-value checkin-view__stat-value--info">
              ${{ totalStatistics.totalQuota.toFixed(2) }}
            </p>
          </div>
          <div class="checkin-view__stat-icon checkin-view__stat-icon--info">
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
                d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6"
              />
            </svg>
          </div>
        </div>
        <!-- 已消耗-->
        <div class="checkin-view__stat-card">
          <div>
            <p class="checkin-view__stat-label">
              {{ t('checkin.stats.totalConsumed') }}
            </p>
            <p class="checkin-view__stat-value checkin-view__stat-value--warning">
              ${{ totalStatistics.totalConsumed.toFixed(2) }}
            </p>
          </div>
          <div class="checkin-view__stat-icon checkin-view__stat-icon--warning">
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
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
        </div>
      </div>

      <!-- Tab 导航 -->
      <div class="checkin-view__tabs-shell">
        <nav class="checkin-view__tabs">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            :class="[
              'checkin-view__tab-button',
              activeTab === tab.id
                ? 'checkin-view__tab-button--active'
                : 'checkin-view__tab-button--inactive',
            ]"
            @click="activeTab = tab.id"
          >
            <SIcon
              :name="tab.icon"
              size="w-4 h-4"
            />
            {{ t(tab.nameKey) }}
          </button>
        </nav>
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
        @refresh="loadAllData"
        @checkin="executeCheckinSingle"
        @refresh-balance="refreshAccountBalance"
        @navigate="openAccountDashboard"
        @show-oauth-wizard="showOAuthWizard = true"
      />
      <CheckinRecordsTab
        v-if="activeTab === 'records'"
        :records="records"
        :providers="providers"
        :accounts="accounts"
        :today-stats="todayStats"
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
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'CheckinView' })

import SIcon from '@/components/ui/SIcon.vue'
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
  // Tab 配置
  tabs,
  // 数据加载
  loadAllData,
  // 签到操作
  executeCheckinSingle,
  handleCheckinConfirm,
  closeCheckinModal,
  handleOAuthSuccess,
  // 余额操作
  refreshAllBalances,
  refreshAccountBalance,
  // 内置提供商操作
  addBuiltinProvider,
  // 结果详情格式化
  getSuccessDetail,
  getAlreadyCheckedInDetail,
  getFailedDetail,
  getErrorLabel,
} = useCheckinState()

const openAccountDashboard = (accountId: string) => {
  router.push({ name: 'checkin-account-dashboard', params: { accountId } })
}
</script>

<style scoped>
.checkin-view {
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
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
  color: var(--text-primary);
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
  box-shadow: inset 0 1px 0 rgb(255 248 240 / 42%);
}

.checkin-view__title-icon {
  color: var(--accent-primary);
}

.dark .checkin-view__title-icon-shell {
  border-color: rgb(var(--color-accent-primary-rgb) / 26%);
  background: rgb(var(--color-accent-primary-rgb) / 14%);
  box-shadow: inset 0 1px 0 rgb(255 248 240 / 10%);
}

.checkin-view__title-label {
  min-width: 0;
}

.checkin-view__subtitle {
  max-width: 40ch;
  margin-top: 0.625rem;
  color: var(--text-secondary);
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
  color: rgb(255 255 255 / 100%);
  font-weight: 700;
  letter-spacing: -0.01em;
  white-space: nowrap;
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 14%),
    0 14px 30px rgb(15 23 42 / 14%);
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
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 8%);
}

.checkin-view__action-button:hover:not(:disabled) {
  transform: translateY(-1px);
}

.checkin-view__action-button--checkin {
  border-color: rgb(74 222 128 / 34%);
  background:
    radial-gradient(circle at 15% 0%, rgb(34 197 94 / 34%), transparent 35%),
    linear-gradient(135deg, rgb(20 184 166 / 96%), rgb(22 163 74 / 98%));
  box-shadow:
    inset 0 1px 0 rgb(236 253 245 / 18%),
    0 0 0 1px rgb(34 197 94 / 10%),
    0 16px 34px rgb(22 163 74 / 24%);
}

.checkin-view__action-button--checkin:hover:not(:disabled) {
  border-color: rgb(134 239 172 / 48%);
  box-shadow:
    inset 0 1px 0 rgb(236 253 245 / 22%),
    0 0 0 1px rgb(34 197 94 / 18%),
    0 20px 42px rgb(22 163 74 / 32%);
}

.checkin-view__action-button--balance {
  border-color: rgb(96 165 250 / 32%);
  background:
    radial-gradient(circle at 15% 0%, rgb(125 211 252 / 30%), transparent 35%),
    linear-gradient(135deg, rgb(37 99 235 / 96%), rgb(79 70 229 / 96%));
}

.checkin-view__action-button--balance:hover:not(:disabled) {
  border-color: rgb(147 197 253 / 46%);
  box-shadow:
    inset 0 1px 0 rgb(239 246 255 / 18%),
    0 18px 38px rgb(37 99 235 / 26%);
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
  border-bottom: 2px solid rgb(37 99 235 / 100%);
  animation: spin 1s linear infinite;
}

.checkin-view__error {
  border-radius: 0.5rem;
  border: 1px solid rgb(254 202 202 / 100%);
  background: rgb(254 242 242 / 100%);
  padding: 1rem;
}

.dark .checkin-view__error {
  border-color: rgb(153 27 27 / 100%);
  background: rgb(127 29 29 / 20%);
}

.checkin-view__error-content {
  display: flex;
}

.checkin-view__error-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: rgb(248 113 113 / 100%);
}

.checkin-view__error-body {
  margin-left: 0.75rem;
}

.checkin-view__error-title {
  color: rgb(153 27 27 / 100%);
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}

.dark .checkin-view__error-title {
  color: rgb(254 202 202 / 100%);
}

.checkin-view__error-message {
  margin-top: 0.5rem;
  color: rgb(185 28 28 / 100%);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.dark .checkin-view__error-message {
  color: rgb(252 165 165 / 100%);
}

.checkin-view__result {
  border-radius: 0.5rem;
  border-width: 1px;
  padding: 1rem;
  box-shadow: 0 1px 2px rgb(15 23 42 / 8%);
}

.checkin-view__result--recovering {
  border-color: rgb(186 230 253 / 100%);
  background: rgb(240 249 255 / 100%);
}

.dark .checkin-view__result--recovering {
  border-color: rgb(7 89 133 / 100%);
  background: rgb(12 74 110 / 20%);
}

.checkin-view__result--warning {
  border-color: rgb(253 230 138 / 100%);
  background: rgb(255 251 235 / 100%);
}

.dark .checkin-view__result--warning {
  border-color: rgb(146 64 14 / 100%);
  background: rgb(120 53 15 / 20%);
}

.checkin-view__result--success {
  border-color: rgb(187 247 208 / 100%);
  background: rgb(240 253 244 / 100%);
}

.dark .checkin-view__result--success {
  border-color: rgb(21 128 61 / 100%);
  background: rgb(20 83 45 / 20%);
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
  color: rgb(14 165 233 / 100%);
}

.checkin-view__result-status-icon--warning {
  color: rgb(245 158 11 / 100%);
}

.checkin-view__result-status-icon--success {
  color: rgb(74 222 128 / 100%);
}

.checkin-view__result-status-title {
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}

.checkin-view__result-status-title--recovering {
  color: rgb(7 89 133 / 100%);
}

.dark .checkin-view__result-status-title--recovering {
  color: rgb(186 230 253 / 100%);
}

.checkin-view__result-status-title--warning {
  color: rgb(146 64 14 / 100%);
}

.dark .checkin-view__result-status-title--warning {
  color: rgb(253 230 138 / 100%);
}

.checkin-view__result-status-title--success {
  color: rgb(22 101 52 / 100%);
}

.dark .checkin-view__result-status-title--success {
  color: rgb(187 247 208 / 100%);
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
  background: rgb(220 252 231 / 100%);
  color: rgb(21 128 61 / 100%);
}

.dark .checkin-view__result-badge--success {
  background: rgb(20 83 45 / 40%);
  color: rgb(187 247 208 / 100%);
}

.checkin-view__result-badge--info {
  background: rgb(219 234 254 / 100%);
  color: rgb(29 78 216 / 100%);
}

.dark .checkin-view__result-badge--info {
  background: rgb(30 64 175 / 40%);
  color: rgb(191 219 254 / 100%);
}

.checkin-view__result-badge--danger {
  background: rgb(254 226 226 / 100%);
  color: rgb(185 28 28 / 100%);
}

.dark .checkin-view__result-badge--danger {
  background: rgb(127 29 29 / 40%);
  color: rgb(254 202 202 / 100%);
}

.checkin-view__result-badge--neutral {
  background: rgb(241 245 249 / 100%);
  color: rgb(51 65 85 / 100%);
}

.dark .checkin-view__result-badge--neutral {
  background: rgb(30 41 59 / 100%);
  color: rgb(226 232 240 / 100%);
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
  border-color: rgb(186 230 253 / 100%);
  background: rgb(240 249 255 / 90%);
}

.dark .checkin-view__callout--recovery {
  border-color: rgb(7 89 133 / 100%);
  background: rgb(12 74 110 / 20%);
}

.checkin-view__callout--waf {
  border-color: rgb(254 215 170 / 100%);
  background: rgb(255 247 237 / 90%);
}

.dark .checkin-view__callout--waf {
  border-color: rgb(154 52 18 / 100%);
  background: rgb(124 45 18 / 20%);
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
  color: rgb(2 132 199 / 100%);
}

.dark .checkin-view__callout-icon--recovery {
  color: rgb(125 211 252 / 100%);
}

.checkin-view__callout-title {
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}

.checkin-view__callout-title--recovery {
  color: rgb(12 74 110 / 100%);
}

.dark .checkin-view__callout-title--recovery {
  color: rgb(224 242 254 / 100%);
}

.checkin-view__callout-title--waf {
  color: rgb(124 45 18 / 100%);
}

.dark .checkin-view__callout-title--waf {
  color: rgb(255 237 213 / 100%);
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
  color: rgb(7 89 133 / 100%);
}

.dark .checkin-view__callout-message--recovery,
.dark .checkin-view__callout-meta--recovery {
  color: rgb(186 230 253 / 100%);
}

.checkin-view__callout-message--waf {
  color: rgb(154 52 18 / 100%);
}

.dark .checkin-view__callout-message--waf {
  color: rgb(254 215 170 / 100%);
}

.checkin-view__callout-action {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  border-radius: 0.5rem;
  background: rgb(234 88 12 / 100%);
  padding: 0.5rem 0.75rem;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 500;
  color: rgb(255 255 255 / 100%);
  transition: background-color 0.2s ease;
}

.checkin-view__callout-action:hover {
  background: rgb(194 65 12 / 100%);
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
  color: rgb(21 128 61 / 100%);
}

.dark .checkin-view__result-section-title--success {
  color: rgb(134 239 172 / 100%);
}

.checkin-view__result-section-title--danger {
  color: rgb(220 38 38 / 100%);
}

.dark .checkin-view__result-section-title--danger {
  color: rgb(248 113 113 / 100%);
}

.checkin-view__result-section-title--info {
  color: rgb(29 78 216 / 100%);
}

.dark .checkin-view__result-section-title--info {
  color: rgb(147 197 253 / 100%);
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
  border-color: rgb(187 247 208 / 100%);
  background: rgb(240 253 244 / 100%);
}

.dark .checkin-view__result-item--success {
  border-color: rgb(21 128 61 / 100%);
  background: rgb(20 83 45 / 20%);
}

.checkin-view__result-item--danger {
  border-color: rgb(254 202 202 / 100%);
  background: rgb(254 242 242 / 100%);
}

.dark .checkin-view__result-item--danger {
  border-color: rgb(153 27 27 / 100%);
  background: rgb(127 29 29 / 30%);
}

.checkin-view__result-item--info {
  border-color: rgb(191 219 254 / 100%);
  background: rgb(239 246 255 / 100%);
}

.dark .checkin-view__result-item--info {
  border-color: rgb(30 64 175 / 100%);
  background: rgb(30 58 138 / 20%);
}

.checkin-view__result-item-icon {
  margin-top: 0.125rem;
  flex-shrink: 0;
}

.checkin-view__result-item-icon--success {
  color: rgb(34 197 94 / 100%);
}

.checkin-view__result-item-icon--danger {
  color: rgb(239 68 68 / 100%);
}

.checkin-view__result-item-icon--info {
  color: rgb(59 130 246 / 100%);
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
  color: rgb(22 101 52 / 100%);
}

.dark .checkin-view__result-item-name--success {
  color: rgb(187 247 208 / 100%);
}

.checkin-view__result-item-name--danger {
  color: rgb(153 27 27 / 100%);
}

.dark .checkin-view__result-item-name--danger {
  color: rgb(254 202 202 / 100%);
}

.checkin-view__result-item-name--info {
  color: rgb(30 64 175 / 100%);
}

.dark .checkin-view__result-item-name--info {
  color: rgb(191 219 254 / 100%);
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
  background: rgb(220 252 231 / 100%);
  color: rgb(21 128 61 / 100%);
}

.dark .checkin-view__result-tag--success {
  background: rgb(22 101 52 / 100%);
  color: rgb(220 252 231 / 100%);
}

.checkin-view__result-tag--reward {
  background: rgb(209 250 229 / 100%);
  color: rgb(4 120 87 / 100%);
}

.dark .checkin-view__result-tag--reward {
  background: rgb(6 78 59 / 100%);
  color: rgb(209 250 229 / 100%);
}

.checkin-view__result-tag--danger {
  background: rgb(254 226 226 / 100%);
  color: rgb(220 38 38 / 100%);
}

.dark .checkin-view__result-tag--danger {
  background: rgb(153 27 27 / 100%);
  color: rgb(254 202 202 / 100%);
}

.checkin-view__result-tag--warning {
  background: rgb(255 237 213 / 100%);
  color: rgb(194 65 12 / 100%);
}

.dark .checkin-view__result-tag--warning {
  background: rgb(124 45 18 / 100%);
  color: rgb(255 237 213 / 100%);
}

.checkin-view__result-tag--info {
  background: rgb(219 234 254 / 100%);
  color: rgb(29 78 216 / 100%);
}

.dark .checkin-view__result-tag--info {
  background: rgb(30 64 175 / 100%);
  color: rgb(219 234 254 / 100%);
}

.checkin-view__result-tag--recovery {
  background: rgb(224 242 254 / 100%);
  color: rgb(3 105 161 / 100%);
}

.dark .checkin-view__result-tag--recovery {
  background: rgb(12 74 110 / 100%);
  color: rgb(224 242 254 / 100%);
}

.checkin-view__result-message {
  margin-top: 0.125rem;
  word-break: break-all;
}

.checkin-view__result-message--success {
  color: rgb(21 128 61 / 100%);
}

.dark .checkin-view__result-message--success {
  color: rgb(134 239 172 / 100%);
}

.checkin-view__result-message--danger {
  color: rgb(220 38 38 / 100%);
}

.dark .checkin-view__result-message--danger {
  color: rgb(248 113 113 / 100%);
}

.checkin-view__result-message--info {
  color: rgb(29 78 216 / 100%);
}

.dark .checkin-view__result-message--info {
  color: rgb(147 197 253 / 100%);
}

.checkin-view__result-close {
  margin-left: 0.75rem;
  flex-shrink: 0;
  transition: color 0.2s ease;
}

.checkin-view__result-close--warning {
  color: rgb(251 191 36 / 100%);
}

.checkin-view__result-close--warning:hover {
  color: rgb(245 158 11 / 100%);
}

.checkin-view__result-close--success {
  color: rgb(74 222 128 / 100%);
}

.checkin-view__result-close--success:hover {
  color: rgb(34 197 94 / 100%);
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
  border: 1px solid rgb(243 244 246 / 100%);
  background: rgb(255 255 255 / 100%);
  padding: 1.5rem;
  box-shadow: 0 1px 2px rgb(15 23 42 / 8%);
  transition: box-shadow 0.2s ease, transform 0.2s ease;
}

.checkin-view__stat-card:hover {
  transform: scale(1.02);
  box-shadow: 0 10px 24px rgb(15 23 42 / 12%);
}

.dark .checkin-view__stat-card {
  border-color: rgb(55 65 81 / 100%);
  background: rgb(31 41 55 / 100%);
}

.checkin-view__stat-label {
  color: var(--text-muted);
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
  color: rgb(22 163 74 / 100%);
}

.dark .checkin-view__stat-value--success {
  color: rgb(74 222 128 / 100%);
}

.checkin-view__stat-value--info {
  color: rgb(37 99 235 / 100%);
}

.dark .checkin-view__stat-value--info {
  color: rgb(96 165 250 / 100%);
}

.checkin-view__stat-value--warning {
  color: rgb(234 88 12 / 100%);
}

.dark .checkin-view__stat-value--warning {
  color: rgb(251 146 60 / 100%);
}

.checkin-view__stat-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 9999px;
  padding: 0.75rem;
}

.checkin-view__stat-icon--success {
  background: rgb(240 253 244 / 100%);
  color: rgb(22 163 74 / 100%);
}

.dark .checkin-view__stat-icon--success {
  background: rgb(20 83 45 / 20%);
  color: rgb(74 222 128 / 100%);
}

.checkin-view__stat-icon--info {
  background: rgb(239 246 255 / 100%);
  color: rgb(37 99 235 / 100%);
}

.dark .checkin-view__stat-icon--info {
  background: rgb(30 64 175 / 20%);
  color: rgb(96 165 250 / 100%);
}

.checkin-view__stat-icon--warning {
  background: rgb(255 247 237 / 100%);
  color: rgb(234 88 12 / 100%);
}

.dark .checkin-view__stat-icon--warning {
  background: rgb(154 52 18 / 20%);
  color: rgb(251 146 60 / 100%);
}

.checkin-view__tabs-shell {
  border-bottom: 1px solid rgb(229 231 235 / 100%);
}

.dark .checkin-view__tabs-shell {
  border-color: rgb(55 65 81 / 100%);
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
  border-color: var(--accent-primary);
  color: var(--accent-primary);
}

.checkin-view__tab-button--inactive {
  border-color: transparent;
  color: var(--text-muted);
}

.checkin-view__tab-button--inactive:hover {
  border-color: rgb(var(--color-border-default-rgb) / 42%);
  color: var(--text-primary);
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
