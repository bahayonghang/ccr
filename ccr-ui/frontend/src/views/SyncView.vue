<template>
  <div class="min-h-screen" :style="{ background: 'linear-gradient(to bottom right, #f8fafc, #dbeafe, #e0e7ff)' }">
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-8 pb-8">
      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="[
          { label: '首页', path: '/', icon: Home },
          { label: 'Claude Code', path: '/claude-code', icon: Code2 },
          { label: '云同步', path: '/sync', icon: Cloud }
        ]"
        moduleColor="#6366f1"
      />
      <div class="mb-8">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-3">
            <Cloud class="w-8 h-8" :style="{ color: '#2563eb' }" />
            <h1 class="text-3xl font-bold" :style="{ color: '#111827' }">WebDAV 云同步</h1>
          </div>
          <RouterLink
            to="/"
            class="flex items-center gap-2 px-4 py-2 rounded-lg border transition-colors shadow-sm"
            :style="{
              background: 'white',
              color: '#374151',
              borderColor: '#d1d5db'
            }"
          >
            <Home class="w-4 h-4" />
            <span class="font-medium">返回首页</span>
          </RouterLink>
        </div>
        <p :style="{ color: '#4b5563' }">使用 WebDAV 协议同步配置文件到云端存储（坚果云、Nextcloud、ownCloud 等）</p>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading" class="flex items-center justify-center py-12">
        <RefreshCw class="w-8 h-8 animate-spin" :style="{ color: '#2563eb' }" />
      </div>

      <!-- 错误状态 -->
      <div
        v-else-if="error"
        class="rounded-lg p-4 flex items-start gap-3"
        :style="{
          background: '#fef2f2',
          border: '1px solid #fecaca'
        }"
      >
        <XCircle class="w-5 h-5 flex-shrink-0 mt-0.5" :style="{ color: '#dc2626' }" />
        <div>
          <h3 class="font-semibold mb-1" :style="{ color: '#7f1d1d' }">加载失败</h3>
          <p class="text-sm" :style="{ color: '#b91c1c' }">{{ error }}</p>
        </div>
      </div>

      <!-- 主要内容 -->
      <div v-else class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- 左侧主内容区 -->
        <div class="lg:col-span-2 space-y-6">
          <div
            class="rounded-xl shadow-sm border overflow-hidden"
            :style="{
              background: 'white',
              borderColor: '#e5e7eb'
            }"
          >
            <!-- 头部 -->
            <div
              class="px-6 py-4"
              :style="{
                background: 'linear-gradient(to right, #2563eb, #4f46e5)'
              }"
            >
              <h2 class="text-xl font-bold text-white flex items-center gap-2">
                <Cloud class="w-6 h-6" />
                同步状态
              </h2>
            </div>

            <div class="p-6">
              <!-- 已配置状态 -->
              <div v-if="syncStatus?.configured && syncStatus.config" class="space-y-4">
                <div
                  class="flex items-center gap-2 px-4 py-3 rounded-lg"
                  :style="{
                    color: '#15803d',
                    background: '#f0fdf4',
                    border: '1px solid #bbf7d0'
                  }"
                >
                  <CheckCircle class="w-5 h-5" />
                  <span class="font-medium">同步功能已配置</span>
                </div>

                <!-- 配置详情卡片 -->
                <div class="grid grid-cols-1 gap-3">
                  <!-- WebDAV 服务器 -->
                  <div
                    class="border rounded-lg p-4"
                    :style="{
                      background: 'white',
                      borderColor: '#e5e7eb'
                    }"
                  >
                    <div class="flex items-start gap-3">
                      <div class="p-2 rounded-lg" :style="{ background: '#eff6ff' }">
                        <Server class="w-5 h-5" :style="{ color: '#2563eb' }" />
                      </div>
                      <div class="flex-1">
                        <div class="text-xs mb-1" :style="{ color: '#6b7280' }">WebDAV 服务器</div>
                        <div class="text-sm font-mono break-all" :style="{ color: '#111827' }">
                          {{ syncStatus.config.webdav_url }}
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- 用户名 -->
                  <div
                    class="border rounded-lg p-4"
                    :style="{
                      background: 'white',
                      borderColor: '#e5e7eb'
                    }"
                  >
                    <div class="flex items-start gap-3">
                      <div class="p-2 rounded-lg" :style="{ background: '#faf5ff' }">
                        <User class="w-5 h-5" :style="{ color: '#9333ea' }" />
                      </div>
                      <div class="flex-1">
                        <div class="text-xs mb-1" :style="{ color: '#6b7280' }">用户名</div>
                        <div class="text-sm font-mono" :style="{ color: '#111827' }">
                          {{ syncStatus.config.username }}
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- 远程路径 -->
                  <div
                    class="border rounded-lg p-4"
                    :style="{
                      background: 'white',
                      borderColor: '#e5e7eb'
                    }"
                  >
                    <div class="flex items-start gap-3">
                      <div class="p-2 rounded-lg" :style="{ background: '#fdf2f8' }">
                        <FolderOpen class="w-5 h-5" :style="{ color: '#db2777' }" />
                      </div>
                      <div class="flex-1">
                        <div class="text-xs mb-1" :style="{ color: '#6b7280' }">远程路径</div>
                        <div class="text-sm font-mono break-all" :style="{ color: '#111827' }">
                          {{ syncStatus.config.remote_path }}
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- 自动同步 -->
                  <div
                    class="border rounded-lg p-4"
                    :style="{
                      background: 'white',
                      borderColor: '#e5e7eb'
                    }"
                  >
                    <div class="flex items-start gap-3">
                      <div class="p-2 rounded-lg" :style="{ background: '#fffbeb' }">
                        <Settings class="w-5 h-5" :style="{ color: '#d97706' }" />
                      </div>
                      <div class="flex-1">
                        <div class="text-xs mb-1" :style="{ color: '#6b7280' }">自动同步</div>
                        <div class="text-sm" :style="{ color: '#111827' }">
                          {{ syncStatus.config.auto_sync ? '✓ 开启' : '✗ 关闭' }}
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- 远程文件存在 -->
                  <div
                    v-if="typeof syncStatus.config.remote_file_exists === 'boolean'"
                    class="border rounded-lg p-4"
                    :style="{
                      background: 'white',
                      borderColor: '#e5e7eb'
                    }"
                  >
                    <div class="flex items-start gap-3">
                      <div class="p-2 rounded-lg" :style="{ background: '#f0fdf4' }">
                        <Info class="w-5 h-5" :style="{ color: '#16a34a' }" />
                      </div>
                      <div class="flex-1">
                        <div class="text-xs mb-1" :style="{ color: '#6b7280' }">远程文件状态</div>
                        <div class="text-sm" :style="{ color: '#111827' }">
                          {{ syncStatus.config.remote_file_exists ? '✓ 存在' : '✗ 不存在' }}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 操作按钮 -->
                <div class="flex flex-wrap gap-3">
                  <button
                    class="flex items-center gap-2 px-4 py-2 rounded-lg shadow-sm disabled:opacity-50 transition-all hover:scale-105"
                    :style="{
                      background: '#2563eb',
                      color: 'white'
                    }"
                    :disabled="operating"
                    @click="handlePush(false)"
                  >
                    <CloudUpload class="w-4 h-4" />
                    <span>上传到云端</span>
                  </button>

                  <button
                    class="flex items-center gap-2 px-4 py-2 rounded-lg shadow-sm disabled:opacity-50 transition-all hover:scale-105"
                    :style="{
                      background: '#4f46e5',
                      color: 'white'
                    }"
                    :disabled="operating"
                    @click="handlePull(false)"
                  >
                    <CloudDownload class="w-4 h-4" />
                    <span>从云端下载</span>
                  </button>

                  <button
                    class="px-3 py-2 text-sm rounded-lg border transition-all hover:scale-105"
                    :style="{
                      borderColor: '#bfdbfe',
                      color: '#1d4ed8',
                      background: '#eff6ff'
                    }"
                    :disabled="operating"
                    @click="handlePush(true)"
                  >
                    强制上传
                  </button>

                  <button
                    class="px-3 py-2 text-sm rounded-lg border transition-all hover:scale-105"
                    :style="{
                      borderColor: '#c7d2fe',
                      color: '#4338ca',
                      background: '#eef2ff'
                    }"
                    :disabled="operating"
                    @click="handlePull(true)"
                  >
                    强制下载
                  </button>
                </div>

                <!-- 操作结果 -->
                <div
                  v-if="operationResult"
                  class="mt-4 p-4 rounded-lg border"
                  :style="{
                    background: operationResult.success ? '#f0fdf4' : '#fef2f2',
                    borderColor: operationResult.success ? '#bbf7d0' : '#fecaca',
                    color: operationResult.success ? '#15803d' : '#b91c1c'
                  }"
                >
                  <div class="flex items-center gap-2">
                    <CheckCircle v-if="operationResult.success" class="w-5 h-5" />
                    <AlertCircle v-else class="w-5 h-5" />
                    <span class="font-medium">
                      {{ operationResult.success ? '操作成功' : '操作失败' }}
                    </span>
                  </div>
                  <pre class="mt-2 text-xs whitespace-pre-wrap">{{ operationResult.message }}</pre>
                </div>
              </div>

              <!-- 未配置状态 -->
              <div
                v-else
                class="rounded-lg p-4 flex items-start gap-3"
                :style="{
                  background: '#fffbeb',
                  border: '1px solid #fef3c7'
                }"
              >
                <AlertCircle class="w-5 h-5 flex-shrink-0 mt-0.5" :style="{ color: '#a16207' }" />
                <div>
                  <h3 class="font-semibold mb-1" :style="{ color: '#78350f' }">同步功能未配置</h3>
                  <p class="text-sm" :style="{ color: '#92400e' }">
                    请在终端中运行
                    <code class="font-mono">ccr sync config</code>
                    设置 WebDAV 连接
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧信息栏 -->
        <aside class="space-y-6">
          <div
            class="rounded-xl shadow-sm border overflow-hidden"
            :style="{
              background: 'white',
              borderColor: '#e5e7eb'
            }"
          >
            <!-- 头部 -->
            <div
              class="px-6 py-4"
              :style="{
                background: 'linear-gradient(to right, #9333ea, #db2777)'
              }"
            >
              <h2 class="text-xl font-bold text-white flex items-center gap-2">
                <Info class="w-6 h-6" />
                功能说明
              </h2>
            </div>

            <div class="p-6">
              <div v-if="syncInfo" class="space-y-4">
                <div>
                  <h3 class="text-base font-semibold" :style="{ color: '#111827' }">
                    {{ syncInfo.feature_name }}
                  </h3>
                  <p class="text-sm mt-1" :style="{ color: '#374151' }">
                    {{ syncInfo.description }}
                  </p>
                </div>

                <div>
                  <h4 class="text-sm font-semibold" :style="{ color: '#111827' }">支持的服务</h4>
                  <ul class="mt-2 list-disc list-inside text-sm space-y-1" :style="{ color: '#374151' }">
                    <li v-for="service in syncInfo.supported_services" :key="service">
                      {{ service }}
                    </li>
                  </ul>
                </div>

                <div>
                  <h4 class="text-sm font-semibold" :style="{ color: '#111827' }">配置步骤</h4>
                  <ol class="mt-2 list-decimal list-inside text-sm space-y-1" :style="{ color: '#374151' }">
                    <li v-for="step in syncInfo.setup_steps" :key="step">
                      {{ step }}
                    </li>
                  </ol>
                </div>

                <div>
                  <h4 class="text-sm font-semibold" :style="{ color: '#111827' }">安全说明</h4>
                  <ul class="mt-2 list-disc list-inside text-sm space-y-1" :style="{ color: '#374151' }">
                    <li v-for="note in syncInfo.security_notes" :key="note">
                      {{ note }}
                    </li>
                  </ul>
                </div>
              </div>

              <div v-else class="text-sm" :style="{ color: '#6b7280' }">加载中...</div>
            </div>
          </div>
        </aside>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import Breadcrumb from '@/components/Breadcrumb.vue'
