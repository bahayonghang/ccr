<template>
  <div class="checkin-view p-6 space-y-6">
    <!-- 页面标题 -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-text-primary flex items-center gap-3">
          <ClipboardList class="w-8 h-8 text-accent-primary" />
          签到管理
        </h1>
        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
          管理中转站签到账号，执行一键签到并追踪余额
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

    <!-- 签到结果弹窗 -->
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
              {{ checkinResult.summary.failed > 0 ? '签到完成（部分失败）' : '签到完成' }}
            </h3>
          </div>
          <!-- 汇总统计 -->
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
            <!-- 成功账号详情 -->
            <div
              v-if="successCheckinResults.length > 0"
              class="space-y-2"
            >
              <p class="text-xs font-medium text-green-700 dark:text-green-300">
                成功账号 ({{ successCheckinResults.length }}):
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
            <!-- 失败账号详情 -->
            <div
              v-if="failedCheckinResults.length > 0"
              class="space-y-2"
            >
              <p class="text-xs font-medium text-red-600 dark:text-red-400">
                失败账号 ({{ failedCheckinResults.length }}):
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
          <!-- 已签到账号详情 -->
          <div
            v-if="alreadyCheckedInResults.length > 0"
            class="mt-4 space-y-2"
          >
            <p class="text-xs font-medium text-blue-700 dark:text-blue-300">
              已签到账号 ({{ alreadyCheckedInResults.length }}):
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

    <!-- 主内容区域 -->
    <div
      v-if="!loading && !error"
      class="space-y-6"
    >
      <!-- 统计卡片（NeuraDock 风格） -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- 当前余额 -->
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700 p-6 flex items-center justify-between transition-all duration-200 hover:shadow-md hover:scale-[1.02] cursor-pointer">
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
        <!-- 总额度 -->
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700 p-6 flex items-center justify-between transition-all duration-200 hover:shadow-md hover:scale-[1.02] cursor-pointer">
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
        <!-- 历史消耗 -->
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700 p-6 flex items-center justify-between transition-all duration-200 hover:shadow-md hover:scale-[1.02] cursor-pointer">
          <div>
            <p class="text-sm font-medium text-gray-500 dark:text-gray-400">
              历史消耗
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

      <!-- Tab 切换 -->
      <div class="border-b border-gray-200 dark:border-gray-700">
        <nav class="-mb-px flex space-x-8">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            class="py-4 px-1 border-b-2 font-medium text-sm transition-colors flex items-center gap-2"
            :class="activeTab === tab.id
              ? 'border-accent-primary text-accent-primary'
              : 'border-transparent text-text-secondary hover:text-text-primary hover:border-border-default'"
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

      <!-- 提供商管理 Tab -->
      <div
        v-if="activeTab === 'providers'"
        class="space-y-6"
      >
        <!-- 内置中转站区域 -->
        <div v-if="availableBuiltinProviders.length > 0">
          <div class="flex items-center space-x-2 mb-4">
            <Store class="w-5 h-5 text-accent-primary" />
            <h2 class="text-lg font-semibold text-text-primary">
              内置中转站
            </h2>
            <span class="text-sm text-gray-500 dark:text-gray-400">
              ({{ availableBuiltinProviders.length }})
            </span>
          </div>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div
              v-for="bp in availableBuiltinProviders"
              :key="bp.id"
              class="bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-gray-800 dark:to-gray-700 rounded-xl shadow-sm p-4 border border-blue-100 dark:border-gray-600 hover:shadow-md transition-all"
            >
              <div class="flex items-start justify-between">
                <div class="flex items-center space-x-3">
                  <span class="text-2xl">{{ bp.icon }}</span>
                  <div>
                    <div class="flex items-center space-x-2">
                      <h3 class="font-semibold text-gray-900 dark:text-white">
                        {{ bp.name }}
                      </h3>
                      <span class="px-1.5 py-0.5 text-xs bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300 rounded">
                        内置
                      </span>
                    </div>
                    <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
                      {{ bp.domain }}
                    </p>
                  </div>
                </div>
                <button
                  class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg transition-colors flex items-center space-x-1"
                  @click="addBuiltinProvider(bp.id)"
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
              <p class="mt-3 text-sm text-gray-600 dark:text-gray-300">
                {{ bp.description }}
              </p>
              <div class="mt-3 flex flex-wrap gap-2">
                <span
                  v-if="bp.supports_checkin"
                  class="px-2 py-0.5 text-xs rounded-full"
                  :class="bp.checkin_bugged 
                    ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300' 
                    : 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'"
                >
                  <component
                    :is="bp.checkin_bugged ? AlertTriangle : CheckCircle"
                    class="w-3 h-3 mr-1 inline"
                  />
                  {{ bp.checkin_bugged ? '自动签到' : '支持签到' }}
                </span>
                <span
                  v-else
                  class="px-2 py-0.5 text-xs bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 rounded-full flex items-center"
                >
                  <XCircle class="w-3 h-3 mr-1" /> 无签到
                </span>
                <span
                  v-if="bp.requires_waf_bypass"
                  class="px-2 py-0.5 text-xs bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300 rounded-full flex items-center"
                >
                  <Shield class="w-3 h-3 mr-1" /> 需要 WAF 绕过
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- 已添加的提供商 -->
        <div>
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center space-x-2">
              <Building2 class="w-5 h-5 text-accent-secondary" />
              <h2 class="text-lg font-semibold text-text-primary">
                已添加的提供商
              </h2>
              <span class="text-sm text-gray-500 dark:text-gray-400">
                ({{ providers.length }})
              </span>
            </div>
            <button
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg flex items-center space-x-2 transition-colors"
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
            class="text-center py-12 text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800/50 rounded-lg"
          >
            <p class="text-4xl mb-3">
              <Package class="w-12 h-12 mx-auto text-text-muted" />
            </p>
            <p>暂无提供商配置</p>
            <p class="text-sm mt-1">
              点击上方内置中转站快速添加，或自定义添加
            </p>
          </div>
          <div
            v-else
            class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
          >
            <div
              v-for="provider in providers"
              :key="provider.id"
              class="bg-white dark:bg-gray-800 rounded-lg shadow p-4 border-l-4"
              :class="provider.enabled ? 'border-l-green-500' : 'border-l-gray-400'"
            >
              <div class="flex items-start justify-between">
                <div>
                  <h3 class="font-semibold text-gray-900 dark:text-white">
                    {{ provider.name }}
                  </h3>
                  <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 truncate">
                    {{ provider.base_url }}
                  </p>
                </div>
                <div class="flex items-center space-x-2">
                  <button
                    class="text-blue-600 hover:text-blue-700 dark:text-blue-400"
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
                    class="text-red-600 hover:text-red-700 dark:text-red-400"
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
              <div class="mt-3 flex items-center space-x-4 text-xs text-gray-500 dark:text-gray-400">
                <span>签到路径: {{ provider.checkin_path }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 账号管理 Tab -->
      <div
        v-if="activeTab === 'accounts'"
        class="space-y-4"
      >
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
                class="hover:bg-gray-50/60 dark:hover:bg-gray-700/50 transition-all duration-200 cursor-pointer"
                @click="openAccountDashboard(account.id)"
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
                      class="inline-flex items-center px-3 py-1.5 rounded-lg text-xs font-medium bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white shadow-sm transition-all duration-200"
                      @click="executeCheckinSingle(account.id)"
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
                          @click="refreshAccountBalance(account.id); openMenuAccountId = null"
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

      <!-- 签到记录 Tab -->
      <div
        v-if="activeTab === 'records'"
        class="space-y-4"
      >
        <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
          签到记录
        </h2>
        <div
          v-if="records.length === 0"
          class="text-center py-12 text-gray-500 dark:text-gray-400"
        >
          暂无签到记录
        </div>
        <div
          v-else
          class="space-y-4"
        >
          <details class="bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-800/60 rounded-lg overflow-hidden">
            <summary class="cursor-pointer select-none px-4 py-3 text-sm font-medium text-red-700 dark:text-red-200 flex items-center justify-between">
              <div class="flex items-center gap-2">
                <XCircle class="w-4 h-4" />
                失败历史记录 ({{ failedHistoryTotal }})
              </div>
              <span class="text-xs text-red-600/80 dark:text-red-300/80">
                点击展开详情
              </span>
            </summary>
            <div class="px-4 pb-4 pt-2">
              <div class="flex flex-wrap items-center gap-2 pb-3">
                <select
                  v-model="failedHistoryProviderFilter"
                  class="px-2 py-1 rounded border border-red-200 dark:border-red-800 bg-white/80 dark:bg-red-950/30 text-xs text-red-700 dark:text-red-200"
                >
                  <option value="all">
                    全部提供商
                  </option>
                  <option
                    v-for="provider in providers"
                    :key="provider.id"
                    :value="provider.id"
                  >
                    {{ provider.name }}
                  </option>
                </select>
                <input
                  v-model="failedHistoryKeyword"
                  type="text"
                  placeholder="账号 / ID / 消息"
                  class="px-2 py-1 rounded border border-red-200 dark:border-red-800 bg-white/80 dark:bg-red-950/30 text-xs text-red-700 dark:text-red-200"
                >
                <button
                  class="px-2 py-1 rounded border border-red-200 dark:border-red-800 text-xs text-red-700 dark:text-red-200 hover:bg-red-100 dark:hover:bg-red-900/30"
                  :disabled="failedHistoryLoading"
                  @click="applyFailedHistoryFilters"
                >
                  筛选
                </button>
                <button
                  class="px-2 py-1 rounded border border-red-200 dark:border-red-800 text-xs text-red-700 dark:text-red-200 hover:bg-red-100 dark:hover:bg-red-900/30"
                  :disabled="failedHistoryLoading"
                  @click="resetFailedHistoryFilters"
                >
                  重置
                </button>
                <button
                  class="px-2 py-1 rounded border border-red-200 dark:border-red-800 text-xs text-red-700 dark:text-red-200 hover:bg-red-100 dark:hover:bg-red-900/30"
                  :disabled="failedHistoryLoading"
                  @click="exportFailedHistory"
                >
                  导出
                </button>
              </div>
              <div
                v-if="failedHistoryLoading"
                class="text-sm text-red-500/80 dark:text-red-300/80"
              >
                加载中...
              </div>
              <div
                v-else-if="failedHistoryTotal === 0"
                class="text-sm text-red-500/80 dark:text-red-300/80"
              >
                暂无失败记录
              </div>
              <div
                v-else
                class="space-y-2"
              >
                <div
                  v-for="record in failedHistoryRecords"
                  :key="record.id"
                  class="p-3 rounded-md border border-red-200 dark:border-red-800 bg-white/70 dark:bg-red-950/30"
                >
                  <div class="flex items-start justify-between gap-4 flex-wrap">
                    <div class="text-sm font-medium text-red-800 dark:text-red-200">
                      {{ getAccountName(record.account_id) }}
                    </div>
                    <div class="text-xs text-red-600 dark:text-red-300">
                      {{ formatDate(record.checked_in_at) }}
                    </div>
                  </div>
                  <div class="mt-1 text-xs text-red-600 dark:text-red-300">
                    提供商: {{ getRecordProviderName(record) }} · 账号ID: {{ record.account_id }}
                  </div>
                  <div class="mt-2 text-xs text-red-600 dark:text-red-300 break-all">
                    原因: {{ getRecordReason(record) }}
                  </div>
                </div>
                <div class="flex items-center justify-between pt-2 text-xs text-red-600 dark:text-red-300">
                  <span>
                    第 {{ failedHistoryPage }} / {{ failedHistoryTotalPages }} 页
                  </span>
                  <div class="flex items-center gap-2">
                    <button
                      class="px-2 py-1 rounded border border-red-200 dark:border-red-800 hover:bg-red-100 dark:hover:bg-red-900/30 disabled:opacity-50"
                      :disabled="failedHistoryPage === 1"
                      @click="goToFailedHistoryPage(failedHistoryPage - 1)"
                    >
                      上一页
                    </button>
                    <button
                      class="px-2 py-1 rounded border border-red-200 dark:border-red-800 hover:bg-red-100 dark:hover:bg-red-900/30 disabled:opacity-50"
                      :disabled="failedHistoryPage === failedHistoryTotalPages"
                      @click="goToFailedHistoryPage(failedHistoryPage + 1)"
                    >
                      下一页
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </details>

          <div class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead class="bg-gray-50 dark:bg-gray-700/50">
                <tr>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    时间
                  </th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    账号
                  </th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    状态
                  </th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    奖励
                  </th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    余额
                  </th>
                  <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    原因
                  </th>
                  <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                    详情
                  </th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                <template
                  v-for="record in records"
                  :key="record.id"
                >
                  <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {{ formatDate(record.checked_in_at) }}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                      {{ getAccountName(record.account_id) }}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap">
                      <span
                        class="px-2 py-1 text-xs font-medium rounded-full"
                        :class="getStatusClass(record.status)"
                      >
                        {{ getStatusText(record.status) }}
                      </span>
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-green-600 dark:text-green-400">
                      {{ record.reward || '-' }}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {{ record.balance_after !== undefined && record.balance_after !== null ? `$${record.balance_after.toFixed(2)}` : '-' }}
                    </td>
                    <td class="px-6 py-4 text-sm text-gray-500 dark:text-gray-400 max-w-xs truncate">
                      {{ getRecordReason(record) }}
                    </td>
                    <td class="px-6 py-4 text-right">
                      <button
                        class="inline-flex items-center gap-1 text-xs text-blue-600 hover:text-blue-700 dark:text-blue-300 dark:hover:text-blue-200"
                        :aria-expanded="isRecordExpanded(record.id)"
                        @click="toggleRecordExpanded(record.id)"
                      >
                        <ChevronUp
                          v-if="isRecordExpanded(record.id)"
                          class="w-4 h-4"
                        />
                        <ChevronDown
                          v-else
                          class="w-4 h-4"
                        />
                        详情
                      </button>
                    </td>
                  </tr>
                  <tr
                    v-if="isRecordExpanded(record.id)"
                    class="bg-gray-50/70 dark:bg-gray-800/60"
                  >
                    <td
                      colspan="7"
                      class="px-6 py-4 text-sm text-gray-600 dark:text-gray-300"
                    >
                      <div class="grid gap-3 md:grid-cols-3">
                        <div class="space-y-1">
                          <div class="text-xs text-gray-500 dark:text-gray-400">
                            提供商
                          </div>
                          <div class="text-sm">
                            {{ getRecordProviderName(record) }}
                          </div>
                        </div>
                        <div class="space-y-1">
                          <div class="text-xs text-gray-500 dark:text-gray-400">
                            账号ID
                          </div>
                          <div class="text-sm break-all">
                            {{ record.account_id }}
                          </div>
                        </div>
                        <div class="space-y-1">
                          <div class="text-xs text-gray-500 dark:text-gray-400">
                            原因
                          </div>
                          <div class="text-sm break-all">
                            {{ getRecordReason(record) }}
                          </div>
                        </div>
                        <div class="space-y-1">
                          <div class="text-xs text-gray-500 dark:text-gray-400">
                            原始消息
                          </div>
                          <div class="text-sm break-all">
                            {{ getRecordRawMessage(record) }}
                          </div>
                        </div>
                        <div class="space-y-1">
                          <div class="text-xs text-gray-500 dark:text-gray-400">
                            奖励 / 余额变化
                          </div>
                          <div class="text-sm">
                            {{ record.reward || '-' }} ·
                            {{ record.balance_change !== undefined && record.balance_change !== null ? `$${record.balance_change.toFixed(2)}` : '-' }}
                          </div>
                        </div>
                        <div class="space-y-1">
                          <div class="text-xs text-gray-500 dark:text-gray-400">
                            余额前 / 后
                          </div>
                          <div class="text-sm">
                            {{ record.balance_before !== undefined && record.balance_before !== null ? `$${record.balance_before.toFixed(2)}` : '-' }}
                            →
                            {{ record.balance_after !== undefined && record.balance_after !== null ? `$${record.balance_after.toFixed(2)}` : '-' }}
                          </div>
                        </div>
                      </div>
                    </td>
                  </tr>
                </template>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- 导入导出 Tab -->
      <div
        v-if="activeTab === 'import-export'"
        class="space-y-6"
      >
        <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
          导入 / 导出
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- 导出 -->
          <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              导出配置
            </h3>
            <div class="space-y-4">
              <label class="flex items-center">
                <input
                  v-model="exportOptions.include_plaintext_keys"
                  type="checkbox"
                  class="w-4 h-4 text-blue-600 border-gray-300 rounded"
                >
                <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">
                  包含明文 API Key (危险)
                </span>
              </label>
              <label class="flex items-center">
                <input
                  v-model="exportOptions.providers_only"
                  type="checkbox"
                  class="w-4 h-4 text-blue-600 border-gray-300 rounded"
                >
                <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">
                  仅导出提供商
                </span>
              </label>
              <button
                class="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
                @click="handleExport"
              >
                导出 JSON
              </button>
            </div>
          </div>

          <!-- 导入 -->
          <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              导入配置
            </h3>
            <div class="space-y-4">
              <div class="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-4">
                <input
                  ref="importFileInput"
                  type="file"
                  accept=".json"
                  class="hidden"
                  @change="handleFileSelect"
                >
                <button
                  class="w-full text-center text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
                  @click="($refs.importFileInput as HTMLInputElement).click()"
                >
                  点击选择 JSON 文件
                </button>
              </div>
              <div
                v-if="importPreview"
                class="text-sm text-gray-600 dark:text-gray-400"
              >
                <p>新提供商: {{ importPreview.new_providers }}</p>
                <p>新账号: {{ importPreview.new_accounts }}</p>
                <p>冲突项: {{ importPreview.conflicting_providers + importPreview.conflicting_accounts }}</p>
              </div>
              <select
                v-model="importConflictStrategy"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              >
                <option value="skip">
                  跳过冲突项
                </option>
                <option value="overwrite">
                  覆盖冲突项
                </option>
              </select>
              <button
                :disabled="!importData"
                class="w-full px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors disabled:opacity-50"
                @click="handleImport"
              >
                执行导入
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 提供商编辑弹窗 -->
    <div
      v-if="showProviderModal"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="showProviderModal = false"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 w-full max-w-lg mx-4">
        <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          {{ editingProvider ? '编辑提供商' : '添加提供商' }}
        </h3>
        <form
          class="space-y-4"
          @submit.prevent="saveProvider"
        >
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              名称 *
            </label>
            <input
              v-model="providerForm.name"
              type="text"
              required
              class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              placeholder="例如: OpenRouter"
            >
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Base URL *
            </label>
            <input
              v-model="providerForm.base_url"
              type="url"
              required
              class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              placeholder="https://api.example.com"
            >
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                签到路径
              </label>
              <input
                v-model="providerForm.checkin_path"
                type="text"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="/api/user/checkin"
              >
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                余额路径
              </label>
              <input
                v-model="providerForm.balance_path"
                type="text"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="/api/user/dashboard"
              >
            </div>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                认证 Header
              </label>
              <input
                v-model="providerForm.auth_header"
                type="text"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="Authorization"
              >
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                认证前缀
              </label>
              <input
                v-model="providerForm.auth_prefix"
                type="text"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="Bearer "
              >
            </div>
          </div>
          <div class="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              class="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
              @click="showProviderModal = false"
            >
              取消
            </button>
            <button
              type="submit"
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg"
            >
              保存
            </button>
          </div>
        </form>
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
  </div>

  <!-- 签到确认弹窗 -->
  <ConfirmModal
    :is-open="showCheckinConfirm"
    title="确认一键签到"
    :message="`即将对 ${enabledAccounts.length} 个启用账号执行签到操作，是否继续？`"
    confirm-text="开始签到"
    cancel-text="取消"
    type="info"
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
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  ClipboardList,
  Store,
  Building2,
  Package,
  CheckCircle,
  AlertTriangle,
  XCircle,
  Shield,
  Calendar,
  Users,
  FileText,
  ChevronDown,
  ChevronUp,
} from 'lucide-vue-next'
import ConfirmModal from '@/components/ConfirmModal.vue'
import CheckinProgressModal from '@/components/CheckinProgressModal.vue'
import {
  listCheckinProviders,
  createCheckinProvider,
  updateCheckinProvider,
  deleteCheckinProvider as apiDeleteProvider,
  listCheckinAccounts,
  createCheckinAccount,
  updateCheckinAccount,
  deleteCheckinAccount as apiDeleteAccount,
  listCheckinRecords,
  exportCheckinRecords,
  getTodayCheckinStats,
  exportCheckinConfig,
  previewCheckinImport,
  importCheckinConfig,
  listBuiltinProviders,
  addBuiltinProvider as apiAddBuiltinProvider,
  queryCheckinBalance,
  checkinAccount,
} from '@/api/client'
import type {
  CheckinProvider,
  AccountInfo,
  CheckinRecordInfo,
  TodayCheckinStats,
  CheckinResponse,
  CheckinExecutionResult,
  CheckinRecordsQuery,
  ExportData,
  ImportPreviewResponse,
  BuiltinProvider,
  CheckinLogEntry,
} from '@/types/checkin'

