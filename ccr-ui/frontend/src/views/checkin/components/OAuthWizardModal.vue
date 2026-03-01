<template>
  <BaseModal
    v-model="isVisible"
    title="OAuth 引导登录"
    size="lg"
    :persistent="loading"
    @close="handleClose"
  >
    <!-- 步骤指示器 -->
    <div class="flex items-center justify-center mb-6 gap-2">
      <template
        v-for="(stepLabel, idx) in stepLabels"
        :key="idx"
      >
        <div
          class="flex items-center gap-1.5"
          :class="step > idx ? 'text-green-500' : step === idx ? 'text-blue-500' : 'text-zinc-400'"
        >
          <div
            class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold border-2 transition-colors"
            :class="
              step > idx
                ? 'bg-green-500/20 border-green-500 text-green-500'
                : step === idx
                  ? 'bg-blue-500/20 border-blue-500 text-blue-500'
                  : 'bg-zinc-800 border-zinc-600 text-zinc-500'
            "
          >
            <CheckCircle
              v-if="step > idx"
              class="w-4 h-4"
            />
            <span v-else>{{ idx + 1 }}</span>
          </div>
          <span class="text-xs hidden sm:inline">{{ stepLabel }}</span>
        </div>
        <ChevronRight
          v-if="idx < stepLabels.length - 1"
          class="w-4 h-4 text-zinc-600"
        />
      </template>
    </div>

    <!-- Step 0: 选择提供商和 OAuth 方式 -->
    <div
      v-if="step === 0"
      class="space-y-4"
    >
      <!-- 提供商选择 -->
      <div>
        <label class="block text-sm font-medium text-zinc-300 mb-2">选择提供商</label>
        <select
          v-model="selectedProviderId"
          class="w-full rounded-lg bg-zinc-800 border border-zinc-700 text-zinc-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option
            value=""
            disabled
          >
            请选择...
          </option>
          <option
            v-for="provider in oauthProviders"
            :key="provider.id"
            :value="provider.id"
          >
            {{ provider.icon }} {{ provider.name }} ({{ provider.domain }})
          </option>
        </select>
      </div>

      <!-- OAuth 方式选择 -->
      <div v-if="selectedProvider">
        <label class="block text-sm font-medium text-zinc-300 mb-2">选择登录方式</label>
        <div class="grid grid-cols-2 gap-3">
          <button
            v-if="selectedProvider.oauth_config?.linuxdo_client_id"
            :class="[
              'flex items-center gap-2 px-4 py-3 rounded-lg border text-sm font-medium transition-colors',
              selectedOAuthType === 'linuxdo'
                ? 'border-blue-500 bg-blue-500/10 text-blue-400'
                : 'border-zinc-700 bg-zinc-800 text-zinc-300 hover:border-zinc-500',
            ]"
            @click="selectedOAuthType = 'linuxdo'"
          >
            <Globe class="w-5 h-5" />
            LinuxDo
          </button>
          <button
            v-if="selectedProvider.oauth_config?.github_client_id"
            :class="[
              'flex items-center gap-2 px-4 py-3 rounded-lg border text-sm font-medium transition-colors',
              selectedOAuthType === 'github'
                ? 'border-blue-500 bg-blue-500/10 text-blue-400'
                : 'border-zinc-700 bg-zinc-800 text-zinc-300 hover:border-zinc-500',
            ]"
            @click="selectedOAuthType = 'github'"
          >
            <Github class="w-5 h-5" />
            GitHub
          </button>
        </div>
        <p
          v-if="!selectedProvider.oauth_config?.linuxdo_client_id && !selectedProvider.oauth_config?.github_client_id"
          class="text-amber-400 text-xs mt-2"
        >
          该提供商的 OAuth client_id 尚未配置
        </p>
      </div>
    </div>

    <!-- Step 1: 获取授权链接 -->
    <div
      v-else-if="step === 1"
      class="space-y-4"
    >
      <div
        v-if="loading"
        class="flex flex-col items-center gap-3 py-8"
      >
        <Loader2 class="w-8 h-8 animate-spin text-blue-500" />
        <p class="text-sm text-zinc-400">
          正在获取授权链接...
        </p>
      </div>

      <div
        v-else-if="oauthError"
        class="bg-red-500/10 border border-red-500/20 rounded-lg p-4"
      >
        <p class="text-red-400 text-sm">
          {{ oauthError }}
        </p>
        <button
          class="mt-2 text-xs text-blue-400 hover:underline"
          @click="step = 0"
        >
          返回重新选择
        </button>
      </div>

      <div
        v-else-if="authorizeUrl"
        class="space-y-4"
      >
        <div class="bg-blue-500/10 border border-blue-500/20 rounded-lg p-4">
          <p class="text-blue-300 text-sm font-medium mb-2">
            🔗 请在浏览器中打开以下链接完成授权：
          </p>
          <div class="flex items-center gap-2">
            <input
              :value="authorizeUrl"
              readonly
              class="flex-1 bg-zinc-900 border border-zinc-700 rounded px-3 py-1.5 text-xs text-zinc-300 font-mono"
            >
            <button
              class="shrink-0 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-xs rounded transition-colors"
              @click="copyUrl"
            >
              {{ copied ? '已复制' : '复制' }}
            </button>
          </div>
          <a
            :href="authorizeUrl"
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center gap-1 mt-2 text-sm text-blue-400 hover:text-blue-300 hover:underline"
          >
            <ExternalLink class="w-3.5 h-3.5" />
            在新标签页打开
          </a>
        </div>

        <!-- 引导说明 -->
        <div class="bg-zinc-800/50 rounded-lg p-4">
          <p class="text-sm font-medium text-zinc-300 mb-3">
            📋 操作步骤：
          </p>
          <ol class="space-y-2">
            <li
              v-for="(guide, idx) in extractionGuide"
              :key="idx"
              class="text-xs text-zinc-400 pl-4 relative before:absolute before:left-0 before:text-zinc-500 before:content-[attr(data-index)]"
              :data-index="(idx + 1) + '.'"
            >
              {{ guide }}
            </li>
          </ol>
        </div>
      </div>
    </div>

    <!-- Step 2: 粘贴 Cookies -->
    <div
      v-else-if="step === 2"
      class="space-y-4"
    >
      <div>
        <label class="block text-sm font-medium text-zinc-300 mb-2">
          粘贴 Cookies JSON 或 document.cookie 字符串
        </label>
        <textarea
          v-model="pastedCredentials"
          rows="6"
          placeholder="{&quot;session&quot;: &quot;xxx&quot;, &quot;token&quot;: &quot;yyy&quot;} 或 session=xxx; token=yyy"
          class="w-full rounded-lg bg-zinc-800 border border-zinc-700 text-zinc-200 px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
        />
      </div>

      <div>
        <label class="block text-sm font-medium text-zinc-300 mb-2">
          API User (可选，通常为数字 ID)
        </label>
        <input
          v-model="pastedApiUser"
          placeholder="从 localStorage 中获取，留空则自动获取"
          class="w-full rounded-lg bg-zinc-800 border border-zinc-700 text-zinc-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
      </div>

      <div>
        <label class="block text-sm font-medium text-zinc-300 mb-2">账号备注名称</label>
        <input
          v-model="accountName"
          :placeholder="`${selectedProvider?.name ?? ''} 账号`"
          class="w-full rounded-lg bg-zinc-800 border border-zinc-700 text-zinc-200 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
      </div>

      <div
        v-if="parseError"
        class="bg-red-500/10 border border-red-500/20 rounded-lg p-3"
      >
        <p class="text-red-400 text-xs">
          {{ parseError }}
        </p>
      </div>
    </div>

    <!-- Step 3: 确认创建 -->
    <div
      v-else-if="step === 3"
      class="space-y-4"
    >
      <div
        v-if="creatingAccount"
        class="flex flex-col items-center gap-3 py-8"
      >
        <Loader2 class="w-8 h-8 animate-spin text-blue-500" />
        <p class="text-sm text-zinc-400">
          正在创建账号...
        </p>
      </div>

      <div
        v-else-if="createSuccess"
        class="flex flex-col items-center gap-3 py-8"
      >
        <CheckCircle class="w-12 h-12 text-green-500" />
        <p class="text-green-400 font-medium">
          账号创建成功！
        </p>
        <p class="text-xs text-zinc-500">
          {{ selectedProvider?.name }} - {{ accountName || '新账号' }}
        </p>
      </div>

      <div
        v-else
        class="space-y-3"
      >
        <div class="bg-zinc-800/50 rounded-lg p-4 space-y-2">
          <div class="flex justify-between text-sm">
            <span class="text-zinc-400">提供商</span>
            <span class="text-zinc-200">{{ selectedProvider?.name }}</span>
          </div>
          <div class="flex justify-between text-sm">
            <span class="text-zinc-400">账号名称</span>
            <span class="text-zinc-200">{{ accountName || selectedProvider?.name + ' 账号' }}</span>
          </div>
          <div class="flex justify-between text-sm">
            <span class="text-zinc-400">Cookies 数量</span>
            <span class="text-zinc-200">{{ parsedCookieCount }} 个</span>
          </div>
          <div class="flex justify-between text-sm">
            <span class="text-zinc-400">API User</span>
            <span class="text-zinc-200">{{ pastedApiUser || '(未设置)' }}</span>
          </div>
        </div>

        <div
          v-if="createError"
          class="bg-red-500/10 border border-red-500/20 rounded-lg p-3"
        >
          <p class="text-red-400 text-xs">
            {{ createError }}
          </p>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-between w-full">
        <button
          v-if="step > 0 && !createSuccess"
          class="px-4 py-2 text-sm text-zinc-400 hover:text-zinc-200 transition-colors"
          :disabled="loading || creatingAccount"
          @click="step--"
        >
          上一步
        </button>
        <div v-else />

        <div class="flex gap-2">
          <button
            class="px-4 py-2 text-sm text-zinc-400 hover:text-zinc-200 border border-zinc-700 rounded-lg transition-colors"
            @click="handleClose"
          >
            {{ createSuccess ? '关闭' : '取消' }}
          </button>

          <button
            v-if="step === 0"
            :disabled="!canProceedStep0"
            class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
            @click="goToStep1"
          >
            获取授权链接
          </button>

          <button
            v-else-if="step === 1 && authorizeUrl"
            class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
            @click="step = 2"
          >
            我已完成授权
          </button>

          <button
            v-else-if="step === 2"
            :disabled="!pastedCredentials.trim()"
            class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
            @click="goToStep3"
          >
            下一步
          </button>

          <button
            v-else-if="step === 3 && !createSuccess"
            :disabled="creatingAccount"
            class="px-4 py-2 text-sm font-medium text-white bg-green-600 hover:bg-green-700 disabled:opacity-50 rounded-lg transition-colors"
            @click="createAccount"
          >
            确认创建
          </button>
        </div>
      </div>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  CheckCircle,
  ChevronRight,
  ExternalLink,
  Github,
  Globe,
  Loader2,
} from 'lucide-vue-next'
import BaseModal from '@/components/common/BaseModal.vue'
import { getOAuthAuthorizeUrl, createCheckinAccount } from '@/api/modules/checkin'
import type { BuiltinProvider } from '@/types/checkin'

