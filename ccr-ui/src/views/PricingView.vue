<template>
  <div class="pricing-view">
    <!-- 页面标题 -->
    <div class="pricing-header">
      <div>
        <h1 class="pricing-title text-text-primary">
          💲 定价管理
        </h1>
        <p class="pricing-subtitle text-text-secondary">
          配置各个模型的价格和默认定价策略
        </p>
      </div>
      <button
        :disabled="loading"
        class="pricing-action-button pricing-action-button--primary"
        @click="loadData"
      >
        <svg
          class="pricing-action-button__icon"
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
      class="pricing-loading"
    >
      <div class="pricing-loading__spinner animate-spin" />
    </div>

    <!-- 错误提示 -->
    <div
      v-if="error"
      class="pricing-error"
    >
      <div class="pricing-error__layout">
        <svg
          class="pricing-error__icon"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd"
          />
        </svg>
        <div class="pricing-error__content">
          <h3 class="pricing-error__title">
            加载失败
          </h3>
          <p class="pricing-error__message">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <!-- 定价内容 -->
    <div
      v-if="!loading && !error && pricingData"
      class="pricing-content"
    >
      <!-- 默认定价卡片 -->
      <div class="pricing-card">
        <h2 class="pricing-section-title text-text-primary">
          默认定价策略
        </h2>
        <div class="pricing-default-grid">
          <div class="pricing-metric-card pricing-metric-card--blue">
            <p class="pricing-metric-card__label text-text-secondary">
              输入价格
            </p>
            <p class="pricing-metric-card__value text-text-primary">
              ${{ pricingData.default_pricing.input_price.toFixed(4) }}
            </p>
            <p class="pricing-metric-card__meta text-text-muted">
              / 1K tokens
            </p>
          </div>
          <div class="pricing-metric-card pricing-metric-card--green">
            <p class="pricing-metric-card__label text-text-secondary">
              输出价格
            </p>
            <p class="pricing-metric-card__value text-text-primary">
              ${{ pricingData.default_pricing.output_price.toFixed(4) }}
            </p>
            <p class="pricing-metric-card__meta text-text-muted">
              / 1K tokens
            </p>
          </div>
          <div class="pricing-metric-card pricing-metric-card--purple">
            <p class="pricing-metric-card__label text-text-secondary">
              缓存读取
            </p>
            <p class="pricing-metric-card__value text-text-primary">
              ${{ (pricingData.default_pricing.cache_read_price || 0).toFixed(4) }}
            </p>
            <p class="pricing-metric-card__meta text-text-muted">
              / 1K tokens
            </p>
          </div>
          <div class="pricing-metric-card pricing-metric-card--orange">
            <p class="pricing-metric-card__label text-text-secondary">
              缓存写入
            </p>
            <p class="pricing-metric-card__value text-text-primary">
              ${{ (pricingData.default_pricing.cache_write_price || 0).toFixed(4) }}
            </p>
            <p class="pricing-metric-card__meta text-text-muted">
              / 1K tokens
            </p>
          </div>
        </div>
      </div>

      <!-- 模型定价列表 -->
      <div class="pricing-card">
        <div class="pricing-section-header">
          <h2 class="pricing-section-title text-text-primary">
            模型定价配置
          </h2>
          <button
            class="pricing-action-button pricing-action-button--success pricing-action-button--compact"
            @click="showAddForm"
          >
            ➕ 添加模型定价
          </button>
        </div>

        <!-- 定价列表 -->
        <div
          v-if="pricingData.pricings.length > 0"
          class="pricing-list"
        >
          <div
            v-for="pricing in pricingData.pricings"
            :key="pricing.model"
            class="pricing-list-item"
          >
            <div class="pricing-list-item__body">
              <h3 class="pricing-list-item__title text-text-primary">
                {{ pricing.model }}
              </h3>
              <div class="pricing-list-item__metrics">
                <div>
                  <span class="text-text-secondary">输入:</span>
                  <span class="pricing-list-item__value text-text-primary">
                    ${{ pricing.input_price.toFixed(4) }}
                  </span>
                </div>
                <div>
                  <span class="text-text-secondary">输出:</span>
                  <span class="pricing-list-item__value text-text-primary">
                    ${{ pricing.output_price.toFixed(4) }}
                  </span>
                </div>
                <div>
                  <span class="text-text-secondary">缓存读:</span>
                  <span class="pricing-list-item__value text-text-primary">
                    ${{ (pricing.cache_read_price || 0).toFixed(4) }}
                  </span>
                </div>
                <div>
                  <span class="text-text-secondary">缓存写:</span>
                  <span class="pricing-list-item__value text-text-primary">
                    ${{ (pricing.cache_write_price || 0).toFixed(4) }}
                  </span>
                </div>
              </div>
            </div>
            <div class="pricing-list-item__actions">
              <button
                class="pricing-action-button pricing-action-button--primary pricing-action-button--small"
                @click="editPricing(pricing)"
              >
                编辑
              </button>
              <button
                class="pricing-action-button pricing-action-button--danger pricing-action-button--small"
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
          class="pricing-empty text-text-muted"
        >
          暂无模型定价配置，点击上方按钮添加
        </div>
      </div>

      <!-- 添加/编辑表单 -->
      <div
        v-if="showForm"
        class="pricing-card"
      >
        <h2 class="pricing-section-title text-text-primary">
          {{ isEditing ? '编辑模型定价' : '添加模型定价' }}
        </h2>
        <form
          class="pricing-form"
          @submit.prevent="savePricing"
        >
          <!-- 模型名称 -->
          <div>
            <label class="pricing-label text-text-secondary">
              模型名称 *
            </label>
            <input
              v-model="form.model"
              :disabled="isEditing"
              type="text"
              required
              class="pricing-input"
              placeholder="例如: claude-sonnet-4-5"
            >
          </div>

          <!-- 价格输入 -->
          <div class="pricing-form-grid">
            <div>
              <label class="pricing-label text-text-secondary">
                输入价格 ($) *
              </label>
              <input
                v-model.number="form.input_price"
                type="number"
                step="0.000001"
                min="0"
                required
                class="pricing-input"
                placeholder="每1K tokens价格"
              >
            </div>
            <div>
              <label class="pricing-label text-text-secondary">
                输出价格 ($) *
              </label>
              <input
                v-model.number="form.output_price"
                type="number"
                step="0.000001"
                min="0"
                required
                class="pricing-input"
                placeholder="每1K tokens价格"
              >
            </div>
          </div>

          <div class="pricing-form-grid">
            <div>
              <label class="pricing-label text-text-secondary">
                缓存读取价格 ($)
              </label>
              <input
                v-model.number="form.cache_read_price"
                type="number"
                step="0.000001"
                min="0"
                class="pricing-input"
                placeholder="可选"
              >
            </div>
            <div>
              <label class="pricing-label text-text-secondary">
                缓存写入价格 ($)
              </label>
              <input
                v-model.number="form.cache_write_price"
                type="number"
                step="0.000001"
                min="0"
                class="pricing-input"
                placeholder="可选"
              >
            </div>
          </div>

          <!-- 按钮 -->
          <div class="pricing-form-actions">
            <button
              type="submit"
              :disabled="saving"
              class="pricing-action-button pricing-action-button--primary pricing-action-button--wide"
            >
              {{ saving ? '保存中...' : '保存' }}
            </button>
            <button
              type="button"
              :disabled="saving"
              class="pricing-action-button pricing-action-button--neutral pricing-action-button--wide"
              @click="cancelForm"
            >
              取消
            </button>
          </div>
        </form>
      </div>

      <!-- 操作按钮 -->
      <div class="pricing-card">
        <h2 class="pricing-section-title text-text-primary">
          批量操作
        </h2>
        <button
          :disabled="saving"
          class="pricing-action-button pricing-action-button--danger pricing-action-button--wide"
          @click="handleReset"
        >
          重置所有定价为默认值
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getPricingList, setPricing, removePricing, resetPricing } from '@/api'
import type { PricingListResponse, ModelPricing, SetPricingRequest } from '@/types'
import { logger } from '@/utils/logger'

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
    logger.error('Failed to load pricing:', e)
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
    logger.error('Failed to save pricing:', e)
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
    logger.error('Failed to delete pricing:', e)
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
    logger.error('Failed to reset pricing:', e)
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.pricing-view,
.pricing-content,
.pricing-form,
.pricing-list {
  display: flex;
  flex-direction: column;
}