// 状态
const loading = ref(false)
const checkinLoading = ref(false)
const balanceRefreshing = ref(false)
const error = ref<string | null>(null)
const checkinResultRef = ref<HTMLElement | null>(null)
const activeTab = ref<'providers' | 'accounts' | 'records' | 'import-export'>('accounts')
const router = useRouter()
const openMenuAccountId = ref<string | null>(null)
const searchQuery = ref('')
const providerFilter = ref<string>('all')
const showCheckinConfirm = ref(false)
const showProgressModal = ref(false)
const isCheckinFinished = ref(false)
const checkinProgress = ref({ total: 0, completed: 0, currentAccountName: '' })
const checkinLogs = ref<CheckinLogEntry[]>([])

// 数据
const providers = ref<CheckinProvider[]>([])
const accounts = ref<AccountInfo[]>([])
const records = ref<CheckinRecordInfo[]>([])
const todayStats = ref<TodayCheckinStats | null>(null)
const checkinResult = ref<CheckinResponse | null>(null)
const builtinProviders = ref<BuiltinProvider[]>([])
const expandedRecordIds = ref<string[]>([])
const failedHistoryRecords = ref<CheckinRecordInfo[]>([])
const failedHistoryTotal = ref(0)
const failedHistoryLoading = ref(false)
const failedHistoryPage = ref(1)
const failedHistoryPageSize = ref(5)
const failedHistoryProviderFilter = ref<string>('all')
const failedHistoryKeyword = ref('')