const props = defineProps<{
  isOpen: boolean
  builtinProviders: BuiltinProvider[]
}>()

const emit = defineEmits<{
  (e: 'update:isOpen', value: boolean): void
  (e: 'close'): void
  (e: 'success'): void
}>()

const isVisible = computed({
  get: () => props.isOpen,
  set: (val: boolean) => emit('update:isOpen', val),
})

// Steps
const stepLabels = ['选择方式', '获取链接', '粘贴凭证', '确认创建']
const step = ref(0)

// Step 0 state
const selectedProviderId = ref('')
const selectedOAuthType = ref<'github' | 'linuxdo'>('linuxdo')

// Step 1 state
const loading = ref(false)
const oauthError = ref('')
const authorizeUrl = ref('')
const extractionGuide = ref<string[]>([])
const copied = ref(false)

// Step 2 state
const pastedCredentials = ref('')
const pastedApiUser = ref('')
const accountName = ref('')
const parseError = ref('')

// Step 3 state
const creatingAccount = ref(false)
const createSuccess = ref(false)
const createError = ref('')

// Computed
const oauthProviders = computed(() =>
  props.builtinProviders.filter((p) => p.oauth_config != null)
)

const selectedProvider = computed(() =>
  props.builtinProviders.find((p) => p.id === selectedProviderId.value)
)

