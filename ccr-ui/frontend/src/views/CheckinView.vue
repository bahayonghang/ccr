<template>
  <div class="checkin-view p-6 space-y-6">
    <!-- 页面标题 -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
          📋 签到管理
        </h1>
        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
          管理中转站签到账号，执行一键签到并追踪余额
        </p>
      </div>
      <div class="flex items-center space-x-3">
        <button
          :disabled="loading || checkinLoading"
          class="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg flex items-center space-x-2 disabled:opacity-50 transition-colors"
          @click="executeCheckinAll"
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
      class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4"
    >
      <div class="flex items-start justify-between">
        <div class="flex">
          <svg
            class="h-5 w-5 text-green-400"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fill-rule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clip-rule="evenodd"
            />
          </svg>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-green-800 dark:text-green-200">
              签到完成
            </h3>
            <div class="mt-2 text-sm text-green-700 dark:text-green-300">
              <p>
                成功: {{ checkinResult.summary.success }} /
                已签到: {{ checkinResult.summary.already_checked_in }} /
                失败: {{ checkinResult.summary.failed }} /
                总计: {{ checkinResult.summary.total }}
              </p>
            </div>
          </div>
        </div>
        <button
          class="text-green-400 hover:text-green-500"
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
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
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
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
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
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
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
            class="py-4 px-1 border-b-2 font-medium text-sm transition-colors"
            :class="activeTab === tab.id
              ? 'border-blue-500 text-blue-600 dark:text-blue-400'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'"
            @click="activeTab = tab.id"
          >
            {{ tab.icon }} {{ tab.name }}
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
            <span class="text-lg">🏪</span>
            <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
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
                  {{ bp.checkin_bugged ? '⚠️ 自动签到' : '✅ 支持签到' }}
                </span>
                <span
                  v-else
                  class="px-2 py-0.5 text-xs bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 rounded-full"
                >
                  ❌ 无签到
                </span>
                <span
                  v-if="bp.requires_waf_bypass"
                  class="px-2 py-0.5 text-xs bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300 rounded-full"
                >
                  🛡️ 需要 WAF 绕过
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- 已添加的提供商 -->
        <div>
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center space-x-2">
              <span class="text-lg">🏢</span>
              <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
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
              📦
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
              <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
            </div>
            <!-- 提供商过滤 -->
            <select
              v-model="providerFilter"
              class="px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">全部提供商</option>
              <option v-for="p in providers" :key="p.id" :value="p.id">{{ p.name }}</option>
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
          class="bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700 overflow-hidden"
        >
          <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead class="bg-gray-50/80 dark:bg-gray-700/50 backdrop-blur-sm sticky top-0">
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
                  <span v-else class="text-xs text-gray-400">-</span>
                </td>
                <!-- 总额度 -->
                <td class="px-4 py-3 text-right">
                  <span
                    v-if="account.total_quota !== undefined && account.total_quota !== null"
                    class="font-mono text-sm font-semibold text-blue-600 dark:text-blue-400"
                  >
                    ${{ account.total_quota.toFixed(2) }}
                  </span>
                  <span v-else class="text-xs text-gray-400">-</span>
                </td>
                <!-- 历史消耗 -->
                <td class="px-4 py-3 text-right">
                  <span
                    v-if="account.total_consumed !== undefined && account.total_consumed !== null"
                    class="font-mono text-sm font-semibold text-orange-600 dark:text-orange-400"
                  >
                    ${{ account.total_consumed.toFixed(2) }}
                  </span>
                  <span v-else class="text-xs text-gray-400">-</span>
                </td>
                <!-- 最后签到 -->
                <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400 font-mono text-xs">
                  {{ account.last_checkin_at ? formatDate(account.last_checkin_at) : '-' }}
                </td>
                <!-- 操作 -->
                <td class="px-4 py-3" @click.stop>
                  <div class="flex items-center justify-center gap-2">
                    <button
                      class="inline-flex items-center px-3 py-1.5 rounded-lg text-xs font-medium bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white shadow-sm transition-all duration-200"
                      @click="executeCheckinSingle(account.id)"
                    >
                      📅 签到
                    </button>
                    <div class="relative">
                      <button
                        class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 dark:hover:text-gray-300 transition-colors"
                        @click="toggleAccountMenu(account.id)"
                      >
                        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                          <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
                        </svg>
                      </button>
                      <!-- 下拉菜单 -->
                      <div
                        v-if="openMenuAccountId === account.id"
                        class="absolute right-0 mt-1 w-32 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-10"
                      >
                        <button
                          class="w-full px-3 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-t-lg"
                          @click="refreshAccountBalance(account.id)"
                        >
                          刷新余额
                        </button>
                        <button
                          class="w-full px-3 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                          @click="openAccountModal(account)"
                        >
                          编辑
                        </button>
                        <button
                          class="w-full px-3 py-2 text-left text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-b-lg"
                          @click="deleteAccount(account.id)"
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
          class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden"
        >
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
                  消息
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
              <tr
                v-for="record in records"
                :key="record.id"
                class="hover:bg-gray-50 dark:hover:bg-gray-700/50"
              >
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
                  {{ record.message || '-' }}
                </td>
              </tr>
            </tbody>
          </table>
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
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="showAccountModal = false"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 w-full max-w-lg mx-4">
        <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          {{ editingAccount ? '编辑账号' : '添加账号' }}
        </h3>
        <form
          class="space-y-4"
          @submit.prevent="saveAccount"
        >
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              提供商 *
            </label>
            <select
              v-model="accountForm.provider_id"
              required
              :disabled="!!editingAccount"
              class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white disabled:opacity-50"
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
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              账号名称 *
            </label>
            <input
              v-model="accountForm.name"
              type="text"
              required
              class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
              placeholder="例如: 主账号"
            >
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Cookies JSON {{ editingAccount ? '(留空不修改)' : '*' }}
            </label>
            <textarea
              v-model="accountForm.cookies_json"
              :required="!editingAccount"
              rows="4"
              class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono text-sm"
              placeholder="{&quot;session&quot;:&quot;xxx&quot;}"
            />
            <div class="mt-2 flex items-center space-x-2">
              <button
                type="button"
                class="px-3 py-1 text-xs bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded transition-colors"
                @click="formatCookiesJson"
              >
                📋 格式化 JSON
              </button>
              <span
                v-if="jsonError"
                class="text-xs text-red-600 dark:text-red-400"
              >
                {{ jsonError }}
              </span>
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              API User (可选)
            </label>
            <input
              v-model="accountForm.api_user"
              type="text"
              class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white font-mono"
              placeholder="12345"
            >
            <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
              通常为 5 位数字，可在浏览器开发者工具 Network 标签的请求头中找到 "New-Api-User"
            </p>
          </div>
          <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3">
            <p class="text-xs font-medium text-blue-800 dark:text-blue-200 mb-2">
              📋 如何获取 Cookies：
            </p>
            <ol class="text-xs text-blue-700 dark:text-blue-300 space-y-1 list-decimal list-inside">
              <li>按 F12 打开浏览器开发者工具</li>
              <li>转到 Application 标签页 → Cookies</li>
              <li>选择目标站点，复制需要的 Cookie 值</li>
              <li>以 JSON 格式填入上方输入框，如：{"session": "值"}</li>
            </ol>
          </div>
          <div class="flex items-center">
            <input
              id="account-enabled"
              v-model="accountForm.enabled"
              type="checkbox"
              class="w-4 h-4 text-blue-600 border-gray-300 rounded"
            >
            <label
              for="account-enabled"
              class="ml-2 text-sm text-gray-700 dark:text-gray-300"
            >
              启用此账号
            </label>
          </div>
          <div class="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              class="px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700"
              @click="showAccountModal = false"
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  listCheckinProviders,
  createCheckinProvider,
  updateCheckinProvider,
  deleteCheckinProvider as apiDeleteProvider,
  listCheckinAccounts,
  createCheckinAccount,
  updateCheckinAccount,
  deleteCheckinAccount as apiDeleteAccount,
  executeCheckin,
  listCheckinRecords,
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
  ExportData,
  ImportPreviewResponse,
  BuiltinProvider,
} from '@/types/checkin'