// 计算属性：过滤出尚未添加的内置提供商
const availableBuiltinProviders = computed(() => {
  const addedNames = new Set(providers.value.map(p => p.name))
  return builtinProviders.value.filter(bp => !addedNames.has(bp.name))
})

// 计算属性：汇总统计数据
const totalStatistics = computed(() => {
  const result = {
    currentBalance: 0,
    totalQuota: 0,
    totalConsumed: 0,
  }
  for (const account of accounts.value) {
    if (account.latest_balance !== undefined && account.latest_balance !== null) {
      result.currentBalance += account.latest_balance
    }
    if (account.total_quota !== undefined && account.total_quota !== null) {
      result.totalQuota += account.total_quota
    }
    if (account.total_consumed !== undefined && account.total_consumed !== null) {
      result.totalConsumed += account.total_consumed
    }
  }
  return result
})

// 计算属性：过滤后的账号列表
const filteredAccounts = computed(() => {
  let result = accounts.value

  // 按搜索词过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(account =>
      account.name.toLowerCase().includes(query) ||
      (account.provider_name && account.provider_name.toLowerCase().includes(query))
    )
  }

  // 按提供商过滤
  if (providerFilter.value !== 'all') {
    result = result.filter(account => account.provider_id === providerFilter.value)
  }

  return result
})