const canProceedStep0 = computed(
  () => selectedProviderId.value && selectedOAuthType.value
)

const parsedCookieCount = computed(() => {
  try {
    const parsed = parseCookies(pastedCredentials.value)
    return Object.keys(parsed).length
  } catch {
    return 0
  }
})

// Reset when modal opens/closes
watch(
  () => props.isOpen,
  (open) => {
    if (open) {
      step.value = 0
      selectedProviderId.value = ''
      selectedOAuthType.value = 'linuxdo'
      loading.value = false
      oauthError.value = ''
      authorizeUrl.value = ''
      extractionGuide.value = []
      pastedCredentials.value = ''
      pastedApiUser.value = ''
      accountName.value = ''
      parseError.value = ''
      createSuccess.value = false
      createError.value = ''
    }
  }
)

// Auto-select first available OAuth type when provider changes
watch(selectedProviderId, () => {
  const provider = selectedProvider.value
  if (!provider?.oauth_config) return
  if (provider.oauth_config.linuxdo_client_id) {
    selectedOAuthType.value = 'linuxdo'
  } else if (provider.oauth_config.github_client_id) {
    selectedOAuthType.value = 'github'
  }
})

// Methods
function handleClose() {
  emit('close')
  emit('update:isOpen', false)
}

async function goToStep1() {
  step.value = 1
  loading.value = true
  oauthError.value = ''
  authorizeUrl.value = ''

  try {
    const response = await getOAuthAuthorizeUrl({
      provider_id: selectedProviderId.value,
      oauth_type: selectedOAuthType.value,
    })

    if (response.success && response.authorize_url) {
      authorizeUrl.value = response.authorize_url
      extractionGuide.value = response.extraction_guide
    } else {
      oauthError.value = response.message || '获取授权链接失败'
    }
  } catch (err: unknown) {
    oauthError.value = err instanceof Error ? err.message : '网络请求失败'
  } finally {
    loading.value = false
  }
}