.pricing-view {
  gap: 1.5rem;
  padding: 1.5rem;
}

.pricing-content,
.pricing-form,
.pricing-list {
  gap: 1.5rem;
}

.pricing-header,
.pricing-section-header,
.pricing-list-item,
.pricing-list-item__actions,
.pricing-form-actions,
.pricing-action-button {
  display: flex;
  align-items: center;
}

.pricing-header,
.pricing-section-header,
.pricing-list-item {
  justify-content: space-between;
  gap: 1rem;
}

.pricing-title,
.pricing-section-title,
.pricing-list-item__title {
  font-weight: 700;
}

.pricing-title {
  font-size: 1.875rem;
}

.pricing-subtitle {
  margin-top: 0.5rem;
  font-size: 0.875rem;
}

.pricing-card {
  border-radius: 0.75rem;
  background: white;
  padding: 1.5rem;
  box-shadow: 0 8px 24px rgb(15 23 42 / 8%);
}

:global(.dark) .pricing-card {
  background: rgb(31 41 55);
}

.pricing-section-title {
  margin-bottom: 1rem;
  font-size: 1.25rem;
  line-height: 1.3;
}

.pricing-action-button {
  justify-content: center;
  gap: 0.5rem;
  border-radius: 0.5rem;
  color: white;
  font-weight: 500;
  transition: background-color 0.2s ease, opacity 0.2s ease;
}