// 计算属性：启用的账号列表
const enabledAccounts = computed(() => {
  return accounts.value.filter(a => a.enabled)
})

// 计算属性：失败的签到结果
const failedCheckinResults = computed(() => {
  if (!checkinResult.value) return []
  return checkinResult.value.results.filter(r => r.status === 'Failed')
})

const failedHistoryTotalPages = computed(() => {
  const total = Math.ceil(failedHistoryTotal.value / failedHistoryPageSize.value)
  return total > 0 ? total : 1
})

const successCheckinResults = computed(() => {
  if (!checkinResult.value) return []
  return checkinResult.value.results.filter(r => r.status === 'Success')
})

const alreadyCheckedInResults = computed(() => {
  if (!checkinResult.value) return []
  return checkinResult.value.results.filter(r => r.status === 'AlreadyCheckedIn')
})

const buildCheckinDetail = (item: CheckinExecutionResult, fallback: string) => {
  const details: string[] = []
  if (item.reward) {
    details.push(`奖励: ${item.reward}`)
  }
  if (item.balance !== undefined && item.balance !== null) {
    details.push(`余额: ${item.balance}`)
  }
  if (item.message) {
    details.push(item.message)
  }
  return details.length > 0 ? details.join(' · ') : fallback
}

const getSuccessDetail = (item: CheckinExecutionResult) => buildCheckinDetail(item, '签到成功')