// 状态
const loading = ref(false)
const checkinLoading = ref(false)
const balanceRefreshing = ref(false)
const error = ref<string | null>(null)
const activeTab = ref<'providers' | 'accounts' | 'records' | 'import-export'>('accounts')
const router = useRouter()
const openMenuAccountId = ref<string | null>(null)
const searchQuery = ref('')
const providerFilter = ref<string>('all')

// 数据
const providers = ref<CheckinProvider[]>([])
const accounts = ref<AccountInfo[]>([])
const records = ref<CheckinRecordInfo[]>([])
const todayStats = ref<TodayCheckinStats | null>(null)
const checkinResult = ref<CheckinResponse | null>(null)
const builtinProviders = ref<BuiltinProvider[]>([])

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

// Tab 配置
const tabs = [
  { id: 'accounts' as const, name: '账号管理', icon: '👤' },
  { id: 'providers' as const, name: '提供商', icon: '🏢' },
  { id: 'records' as const, name: '签到记录', icon: '📜' },
  { id: 'import-export' as const, name: '导入导出', icon: '📦' },
]

// 弹窗状态
const showProviderModal = ref(false)
const showAccountModal = ref(false)
const editingProvider = ref<CheckinProvider | null>(null)
const editingAccount = ref<AccountInfo | null>(null)
const jsonError = ref<string | null>(null)

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
  cookies_json: '',
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
      listCheckinRecords(100),
      getTodayCheckinStats(),
      listBuiltinProviders(),
    ])

    providers.value = providersRes.providers
    accounts.value = accountsRes.accounts
    records.value = recordsRes.records
    todayStats.value = statsRes
    builtinProviders.value = builtinRes.providers
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

