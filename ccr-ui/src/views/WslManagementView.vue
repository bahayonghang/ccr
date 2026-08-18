<script setup lang="ts">
/**
 * WSL 管理视图 — WSL 发行版列表、配置浏览、同步操作
 */
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { computed, ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
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

const { locale } = useI18n()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)
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
const stateBullet = '●'

const platforms = ['claude', 'codex', 'gemini']

const stateColor = (state: string) => {
  switch (state.toLowerCase()) {
    case 'running': return 'text-accent-success'
    case 'stopped': return 'text-text-muted'
    default: return 'text-accent-warning'
  }
}

const formatCacheAge = (secs: number | null): string => {
  if (secs === null) return tt('未知', 'Unknown')
  if (secs < 60) return isZh.value ? `${secs}秒前` : `${secs}s ago`
  if (secs < 3600) return isZh.value ? `${Math.floor(secs / 60)}分钟前` : `${Math.floor(secs / 60)}m ago`
  if (secs < 86400) return isZh.value ? `${Math.floor(secs / 3600)}小时前` : `${Math.floor(secs / 3600)}h ago`
  return isZh.value ? `${Math.floor(secs / 86400)}天前` : `${Math.floor(secs / 86400)}d ago`
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
    configContent.value = `${tt('读取失败', 'Read failed')}: ${e}`
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
    syncMessage.value = `${tt('同步失败', 'Sync failed')}: ${e}`
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
  <PageShell class="wsl-page">
    <template #header>
      <PageHeader
        :title="tt('WSL 环境管理', 'WSL Environment Management')"
        :description="tt('管理 Windows Subsystem for Linux 发行版配置', 'Manage Windows Subsystem for Linux distribution configuration')"
      >
        <template #actions>
          <button
            class="flex items-center gap-2 px-4 py-2 rounded-lg border border-border-default/25 text-text-primary hover:border-accent-primary/30 transition-colors text-sm"
            :disabled="isRefreshing"
            @click="refresh"
          >
            <SIcon
              name="RefreshCw"
              size="w-4 h-4"
              :class="{ 'animate-spin': isRefreshing }"
            />
            {{ tt('刷新', 'Refresh') }}
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
            {{ tt('强制刷新', 'Force refresh') }}
          </button>
        </template>
      </PageHeader>
    </template>

    <!-- 缓存状态 -->
    <div
      v-if="cacheStatus"
      class="flex items-center justify-between px-4 py-2 rounded-lg border border-border-default/25 bg-bg-surface text-sm"
    >
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <SIcon
            name="Database"
            size="w-4 h-4"
            class="text-text-muted"
          />
          <span class="text-text-primary">{{ `${tt('缓存状态', 'Cache status')}:` }}</span>
          <span
            :class="cacheStatus.has_disk_cache ? 'text-accent-success' : 'text-text-muted'"
          >
            {{ cacheStatus.has_disk_cache ? tt('已缓存', 'Cached') : tt('未缓存', 'Not cached') }}
          </span>
        </div>
        <div
          v-if="cacheStatus.has_disk_cache"
          class="flex items-center gap-2"
        >
          <span class="text-text-muted">|</span>
          <span class="text-text-primary">{{ `${tt('缓存时间', 'Cache age')}:` }}</span>
          <span class="text-text-primary">{{ formatCacheAge(cacheStatus.age_secs) }}</span>
          <span
            v-if="cacheStatus.is_expired"
            class="px-1.5 py-0.5 rounded text-[10px] bg-accent-warning/20 text-accent-warning"
          >
            {{ tt('已过期', 'Expired') }}
          </span>
        </div>
      </div>
      <button
        class="flex items-center gap-1.5 px-2 py-1 rounded text-xs text-text-muted hover:text-text-primary hover: transition-colors"
        @click="clearCache"
      >
        <SIcon
          name="Trash2"
          size="w-3 h-3"
        />
        {{ tt('清除缓存', 'Clear cache') }}
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
      class="rounded-xl border border-border-default/15 bg-bg-surface p-8 text-center"
    >
      <SIcon
        name="Terminal"
        size="w-12 h-12"
        class="mx-auto text-text-muted mb-3"
      />
      <p class="text-text-primary font-medium">
        {{ tt('未检测到 WSL 发行版', 'No WSL distributions detected') }}
      </p>
      <p class="text-sm text-text-muted mt-1">
        {{ tt('请先安装 WSL 并配置至少一个 Linux 发行版', 'Install WSL and configure at least one Linux distribution first') }}
      </p>
    </div>

    <!-- 主内容 -->
    <div
      v-else
      class="grid grid-cols-12 gap-6"
    >
      <!-- 左侧：发行版列表 -->
      <div class="col-span-4 space-y-3">
        <h2 class="text-xs font-medium text-text-muted px-1">
          {{ tt('发行版', 'Distributions') }}
        </h2>
        <div class="space-y-2">
          <button
            v-for="distro in distros"
            :key="distro.name"
            class="w-full flex items-center gap-3 p-3 rounded-xl border transition-colors text-left"
            :class="[ selectedDistro === distro.name ? 'bg-accent-primary/10 border-accent-primary/30 text-accent-primary' : 'bg-bg-surface border-border-default/25 text-text-primary hover:border-border-accent' ]"
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
                <span class="opacity-60">{{ `WSL${distro.version === 'Wsl2' ? '2' : '1'}` }}</span>
                <span :class="stateColor(distro.state)">{{ `${stateBullet} ${distro.state}` }}</span>
              </div>
            </div>
            <span
              v-if="distro.is_default"
              class="px-1.5 py-0.5 rounded text-[9px] font-bold uppercase bg-accent-primary/20 text-accent-primary"
            >
              {{ tt('默认', 'Default') }}
            </span>
          </button>
        </div>
      </div>

      <!-- 右侧：详情面板 -->
      <div class="col-span-8 space-y-6">
        <!-- CLI 工具检测 -->
        <div class="rounded-xl border border-border-default/15 bg-bg-surface p-4">
          <h3 class="text-sm font-semibold text-text-primary mb-3">
            {{ tt('AI CLI 工具状态', 'AI CLI tool status') }}
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
                class="text-accent-success"
              />
              <SIcon
                v-else
                name="XCircle"
                size="w-4 h-4"
                class="text-text-muted"
              />
              <span :class="installed ? 'text-text-primary' : 'text-text-muted'">
                {{ tool }}
              </span>
            </div>
          </div>
        </div>

        <!-- 配置浏览 -->
        <div class="rounded-xl border border-border-default/15 bg-bg-surface p-4">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-semibold text-text-primary flex items-center gap-2">
              <SIcon
                name="FileText"
                size="w-4 h-4"
              />
              {{ tt('配置文件', 'Config file') }}
            </h3>
            <select
              v-model="selectedPlatform"
              class="px-2 py-1 rounded-lg border border-border-default/15 text-xs text-text-primary"
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
          <pre class="p-3 rounded-lg text-xs text-text-primary font-mono overflow-auto max-h-64 whitespace-pre-wrap">{{ configContent || '(空)' }}</pre>
        </div>

        <!-- 同步操作 -->
        <div class="rounded-xl border border-border-default/15 bg-bg-surface p-4">
          <h3 class="text-sm font-semibold text-text-primary mb-3">
            {{ tt('配置同步', 'Config sync') }}
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
              {{ tt('推送到 WSL', 'Push to WSL') }}
            </button>
            <button
              class="flex items-center gap-2 px-4 py-2 rounded-lg border border-border-default/15 text-text-primary text-sm font-medium hover:border-accent-primary/30 transition-colors"
              :disabled="isSyncing"
              @click="syncConfig('wslToLocal')"
            >
              <SIcon
                name="Download"
                size="w-4 h-4"
              />
              {{ tt('从 WSL 拉取', 'Pull from WSL') }}
            </button>
          </div>
          <p
            v-if="syncMessage"
            class="mt-2 text-xs text-text-muted"
          >
            {{ syncMessage }}
          </p>
        </div>
      </div>
    </div>
  </PageShell>
</template>
