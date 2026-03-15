<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <AnimatedBackground variant="minimal" />

    <div class="max-w-5xl mx-auto space-y-5">
      <!-- 页面标题 -->
      <div class="flex items-center justify-between animate-slide-up">
        <div class="flex items-center gap-3">
          <RouterLink
            to="/opencode"
            class="p-2 rounded-lg text-white/50 hover:text-white transition-colors"
          >
            <SIcon
              name="ChevronLeft"
              size="w-5 h-5"
            />
          </RouterLink>
          <div>
            <h1 class="text-2xl font-bold text-white">
              Provider 管理
            </h1>
            <p class="text-white/50 text-sm">
              管理 OpenCode npm AI SDK Provider 配置
            </p>
          </div>
        </div>
        <button
          class="flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-transform hover:scale-105"
          style="background: var(--accent-primary); color: white;"
          @click="showAddDialog = true"
        >
          <SIcon
            name="Plus"
            size="w-4 h-4"
          />
          添加 Provider
        </button>
      </div>

      <!-- 加载状态 -->
      <div
        v-if="loading"
        class="flex items-center justify-center py-16"
      >
        <div class="w-8 h-8 border-2 border-violet-500 border-t-transparent rounded-full animate-spin" />
      </div>

      <!-- 错误状态 -->
      <Card
        v-else-if="error"
        variant="elevated"
        class="p-6 text-center"
      >
        <p class="text-red-400 mb-3">
          {{ error }}
        </p>
        <button
          class="text-sm text-accent-primary hover:underline"
          @click="loadProviders"
        >
          重新加载
        </button>
      </Card>

      <!-- 空状态 -->
      <Card
        v-else-if="providers.length === 0"
        variant="glass"
        class="p-10 text-center"
      >
        <SIcon
          name="Layers"
          size="w-12 h-12"
          class="text-white/50 mx-auto mb-4"
        />
        <h3 class="text-lg font-bold text-white mb-2">
          暂无 Provider
        </h3>
        <p class="text-white/50 text-sm mb-4">
          添加 npm AI SDK Provider 来开始使用 OpenCode
        </p>
        <button
          class="px-4 py-2 rounded-lg font-medium text-sm transition-transform hover:scale-105"
          style="background: var(--accent-primary); color: white;"
          @click="showAddDialog = true"
        >
          添加第一个 Provider
        </button>
      </Card>

      <!-- Provider 列表 -->
      <div
        v-else
        class="space-y-3"
      >
        <Card
          v-for="provider in providers"
          :key="provider.id"
          variant="elevated"
          class="p-4 animate-slide-up"
        >
          <div class="flex items-start justify-between gap-4">
            <div class="flex items-start gap-3 min-w-0">
              <!-- Provider 图标 -->
              <div class="w-10 h-10 rounded-lg bg-violet-500/10 flex items-center justify-center shrink-0">
                <SIcon
                  name="Layers"
                  size="w-5 h-5"
                  class="text-violet-500"
                />
              </div>

              <!-- Provider 信息 -->
              <div class="min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <h3 class="font-bold text-white truncate">
                    {{ provider.id }}
                  </h3>
                  <span class="px-2 py-0.5 rounded text-xs bg-violet-500/10 text-violet-400 font-mono shrink-0">
                    {{ provider.npm }}
                  </span>
                </div>

                <div class="flex flex-wrap gap-3 text-xs text-white/50">
                  <span
                    v-if="provider.options?.apiKey"
                    class="flex items-center gap-1"
                  >
                    <SIcon
                      name="Key"
                      size="w-3 h-3"
                    />
                    {{ maskApiKey(provider.options.apiKey) }}
                  </span>
                  <span
                    v-if="provider.options?.baseURL"
                    class="flex items-center gap-1"
                  >
                    <SIcon
                      name="Globe"
                      size="w-3 h-3"
                    />
                    {{ provider.options.baseURL }}
                  </span>
                  <span class="flex items-center gap-1">
                    <SIcon
                      name="Cpu"
                      size="w-3 h-3"
                    />
                    {{ Object.keys(provider.models || {}).length }} 个模型
                  </span>
                </div>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="flex items-center gap-2 shrink-0">
              <button
                class="p-2 rounded-lg text-white/50 hover:text-blue-400 hover:bg-blue-500/10 transition-colors"
                title="编辑"
                @click="editProvider(provider)"
              >
                <SIcon
                  name="Pencil"
                  size="w-4 h-4"
                />
              </button>
              <button
                class="p-2 rounded-lg text-white/50 hover:text-red-400 hover:bg-red-500/10 transition-colors"
                title="删除"
                @click="confirmDelete(provider)"
              >
                <SIcon
                  name="Trash2"
                  size="w-4 h-4"
                />
              </button>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <!-- 添加/编辑 Provider 弹窗 -->
    <div
      v-if="showAddDialog || editingProvider"
      class="fixed inset-0 flex items-center justify-center z-50 p-4"
      style="background: rgb(0 0 0 / 50%); backdrop-filter: blur(4px);"
      @click.self="closeDialog"
    >
      <Card
        variant="glass"
        class="w-full max-w-lg p-6 space-y-4 max-h-[90vh] overflow-y-auto"
      >
        <div class="flex items-center justify-between">
          <h2 class="text-lg font-bold text-white">
            {{ editingProvider ? '编辑 Provider' : '添加 Provider' }}
          </h2>
          <button
            class="p-1 rounded text-white/50 hover:text-white transition-colors"
            @click="closeDialog"
          >
            <SIcon
              name="X"
              size="w-5 h-5"
            />
          </button>
        </div>

        <!-- Provider ID -->
        <div>
          <label class="block text-xs font-bold text-white/50 uppercase tracking-wider mb-1">Provider ID *</label>
          <input
            v-model="form.id"
            :disabled="!!editingProvider"
            type="text"
            placeholder="例：my-claude"
            class="w-full px-3 py-2 rounded-lg text-sm glass-surface border border-white/20 text-white placeholder:text-white/50 focus:outline-none focus:border-violet-500 disabled:opacity-50"
          >
        </div>

        <!-- npm 包名（预设选择） -->
        <div>
          <label class="block text-xs font-bold text-white/50 uppercase tracking-wider mb-1">npm 包 *</label>
          <div class="grid grid-cols-2 gap-2 mb-2">
            <button
              v-for="preset in OPENCODE_PROVIDER_PRESETS"
              :key="preset.npm"
              class="px-3 py-2 rounded-lg text-xs text-left transition-colors border"
              :class="form.npm === preset.npm
                ? 'bg-violet-500/20 border-violet-500 text-violet-400'
                : 'glass-surface border-white/20 text-white/50 hover:border-violet-500/50'"
              @click="selectPreset(preset)"
            >
              <div class="font-bold truncate">
                {{ preset.label }}
              </div>
              <div class="opacity-70 truncate font-mono text-xs">
                {{ preset.npm }}
              </div>
            </button>
          </div>
          <input
            v-model="form.npm"
            type="text"
            placeholder="或输入自定义 npm 包名"
            class="w-full px-3 py-2 rounded-lg text-sm glass-surface border border-white/20 text-white placeholder:text-white/50 focus:outline-none focus:border-violet-500"
          >
        </div>

        <!-- API Key -->
        <div>
          <label class="block text-xs font-bold text-white/50 uppercase tracking-wider mb-1">API Key</label>
          <input
            v-model="form.apiKey"
            type="password"
            placeholder="sk-... 或 {env:VAR_NAME}"
            class="w-full px-3 py-2 rounded-lg text-sm glass-surface border border-white/20 text-white placeholder:text-white/50 focus:outline-none focus:border-violet-500"
          >
          <p class="text-xs text-white/50 mt-1">
            支持环境变量格式：{env:MY_API_KEY}
          </p>
        </div>

        <!-- Base URL -->
        <div>
          <label class="block text-xs font-bold text-white/50 uppercase tracking-wider mb-1">Base URL（可选）</label>
          <input
            v-model="form.baseURL"
            type="text"
            placeholder="https://api.example.com/v1"
            class="w-full px-3 py-2 rounded-lg text-sm glass-surface border border-white/20 text-white placeholder:text-white/50 focus:outline-none focus:border-violet-500"
          >
        </div>

        <!-- 操作按钮 -->
        <div class="flex justify-end gap-3 pt-2">
          <button
            class="px-4 py-2 rounded-lg text-sm text-white/50 hover:text-white transition-colors"
            @click="closeDialog"
          >
            取消
          </button>
          <button
            :disabled="!form.id || !form.npm || saving"
            class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-transform hover:scale-105 disabled:opacity-50 disabled:hover:scale-100"
            style="background: var(--accent-primary); color: white;"
            @click="saveProvider"
          >
            <SIcon
              v-if="saving"
              name="Loader2"
              size="w-4 h-4"
              class="animate-spin"
            />
            {{ editingProvider ? '更新' : '添加' }}
          </button>
        </div>
      </Card>
    </div>

    <!-- 删除确认弹窗 -->
    <div
      v-if="deletingProvider"
      class="fixed inset-0 flex items-center justify-center z-50 p-4"
      style="background: rgb(0 0 0 / 50%); backdrop-filter: blur(4px);"
      @click.self="deletingProvider = null"
    >
      <Card
        variant="glass"
        class="w-full max-w-sm p-6 space-y-4"
      >
        <h2 class="text-lg font-bold text-white">
          确认删除
        </h2>
        <p class="text-white/80 text-sm">
          确定要删除 Provider <strong>{{ deletingProvider.id }}</strong>（{{ deletingProvider.npm }}）吗？此操作无法撤销。
        </p>
        <div class="flex justify-end gap-3">
          <button
            class="px-4 py-2 rounded-lg text-sm text-white/50 hover:text-white"
            @click="deletingProvider = null"
          >
            取消
          </button>
          <button
            :disabled="saving"
            class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-red-500 text-white hover:bg-red-600 transition-colors disabled:opacity-50"
            @click="doDelete"
          >
            <SIcon
              v-if="saving"
              name="Loader2"
              size="w-4 h-4"
              class="animate-spin"
            />
            删除
          </button>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted, reactive } from 'vue'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import {
  listOpenCodeProviders,
  addOpenCodeProvider,
  updateOpenCodeProvider,
  deleteOpenCodeProvider,
} from '@/api'
import { OPENCODE_PROVIDER_PRESETS } from '@/types/opencode'
import type { OpenCodeProvider, OpenCodeProviderPreset } from '@/types/opencode'