import { RouterLink } from 'vue-router'
import {
  Cloud,
  CloudUpload,
  CloudDownload,
  Info,
  CheckCircle,
  XCircle,
  RefreshCw,
  AlertCircle,
  Home,
  Server,
  User,
  FolderOpen,
  Settings,
  ArrowLeft,
  Code2,
} from 'lucide-vue-next'
import { getSyncStatus, getSyncInfo, pushSync, pullSync } from '@/api/client'
import type { SyncStatusResponse, SyncInfoResponse } from '@/types'

const syncStatus = ref<SyncStatusResponse | null>(null)
const syncInfo = ref<SyncInfoResponse | null>(null)
const loading = ref(true)
const operating = ref(false)
const error = ref<string | null>(null)
const operationResult = ref<{ success: boolean; message: string } | null>(null)

const loadSyncStatus = async () => {
  try {
    loading.value = true
    error.value = null
    const statusData = await getSyncStatus()
    syncStatus.value = statusData

    const infoData = await getSyncInfo()
    syncInfo.value = infoData
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load sync status'
    console.error('Error loading sync status:', err)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadSyncStatus()
})

const handlePush = async (force: boolean = false) => {
  if (!force && !confirm('确定要上传配置到云端吗？\n如果远程文件已存在将会被覆盖。')) {
    return
  }
  try {
    operating.value = true
    operationResult.value = null
    const data = await pushSync({ force })
    operationResult.value = { success: data.success, message: data.output || data.error }
    if (data.success) {
      await loadSyncStatus()
    }
  } catch (err) {
    operationResult.value = {
      success: false,
      message: err instanceof Error ? err.message : 'Failed to push config'
    }
  } finally {
    operating.value = false
  }
}

const handlePull = async (force: boolean = false) => {
  if (!force && !confirm('确定要从云端下载配置吗？\n本地配置将被覆盖（会先备份）。')) {
    return
  }
  try {
    operating.value = true
    operationResult.value = null
    const data = await pullSync({ force })
    operationResult.value = { success: data.success, message: data.output || data.error }
    if (data.success) {
      await loadSyncStatus()
    }
  } catch (err) {
    operationResult.value = {
      success: false,
      message: err instanceof Error ? err.message : 'Failed to pull config'
    }
  } finally {
    operating.value = false
  }
}
</script>