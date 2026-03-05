<template>
  <div class="pricing-view p-6 space-y-6">
    <!-- 页面标题 -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
          💲 定价管理
        </h1>
        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
          配置各个模型的价格和默认定价策略
        </p>
      </div>
      <button
        :disabled="loading"
        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg flex items-center space-x-2 disabled:opacity-50"
        @click="loadData"
      >
        <svg
          class="w-5 h-5"
          :class="{ 'animate-spin': loading }"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
          />
        </svg>
        <span>刷新</span>
      </button>
    </div>

    <!-- 加载状态 -->
    <div
      v-if="loading"
      class="flex items-center justify-center py-12"
    >
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" />
    </div>

    <!-- 错误提示 -->
    <div
      v-if="error"
      class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4"
    >
      <div class="flex">
        <svg
          class="h-5 w-5 text-red-400"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd"
          />
        </svg>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-red-800 dark:text-red-200">
            加载失败
          </h3>
          <p class="mt-2 text-sm text-red-700 dark:text-red-300">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <!-- 定价内容 -->
    <div
      v-if="!loading && !error && pricingData"
      class="space-y-6"
    >
      <!-- 默认定价卡片 -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          默认定价策略
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              输入价格
            </p>
            <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
              ${{ pricingData.default_pricing.input_price.toFixed(4) }}
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              / 1K tokens
            </p>
          </div>
          <div class="bg-green-50 dark:bg-green-900/20 rounded-lg p-4">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              输出价格
            </p>
            <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
              ${{ pricingData.default_pricing.output_price.toFixed(4) }}
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              / 1K tokens
            </p>
          </div>
          <div class="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-4">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              缓存读取
            </p>
            <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
              ${{ (pricingData.default_pricing.cache_read_price || 0).toFixed(4) }}
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              / 1K tokens
            </p>
          </div>
          <div class="bg-orange-50 dark:bg-orange-900/20 rounded-lg p-4">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              缓存写入
            </p>
            <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
              ${{ (pricingData.default_pricing.cache_write_price || 0).toFixed(4) }}
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
              / 1K tokens
            </p>
          </div>
        </div>
      </div>

      <!-- 模型定价列表 -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
            模型定价配置
          </h2>
          <button
            class="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg text-sm"
            @click="showAddForm"
          >
            ➕ 添加模型定价
          </button>
        </div>

        <!-- 定价列表 -->
        <div
          v-if="pricingData.pricings.length > 0"
          class="space-y-3"
        >
          <div
            v-for="pricing in pricingData.pricings"
            :key="pricing.model"
            class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg"
          >
            <div class="flex-1">
              <h3 class="text-lg font-medium text-gray-900 dark:text-white">
                {{ pricing.model }}
              </h3>
              <div class="mt-2 grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
                <div>
                  <span class="text-gray-600 dark:text-gray-400">输入:</span>
                  <span class="ml-2 font-semibold text-gray-900 dark:text-white">
                    ${{ pricing.input_price.toFixed(4) }}
                  </span>
                </div>
                <div>
                  <span class="text-gray-600 dark:text-gray-400">输出:</span>
                  <span class="ml-2 font-semibold text-gray-900 dark:text-white">
                    ${{ pricing.output_price.toFixed(4) }}
                  </span>
                </div>
                <div>
                  <span class="text-gray-600 dark:text-gray-400">缓存读:</span>
                  <span class="ml-2 font-semibold text-gray-900 dark:text-white">
                    ${{ (pricing.cache_read_price || 0).toFixed(4) }}
                  </span>
                </div>
                <div>
                  <span class="text-gray-600 dark:text-gray-400">缓存写:</span>
                  <span class="ml-2 font-semibold text-gray-900 dark:text-white">
                    ${{ (pricing.cache_write_price || 0).toFixed(4) }}
                  </span>
                </div>
              </div>
            </div>
            <div class="flex items-center space-x-2 ml-4">
              <button
                class="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm"
                @click="editPricing(pricing)"
              >
                编辑
              </button>
              <button
                class="px-3 py-1 bg-red-600 hover:bg-red-700 text-white rounded text-sm"
                @click="deletePricing(pricing.model)"
              >
                删除
              </button>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div
          v-else
          class="text-center py-8 text-gray-500 dark:text-gray-400"
        >
          暂无模型定价配置，点击上方按钮添加
        </div>
      </div>

      <!-- 添加/编辑表单 -->
      <div
        v-if="showForm"
        class="bg-white dark:bg-gray-800 rounded-lg shadow p-6"
      >
        <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          {{ isEditing ? '编辑模型定价' : '添加模型定价' }}
        </h2>
        <form
          class="space-y-4"
          @submit.prevent="savePricing"
        >
          <!-- 模型名称 -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              模型名称 *
            </label>
            <input
              v-model="form.model"
              :disabled="isEditing"
              type="text"
              required
              class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white disabled:opacity-50"
              placeholder="例如: claude-sonnet-4-5"
            >
          </div>

          <!-- 价格输入 -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                输入价格 ($) *
              </label>
              <input
                v-model.number="form.input_price"
                type="number"
                step="0.000001"
                min="0"
                required
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="每1K tokens价格"
              >
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                输出价格 ($) *
              </label>
              <input
                v-model.number="form.output_price"
                type="number"
                step="0.000001"
                min="0"
                required
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="每1K tokens价格"
              >
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                缓存读取价格 ($)
              </label>
              <input
                v-model.number="form.cache_read_price"
                type="number"
                step="0.000001"
                min="0"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="可选"
              >
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                缓存写入价格 ($)
              </label>
              <input
                v-model.number="form.cache_write_price"
                type="number"
                step="0.000001"
                min="0"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="可选"
              >
            </div>
          </div>

          <!-- 按钮 -->
          <div class="flex items-center space-x-4">
            <button
              type="submit"
              :disabled="saving"
              class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50"
            >
              {{ saving ? '保存中...' : '保存' }}
            </button>
            <button
              type="button"
              :disabled="saving"
              class="px-6 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-lg disabled:opacity-50"
              @click="cancelForm"
            >
              取消
            </button>
          </div>
        </form>
      </div>

      <!-- 操作按钮 -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          批量操作
        </h2>
        <button
          :disabled="saving"
          class="px-6 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg disabled:opacity-50"
          @click="handleReset"
        >
          重置所有定价为默认值
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/* eslint-disable no-console -- Development debugging, console output acceptable */
import { ref, onMounted } from 'vue'
import { getPricingList, setPricing, removePricing, resetPricing } from '@/api'
import type { PricingListResponse, ModelPricing, SetPricingRequest } from '@/types'

