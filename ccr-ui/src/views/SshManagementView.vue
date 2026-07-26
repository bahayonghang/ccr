<!-- -->
<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  refreshEnvironments,
  sshAddHost,
  sshConnect,
  sshConfirmHostFingerprint,
  sshDetectCli,
  sshDisconnect,
  sshGetConnectionState,
  sshListHosts,
  sshListKeys,
  sshProbeHostFingerprint,
  sshReadConfig,
  sshReconnect,
  sshTestConnection,
  sshWriteConfig,
  type SshConnectResult,
  type SshConnectionState,
  type SshFingerprintProbeResult,
  type SshHostConfig,
  type SshKeyInfo,
} from '@/api'

const { locale } = useI18n()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)

// SSH 密钥列表，待密钥管理 UI 实装时使用
const discoveredKeys = ref<SshKeyInfo[]>([])

const hosts = ref<SshHostConfig[]>([])
const loading = ref(false)
const error = ref('')
const activeEnvId = ref('')
const activeConnectionState = ref<SshConnectionState | null>(null)
const pendingFingerprint = ref<SshFingerprintProbeResult | null>(null)
const connectPassword = ref('')
const cliStatusText = ref('')
const configContent = ref('')
/** 每个主机的连通性测试结果，key 为 envId。 */
const testResults = ref<Record<string, SshConnectResult>>({})
/** 正在测试中的主机 envId 集合。 */
const testingHosts = ref<Set<string>>(new Set())
const form = ref<SshHostConfig>({
  host: '',
  port: 22,
  user: '',
  name: '',
  identity_file: '',
})
const platform = ref('claude')
const configPath = ref('settings.json')

const selectedHost = computed(() => {
  return hosts.value.find((host) => buildEnvId(host) === activeEnvId.value)
})
const selectedHostLabel = computed(() => (
  selectedHost.value
    ? `${selectedHost.value.user || 'user'}@${selectedHost.value.host}`
    : tt('未连接', 'Not connected')
))
const platformOptions = computed(() => ([
  { value: 'claude', label: 'claude' },
  { value: 'codex', label: 'codex' },
  { value: 'gemini', label: 'gemini' },
  { value: 'opencode', label: 'opencode' },
]))

function buildEnvId(host: SshHostConfig): string {
  return `ssh:${host.id?.trim() || host.host}`
}

async function loadHosts() {
  loading.value = true
  error.value = ''
  try {
    hosts.value = await sshListHosts()
  } catch (e: unknown) {
    error.value = e?.toString?.() || tt('加载 SSH 主机失败', 'Failed to load SSH hosts')
  } finally {
    loading.value = false
  }
}

async function addHost() {
  error.value = ''
  try {
    if (!form.value.host?.trim()) {
      throw new Error(tt('主机地址不能为空', 'Host is required'))
    }
    await sshAddHost({
      id: form.value.id?.trim() || undefined,
      name: form.value.name?.trim() || undefined,
      host: form.value.host.trim(),
      port: Number(form.value.port) || 22,
      user: form.value.user?.trim() || undefined,
      identity_file: form.value.identity_file?.trim() || undefined,
      remote_home: form.value.remote_home?.trim() || undefined,
    })
    await refreshEnvironments()
    await loadHosts()
    form.value = { host: '', port: 22, user: '', name: '', identity_file: '' }
  } catch (e: unknown) {
    error.value = e?.toString?.() || tt('新增 SSH 主机失败', 'Failed to add SSH host')
  }
}

async function connectHost(host: SshHostConfig) {
  error.value = ''
  pendingFingerprint.value = null
  const envId = buildEnvId(host)
  try {
    const probe = await sshProbeHostFingerprint(envId)
    if (probe.status === 'mismatch' || probe.status === 'new') {
      pendingFingerprint.value = probe
      activeEnvId.value = envId
      return
    }

    activeConnectionState.value = await sshConnect(envId, connectPassword.value || undefined)
    activeEnvId.value = envId
    cliStatusText.value = ''
    configContent.value = ''
  } catch (e: unknown) {
    error.value = e?.toString?.() || tt('连接 SSH 主机失败', 'Failed to connect to SSH host')
  }
}

