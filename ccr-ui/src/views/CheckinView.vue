<template>
  <div class="checkin-view p-6 space-y-6">
    <!-- 页面标题与操作 -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-white flex items-center gap-3">
          <ClipboardList class="w-8 h-8 text-accent-primary" />
          签到管理
        </h1>
        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
          管理所有平台的自动签到任务和 Cookie
        </p>
      </div>
      <div class="flex items-center space-x-3">
        <button
          :disabled="loading || checkinLoading || enabledAccounts.length === 0"
          class="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg flex items-center space-x-2 disabled:opacity-50 transition-colors"
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
          <span>{{ checkinLoading ? '签到中...' : '一键签到' }}</span>
        </button>
        <button
          :disabled="balanceRefreshing || accounts.length === 0"
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg flex items-center space-x-2 disabled:opacity-50 transition-colors"
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
          <span>{{ balanceRefreshing ? '刷新中...' : '刷新余额' }}</span>
        </button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div
      v-if="loading"
      class="flex items-center justify-center py-12"
    >
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" />
    </div>

    <!-- 错误提示 -->
    <div
      v-if="error"
      class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4"
    >
      <div class="flex">
        <svg
          class="h-5 w-5 text-red-400"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd"
          />
        </svg>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-red-800 dark:text-red-200">
            加载失败
          </h3>
          <p class="mt-2 text-sm text-red-700 dark:text-red-300">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <!-- 签到结果汇总 -->
    <div
      v-if="checkinResult"
      ref="checkinResultRef"
      class="rounded-lg p-4 border shadow-sm"
      :class="checkinResult.summary.failed > 0
        ? 'bg-amber-50 dark:bg-amber-900/20 border-amber-200 dark:border-amber-800'
        : 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'"
    >
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <div class="flex items-center gap-2">
            <svg
              class="h-5 w-5"
              :class="checkinResult.summary.failed > 0 ? 'text-amber-500' : 'text-green-400'"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                v-if="checkinResult.summary.failed > 0"
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
              class="text-sm font-medium"
              :class="checkinResult.summary.failed > 0
                ? 'text-amber-800 dark:text-amber-200'
                : 'text-green-800 dark:text-green-200'"
            >
              {{ checkinResult.summary.failed > 0 ? '签到完成，部分失败' : '签到完成' }}
            </h3>
          </div>
          <!-- 结果统计 -->
          <div class="mt-3 flex flex-wrap items-center gap-2 text-xs">
            <span class="inline-flex items-center gap-1 rounded-full px-2 py-1 bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-200">
              <CheckCircle class="w-3.5 h-3.5" />
              成功 {{ checkinResult.summary.success }}
            </span>
            <span class="inline-flex items-center gap-1 rounded-full px-2 py-1 bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-200">
              <Calendar class="w-3.5 h-3.5" />
              已签到 {{ checkinResult.summary.already_checked_in }}
            </span>
            <span class="inline-flex items-center gap-1 rounded-full px-2 py-1 bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-200">
              <XCircle class="w-3.5 h-3.5" />
              失败 {{ checkinResult.summary.failed }}
            </span>
            <span class="inline-flex items-center gap-1 rounded-full px-2 py-1 bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-200">
              总计 {{ checkinResult.summary.total }}
            </span>
          </div>
          <div class="mt-4 grid gap-4 md:grid-cols-2">
            <!-- 成功结果 -->
            <div
              v-if="successCheckinResults.length > 0"
              class="space-y-2"
            >
              <p class="text-xs font-medium text-green-700 dark:text-green-300">
                成功结果（{{ successCheckinResults.length }}）
              </p>
              <div class="space-y-1.5">
                <div
                  v-for="item in successCheckinResults"
                  :key="item.account_id"
                  class="flex items-start gap-2 p-2 bg-green-50 dark:bg-green-900/20 rounded-md border border-green-200 dark:border-green-800"
                >
                  <CheckCircle class="w-4 h-4 text-green-500 flex-shrink-0 mt-0.5" />
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 flex-wrap">
                      <span class="text-sm font-medium text-green-800 dark:text-green-200">
                        {{ item.account_name }}
                      </span>
                      <span class="text-xs px-1.5 py-0.5 bg-green-100 dark:bg-green-800 text-green-700 dark:text-green-200 rounded">
                        {{ item.provider_name }}
                      </span>
                      <span
                        v-if="item.reward"
                        class="text-xs px-1.5 py-0.5 bg-emerald-100 dark:bg-emerald-900/40 text-emerald-700 dark:text-emerald-200 rounded"
                      >
                        奖励 {{ item.reward }}
                      </span>
                    </div>
                    <p class="text-xs text-green-700 dark:text-green-300 mt-0.5 break-all">
                      {{ getSuccessDetail(item) }}
                    </p>
                  </div>
                </div>
              </div>
            </div>
            <!-- 失败结果 -->
            <div
              v-if="failedCheckinResults.length > 0"
              class="space-y-2"
            >
              <p class="text-xs font-medium text-red-600 dark:text-red-400">
                失败结果（{{ failedCheckinResults.length }}）
              </p>
              <div class="space-y-1.5">
                <div
                  v-for="item in failedCheckinResults"
                  :key="item.account_id"
                  class="flex items-start gap-2 p-2 bg-red-50 dark:bg-red-900/30 rounded-md border border-red-200 dark:border-red-800"
                >
                  <XCircle class="w-4 h-4 text-red-500 flex-shrink-0 mt-0.5" />
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 flex-wrap">
                      <span class="text-sm font-medium text-red-800 dark:text-red-200">
                        {{ item.account_name }}
                      </span>
                      <span class="text-xs px-1.5 py-0.5 bg-red-100 dark:bg-red-800 text-red-600 dark:text-red-300 rounded">
                        {{ item.provider_name }}
                      </span>
                    </div>
                    <p class="text-xs text-red-600 dark:text-red-400 mt-0.5 break-all">
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
            class="mt-4 space-y-2"
          >
            <p class="text-xs font-medium text-blue-700 dark:text-blue-300">
              已签到（{{ alreadyCheckedInResults.length }}）
            </p>
            <div class="space-y-1.5">
              <div
                v-for="item in alreadyCheckedInResults"
                :key="item.account_id"
                class="flex items-start gap-2 p-2 bg-blue-50 dark:bg-blue-900/20 rounded-md border border-blue-200 dark:border-blue-800"
              >
                <Calendar class="w-4 h-4 text-blue-500 flex-shrink-0 mt-0.5" />
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-sm font-medium text-blue-800 dark:text-blue-200">
                      {{ item.account_name }}
                    </span>
                    <span class="text-xs px-1.5 py-0.5 bg-blue-100 dark:bg-blue-800 text-blue-700 dark:text-blue-200 rounded">
                      {{ item.provider_name }}
                    </span>
                  </div>
                  <p class="text-xs text-blue-700 dark:text-blue-300 mt-0.5 break-all">
                    {{ getAlreadyCheckedInDetail(item) }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
        <button
          class="ml-3 flex-shrink-0"
          :class="checkinResult.summary.failed > 0
            ? 'text-amber-400 hover:text-amber-500'
            : 'text-green-400 hover:text-green-500'"
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
      class="space-y-6"
    >
      <!-- 顶部统计卡片 -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- 当前余额 -->
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700 p-6 flex items-center justify-between transition-[box-shadow,transform] duration-200 hover:shadow-md hover:scale-[1.02] cursor-pointer">
          <div>
            <p class="text-sm font-medium text-gray-500 dark:text-gray-400">
              当前余额
            </p>
            <p class="mt-1 text-2xl font-bold text-green-600 dark:text-green-400 font-mono">
              ${{ totalStatistics.currentBalance.toFixed(2) }}
            </p>
          </div>
          <div class="p-3 rounded-full bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400">
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
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700 p-6 flex items-center justify-between transition-[box-shadow,transform] duration-200 hover:shadow-md hover:scale-[1.02] cursor-pointer">
          <div>
            <p class="text-sm font-medium text-gray-500 dark:text-gray-400">
              总额度
            </p>
            <p class="mt-1 text-2xl font-bold text-blue-600 dark:text-blue-400 font-mono">
              ${{ totalStatistics.totalQuota.toFixed(2) }}
            </p>
          </div>
          <div class="p-3 rounded-full bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400">
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
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700 p-6 flex items-center justify-between transition-[box-shadow,transform] duration-200 hover:shadow-md hover:scale-[1.02] cursor-pointer">
          <div>
            <p class="text-sm font-medium text-gray-500 dark:text-gray-400">
              已消耗
            </p>
            <p class="mt-1 text-2xl font-bold text-orange-600 dark:text-orange-400 font-mono">
              ${{ totalStatistics.totalConsumed.toFixed(2) }}
            </p>
          </div>
          <div class="p-3 rounded-full bg-orange-50 dark:bg-orange-900/20 text-orange-600 dark:text-orange-400">
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
      <div class="border-b border-gray-200 dark:border-gray-700">
        <nav class="-mb-px flex space-x-8">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            class="py-4 px-1 border-b-2 font-medium text-sm transition-colors flex items-center gap-2"
            :class="activeTab === tab.id
              ? 'border-accent-primary text-accent-primary'
              : 'border-transparent text-white/80 hover:text-white hover:border-white/10'"
            @click="activeTab = tab.id"
          >
            <component
              :is="tab.icon"
              class="w-4 h-4"
            />
            {{ tab.name }}
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
      title="确认一键签到"
      :message="`即将对 ${enabledAccounts.length} 个启用账号执行签到操作，是否继续？`"
      confirm-text="开始签到"
      cancel-text="取消"
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
      :is-finished="isCheckinFinished"
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

import { useRouter } from 'vue-router'
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
import {
  ClipboardList,
  CheckCircle,
  XCircle,
  Calendar,
} from 'lucide-vue-next'

const router = useRouter()

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
  isCheckinFinished,
  checkinProgress,
  checkinLogs,
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
} = useCheckinState()

const openAccountDashboard = (accountId: string) => {
  router.push({ name: 'checkin-account-dashboard', params: { accountId } })
}
</script>
