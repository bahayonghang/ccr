<script setup lang="ts">
/**
 * WSL 管理视图 — WSL 发行版列表、配置浏览、同步操作
 */
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import {
  clearWslCache,
  detectWslCli,
  getWslCacheStatus,
  listWslDistros,
  readWslConfig,
  refreshWslDistros,
  syncWslConfig,
  type WslCacheStatus,
  type WslCliStatus,
  type WslDistro,
} from '@/api/runtime/wsl'
import { logger } from '@/utils/logger'

const distros = ref<WslDistro[]>([])
const selectedDistro = ref<string | null>(null)
const isLoading = ref(false)
const isRefreshing = ref(false)
const isSyncing = ref(false)
const syncMessage = ref('')
const configContent = ref('')
const cliStatus = ref<WslCliStatus>({})
const selectedPlatform = ref('claude')
const cacheStatus = ref<WslCacheStatus | null>(null)

const platforms = ['claude', 'codex', 'gemini', 'qwen', 'qoder', 'droid']

const stateColor = (state: string) => {
  switch (state.toLowerCase()) {
    case 'running': return 'text-emerald-400'
    case 'stopped': return 'text-text-muted'
    default: return 'text-amber-400'
  }
}

const formatCacheAge = (secs: number | null): string => {
  if (secs === null) return '未知'
  if (secs < 60) return `${secs}秒前`
  if (secs < 3600) return `${Math.floor(secs / 60)}分钟前`
  if (secs < 86400) return `${Math.floor(secs / 3600)}小时前`
  return `${Math.floor(secs / 86400)}天前`
}

const fetchCacheStatus = async () => {
  try {
    cacheStatus.value = await getWslCacheStatus()
  } catch (e) {
    logger.error('[WSL] Failed to get cache status:', e)
  }
}

const fetchDistros = async (forceRefresh = false) => {
  isLoading.value = true
  try {
    distros.value = await listWslDistros(forceRefresh)
    await fetchCacheStatus()
    if (distros.value.length > 0 && !selectedDistro.value) {
      selectedDistro.value = distros.value[0].name
      await loadDistroDetails()
    }
  } catch (e) {
    logger.error('[WSL] Failed to list distros:', e)
  } finally {
    isLoading.value = false
  }
}

const loadDistroDetails = async () => {
  if (!selectedDistro.value) return

  try {
    // 检测 CLI 工具状态
    const status = await detectWslCli(selectedDistro.value)
    cliStatus.value = status
  } catch (e) {
    logger.error('[WSL] Failed to detect CLI:', e)
  }

  await readConfig()
}

const readConfig = async () => {
  if (!selectedDistro.value) return

  try {
    configContent.value = await readWslConfig({
      distro: selectedDistro.value,
      platform: selectedPlatform.value,
      path: 'settings.json',
    })
  } catch (e) {
    configContent.value = `读取失败: ${e}`
  }
}

const syncConfig = async (direction: string) => {
  if (!selectedDistro.value) return

  isSyncing.value = true
  syncMessage.value = ''
  try {
    const result = await syncWslConfig({
      distro: selectedDistro.value,
      platform: selectedPlatform.value,
      direction,
    })
    syncMessage.value = result
  } catch (e) {
    syncMessage.value = `同步失败: ${e}`
  } finally {
    isSyncing.value = false
  }
}

const selectDistro = async (name: string) => {
  selectedDistro.value = name
  await loadDistroDetails()
}

const refresh = async () => {
  isRefreshing.value = true
  await fetchDistros()
  isRefreshing.value = false
}

const forceRefresh = async () => {
  isRefreshing.value = true
  try {
    distros.value = await refreshWslDistros()
    await fetchCacheStatus()
    if (distros.value.length > 0 && !selectedDistro.value) {
      selectedDistro.value = distros.value[0].name
      await loadDistroDetails()
    }
  } catch (e) {
    logger.error('[WSL] Failed to force refresh:', e)
  } finally {
    isRefreshing.value = false
  }
}