async function confirmFingerprintAndConnect() {
  if (!pendingFingerprint.value || !activeEnvId.value) return
  error.value = ''
  try {
    await sshConfirmHostFingerprint(pendingFingerprint.value.challenge_id)
    activeConnectionState.value = await sshConnect(activeEnvId.value, connectPassword.value || undefined)
    pendingFingerprint.value = null
  } catch (e: unknown) {
    error.value = e?.toString?.() || tt('确认指纹失败', 'Failed to confirm fingerprint')
  }
}

async function reconnectHost() {
  if (!activeEnvId.value) return
  error.value = ''
  try {
    activeConnectionState.value = await sshReconnect(activeEnvId.value, connectPassword.value || undefined)
  } catch (e: unknown) {
    error.value = e?.toString?.() || tt('重连失败', 'Reconnect failed')
  }
}

async function refreshConnectionState() {
  if (!activeEnvId.value) return
  try {
    const state = await sshGetConnectionState(activeEnvId.value)
    if (!Array.isArray(state)) {
      activeConnectionState.value = state
    }
  } catch {
    // 连接状态刷新失败不阻断主流程
  }
}

async function disconnectHost() {
  error.value = ''
  try {
    activeConnectionState.value = await sshDisconnect()
    activeEnvId.value = ''
    pendingFingerprint.value = null
  } catch (e: unknown) {
    error.value = e?.toString?.() || tt('断开连接失败', 'Failed to disconnect')
  }
}

async function detectCli() {
  if (!activeEnvId.value) return
  error.value = ''
  try {
    const data = await sshDetectCli(activeEnvId.value)
    cliStatusText.value = JSON.stringify(data, null, 2)
  } catch (e: unknown) {
    error.value = e?.toString?.() || tt('CLI 检测失败', 'CLI detection failed')
  }
}

async function readConfig() {
  if (!activeEnvId.value) return
  error.value = ''
  try {
    configContent.value = await sshReadConfig(activeEnvId.value, platform.value, configPath.value)
  } catch (e: unknown) {
    error.value = e?.toString?.() || tt('读取配置失败', 'Failed to read config')
  }
}

async function writeConfig() {
  if (!activeEnvId.value) return
  error.value = ''
  try {
    await sshWriteConfig(activeEnvId.value, platform.value, configPath.value, configContent.value, true)
  } catch (e: unknown) {
    error.value = e?.toString?.() || tt('写入配置失败', 'Failed to write config')
  }
}

async function testConnect(host: SshHostConfig) {
  const envId = buildEnvId(host)
  testingHosts.value = new Set([...testingHosts.value, envId])
  try {
    const result = await sshTestConnection(envId)
    testResults.value = { ...testResults.value, [envId]: result }
  } catch (e: unknown) {
    testResults.value = {
      ...testResults.value,
      [envId]: { success: false, latency_ms: 0, error: e?.toString?.() || tt('测试失败', 'Test failed') },
    }
  } finally {
    const next = new Set(testingHosts.value)
    next.delete(envId)
    testingHosts.value = next
  }
}

function formatTestResult(result: SshConnectResult) {
  if (result.success) {
    return `${tt('连通', 'Reachable')} (${result.latency_ms} ms)`
  }
  if (result.error) {
    return `${tt('失败', 'Failed')}: ${result.error}`
  }
  return tt('失败', 'Failed')
}

function formatConnectionState(state: SshConnectionState) {
  return `${tt('连接状态', 'Connection state')}: ${state.connected ? tt('已连接', 'Connected') : tt('未连接', 'Not connected')}`
}

function formatConnectionCheckedAt(value: string) {
  return `(${value})`
}

function formatConnectionError(value: string) {
  return `${tt('错误', 'Error')}: ${value}`
}

onMounted(async () => {
  await loadHosts()
  await refreshConnectionState()
})