const getAlreadyCheckedInDetail = (item: CheckinExecutionResult) =>
  buildCheckinDetail(item, '今日已签到')

const getFailedDetail = (item: CheckinExecutionResult) => item.message || '未知原因'

// Tab 配置
const tabs = [
  { id: 'accounts' as const, name: '账号管理', icon: Users },
  { id: 'providers' as const, name: '提供商', icon: Building2 },
  { id: 'records' as const, name: '签到记录', icon: FileText },
  { id: 'import-export' as const, name: '导入导出', icon: Package },
]

// 弹窗状态
const showProviderModal = ref(false)
const showAccountModal = ref(false)
const editingProvider = ref<CheckinProvider | null>(null)
const editingAccount = ref<AccountInfo | null>(null)

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

const accountForm = ref({
  provider_id: '',
  name: '',
  session: '', // 简化：只需要输入 session 值，后台自动转换成 {"session": "xxx"}
  api_user: '',
  enabled: true,
})

// 导入导出
const exportOptions = ref({
  include_plaintext_keys: false,
  providers_only: false,
})
const importData = ref<ExportData | null>(null)
const importPreview = ref<ImportPreviewResponse | null>(null)
const importConflictStrategy = ref<'skip' | 'overwrite'>('skip')

// 加载所有数据
const loadAllData = async () => {
  loading.value = true
  error.value = null

  try {
    const [providersRes, accountsRes, recordsRes, statsRes, builtinRes] = await Promise.all([
      listCheckinProviders(),
      listCheckinAccounts(),
      listCheckinRecords({ page: 1, page_size: 100 }),
      getTodayCheckinStats(),
      listBuiltinProviders(),
    ])

    providers.value = providersRes.providers
    accounts.value = accountsRes.accounts
    records.value = recordsRes.records
    todayStats.value = statsRes
    builtinProviders.value = builtinRes.providers
    await loadFailedHistory()
  } catch (e: any) {
    error.value = e.message || '加载失败'
    console.error('Failed to load checkin data:', e)
  } finally {
    loading.value = false
  }
}