.pricing-action-button:disabled {
  opacity: 0.5;
}

.pricing-action-button--primary {
  background: rgb(37 99 235);
}

.pricing-action-button--primary:hover:not(:disabled) {
  background: rgb(29 78 216);
}

.pricing-action-button--success {
  background: rgb(22 163 74);
}

.pricing-action-button--success:hover:not(:disabled) {
  background: rgb(21 128 61);
}

.pricing-action-button--danger {
  background: rgb(220 38 38);
}

.pricing-action-button--danger:hover:not(:disabled) {
  background: rgb(185 28 28);
}

.pricing-action-button--neutral {
  background: rgb(107 114 128);
}

.pricing-action-button--neutral:hover:not(:disabled) {
  background: rgb(75 85 99);
}

.pricing-action-button--primary,
.pricing-action-button--success,
.pricing-action-button--neutral,
.pricing-action-button--danger {
  padding: 0.5rem 1rem;
}

.pricing-action-button--compact {
  font-size: 0.875rem;
}

.pricing-action-button--small {
  padding: 0.25rem 0.75rem;
  font-size: 0.875rem;
}

.pricing-action-button--wide {
  padding: 0.5rem 1.5rem;
}

.pricing-action-button__icon {
  width: 1.25rem;
  height: 1.25rem;
}

.pricing-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 3rem 0;
}

.pricing-loading__spinner {
  width: 3rem;
  height: 3rem;
  border-radius: 9999px;
  border-bottom: 2px solid rgb(37 99 235);
}

.pricing-error {
  border: 1px solid rgb(254 202 202);
  border-radius: 0.5rem;
  background: rgb(254 242 242);
  padding: 1rem;
}

:global(.dark) .pricing-error {
  border-color: rgb(153 27 27);
  background: rgb(127 29 29 / 20%);
}

