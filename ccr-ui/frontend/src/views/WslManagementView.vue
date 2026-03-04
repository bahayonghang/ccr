<!-- eslint-disable no-console -->
<script setup lang="ts">
/**
 * WSL 管理视图 — WSL 发行版列表、配置浏览、同步操作
 */
import { ref, onMounted } from 'vue'
import { Terminal, RefreshCw, Upload, Download, FileText, CheckCircle2, XCircle } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'

interface WslDistro {
  name: string
  is_default: boolean
  version: string
  state: string
}

interface CliStatus {
  [key: string]: boolean
}

const distros = ref<WslDistro[]>([])
const selectedDistro = ref<string | null>(null)
const isLoading = ref(false)
const isRefreshing = ref(false)
const isSyncing = ref(false)
const syncMessage = ref('')
const configContent = ref('')
const cliStatus = ref<CliStatus>({})
const selectedPlatform = ref('claude')

const platforms = ['claude', 'codex', 'gemini', 'qwen', 'iflow', 'droid']

const stateColor = (state: string) => {
  switch (state.toLowerCase()) {
    case 'running': return 'text-emerald-400'
    case 'stopped': return 'text-slate-400'
    default: return 'text-amber-400'
  }
}

const fetchDistros = async () => {
  isLoading.value = true
  try {
    distros.value = await invoke<WslDistro[]>('wsl_list_distros')
    if (distros.value.length > 0 && !selectedDistro.value) {
      selectedDistro.value = distros.value[0].name
      await loadDistroDetails()
    }
  } catch (e) {
    console.error('[WSL] Failed to list distros:', e)
  } finally {
    isLoading.value = false
  }
}

const loadDistroDetails = async () => {
  if (!selectedDistro.value) return

  try {
    // 检测 CLI 工具状态
    const status = await invoke<Record<string, boolean>>('wsl_detect_cli', {
      distro: selectedDistro.value
    })
    cliStatus.value = status
  } catch (e) {
    console.error('[WSL] Failed to detect CLI:', e)
  }

  await readConfig()
}

const readConfig = async () => {
  if (!selectedDistro.value) return

  try {
    configContent.value = await invoke<string>('wsl_read_config', {
      distro: selectedDistro.value,
      platform: selectedPlatform.value,
      path: ''
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
    const result = await invoke<string>('wsl_sync_config', {
      distro: selectedDistro.value,
      platform: selectedPlatform.value,
      direction
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

onMounted(fetchDistros)
</script>

<template>
  <div class="space-y-6">
    <!-- 标题 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="p-2 rounded-xl bg-orange-500/10">
          <Terminal class="w-6 h-6 text-orange-400" />
        </div>
        <div>
          <h1 class="text-xl font-bold text-text-primary">
            WSL 环境管理
          </h1>
          <p class="text-sm text-text-muted">
            管理 Windows Subsystem for Linux 发行版配置
          </p>
        </div>
      </div>
      <button
        class="flex items-center gap-2 px-4 py-2 rounded-lg bg-bg-surface border border-border-default text-text-secondary hover:text-text-primary hover:border-accent-primary/30 transition-colors text-sm"
        :disabled="isRefreshing"
        @click="refresh"
      >
        <RefreshCw
          class="w-4 h-4"
          :class="{ 'animate-spin': isRefreshing }"
        />
        刷新
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
      class="rounded-xl border border-border-default bg-bg-surface p-8 text-center"
    >
      <Terminal class="w-12 h-12 mx-auto text-text-muted mb-3" />
      <p class="text-text-secondary font-medium">
        未检测到 WSL 发行版
      </p>
      <p class="text-sm text-text-muted mt-1">
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
        <h2 class="text-xs font-bold uppercase tracking-wider text-text-muted px-1">
          发行版
        </h2>
        <div class="space-y-2">
          <button
            v-for="distro in distros"
            :key="distro.name"
            class="w-full flex items-center gap-3 p-3 rounded-xl border transition-colors text-left"
            :class="[
              selectedDistro === distro.name
                ? 'bg-accent-primary/10 border-accent-primary/30 text-accent-primary'
                : 'bg-bg-surface border-border-default text-text-secondary hover:text-text-primary hover:border-border-accent'
            ]"
            @click="selectDistro(distro.name)"
          >
            <Terminal class="w-5 h-5 flex-shrink-0" />
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
        <div class="rounded-xl border border-border-default bg-bg-surface p-4">
          <h3 class="text-sm font-semibold text-text-primary mb-3">
            AI CLI 工具状态
          </h3>
          <div class="grid grid-cols-3 gap-3">
            <div
              v-for="(installed, tool) in cliStatus"
              :key="tool"
              class="flex items-center gap-2 px-3 py-2 rounded-lg bg-bg-base text-sm"
            >
              <CheckCircle2
                v-if="installed"
                class="w-4 h-4 text-emerald-400"
              />
              <XCircle
                v-else
                class="w-4 h-4 text-slate-400"
              />
              <span :class="installed ? 'text-text-primary' : 'text-text-muted'">
                {{ tool }}
              </span>
            </div>
          </div>
        </div>

        <!-- 配置浏览 -->
        <div class="rounded-xl border border-border-default bg-bg-surface p-4">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-semibold text-text-primary flex items-center gap-2">
              <FileText class="w-4 h-4" />
              配置文件
            </h3>
            <select
              v-model="selectedPlatform"
              class="px-2 py-1 rounded-lg bg-bg-base border border-border-default text-xs text-text-secondary"
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
          <pre class="p-3 rounded-lg bg-bg-base text-xs text-text-secondary font-mono overflow-auto max-h-64 whitespace-pre-wrap">{{ configContent || '(空)' }}</pre>
        </div>

        <!-- 同步操作 -->
        <div class="rounded-xl border border-border-default bg-bg-surface p-4">
          <h3 class="text-sm font-semibold text-text-primary mb-3">
            配置同步
          </h3>
          <div class="flex items-center gap-3">
            <button
              class="flex items-center gap-2 px-4 py-2 rounded-lg bg-accent-primary/10 text-accent-primary text-sm font-medium hover:bg-accent-primary/20 transition-colors"
              :disabled="isSyncing"
              @click="syncConfig('local_to_wsl')"
            >
              <Upload class="w-4 h-4" />
              推送到 WSL
            </button>
            <button
              class="flex items-center gap-2 px-4 py-2 rounded-lg bg-bg-base border border-border-default text-text-secondary text-sm font-medium hover:text-text-primary hover:border-accent-primary/30 transition-colors"
              :disabled="isSyncing"
              @click="syncConfig('wsl_to_local')"
            >
              <Download class="w-4 h-4" />
              从 WSL 拉取
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
  </div>
</template>
