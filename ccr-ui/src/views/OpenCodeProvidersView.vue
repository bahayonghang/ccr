<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <AnimatedBackground
      contained
      variant="minimal"
    />

    <div class="relative z-10 mx-auto max-w-5xl space-y-5">
      <!-- 页面标题 -->
      <div class="flex items-center justify-between animate-slide-up">
        <div class="flex items-center gap-3">
          <RouterLink
            to="/opencode"
            class="inline-flex"
          >
            <Button
              variant="ghost"
              surface="status"
              density="compact"
              motion="subtle"
            >
              <template #leading>
                <SIcon
                  name="ChevronLeft"
                  size="w-5 h-5"
                />
              </template>
            </Button>
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
        <Button
          variant="primary"
          surface="card"
          density="compact"
          motion="standard"
          @click="showAddDialog = true"
        >
          <template #leading>
            <SIcon
              name="Plus"
              size="w-4 h-4"
            />
          </template>
          添加 Provider
        </Button>
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
        surface="card"
        :elevation="2"
        motion="subtle"
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
        surface="workspace"
        :elevation="2"
        motion="subtle"
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
        <Button
          variant="primary"
          surface="card"
          density="compact"
          motion="standard"
          @click="showAddDialog = true"
        >
          添加第一个 Provider
        </Button>
      </Card>

      <!-- Provider 列表 -->
      <div
        v-else
        class="space-y-3"
      >
        <Card
          v-for="provider in providers"
          :key="provider.id"
          surface="card"
          :elevation="2"
          motion="subtle"
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
    <BaseModal
      :model-value="showAddDialog || Boolean(editingProvider)"
      :title="editingProvider ? '编辑 Provider' : '添加 Provider'"
      description="创建或编辑 OpenCode npm AI SDK Provider 配置。"
      size="lg"
      surface="solid"
      content-class="max-w-lg"
      @update:model-value="(value) => !value && closeDialog()"
    >
      <div class="space-y-4 max-h-[90vh] overflow-y-auto">
        <div class="flex items-center justify-between">
          <h2 class="text-lg font-bold text-text-primary">
            {{ editingProvider ? '编辑 Provider' : '添加 Provider' }}
          </h2>
          <Button
            variant="ghost"
            surface="status"
            density="compact"
            motion="subtle"
            @click="closeDialog"
          >
            <template #leading>
              <SIcon
                name="X"
                size="w-5 h-5"
              />
            </template>
          </Button>
        </div>

        <!-- Provider ID -->
        <div>
          <label class="block text-xs font-bold text-white/50 uppercase tracking-wider mb-1">Provider ID *</label>
          <input
            v-model="form.id"
            :disabled="!!editingProvider"
            type="text"
            placeholder="例：my-claude"
            class="opencode-provider-input w-full px-3 py-2 rounded-lg text-sm"
          >
        </div>

        <!-- npm 包名（预设选择） -->
        <div>
          <label class="block text-xs font-bold text-white/50 uppercase tracking-wider mb-1">npm 包 *</label>
          <div class="grid grid-cols-2 gap-2 mb-2">
            <button
              v-for="preset in OPENCODE_PROVIDER_PRESETS"
              :key="preset.npm"
              class="opencode-provider-preset px-3 py-2 rounded-lg text-left text-xs border"
              :class="form.npm === preset.npm
                ? 'bg-violet-500/20 border-violet-500 text-violet-400'
                : 'text-text-secondary hover:border-violet-500/50'"
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
            class="opencode-provider-input w-full px-3 py-2 rounded-lg text-sm"
          >
        </div>

        <!-- API Key -->
        <div>
          <label class="block text-xs font-bold text-white/50 uppercase tracking-wider mb-1">API Key</label>
          <input
            v-model="form.apiKey"
            type="password"
            placeholder="sk-... 或 {env:VAR_NAME}"
            class="opencode-provider-input w-full px-3 py-2 rounded-lg text-sm"
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
            class="opencode-provider-input w-full px-3 py-2 rounded-lg text-sm"
          >
        </div>

        <!-- 操作按钮 -->
        <div class="flex justify-end gap-3 pt-2">
          <Button
            variant="secondary"
            surface="status"
            density="compact"
            motion="subtle"
            @click="closeDialog"
          >
            取消
          </Button>
          <Button
            variant="primary"
            surface="card"
            density="compact"
            motion="standard"
            :disabled="!form.id || !form.npm || saving"
            @click="saveProvider"
          >
            <template #leading>
              <SIcon
                v-if="saving"
                name="Loader2"
                size="w-4 h-4"
                class="animate-spin"
              />
            </template>
            {{ editingProvider ? '更新' : '添加' }}
          </Button>
        </div>
      </div>
    </BaseModal>

    <!-- 删除确认弹窗 -->
    <BaseModal
      :model-value="Boolean(deletingProvider)"
      title="确认删除"
      description="删除后该 Provider 会从 OpenCode 配置中移除。"
      size="sm"
      surface="solid"
      content-class="max-w-sm"
      @update:model-value="(value) => !value && (deletingProvider = null)"
    >
      <div class="space-y-4">
        <h2 class="text-lg font-bold text-white">
          确认删除
        </h2>
        <p class="text-white/80 text-sm">
          确定要删除 Provider <strong>{{ deletingProvider?.id }}</strong>（{{ deletingProvider?.npm }}）吗？此操作无法撤销。
        </p>
        <div class="flex justify-end gap-3">
          <Button
            variant="secondary"
            surface="status"
            density="compact"
            motion="subtle"
            @click="deletingProvider = null"
          >
            取消
          </Button>
          <Button
            variant="danger"
            surface="status"
            density="compact"
            motion="standard"
            :disabled="saving"
            @click="doDelete"
          >
            <template #leading>
              <SIcon
                v-if="saving"
                name="Loader2"
                size="w-4 h-4"
                class="animate-spin"
              />
            </template>
            删除
          </Button>
        </div>
      </div>
    </BaseModal>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted, reactive } from 'vue'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
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

<style scoped>
.opencode-provider-input,
.opencode-provider-preset {
  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  color: var(--color-text-primary);
  backdrop-filter: var(--surface-status-blur);
  box-shadow: var(--elevation-1);
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.opencode-provider-input::placeholder {
  color: var(--color-text-muted);
}

.opencode-provider-input:focus {
  outline: none;
  border-color: rgb(var(--color-accent-primary-rgb) / 42%);
  box-shadow: var(--elevation-2);
}
</style>