// 预留 SSH 密钥管理接口，待 UI 实装时使用
defineExpose({ sshListKeys, discoveredKeys })
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold text-white">
          {{ tt('SSH 远程管理', 'SSH Remote Management') }}
        </h1>
        <p class="text-sm text-text-muted">
          {{ tt('添加主机并连接后执行配置读写和 CLI 检测', 'Add a host, connect, then run config read/write and CLI checks.') }}
        </p>
      </div>
      <button
        class="px-3 py-2 rounded-lg border border-border-default/15"
        :disabled="loading"
        @click="loadHosts"
      >
        {{ tt('刷新主机', 'Refresh hosts') }}
      </button>
    </div>

    <div
      v-if="error"
      class="rounded-lg border border-red-500/30 bg-red-500/10 p-3 text-sm text-red-300"
    >
      {{ error }}
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <section class="rounded-xl border border-border-default/15 glass-surface p-4 space-y-3">
        <h2 class="text-base font-semibold text-white">
          {{ tt('新增 SSH 主机', 'Add SSH host') }}
        </h2>
        <div class="grid grid-cols-2 gap-3">
          <input
            v-model="form.name"
            class="rounded-md border border-border-default/15 px-3 py-2"
            placeholder="名称（可选）"
          >
          <input
            v-model="form.id"
            class="rounded-md border border-border-default/15 px-3 py-2"
            placeholder="ID（可选）"
          >
          <input
            v-model="form.host"
            class="rounded-md border border-border-default/15 px-3 py-2 col-span-2"
            placeholder="主机地址"
          >
          <input
            v-model.number="form.port"
            type="number"
            class="rounded-md border border-border-default/15 px-3 py-2"
            placeholder="端口"
          >
          <input
            v-model="form.user"
            class="rounded-md border border-border-default/15 px-3 py-2"
            placeholder="用户名"
          >
          <input
            v-model="connectPassword"
            type="password"
            class="rounded-md border border-border-default/15 px-3 py-2 col-span-2"
            placeholder="密码（仅内存，可选）"
          >
          <input
            v-model="form.identity_file"
            class="rounded-md border border-border-default/15 px-3 py-2 col-span-2"
            placeholder="私钥路径（可选）"
          >
        </div>
        <button
          class="px-3 py-2 rounded-lg bg-sky-500/20 text-sky-300"
          @click="addHost"
        >
          {{ tt('添加主机', 'Add host') }}
        </button>
      </section>

      <section class="rounded-xl border border-border-default/15 glass-surface p-4 space-y-3">
        <h2 class="text-base font-semibold text-white">
          {{ tt('主机列表', 'Host list') }}
        </h2>
        <div
          v-if="hosts.length === 0"
          class="text-sm text-text-muted"
        >
          {{ tt('暂无 SSH 主机', 'No SSH hosts yet') }}
        </div>
        <div
          v-for="host in hosts"
          :key="buildEnvId(host)"
          class="rounded-lg border border-border-default/15 p-3"
        >
          <div class="flex items-center justify-between gap-3">
            <div>
              <div class="text-sm font-medium text-white">
                {{ host.name || host.host }}
              </div>
              <div class="text-xs text-text-muted">
                {{ host.user || 'user' }}@{{ host.host }}:{{ host.port || 22 }}
              </div>
            </div>
            <div class="flex items-center gap-1">
              <button
                class="px-2 py-1 rounded border border-border-default/15 text-xs"
                :disabled="testingHosts.has(buildEnvId(host))"
                @click="testConnect(host)"
              >
                {{ testingHosts.has(buildEnvId(host)) ? tt('测试中…', 'Testing...') : tt('测试连接', 'Test connection') }}
              </button>
              <button
                class="px-2 py-1 rounded border border-border-default/15 text-xs"
                @click="connectHost(host)"
              >
                {{ tt('连接', 'Connect') }}
              </button>
            </div>
          </div>
          <div
            v-if="testResults[buildEnvId(host)]"
            class="mt-2 text-xs"
          >
            <span
              :class="testResults[buildEnvId(host)].success ? 'text-green-400' : 'text-red-400'"
            >
              {{ formatTestResult(testResults[buildEnvId(host)]) }}
            </span>
          </div>
        </div>
      </section>
    </div>

    <section class="rounded-xl border border-border-default/15 glass-surface p-4 space-y-3">
      <div class="flex items-center justify-between gap-3">
        <h2 class="text-base font-semibold text-white">
          {{ tt('已连接主机', 'Connected host') }}
        </h2>
        <div class="flex items-center gap-2">
          <button
            class="px-2 py-1 rounded border border-border-default/15 text-xs"
            :disabled="!activeEnvId"
            @click="refreshConnectionState"
          >
            {{ tt('刷新状态', 'Refresh state') }}
          </button>
          <button
            class="px-2 py-1 rounded border border-border-default/15 text-xs"
            :disabled="!activeEnvId"
            @click="reconnectHost"
          >
            {{ tt('重连', 'Reconnect') }}
          </button>
          <button
            class="px-2 py-1 rounded border border-border-default/15 text-xs"
            :disabled="!activeEnvId"
            @click="disconnectHost"
          >
            {{ tt('断开', 'Disconnect') }}
          </button>
        </div>
      </div>
      <div class="text-sm text-text-muted">
        {{ selectedHostLabel }}
      </div>
      <div
        v-if="activeConnectionState"
        class="rounded-md border border-border-default/15 p-2 text-xs text-text-muted"
      >
        {{ formatConnectionState(activeConnectionState) }}
        <span v-if="activeConnectionState.last_checked_at">{{ ` ${formatConnectionCheckedAt(activeConnectionState.last_checked_at)}` }}</span>
        <span
          v-if="activeConnectionState.last_error"
          class="text-red-300"
        >{{ ` ${formatConnectionError(activeConnectionState.last_error)}` }}</span>
      </div>
      <div
        v-if="pendingFingerprint"
        class="rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-200 space-y-2"
      >
        <div>{{ tt('检测到主机指纹需要确认：', 'Host fingerprint confirmation required:') }}</div>
        <div class="text-xs font-mono">
          {{ pendingFingerprint.key_type }} {{ pendingFingerprint.fingerprint }}
        </div>
        <button
          class="px-2 py-1 rounded border border-amber-400/60 text-xs"
          @click="confirmFingerprintAndConnect"
        >
          {{ tt('确认并连接', 'Confirm and connect') }}
        </button>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <select
          v-model="platform"
          class="rounded-md border border-border-default/15 px-2 py-1 text-sm"
        >
          <option
            v-for="item in platformOptions"
            :key="item.value"
            :value="item.value"
          >
            {{ item.label }}
          </option>
        </select>
        <input
          v-model="configPath"
          class="rounded-md border border-border-default/15 px-2 py-1 text-sm min-w-[220px]"
          placeholder="配置路径"
        >
        <button
          class="px-2 py-1 rounded border border-border-default/15 text-xs"
          :disabled="!activeEnvId"
          @click="readConfig"
        >
          {{ tt('读取配置', 'Read config') }}
        </button>
        <button
          class="px-2 py-1 rounded border border-border-default/15 text-xs"
          :disabled="!activeEnvId"
          @click="writeConfig"
        >
          {{ tt('写入配置', 'Write config') }}
        </button>
        <button
          class="px-2 py-1 rounded border border-border-default/15 text-xs"
          :disabled="!activeEnvId"
          @click="detectCli"
        >
          {{ tt('检测 CLI', 'Detect CLI') }}
        </button>
      </div>

      <textarea
        v-model="configContent"
        class="w-full min-h-[220px] rounded-md border border-border-default/15 p-3 text-xs font-mono"
        placeholder="读取后的配置内容会显示在这里"
      />

      <pre
        v-if="cliStatusText"
        class="rounded-md border border-border-default/15 p-3 text-xs overflow-x-auto"
      >{{ cliStatusText }}</pre>
    </section>
  </div>
</template>