// 添加内置提供商
const addBuiltinProvider = async (builtinId: string) => {
  try {
    await apiAddBuiltinProvider(builtinId)
    await loadAllData()
  } catch (e: any) {
    alert('添加失败: ' + (e.message || '未知错误'))
    console.error('Failed to add builtin provider:', e)
  }
}

// 确认签到弹窗回调
const handleCheckinConfirm = () => {
  showCheckinConfirm.value = false
  executeCheckinAll()
}

// 关闭签到弹窗
const closeCheckinModal = () => {
  showProgressModal.value = false
  setTimeout(() => {
    isCheckinFinished.value = false
  }, 300)
}

// 执行全部签到（逐个签到模式，实现实时进度）
const executeCheckinAll = async () => {
  const accountsToCheckin = enabledAccounts.value
  if (accountsToCheckin.length === 0) return

  checkinLoading.value = true
  checkinResult.value = null
  showProgressModal.value = true
  isCheckinFinished.value = false

  // 初始化进度
  checkinProgress.value = {
    total: accountsToCheckin.length,
    completed: 0,
    currentAccountName: ''
  }

  // 初始化日志
  checkinLogs.value = accountsToCheckin.map(acc => ({
    accountId: acc.id,
    accountName: acc.name,
    providerName: acc.provider_name || '未知',
    status: 'pending' as const,
    timestamp: new Date()
  }))

  // 收集结果用于最终汇总
  const results: CheckinExecutionResult[] = []
  let successCount = 0
  let alreadyCheckedInCount = 0
  let failedCount = 0

  try {
    for (let i = 0; i < accountsToCheckin.length; i++) {
      const account = accountsToCheckin[i]

      // 更新当前进度
      checkinProgress.value.currentAccountName = account.name

      // 更新日志状态为处理中
      const logIndex = checkinLogs.value.findIndex(l => l.accountId === account.id)
      if (logIndex >= 0) {
        checkinLogs.value[logIndex].status = 'processing'
        checkinLogs.value[logIndex].timestamp = new Date()
      }

      try {
        const result = await checkinAccount(account.id)
        results.push(result)

        // 更新日志
        if (logIndex >= 0) {
          if (result.status === 'Success') {
            checkinLogs.value[logIndex].status = 'success'
            checkinLogs.value[logIndex].message = result.reward ? `奖励: ${result.reward}` : '签到成功'
            successCount++
          } else if (result.status === 'AlreadyCheckedIn') {
            checkinLogs.value[logIndex].status = 'already_checked_in'
            checkinLogs.value[logIndex].message = '今日已签到'
            alreadyCheckedInCount++
          } else {
            checkinLogs.value[logIndex].status = 'failed'
            checkinLogs.value[logIndex].message = result.message || '签到失败'
            failedCount++
          }
          checkinLogs.value[logIndex].balance = result.balance
          checkinLogs.value[logIndex].reward = result.reward
        }
      } catch (e: any) {
        // 单个账号签到失败
        if (logIndex >= 0) {
          checkinLogs.value[logIndex].status = 'failed'
          checkinLogs.value[logIndex].message = e.message || '请求失败'
        }
        failedCount++
        results.push({
          account_id: account.id,
          account_name: account.name,
          provider_name: account.provider_name || '未知',
          status: 'Failed',
          message: e.message || '请求失败'
        })
      }

      // 更新完成进度
      checkinProgress.value.completed = i + 1
    }

    // 构建签到结果
    checkinResult.value = {
      results,
      summary: {
        total: accountsToCheckin.length,
        success: successCount,
        already_checked_in: alreadyCheckedInCount,
        failed: failedCount
      }
    }

    // 标记签到完成
    isCheckinFinished.value = true

    await loadAllData()
    // 签到完成后自动刷新余额
    await refreshAllBalances()

    // 如果有失败的签到，自动滚动到结果区域确保用户能看到详情
    if (failedCount > 0) {
      await nextTick()
      checkinResultRef.value?.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  } catch (e: any) {
    showProgressModal.value = false
    alert('签到失败: ' + (e.message || '未知错误'))
    console.error('Checkin failed:', e)
  } finally {
    checkinLoading.value = false
  }
}

// 批量刷新所有账号余额
const refreshAllBalances = async () => {
  if (accounts.value.length === 0) return
  
  balanceRefreshing.value = true
  const enabledAccounts = accounts.value.filter(a => a.enabled)
  
  try {
    await Promise.allSettled(
      enabledAccounts.map(account => queryCheckinBalance(account.id))
    )
    await loadAllData()
  } catch (e: any) {
    console.error('Batch refresh failed:', e)
  } finally {
    balanceRefreshing.value = false
  }
}

// 刷新单个账号余额
const refreshAccountBalance = async (accountId: string) => {
  openMenuAccountId.value = null
  try {
    await queryCheckinBalance(accountId)
    await loadAllData()
  } catch (e: any) {
    alert('刷新余额失败: ' + (e.message || '未知错误'))
  }
}

// 单账号签到
const executeCheckinSingle = async (accountId: string) => {
  try {
    await checkinAccount(accountId)
    await loadAllData()
  } catch (e: any) {
    alert('签到失败: ' + (e.message || '未知错误'))
    console.error('Single checkin failed:', e)
  }
}

// 切换账号菜单
const toggleAccountMenu = (accountId: string) => {
  if (openMenuAccountId.value === accountId) {
    openMenuAccountId.value = null
  } else {
    openMenuAccountId.value = accountId
  }
}

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
    await loadAllData()
  } catch (e: any) {
    alert('保存失败: ' + (e.message || '未知错误'))
  }
}