async function copyUrl() {
  try {
    await navigator.clipboard.writeText(authorizeUrl.value)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch {
    // fallback
  }
}

function parseCookies(input: string): Record<string, string> {
  const trimmed = input.trim()

  // 尝试 JSON 解析
  try {
    const json = JSON.parse(trimmed)

    // 格式1: { cookies: "a=b; c=d", api_user: "123" }
    if (json.cookies && typeof json.cookies === 'string') {
      if (json.api_user) pastedApiUser.value = String(json.api_user)
      return parseCookieString(json.cookies)
    }

    // 格式2: { "session": "abc", "token": "xyz" }
    if (typeof json === 'object' && !Array.isArray(json)) {
      return json as Record<string, string>
    }
  } catch {
    // not JSON
  }

  // 格式3: cookie string "a=b; c=d"
  if (trimmed.includes('=')) {
    return parseCookieString(trimmed)
  }

  throw new Error('无法识别的格式')
}

function parseCookieString(str: string): Record<string, string> {
  const cookies: Record<string, string> = {}
  for (const part of str.split(';')) {
    const eqIdx = part.indexOf('=')
    if (eqIdx > 0) {
      const key = part.substring(0, eqIdx).trim()
      const value = part.substring(eqIdx + 1).trim()
      cookies[key] = value
    }
  }
  return cookies
}

function goToStep3() {
  parseError.value = ''
  try {
    const cookies = parseCookies(pastedCredentials.value)
    if (Object.keys(cookies).length === 0) {
      parseError.value = 'Cookies 为空，请检查输入格式'
      return
    }
    step.value = 3
  } catch (err: unknown) {
    parseError.value = err instanceof Error ? err.message : '解析失败'
  }
}

async function createAccount() {
  creatingAccount.value = true
  createError.value = ''

  try {
    const provider = selectedProvider.value
    if (!provider) throw new Error('未选择提供商')

    const cookies = parseCookies(pastedCredentials.value)
    const cookiesJson = JSON.stringify(cookies)

    await createCheckinAccount({
      provider_id: provider.id.replace('builtin-', ''),
      name: accountName.value || `${provider.name} 账号`,
      cookies_json: cookiesJson,
      api_user: pastedApiUser.value || '',
    })

    createSuccess.value = true
    emit('success')
  } catch (err: unknown) {
    createError.value = err instanceof Error ? err.message : '创建失败'
  } finally {
    creatingAccount.value = false
  }
}
</script>
