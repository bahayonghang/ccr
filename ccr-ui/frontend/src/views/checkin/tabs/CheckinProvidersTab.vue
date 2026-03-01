<template>
  <div class="space-y-6">
    <!-- 内置中转站区域 -->
    <div v-if="availableBuiltinProviders.length > 0">
      <div class="flex items-center space-x-2 mb-4">
        <Store class="w-5 h-5 text-accent-primary" />
        <h2 class="text-lg font-semibold text-text-primary">
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
          class="bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-gray-800 dark:to-gray-700 rounded-xl shadow-sm p-4 border border-blue-100 dark:border-gray-600 hover:shadow-md transition-[box-shadow]"
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
              @click="emit('add-builtin', bp.id)"
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
              <component
                :is="bp.checkin_bugged ? AlertTriangle : CheckCircle"
                class="w-3 h-3 mr-1 inline"
              />
              {{ bp.checkin_bugged ? '自动签到' : '支持签到' }}
            </span>
            <span
              v-else
              class="px-2 py-0.5 text-xs bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 rounded-full flex items-center"
            >
              <XCircle class="w-3 h-3 mr-1" /> 无签到
            </span>
            <span
              v-if="bp.requires_waf_bypass"
              class="px-2 py-0.5 text-xs bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300 rounded-full flex items-center"
            >
              <Shield class="w-3 h-3 mr-1" /> 需要 WAF 绕过
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 已添加的提供商 -->
    <div>
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center space-x-2">
          <Building2 class="w-5 h-5 text-accent-secondary" />
          <h2 class="text-lg font-semibold text-text-primary">
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
          <Package class="w-12 h-12 mx-auto text-text-muted" />
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
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  Store,
  Building2,
  Package,
  CheckCircle,
  AlertTriangle,
  XCircle,
  Shield,
} from 'lucide-vue-next'
import {
  createCheckinProvider,
  updateCheckinProvider,
  deleteCheckinProvider as apiDeleteProvider,
} from '@/api/modules/checkin'
import type { CheckinProvider, BuiltinProvider } from '@/types/checkin'

const props = defineProps<{
  providers: CheckinProvider[]
  builtinProviders: BuiltinProvider[]
}>()

const emit = defineEmits<{
  (e: 'add-builtin', builtinId: string): void
  (e: 'refresh'): void
}>()

// 计算属性：过滤出尚未添加的内置提供商
const availableBuiltinProviders = computed(() => {
  const addedNames = new Set(props.providers.map(p => p.name))
  return props.builtinProviders.filter(bp => !addedNames.has(bp.name))
})

// 弹窗状态
const showProviderModal = ref(false)
const editingProvider = ref<CheckinProvider | null>(null)

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
    emit('refresh')
  } catch (e: any) {
    alert('保存失败: ' + (e.message || '未知错误'))
  }
}

const deleteProvider = async (id: string) => {
  if (!confirm('确定要删除此提供商吗？相关账号也会被删除。')) return
  try {
    await apiDeleteProvider(id)
    emit('refresh')
  } catch (e: any) {
    alert('删除失败: ' + (e.message || '未知错误'))
  }
}
</script>