const deleteProvider = async (id: string) => {
  if (!confirm('确定要删除此提供商吗？相关账号也会被删除。')) return
  try {
    await apiDeleteProvider(id)
    await loadAllData()
  } catch (e: any) {
    alert('删除失败: ' + (e.message || '未知错误'))
  }
}

// 账号操作
// 从 cookies JSON 中提取 session 值
const extractSessionFromJson = (json: string): string => {
  try {
    const parsed = JSON.parse(json)
    return parsed.session || ''
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

const openAccountModal = async (account?: AccountInfo) => {
  editingAccount.value = account || null
  
  if (account) {
    // 编辑已有账号：从后端获取解密后的 cookies
    try {
      const { getCheckinAccountCookies } = await import('@/api/client')
      const cookiesData = await getCheckinAccountCookies(account.id)
      accountForm.value = {
        provider_id: account.provider_id,
        name: account.name,
        session: extractSessionFromJson(cookiesData.cookies_json), // 提取 session 值
        api_user: cookiesData.api_user || '',
        enabled: account.enabled,
      }
    } catch (e: any) {
      console.error('Failed to get cookies:', e)
      // 如果获取失败，留空
      accountForm.value = {
        provider_id: account.provider_id,
        name: account.name,
        session: '',
        api_user: account.api_user || '',
        enabled: account.enabled,
      }
    }
  } else {
    // 添加新账号：留空
    accountForm.value = {
      provider_id: providers.value[0]?.id || '',
      name: '',
      session: '',
      api_user: '',
      enabled: true,
    }
  }
  showAccountModal.value = true
}

const saveAccount = async () => {
  try {
    // 将 session 值转换为 cookies_json 格式
    const cookiesJson = sessionToCookiesJson(accountForm.value.session)
    
    if (editingAccount.value) {
      const updateData: { name?: string; cookies_json?: string; api_user?: string; enabled?: boolean } = {
        name: accountForm.value.name,
        enabled: accountForm.value.enabled,
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
      })
    }
    showAccountModal.value = false
    await loadAllData()
  } catch (e: any) {
    alert('保存失败: ' + (e.message || '未知错误'))
  }
}

const deleteAccount = async (id: string) => {
  if (!confirm('确定要删除此账号吗？')) return
  try {
    await apiDeleteAccount(id)
    await loadAllData()
  } catch (e: any) {
    alert('删除失败: ' + (e.message || '未知错误'))
  }
}

// 导入导出
const handleExport = async () => {
  try {
    const data = await exportCheckinConfig(exportOptions.value)
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `checkin-config-${new Date().toISOString().slice(0, 10)}.json`
    a.click()
    URL.revokeObjectURL(url)
  } catch (e: any) {
    alert('导出失败: ' + (e.message || '未知错误'))
  }
}

const handleFileSelect = async (event: Event) => {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return

  try {
    const text = await file.text()
    const data = JSON.parse(text) as ExportData
    importData.value = data
    importPreview.value = await previewCheckinImport(data)
  } catch (e: any) {
    alert('解析文件失败: ' + (e.message || '未知错误'))
    importData.value = null
    importPreview.value = null
  }
}

const handleImport = async () => {
  if (!importData.value) return

  try {
    const result = await importCheckinConfig({
      data: importData.value,
      options: { conflict_strategy: importConflictStrategy.value },
    })
    alert(`导入完成: 提供商 ${result.providers_imported} 个, 账号 ${result.accounts_imported} 个`)
    importData.value = null
    importPreview.value = null
    await loadAllData()
  } catch (e: any) {
    alert('导入失败: ' + (e.message || '未知错误'))
  }
}

// 辅助函数
const getProviderName = (providerId: string) => {
  return providers.value.find(p => p.id === providerId)?.name || providerId
}

const getAccountName = (accountId: string) => {
  return accounts.value.find(a => a.id === accountId)?.name || accountId
}

const openAccountDashboard = (accountId: string) => {
  router.push({ name: 'checkin-account-dashboard', params: { accountId } })
}

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleString('zh-CN')
}