const pricingData = ref<PricingListResponse | null>(null)
const loading = ref(false)
const saving = ref(false)
const error = ref<string | null>(null)
const showForm = ref(false)
const isEditing = ref(false)

const form = ref<{
  model: string
  input_price: number
  output_price: number
  cache_read_price: number | null
  cache_write_price: number | null
}>({
  model: '',
  input_price: 0,
  output_price: 0,
  cache_read_price: null,
  cache_write_price: null,
})

const loadData = async () => {
  loading.value = true
  error.value = null

  try {
    pricingData.value = await getPricingList()
  } catch (e) {
    error.value = (e instanceof Error ? e.message : "Error") || '加载失败'
    console.error('Failed to load pricing:', e)
  } finally {
    loading.value = false
  }
}

const showAddForm = () => {
  isEditing.value = false
  form.value = {
    model: '',
    input_price: 0,
    output_price: 0,
    cache_read_price: null,
    cache_write_price: null,
  }
  showForm.value = true
}

const editPricing = (pricing: ModelPricing) => {
  isEditing.value = true
  form.value = {
    model: pricing.model,
    input_price: pricing.input_price,
    output_price: pricing.output_price,
    cache_read_price: pricing.cache_read_price || null,
    cache_write_price: pricing.cache_write_price || null,
  }
  showForm.value = true
}

const savePricing = async () => {
  saving.value = true

  try {
    const request: SetPricingRequest = {
      model: form.value.model,
      input_price: form.value.input_price,
      output_price: form.value.output_price,
      cache_read_price: form.value.cache_read_price ?? undefined,
      cache_write_price: form.value.cache_write_price ?? undefined,
    }

    await setPricing(request)
    await loadData()
    showForm.value = false

    alert(isEditing.value ? '定价已更新' : '定价已添加')
  } catch (e) {
    alert('保存失败: ' + ((e instanceof Error ? e.message : "Error") || '未知错误'))
    console.error('Failed to save pricing:', e)
  } finally {
    saving.value = false
  }
}

const deletePricing = async (model: string) => {
  if (!confirm(`确定要删除模型 "${model}" 的定价吗？`)) return

  saving.value = true

  try {
    await removePricing(model)
    await loadData()

    alert('定价已删除')
  } catch (e) {
    alert('删除失败: ' + ((e instanceof Error ? e.message : "Error") || '未知错误'))
    console.error('Failed to delete pricing:', e)
  } finally {
    saving.value = false
  }
}

const cancelForm = () => {
  showForm.value = false
}

const handleReset = async () => {
  if (!confirm('确定要重置所有模型定价为默认值吗？此操作不可撤销！')) return

  saving.value = true

  try {
    await resetPricing()
    await loadData()

    alert('所有定价已重置为默认值')
  } catch (e) {
    alert('重置失败: ' + ((e instanceof Error ? e.message : "Error") || '未知错误'))
    console.error('Failed to reset pricing:', e)
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadData()
})
</script>