const providers = ref<OpenCodeProvider[]>([])
const loading = ref(true)
const error = ref('')
const saving = ref(false)
const showAddDialog = ref(false)
const editingProvider = ref<OpenCodeProvider | null>(null)
const deletingProvider = ref<OpenCodeProvider | null>(null)

const form = reactive({
  id: '',
  npm: '',
  apiKey: '',
  baseURL: '',
})

const maskApiKey = (key: string) => {
  if (!key || key.startsWith('{env:')) return key
  if (key.length <= 8) return '••••••••'
  return key.slice(0, 4) + '••••••••' + key.slice(-4)
}

const loadProviders = async () => {
  loading.value = true
  error.value = ''
  try {
    providers.value = await listOpenCodeProviders()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '加载 Provider 列表失败'
  } finally {
    loading.value = false
  }
}

const selectPreset = (preset: OpenCodeProviderPreset) => {
  form.npm = preset.npm
  if (!form.id) {
    form.id = preset.id
  }
}

const editProvider = (provider: OpenCodeProvider) => {
  editingProvider.value = provider
  form.id = provider.id
  form.npm = provider.npm
  form.apiKey = provider.options?.apiKey || ''
  form.baseURL = provider.options?.baseURL || ''
}

const confirmDelete = (provider: OpenCodeProvider) => {
  deletingProvider.value = provider
}

const closeDialog = () => {
  showAddDialog.value = false
  editingProvider.value = null
  form.id = ''
  form.npm = ''
  form.apiKey = ''
  form.baseURL = ''
}

const saveProvider = async () => {
  if (!form.id || !form.npm) return
  saving.value = true
  try {
    const request = {
      id: form.id,
      npm: form.npm,
      options: {
        apiKey: form.apiKey || undefined,
        baseURL: form.baseURL || undefined,
      },
      models: {},
    }
    if (editingProvider.value) {
      await updateOpenCodeProvider(form.id, request)
    } else {
      await addOpenCodeProvider(request)
    }
    closeDialog()
    await loadProviders()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '保存失败'
  } finally {
    saving.value = false
  }
}

const doDelete = async () => {
  if (!deletingProvider.value) return
  saving.value = true
  try {
    await deleteOpenCodeProvider(deletingProvider.value.id)
    deletingProvider.value = null
    await loadProviders()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '删除失败'
  } finally {
    saving.value = false
  }
}

onMounted(loadProviders)
</script>