.pricing-error__layout {
  display: flex;
}

.pricing-error__icon {
  width: 1.25rem;
  height: 1.25rem;
  color: rgb(248 113 113);
}

.pricing-error__content {
  margin-left: 0.75rem;
}

.pricing-error__title {
  font-size: 0.875rem;
  font-weight: 500;
  color: rgb(153 27 27);
}

:global(.dark) .pricing-error__title {
  color: rgb(254 202 202);
}

.pricing-error__message {
  margin-top: 0.5rem;
  font-size: 0.875rem;
  color: rgb(185 28 28);
}

:global(.dark) .pricing-error__message {
  color: rgb(252 165 165);
}

.pricing-default-grid,
.pricing-form-grid,
.pricing-list-item__metrics {
  display: grid;
  gap: 1rem;
}

.pricing-default-grid {
  grid-template-columns: repeat(1, minmax(0, 1fr));
}

.pricing-metric-card,
.pricing-list-item {
  border-radius: 0.5rem;
  padding: 1rem;
}

.pricing-metric-card--blue {
  background: rgb(239 246 255);
}

.pricing-metric-card--green {
  background: rgb(240 253 244);
}

.pricing-metric-card--purple {
  background: rgb(250 245 255);
}

.pricing-metric-card--orange {
  background: rgb(255 247 237);
}

:global(.dark) .pricing-metric-card--blue {
  background: rgb(30 58 138 / 20%);
}

:global(.dark) .pricing-metric-card--green {
  background: rgb(20 83 45 / 20%);
}

:global(.dark) .pricing-metric-card--purple {
  background: rgb(88 28 135 / 20%);
}

:global(.dark) .pricing-metric-card--orange {
  background: rgb(154 52 18 / 20%);
}

.pricing-metric-card__label {
  font-size: 0.875rem;
}

.pricing-metric-card__value {
  margin-top: 0.5rem;
  font-size: 1.5rem;
  font-weight: 700;
}

.pricing-metric-card__meta {
  margin-top: 0.25rem;
  font-size: 0.75rem;
}

.pricing-list {
  gap: 0.75rem;
}

.pricing-list-item {
  background: rgb(249 250 251);
}

:global(.dark) .pricing-list-item {
  background: rgb(55 65 81 / 50%);
}

.pricing-list-item__body {
  flex: 1;
}

.pricing-list-item__title {
  font-size: 1.125rem;
}

.pricing-list-item__metrics {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin-top: 0.5rem;
  gap: 0.75rem;
  font-size: 0.875rem;
}

.pricing-list-item__value {
  margin-left: 0.5rem;
  font-weight: 600;
}

.pricing-list-item__actions,
.pricing-form-actions {
  gap: 0.5rem;
}

.pricing-list-item__actions {
  margin-left: 1rem;
}

.pricing-empty {
  padding: 2rem 0;
  text-align: center;
}

.pricing-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
}

.pricing-input {
  display: block;
  width: 100%;
  margin-top: 0.25rem;
  padding: 0.5rem 0.75rem;
  border: 1px solid rgb(209 213 219);
  border-radius: 0.5rem;
  background: white;
  color: rgb(17 24 39);
}

:global(.dark) .pricing-input {
  border-color: rgb(75 85 99);
  background: rgb(55 65 81);
  color: white;
}

.pricing-input:disabled {
  opacity: 0.5;
}

@media (width >= 768px) {
  .pricing-default-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .pricing-form-grid,
  .pricing-list-item__metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width >= 900px) {
  .pricing-list-item__metrics {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}

@media (width <= 767px) {
  .pricing-header,
  .pricing-section-header,
  .pricing-list-item {
    flex-direction: column;
    align-items: flex-start;
  }

  .pricing-list-item__actions,
  .pricing-form-actions {
    width: 100%;
    flex-wrap: wrap;
  }
}
</style>