const clearCache = async () => {
  try {
    await clearWslCache()
    await fetchCacheStatus()
  } catch (e) {
    logger.error('[WSL] Failed to clear cache:', e)
  }
}

onMounted(() => fetchDistros())
</script>

<template>
  <div class="space-y-6">
    <!-- 标题 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="p-2 rounded-xl bg-orange-500/10">
          <SIcon
            name="Terminal"
            size="w-6 h-6"
            class="text-orange-400"
          />
        </div>
        <div>
          <h1 class="text-xl font-bold text-white">
            WSL 环境管理
          </h1>
          <p class="text-sm text-white/50">
            管理 Windows Subsystem for Linux 发行版配置
          </p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="flex items-center gap-2 px-4 py-2 rounded-lg glass-surface border border-white/20 text-white/80 hover:text-white hover:border-accent-primary/30 transition-colors text-sm"
          :disabled="isRefreshing"
          @click="refresh"
        >
          <SIcon
            name="RefreshCw"
            size="w-4 h-4"
            :class="{ 'animate-spin': isRefreshing }"
          />
          刷新
        </button>
        <button
          class="flex items-center gap-2 px-4 py-2 rounded-lg bg-accent-primary/10 border border-accent-primary/30 text-accent-primary hover:bg-accent-primary/20 transition-colors text-sm"
          :disabled="isRefreshing"
          @click="forceRefresh"
        >
          <SIcon
            name="RefreshCw"
            size="w-4 h-4"
            :class="{ 'animate-spin': isRefreshing }"
          />
          强制刷新
        </button>
      </div>
    </div>

    <!-- 缓存状态 -->
    <div
      v-if="cacheStatus"
      class="flex items-center justify-between px-4 py-2 rounded-lg glass-surface border border-white/20 text-sm"
    >
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <SIcon
            name="Database"
            size="w-4 h-4"
            class="text-white/50"
          />
          <span class="text-white/80">缓存状态:</span>
          <span
            :class="cacheStatus.has_disk_cache ? 'text-emerald-400' : 'text-text-muted'"
          >
            {{ cacheStatus.has_disk_cache ? '已缓存' : '未缓存' }}
          </span>
        </div>
        <div
          v-if="cacheStatus.has_disk_cache"
          class="flex items-center gap-2"
        >
          <span class="text-white/50">|</span>
          <span class="text-white/80">缓存时间:</span>
          <span class="text-white">{{ formatCacheAge(cacheStatus.age_secs) }}</span>
          <span
            v-if="cacheStatus.is_expired"
            class="px-1.5 py-0.5 rounded text-[10px] bg-amber-500/20 text-amber-400"
          >
            已过期
          </span>
        </div>
      </div>
      <button
        class="flex items-center gap-1.5 px-2 py-1 rounded text-xs text-white/50 hover:text-white/80 hover: transition-colors"
        @click="clearCache"
      >
        <SIcon
          name="Trash2"
          size="w-3 h-3"
        />
        清除缓存
      </button>
    </div>

    <!-- 加载状态 -->
    <div
      v-if="isLoading"
      class="flex items-center justify-center py-12"
    >
      <div class="loading-spinner w-8 h-8 border-accent-primary/30 border-t-accent-primary" />
    </div>

    <!-- 无发行版 -->
    <div
      v-else-if="distros.length === 0"
      class="rounded-xl border border-white/10 glass-surface p-8 text-center"
    >
      <SIcon
        name="Terminal"
        size="w-12 h-12"
        class="mx-auto text-white/50 mb-3"
      />
      <p class="text-white/80 font-medium">
        未检测到 WSL 发行版
      </p>
      <p class="text-sm text-white/50 mt-1">
        请先安装 WSL 并配置至少一个 Linux 发行版
      </p>
    </div>

    <!-- 主内容 -->
    <div
      v-else
      class="grid grid-cols-12 gap-6"
    >
      <!-- 左侧：发行版列表 -->
      <div class="col-span-4 space-y-3">
        <h2 class="text-xs font-bold uppercase tracking-wider text-white/50 px-1">
          发行版
        </h2>
        <div class="space-y-2">
          <button
            v-for="distro in distros"
            :key="distro.name"
            class="w-full flex items-center gap-3 p-3 rounded-xl border transition-colors text-left"
            :class="[ selectedDistro === distro.name ? 'bg-accent-primary/10 border-accent-primary/30 text-accent-primary' : 'glass-surface border-white/20 text-white/80 hover:text-white hover:border-border-accent' ]"
            @click="selectDistro(distro.name)"
          >
            <SIcon
              name="Terminal"
              size="w-5 h-5"
              class="flex-shrink-0"
            />
            <div class="flex-1 min-w-0">
              <div class="font-medium truncate">
                {{ distro.name }}
              </div>
              <div class="flex items-center gap-2 text-[10px] mt-0.5">
                <span class="opacity-60">WSL{{ distro.version === 'Wsl2' ? '2' : '1' }}</span>
                <span :class="stateColor(distro.state)">● {{ distro.state }}</span>
              </div>
            </div>
            <span
              v-if="distro.is_default"
              class="px-1.5 py-0.5 rounded text-[9px] font-bold uppercase bg-accent-primary/20 text-accent-primary"
            >
              默认
            </span>
          </button>
        </div>
      </div>

      <!-- 右侧：详情面板 -->
      <div class="col-span-8 space-y-6">
        <!-- CLI 工具检测 -->
        <div class="rounded-xl border border-white/10 glass-surface p-4">
          <h3 class="text-sm font-semibold text-white mb-3">
            AI CLI 工具状态
          </h3>
          <div class="grid grid-cols-3 gap-3">
            <div
              v-for="(installed, tool) in cliStatus"
              :key="tool"
              class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm"
            >
              <SIcon
                v-if="installed"
                name="CheckCircle2"
                size="w-4 h-4"
                class="text-emerald-400"
              />
              <SIcon
                v-else
                name="XCircle"
                size="w-4 h-4"
                class="text-text-muted"
              />
              <span :class="installed ? 'text-white' : 'text-white/50'">
                {{ tool }}
              </span>
            </div>
          </div>
        </div>

        <!-- 配置浏览 -->
        <div class="rounded-xl border border-white/10 glass-surface p-4">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-semibold text-white flex items-center gap-2">
              <SIcon
                name="FileText"
                size="w-4 h-4"
              />
              配置文件
            </h3>
            <select
              v-model="selectedPlatform"
              class="px-2 py-1 rounded-lg border border-white/10 text-xs text-white/80"
              @change="readConfig"
            >
              <option
                v-for="p in platforms"
                :key="p"
                :value="p"
              >
                {{ p }}
              </option>
            </select>
          </div>
          <pre class="p-3 rounded-lg text-xs text-white/80 font-mono overflow-auto max-h-64 whitespace-pre-wrap">{{ configContent || '(空)' }}</pre>
        </div>

        <!-- 同步操作 -->
        <div class="rounded-xl border border-white/10 glass-surface p-4">
          <h3 class="text-sm font-semibold text-white mb-3">
            配置同步
          </h3>
          <div class="flex items-center gap-3">
            <button
              class="flex items-center gap-2 px-4 py-2 rounded-lg bg-accent-primary/10 text-accent-primary text-sm font-medium hover:bg-accent-primary/20 transition-colors"
              :disabled="isSyncing"
              @click="syncConfig('localToWsl')"
            >
              <SIcon
                name="Upload"
                size="w-4 h-4"
              />
              推送到 WSL
            </button>
            <button
              class="flex items-center gap-2 px-4 py-2 rounded-lg border border-white/10 text-white/80 text-sm font-medium hover:text-white hover:border-accent-primary/30 transition-colors"
              :disabled="isSyncing"
              @click="syncConfig('wslToLocal')"
            >
              <SIcon
                name="Download"
                size="w-4 h-4"
              />
              从 WSL 拉取
            </button>
          </div>
          <p
            v-if="syncMessage"
            class="mt-2 text-xs text-white/50"
          >
            {{ syncMessage }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