// 执行全部签到
const executeCheckinAll = async () => {
  checkinLoading.value = true
  checkinResult.value = null

  try {
    const result = await executeCheckin()
    checkinResult.value = result
    await loadAllData()
  } catch (e: any) {
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
// Cookies JSON 默认模板
const DEFAULT_COOKIES_TEMPLATE = '{"session":"xxx"}'

const openAccountModal = async (account?: AccountInfo) => {
  editingAccount.value = account || null
  jsonError.value = null
  
  if (account) {
    // 编辑已有账号：从后端获取解密后的 cookies
    try {
      const { getCheckinAccountCookies } = await import('@/api/client')
      const cookiesData = await getCheckinAccountCookies(account.id)
      accountForm.value = {
        provider_id: account.provider_id,
        name: account.name,
        cookies_json: cookiesData.cookies_json, // 使用真实的 cookies
        api_user: cookiesData.api_user || '',
        enabled: account.enabled,
      }
    } catch (e: any) {
      console.error('Failed to get cookies:', e)
      // 如果获取失败，使用默认模板
      accountForm.value = {
        provider_id: account.provider_id,
        name: account.name,
        cookies_json: DEFAULT_COOKIES_TEMPLATE,
        api_user: account.api_user || '',
        enabled: account.enabled,
      }
    }
  } else {
    // 添加新账号：使用默认模板
    accountForm.value = {
      provider_id: providers.value[0]?.id || '',
      name: '',
      cookies_json: DEFAULT_COOKIES_TEMPLATE,
      api_user: '',
      enabled: true,
    }
  }
  showAccountModal.value = true
}

const saveAccount = async () => {
  try {
    if (editingAccount.value) {
      const updateData: { name?: string; cookies_json?: string; api_user?: string; enabled?: boolean } = {
        name: accountForm.value.name,
        enabled: accountForm.value.enabled,
      }
      if (accountForm.value.cookies_json) {
        updateData.cookies_json = accountForm.value.cookies_json
      }
      if (accountForm.value.api_user) {
        updateData.api_user = accountForm.value.api_user
      }
      await updateCheckinAccount(editingAccount.value.id, updateData)
    } else {
      await createCheckinAccount({
        provider_id: accountForm.value.provider_id,
        name: accountForm.value.name,
        cookies_json: accountForm.value.cookies_json,
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

// 格式化 Cookies JSON
const formatCookiesJson = () => {
  jsonError.value = null
  const input = accountForm.value.cookies_json.trim()
  
  if (!input) {
    jsonError.value = '请输入 JSON 内容'
    return
  }

  try {
    const parsed = JSON.parse(input)
    // 验证是否为对象
    if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
      jsonError.value = 'JSON 必须是对象格式，如 {"key": "value"}'
      return
    }
    // 格式化并重新赋值
    accountForm.value.cookies_json = JSON.stringify(parsed, null, 2)
  } catch (e: any) {
    jsonError.value = '无效的 JSON 格式: ' + (e.message || '未知错误')
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

onMounted(() => {
  loadAllData()
})
</script>