const getStatusClass = (status: string) => {
  switch (status) {
    case 'Success':
      return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
    case 'AlreadyCheckedIn':
      return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400'
    case 'Failed':
      return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'
    default:
      return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-400'
  }
}

const getStatusText = (status: string) => {
  switch (status) {
    case 'Success':
      return '成功'
    case 'AlreadyCheckedIn':
      return '已签到'
    case 'Failed':
      return '失败'
    default:
      return status
  }
}

const getRecordProviderName = (record: CheckinRecordInfo) => {
  if (record.provider_name) return record.provider_name
  const account = accounts.value.find(a => a.id === record.account_id)
  return account?.provider_id ? getProviderName(account.provider_id) : '-'
}

const getRecordReason = (record: CheckinRecordInfo) => {
  if (record.message) return record.message
  switch (record.status) {
    case 'Success':
      return record.reward ? `签到成功 · 奖励 ${record.reward}` : '签到成功'
    case 'AlreadyCheckedIn':
      return '今日已签到'
    case 'Failed':
      return '未知原因'
    default:
      return '-'
  }
}

const getRecordRawMessage = (record: CheckinRecordInfo) => record.message || '-'

const isRecordExpanded = (recordId: string) => {
  return expandedRecordIds.value.includes(recordId)
}

const toggleRecordExpanded = (recordId: string) => {
  expandedRecordIds.value = expandedRecordIds.value.includes(recordId)
    ? expandedRecordIds.value.filter(id => id !== recordId)
    : [...expandedRecordIds.value, recordId]
}

const loadFailedHistory = async () => {
  failedHistoryLoading.value = true
  try {
    const params: CheckinRecordsQuery = {
      status: 'failed',
      page: failedHistoryPage.value,
      page_size: failedHistoryPageSize.value,
    }
    if (failedHistoryProviderFilter.value !== 'all') {
      params.provider_id = failedHistoryProviderFilter.value
    }
    if (failedHistoryKeyword.value.trim()) {
      params.keyword = failedHistoryKeyword.value.trim()
    }
    const response = await listCheckinRecords(params)
    failedHistoryRecords.value = response.records
    failedHistoryTotal.value = response.total
  } catch (e: any) {
    console.error('Failed to load failed history:', e)
  } finally {
    failedHistoryLoading.value = false
  }
}

const applyFailedHistoryFilters = async () => {
  failedHistoryPage.value = 1
  await loadFailedHistory()
}

const resetFailedHistoryFilters = async () => {
  failedHistoryProviderFilter.value = 'all'
  failedHistoryKeyword.value = ''
  failedHistoryPage.value = 1
  await loadFailedHistory()
}

const goToFailedHistoryPage = async (page: number) => {
  const nextPage = Math.min(Math.max(page, 1), failedHistoryTotalPages.value)
  if (nextPage === failedHistoryPage.value) return
  failedHistoryPage.value = nextPage
  await loadFailedHistory()
}

const exportFailedHistory = async () => {
  try {
    const params: CheckinRecordsQuery = { status: 'failed' }
    if (failedHistoryProviderFilter.value !== 'all') {
      params.provider_id = failedHistoryProviderFilter.value
    }
    if (failedHistoryKeyword.value.trim()) {
      params.keyword = failedHistoryKeyword.value.trim()
    }
    const { blob, filename } = await exportCheckinRecords(params)
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    link.remove()
    URL.revokeObjectURL(url)
  } catch (e: any) {
    alert('导出失败: ' + (e.message || '未知错误'))
  }
}

watch(
  () => failedHistoryTotal.value,
  () => {
    if (failedHistoryPage.value > failedHistoryTotalPages.value) {
      failedHistoryPage.value = failedHistoryTotalPages.value
    }
  }
)

// 点击页面其他地方关闭菜单
const closeMenuOnClickOutside = (e: MouseEvent) => {
  if (openMenuAccountId.value && !(e.target as HTMLElement).closest('.relative')) {
    openMenuAccountId.value = null
  }
}

onMounted(() => {
  loadAllData()
  document.addEventListener('click', closeMenuOnClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', closeMenuOnClickOutside)
})
</script>
