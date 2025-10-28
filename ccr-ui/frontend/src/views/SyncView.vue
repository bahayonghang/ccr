<template>
  <div class="min-h-screen relative overflow-hidden" :style="{ 
    background: 'linear-gradient(135deg, #1e1b4b 0%, #312e81 25%, #4c1d95 50%, #5b21b6 75%, #6d28d9 100%)'
  }">
    <!-- 动态背景装饰 -->
    <div class="absolute inset-0 overflow-hidden pointer-events-none">
      <div class="absolute -top-40 -right-40 w-80 h-80 bg-purple-500/20 rounded-full blur-3xl"></div>
      <div class="absolute top-1/2 -left-40 w-96 h-96 bg-indigo-500/20 rounded-full blur-3xl"></div>
      <div class="absolute bottom-20 right-1/3 w-72 h-72 bg-violet-500/20 rounded-full blur-3xl"></div>
    </div>
    
    <main class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-8 pb-8">
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
          <div class="flex items-center gap-4">
            <div class="p-3 rounded-2xl backdrop-blur-xl bg-white/20 border border-white/30 shadow-xl">
              <Cloud class="w-8 h-8 text-white drop-shadow-lg" />
            </div>
            <h1 class="text-4xl font-bold text-white drop-shadow-lg">WebDAV 云同步</h1>
          </div>
          <RouterLink
            to="/"
            class="group flex items-center gap-2 px-5 py-2.5 rounded-xl backdrop-blur-md bg-white/20 border border-white/30 transition-all duration-300 hover:bg-white/30 hover:scale-105 shadow-lg"
          >
            <Home class="w-4 h-4 text-white" />
            <span class="font-medium text-white">返回首页</span>
          </RouterLink>
        </div>
        <p class="text-white/90 text-lg drop-shadow-md">使用 WebDAV 协议同步配置文件到云端存储，支持目录同步，智能排除备份和临时文件</p>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading" class="flex items-center justify-center py-16">
        <div class="p-6 rounded-2xl backdrop-blur-xl bg-white/20 border border-white/30 shadow-2xl">
          <RefreshCw class="w-10 h-10 animate-spin text-white drop-shadow-lg" />
        </div>
      </div>

      <!-- 错误状态 -->
      <div
        v-else-if="error"
        class="rounded-2xl backdrop-blur-xl bg-red-500/20 border border-red-400/30 p-6 flex items-start gap-4 shadow-xl"
      >
        <XCircle class="w-7 h-7 flex-shrink-0 mt-0.5 text-red-200 drop-shadow-md" />
        <div>
          <h3 class="font-bold text-xl mb-2 text-white drop-shadow-md">加载失败</h3>
          <p class="text-base text-white/90 drop-shadow-md">{{ error }}</p>
        </div>
      </div>

      <!-- 主要内容 -->
      <div v-else class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- 左侧主内容区 -->
        <div class="lg:col-span-2 space-y-6">
          <div
            class="rounded-2xl backdrop-blur-xl bg-white/15 border border-white/30 shadow-2xl overflow-hidden transition-all duration-300 hover:bg-white/20"
          >
            <!-- 头部 -->
            <div class="px-6 py-5 bg-gradient-to-r from-white/25 to-white/15 border-b border-white/30">
              <h2 class="text-2xl font-bold text-white flex items-center gap-3 drop-shadow-[0_2px_8px_rgba(0,0,0,0.8)]">
                <div class="p-2 rounded-xl bg-white/30">
                  <Cloud class="w-6 h-6" />
                </div>
                同步状态
              </h2>
            </div>

            <div class="p-6">
              <!-- 已配置状态 -->
              <div v-if="syncStatus?.configured && syncStatus.config" class="space-y-4">
                <div class="flex items-center gap-3 px-5 py-3.5 rounded-xl backdrop-blur-md bg-emerald-400/20 border border-emerald-300/30 shadow-lg">
                  <CheckCircle class="w-6 h-6 text-emerald-100 drop-shadow-md" />
                  <span class="font-semibold text-emerald-50 text-lg drop-shadow-md">同步功能已配置</span>
                </div>

                <!-- 配置详情卡片 -->
                <div class="grid grid-cols-1 gap-4">
                  <!-- WebDAV 服务器 -->
                  <div class="rounded-xl backdrop-blur-md bg-white/15 border border-white/30 p-5 transition-all duration-300 hover:bg-white/20 hover:scale-[1.02]">
                    <div class="flex items-start gap-4">
                      <div class="p-3 rounded-xl bg-blue-500/40 backdrop-blur-sm">
                        <Server class="w-6 h-6 text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]" />
                      </div>
                      <div class="flex-1">
                        <div class="text-sm font-medium mb-2 text-white/90">WebDAV 服务器</div>
                        <div class="text-base font-mono break-all text-white font-semibold drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]">
                          {{ syncStatus.config.webdav_url }}
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- 用户名 -->
                  <div class="rounded-xl backdrop-blur-md bg-white/15 border border-white/30 p-5 transition-all duration-300 hover:bg-white/20 hover:scale-[1.02]">
                    <div class="flex items-start gap-4">
                      <div class="p-3 rounded-xl bg-purple-500/40 backdrop-blur-sm">
                        <User class="w-6 h-6 text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]" />
                      </div>
                      <div class="flex-1">
                        <div class="text-sm font-medium mb-2 text-white/90">用户名</div>
                        <div class="text-base font-mono text-white font-semibold drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]">
                          {{ syncStatus.config.username }}
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- 远程路径 -->
                  <div class="rounded-xl backdrop-blur-md bg-white/15 border border-white/30 p-5 transition-all duration-300 hover:bg-white/20 hover:scale-[1.02]">
                    <div class="flex items-start gap-4">
                      <div class="p-3 rounded-xl bg-pink-500/40 backdrop-blur-sm">
                        <FolderOpen class="w-6 h-6 text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]" />
                      </div>
                      <div class="flex-1">
                        <div class="text-sm font-medium mb-2 text-white/90">远程路径</div>
                        <div class="text-base font-mono break-all text-white font-semibold drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]">
                          {{ syncStatus.config.remote_path }}
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- 自动同步 -->
                  <div class="rounded-xl backdrop-blur-md bg-white/15 border border-white/30 p-5 transition-all duration-300 hover:bg-white/20 hover:scale-[1.02]">
                    <div class="flex items-start gap-4">
                      <div class="p-3 rounded-xl bg-amber-500/40 backdrop-blur-sm">
                        <Settings class="w-6 h-6 text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]" />
                      </div>
                      <div class="flex-1">
                        <div class="text-sm font-medium mb-2 text-white/90">自动同步</div>
                        <div class="text-base text-white font-semibold drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]">
                          {{ syncStatus.config.auto_sync ? '✓ 开启' : '✗ 关闭' }}
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- 远程文件存在 -->
                  <div
                    v-if="typeof syncStatus.config.remote_file_exists === 'boolean'"
                    class="rounded-xl backdrop-blur-md bg-white/15 border border-white/30 p-5 transition-all duration-300 hover:bg-white/20 hover:scale-[1.02]"
                  >
                    <div class="flex items-start gap-4">
                      <div class="p-3 rounded-xl bg-emerald-500/40 backdrop-blur-sm">
                        <Info class="w-6 h-6 text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]" />
                      </div>
                      <div class="flex-1">
                        <div class="text-sm font-medium mb-2 text-white/90">远程文件状态</div>
                        <div class="text-base text-white font-semibold drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]">
                          {{ syncStatus.config.remote_file_exists ? '✓ 存在' : '✗ 不存在' }}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 操作按钮 -->
                <div class="flex flex-wrap gap-3">
                  <button
                    class="group flex items-center gap-2 px-6 py-3 rounded-xl backdrop-blur-md bg-gradient-to-r from-blue-500/80 to-indigo-500/80 border border-white/30 shadow-xl disabled:opacity-50 transition-all duration-300 hover:scale-105 hover:shadow-2xl hover:from-blue-600/90 hover:to-indigo-600/90"
                    :disabled="operating"
                    @click="handlePush(false)"
                  >
                    <CloudUpload class="w-5 h-5 text-white drop-shadow-md" />
                    <span class="font-semibold text-white drop-shadow-md">上传到云端</span>
                  </button>

                  <button
                    class="group flex items-center gap-2 px-6 py-3 rounded-xl backdrop-blur-md bg-gradient-to-r from-purple-500/80 to-pink-500/80 border border-white/30 shadow-xl disabled:opacity-50 transition-all duration-300 hover:scale-105 hover:shadow-2xl hover:from-purple-600/90 hover:to-pink-600/90"
                    :disabled="operating"
                    @click="handlePull(false)"
                  >
                    <CloudDownload class="w-5 h-5 text-white drop-shadow-md" />
                    <span class="font-semibold text-white drop-shadow-md">从云端下载</span>
                  </button>

                  <button
                    class="px-4 py-2.5 text-sm rounded-xl backdrop-blur-md bg-white/20 border border-white/30 text-white font-medium shadow-lg disabled:opacity-50 transition-all duration-300 hover:bg-white/30 hover:scale-105"
                    :disabled="operating"
                    @click="handlePush(true)"
                  >
                    强制上传
                  </button>

                  <button
                    class="px-4 py-2.5 text-sm rounded-xl backdrop-blur-md bg-white/20 border border-white/30 text-white font-medium shadow-lg disabled:opacity-50 transition-all duration-300 hover:bg-white/30 hover:scale-105"
                    :disabled="operating"
                    @click="handlePull(true)"
                  >
                    强制下载
                  </button>
                </div>

                <!-- 操作结果 -->
                <div
                  v-if="operationResult"
                  class="mt-4 p-5 rounded-xl backdrop-blur-md border shadow-lg"
                  :class="operationResult.success ? 'bg-emerald-400/20 border-emerald-300/30' : 'bg-red-400/20 border-red-300/30'"
                >
                  <div class="flex items-center gap-3">
                    <CheckCircle v-if="operationResult.success" class="w-6 h-6 text-emerald-100 drop-shadow-md" />
                    <AlertCircle v-else class="w-6 h-6 text-red-100 drop-shadow-md" />
                    <span class="font-bold text-lg text-white drop-shadow-md">
                      {{ operationResult.success ? '操作成功' : '操作失败' }}
                    </span>
                  </div>
                  <pre class="mt-3 text-sm whitespace-pre-wrap text-white/90 drop-shadow-md leading-relaxed">{{ operationResult.message }}</pre>
                </div>
              </div>

              <!-- 未配置状态 -->
              <div
                v-else
                class="rounded-xl backdrop-blur-md bg-amber-400/20 border border-amber-300/30 p-6 flex items-start gap-4 shadow-lg"
              >
                <AlertCircle class="w-6 h-6 flex-shrink-0 mt-0.5 text-amber-100 drop-shadow-md" />
                <div>
                  <h3 class="font-bold text-lg mb-2 text-white drop-shadow-md">同步功能未配置</h3>
                  <p class="text-base text-white/90 drop-shadow-md">
                    请在终端中运行
                    <code class="font-mono bg-white/20 px-2 py-1 rounded-lg">ccr sync config</code>
                    设置 WebDAV 连接
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧信息栏 -->
        <aside class="space-y-6">
          <!-- 功能说明卡片 -->
          <div class="rounded-2xl backdrop-blur-xl bg-white/15 border border-white/30 shadow-2xl overflow-hidden transition-all duration-300 hover:bg-white/20">
            <!-- 头部 -->
            <div class="px-6 py-5 bg-gradient-to-r from-white/25 to-white/15 border-b border-white/30">
              <h2 class="text-2xl font-bold text-white flex items-center gap-3 drop-shadow-[0_2px_8px_rgba(0,0,0,0.8)]">
                <div class="p-2 rounded-xl bg-white/30">
                  <Info class="w-6 h-6" />
                </div>
                功能说明
              </h2>
            </div>

            <div class="p-6">
              <div v-if="syncInfo" class="space-y-5">
                <div>
                  <h3 class="text-lg font-bold text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)] mb-2">
                    {{ syncInfo.feature_name }}
                  </h3>
                  <p class="text-sm text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)] leading-relaxed">
                    {{ syncInfo.description }}
                  </p>
                </div>

                <div>
                  <h4 class="text-base font-semibold flex items-center gap-2 text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)] mb-3">
                    <Server class="w-5 h-5" />
                    支持的服务
                  </h4>
                  <ul class="space-y-2">
                    <li v-for="service in syncInfo.supported_services" :key="service" class="flex items-center gap-2.5">
                      <span class="w-2 h-2 rounded-full bg-white/80 drop-shadow-md"></span>
                      <span class="text-sm text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]">{{ service }}</span>
                    </li>
                  </ul>
                </div>

                <div>
                  <h4 class="text-base font-semibold flex items-center gap-2 text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)] mb-3">
                    <Settings class="w-5 h-5" />
                    配置步骤
                  </h4>
                  <ol class="space-y-2.5">
                    <li v-for="(step, index) in syncInfo.setup_steps" :key="step" class="flex gap-3">
                      <span class="flex-shrink-0 w-6 h-6 rounded-full backdrop-blur-md bg-amber-500/50 border border-amber-300/50 flex items-center justify-center text-xs font-bold text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]">
                        {{ index + 1 }}
                      </span>
                      <span class="text-sm text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)] leading-relaxed">{{ step }}</span>
                    </li>
                  </ol>
                </div>

                <div>
                  <h4 class="text-base font-semibold flex items-center gap-2 text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)] mb-3">
                    <AlertCircle class="w-5 h-5" />
                    安全与同步说明
                  </h4>
                  <ul class="space-y-2.5">
                    <li v-for="note in syncInfo.security_notes" :key="note" class="flex items-start gap-2.5">
                      <CheckCircle class="w-5 h-5 flex-shrink-0 mt-0.5 text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]" />
                      <span class="text-sm text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)] leading-relaxed">{{ note }}</span>
                    </li>
                  </ul>
                </div>
              </div>

              <div v-else class="text-sm text-white drop-shadow-[0_2px_4px_rgba(0,0,0,0.5)]">加载中...</div>
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