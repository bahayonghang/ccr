<template>
  <div class="pricing-view">
    <header class="pricing-hero">
      <div class="pricing-hero__copy">
        <p class="pricing-hero__eyebrow">
          {{ t('pricing.eyebrow') }}
        </p>
        <div class="pricing-hero__title-row">
          <h1>{{ t('pricing.title') }}</h1>
          <span class="pricing-badge">{{ t('pricing.legacyBadge') }}</span>
        </div>
        <p class="pricing-hero__subtitle">
          {{ t('pricing.subtitle') }}
        </p>
      </div>

      <div class="pricing-hero__actions">
        <a
          class="pricing-button pricing-button--secondary"
          href="/usage"
        >
          {{ t('pricing.actions.openUsage') }}
        </a>
        <button
          type="button"
          :disabled="loading"
          class="pricing-button pricing-button--primary"
          @click="loadData"
        >
          <span
            class="pricing-button__spinner"
            :class="{ 'pricing-button__spinner--active': loading }"
            aria-hidden="true"
          />
          {{ t('pricing.actions.refresh') }}
        </button>
      </div>
    </header>

    <section
      class="pricing-boundary"
      :aria-label="t('pricing.boundary.title')"
    >
      <article class="pricing-boundary__item">
        <span>{{ t('pricing.boundary.sourceLabel') }}</span>
        <strong>{{ legacyPricingSourcePath }}</strong>
        <p>{{ t('pricing.boundary.sourceCopy') }}</p>
      </article>
      <article class="pricing-boundary__item">
        <span>{{ t('pricing.boundary.effectLabel') }}</span>
        <strong>{{ t('pricing.boundary.effectTitle') }}</strong>
        <p>{{ t('pricing.boundary.effectCopy') }}</p>
      </article>
      <article class="pricing-boundary__item">
        <span>{{ t('pricing.boundary.unitLabel') }}</span>
        <strong>{{ t('pricing.unitPerMtok') }}</strong>
        <p>{{ t('pricing.boundary.unitCopy') }}</p>
      </article>
    </section>

    <div
      v-if="statusMessage"
      class="pricing-status pricing-status--success"
      role="status"
      aria-live="polite"
    >
      {{ statusMessage }}
    </div>

    <div
      v-if="error"
      class="pricing-status pricing-status--error"
      role="alert"
    >
      <strong>{{ t('pricing.states.errorTitle') }}</strong>
      <span>{{ error }}</span>
    </div>

    <div
      v-if="loading && !pricingData"
      class="pricing-loading"
    >
      <span
        class="pricing-loading__spinner"
        aria-hidden="true"
      />
      <span>{{ t('pricing.states.loading') }}</span>
    </div>

    <main
      v-else
      class="pricing-content"
    >
      <section
        v-if="normalizedData.defaultPricing"
        class="pricing-card"
      >
        <div class="pricing-section-heading">
          <div>
            <p class="pricing-section-heading__eyebrow">
              {{ t('pricing.default.eyebrow') }}
            </p>
            <h2>{{ t('pricing.default.title') }}</h2>
          </div>
          <span class="pricing-pill">{{ t('pricing.unitPerMtok') }}</span>
        </div>
        <div class="pricing-metric-grid">
          <article
            v-for="metric in priceMetrics(normalizedData.defaultPricing)"
            :key="metric.key"
            class="pricing-metric"
          >
            <span>{{ metric.label }}</span>
            <strong>{{ metric.value }}</strong>
            <small>{{ t('pricing.unitPerMtok') }}</small>
          </article>
        </div>
      </section>

      <section class="pricing-card pricing-card--models">
        <div class="pricing-section-heading pricing-section-heading--split">
          <div>
            <p class="pricing-section-heading__eyebrow">
              {{ t('pricing.models.eyebrow') }}
            </p>
            <h2>{{ t('pricing.models.title') }}</h2>
            <p>{{ t('pricing.models.subtitle', { count: normalizedData.items.length }) }}</p>
          </div>
          <button
            type="button"
            class="pricing-button pricing-button--primary"
            @click="showAddForm"
          >
            {{ t('pricing.actions.add') }}
          </button>
        </div>

        <div
          v-if="normalizedData.items.length > 0"
          class="pricing-list"
        >
          <article
            v-for="pricing in normalizedData.items"
            :key="pricing.model"
            class="pricing-row"
          >
            <div class="pricing-row__main">
              <div class="pricing-row__title-line">
                <h3>{{ pricing.model }}</h3>
                <span class="pricing-row__tag">{{ t('pricing.models.configuredRow') }}</span>
              </div>
              <div class="pricing-row__metrics">
                <span
                  v-for="metric in priceMetrics(pricing)"
                  :key="metric.key"
                >
                  <small>{{ metric.label }}</small>
                  <strong>{{ metric.value }}</strong>
                  <em>{{ t('pricing.unitShort') }}</em>
                </span>
              </div>
            </div>
            <div class="pricing-row__actions">
              <button
                type="button"
                class="pricing-button pricing-button--secondary pricing-button--compact"
                @click="editPricing(pricing)"
              >
                {{ t('pricing.actions.edit') }}
              </button>
              <button
                type="button"
                class="pricing-button pricing-button--ghost-danger pricing-button--compact"
                @click="requestDelete(pricing.model)"
              >
                {{ t('pricing.actions.remove') }}
              </button>
            </div>
          </article>
        </div>

        <div
          v-else
          class="pricing-empty"
        >
          <strong>{{ t('pricing.empty.title') }}</strong>
          <p>{{ t('pricing.empty.subtitle') }}</p>
        </div>
      </section>

      <section
        v-if="showForm"
        class="pricing-card"
      >
        <div class="pricing-section-heading">
          <p class="pricing-section-heading__eyebrow">
            {{ isEditing ? t('pricing.form.editEyebrow') : t('pricing.form.addEyebrow') }}
          </p>
          <h2>{{ isEditing ? t('pricing.form.editTitle') : t('pricing.form.addTitle') }}</h2>
          <p>{{ t('pricing.form.unitHint') }}</p>
        </div>

        <form
          class="pricing-form"
          @submit.prevent="savePricing"
        >
          <label class="pricing-field">
            <span>{{ t('pricing.form.model') }}</span>
            <input
              v-model.trim="form.model"
              :disabled="isEditing || saving"
              type="text"
              required
              class="pricing-input"
              :placeholder="t('pricing.form.modelPlaceholder')"
            >
          </label>

          <div class="pricing-form__grid">
            <label
              v-for="field in priceFields"
              :key="field.key"
              class="pricing-field"
            >
              <span>{{ field.label }}</span>
              <input
                v-model.number="form[field.key]"
                :required="field.required"
                :disabled="saving"
                type="number"
                step="0.000001"
                min="0"
                class="pricing-input"
                :placeholder="t('pricing.form.pricePlaceholder')"
              >
            </label>
          </div>

          <div class="pricing-form__actions">
            <button
              type="submit"
              :disabled="saving"
              class="pricing-button pricing-button--primary"
            >
              {{ saving ? t('pricing.actions.saving') : t('pricing.actions.save') }}
            </button>
            <button
              type="button"
              :disabled="saving"
              class="pricing-button pricing-button--secondary"
              @click="cancelForm"
            >
              {{ t('pricing.actions.cancel') }}
            </button>
          </div>
        </form>
      </section>

      <section
        v-if="normalizedData.items.length > 0"
        class="pricing-card pricing-card--operations"
      >
        <div class="pricing-section-heading pricing-section-heading--split">
          <div>
            <p class="pricing-section-heading__eyebrow">
              {{ t('pricing.operations.eyebrow') }}
            </p>
            <h2>{{ t('pricing.operations.title') }}</h2>
            <p>{{ t('pricing.operations.subtitle') }}</p>
          </div>
          <button
            type="button"
            :disabled="saving"
            class="pricing-button pricing-button--ghost-danger"
            @click="requestReset"
          >
            {{ t('pricing.actions.reset') }}
          </button>
        </div>
      </section>

      <section
        v-if="pendingAction"
        class="pricing-confirm"
        role="dialog"
        aria-modal="false"
        :aria-label="confirmTitle"
      >
        <div>
          <p class="pricing-confirm__eyebrow">
            {{ t('pricing.confirm.eyebrow') }}
          </p>
          <h2>{{ confirmTitle }}</h2>
          <p>{{ confirmCopy }}</p>
        </div>
        <div class="pricing-confirm__actions">
          <button
            type="button"
            :disabled="saving"
            class="pricing-button pricing-button--danger"
            @click="confirmPendingAction"
          >
            {{ saving ? t('pricing.actions.saving') : t('pricing.confirm.confirm') }}
          </button>
          <button
            type="button"
            :disabled="saving"
            class="pricing-button pricing-button--secondary"
            @click="pendingAction = null"
          >
            {{ t('pricing.actions.cancel') }}
          </button>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getPricingList, removePricing, resetPricing, setPricing } from '@/api'
import type { ModelPricing, SetPricingRequest } from '@/types'
import { logger } from '@/utils/logger'

type PriceFormKey = 'input_price' | 'output_price' | 'cache_read_price' | 'cache_write_price'

type PriceForm = {
  model: string
  input_price: number
  output_price: number
  cache_read_price: number | null
  cache_write_price: number | null
}

type RawPricingListItem = {
  model?: string
  pricing?: Partial<ModelPricing>
}

type RawPricingListResponse = {
  items?: RawPricingListItem[]
  pricings?: Partial<ModelPricing>[]
  models?: Record<string, Partial<ModelPricing>>
  default_pricing?: Partial<ModelPricing> | null
  total?: number
}

type NormalizedPricingList = {
  items: ModelPricing[]
  defaultPricing: ModelPricing | null
  total: number
}

type PendingAction =
  | { type: 'delete'; model: string }
  | { type: 'reset' }

const { t } = useI18n()
const legacyPricingSourcePath = '~/.claude/pricing.toml'

const pricingData = ref<RawPricingListResponse | null>(null)
const loading = ref(false)
const saving = ref(false)
const error = ref<string | null>(null)
const statusMessage = ref<string | null>(null)
const showForm = ref(false)
const isEditing = ref(false)
const pendingAction = ref<PendingAction | null>(null)

const createEmptyForm = (): PriceForm => ({
  model: '',
  input_price: 0,
  output_price: 0,
  cache_read_price: null,
  cache_write_price: null,
})

const form = ref<PriceForm>(createEmptyForm())

const normalizePricing = (model: string, pricing: Partial<ModelPricing> | undefined): ModelPricing => ({
  model: pricing?.model || model,
  input_price: Number(pricing?.input_price ?? 0),
  output_price: Number(pricing?.output_price ?? 0),
  cache_read_price: pricing?.cache_read_price ?? undefined,
  cache_write_price: pricing?.cache_write_price ?? undefined,
})

const normalizePricingList = (response: RawPricingListResponse | null): NormalizedPricingList => {
  if (!response) {
    return { items: [], defaultPricing: null, total: 0 }
  }

  const items = Array.isArray(response.items)
    ? response.items.map((item) => normalizePricing(item.model ?? item.pricing?.model ?? '', item.pricing))
    : Array.isArray(response.pricings)
      ? response.pricings.map((pricing) => normalizePricing(pricing.model ?? '', pricing))
      : response.models
        ? Object.entries(response.models).map(([model, pricing]) => normalizePricing(model, pricing))
        : []

  const sortedItems = items
    .filter((pricing) => pricing.model.length > 0)
    .sort((a, b) => a.model.localeCompare(b.model))

  return {
    items: sortedItems,
    defaultPricing: response.default_pricing
      ? normalizePricing(response.default_pricing.model ?? 'default', response.default_pricing)
      : null,
    total: typeof response.total === 'number' ? response.total : sortedItems.length,
  }
}

const normalizedData = computed(() => normalizePricingList(pricingData.value))

const priceFields: Array<{ key: PriceFormKey; label: string; required: boolean }> = [
  { key: 'input_price', label: t('pricing.fields.input'), required: true },
  { key: 'output_price', label: t('pricing.fields.output'), required: true },
  { key: 'cache_read_price', label: t('pricing.fields.cacheRead'), required: false },
  { key: 'cache_write_price', label: t('pricing.fields.cacheWrite'), required: false },
]

const priceMetrics = (pricing: ModelPricing) => [
  { key: 'input', label: t('pricing.fields.input'), value: formatPrice(pricing.input_price) },
  { key: 'output', label: t('pricing.fields.output'), value: formatPrice(pricing.output_price) },
  { key: 'cache-read', label: t('pricing.fields.cacheRead'), value: formatPrice(pricing.cache_read_price) },
  { key: 'cache-write', label: t('pricing.fields.cacheWrite'), value: formatPrice(pricing.cache_write_price) },
]

const confirmTitle = computed(() => {
  if (!pendingAction.value) {
    return ''
  }

  return pendingAction.value.type === 'delete'
    ? t('pricing.confirm.deleteTitle', { model: pendingAction.value.model })
    : t('pricing.confirm.resetTitle')
})

const confirmCopy = computed(() => {
  if (!pendingAction.value) {
    return ''
  }

  return pendingAction.value.type === 'delete'
    ? t('pricing.confirm.deleteCopy')
    : t('pricing.confirm.resetCopy')
})

function formatPrice(value?: number | null): string {
  return typeof value === 'number' ? `$${value.toFixed(4)}` : '—'
}

function humanizeError(value: unknown): string {
  return value instanceof Error ? value.message : String(value || t('pricing.messages.unknownError'))
}

async function loadData() {
  loading.value = true
  error.value = null

  try {
    pricingData.value = await getPricingList<RawPricingListResponse>()
  } catch (value) {
    error.value = humanizeError(value)
    logger.error('Failed to load legacy CCR pricing:', value)
  } finally {
    loading.value = false
  }
}

function showAddForm() {
  statusMessage.value = null
  error.value = null
  pendingAction.value = null
  isEditing.value = false
  form.value = createEmptyForm()
  showForm.value = true
}

function editPricing(pricing: ModelPricing) {
  statusMessage.value = null
  error.value = null
  pendingAction.value = null
  isEditing.value = true
  form.value = {
    model: pricing.model,
    input_price: pricing.input_price,
    output_price: pricing.output_price,
    cache_read_price: pricing.cache_read_price ?? null,
    cache_write_price: pricing.cache_write_price ?? null,
  }
  showForm.value = true
}

function cancelForm() {
  showForm.value = false
  error.value = null
}

async function savePricing() {
  const model = form.value.model.trim()
  if (!model) {
    error.value = t('pricing.messages.modelRequired')
    return
  }

  saving.value = true
  error.value = null
  statusMessage.value = null

  const request: SetPricingRequest = {
    model,
    input_price: form.value.input_price,
    output_price: form.value.output_price,
    cache_read_price: form.value.cache_read_price ?? undefined,
    cache_write_price: form.value.cache_write_price ?? undefined,
  }

  try {
    await setPricing(request)
    await loadData()
    showForm.value = false
    statusMessage.value = isEditing.value
      ? t('pricing.messages.updated', { model })
      : t('pricing.messages.created', { model })
  } catch (value) {
    error.value = t('pricing.messages.saveFailed', { error: humanizeError(value) })
    logger.error('Failed to save legacy CCR pricing:', value)
  } finally {
    saving.value = false
  }
}

function requestDelete(model: string) {
  statusMessage.value = null
  error.value = null
  showForm.value = false
  pendingAction.value = { type: 'delete', model }
}

function requestReset() {
  statusMessage.value = null
  error.value = null
  showForm.value = false
  pendingAction.value = { type: 'reset' }
}

async function confirmPendingAction() {
  if (!pendingAction.value) {
    return
  }

  saving.value = true
  error.value = null
  statusMessage.value = null

  const action = pendingAction.value

  try {
    if (action.type === 'delete') {
      await removePricing(action.model)
      statusMessage.value = t('pricing.messages.removed', { model: action.model })
    } else {
      await resetPricing()
      statusMessage.value = t('pricing.messages.reset')
    }

    pendingAction.value = null
    await loadData()
  } catch (value) {
    error.value = action.type === 'delete'
      ? t('pricing.messages.removeFailed', { error: humanizeError(value) })
      : t('pricing.messages.resetFailed', { error: humanizeError(value) })
    logger.error('Failed to apply legacy CCR pricing action:', value)
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  void loadData()
})
</script>

<style scoped>
.pricing-view {
  display: grid;
  gap: 1rem;
  padding: 1.1rem;
}

.pricing-hero,
.pricing-boundary,
.pricing-card,
.pricing-confirm,
.pricing-status,
.pricing-loading {
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  background: rgb(var(--color-bg-elevated-rgb) / 78%);
  box-shadow: var(--elevation-1), inset 0 1px 0 rgb(255 255 255 / 7%);
}

.pricing-hero {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
  border-radius: 1.45rem;
  background:
    linear-gradient(135deg, rgb(var(--color-bg-elevated-rgb) / 90%), rgb(var(--color-bg-surface-rgb) / 72%)),
    radial-gradient(circle at 12% 0%, rgb(var(--color-accent-primary-rgb) / 7%), transparent 38%);
  padding: 1rem 1.08rem;
}

.pricing-hero__copy {
  display: grid;
  gap: 0.28rem;
  max-width: 54rem;
}

.pricing-hero__eyebrow,
.pricing-section-heading__eyebrow,
.pricing-confirm__eyebrow,
.pricing-boundary__item span,
.pricing-field span {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 760;
  letter-spacing: 0.11em;
  text-transform: uppercase;
}

.pricing-hero__title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.55rem;
}

.pricing-hero h1 {
  color: var(--color-text-primary);
  font-size: clamp(1.45rem, 1.1vw + 1rem, 2.05rem);
  font-weight: 780;
  letter-spacing: -0.04em;
  line-height: 1.05;
}

.pricing-hero__subtitle,
.pricing-section-heading p,
.pricing-boundary__item p,
.pricing-confirm p,
.pricing-empty p {
  color: var(--color-text-secondary);
  font-size: 0.88rem;
  line-height: 1.5;
}

.pricing-hero__actions,
.pricing-section-heading--split,
.pricing-form__actions,
.pricing-confirm__actions,
.pricing-row__actions,
.pricing-row__title-line {
  display: flex;
  align-items: center;
  gap: 0.55rem;
}

.pricing-hero__actions,
.pricing-row__actions {
  flex-wrap: wrap;
  justify-content: flex-end;
}

.pricing-badge,
.pricing-pill,
.pricing-row__tag {
  display: inline-flex;
  align-items: center;
  width: fit-content;
  border: 1px solid rgb(var(--color-border-default-rgb) / 16%);
  border-radius: 9999px;
  background: rgb(var(--color-bg-surface-rgb) / 58%);
  color: var(--color-text-secondary);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.pricing-badge,
.pricing-pill {
  padding: 0.28rem 0.58rem;
}

.pricing-row__tag {
  padding: 0.18rem 0.48rem;
}

.pricing-button {
  display: inline-flex;
  min-height: 2.35rem;
  align-items: center;
  justify-content: center;
  gap: 0.42rem;
  border: 1px solid transparent;
  border-radius: 9999px;
  padding: 0 0.82rem;
  font-size: 0.82rem;
  font-weight: 700;
  text-decoration: none;
  transition:
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    opacity var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.pricing-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.pricing-button--primary {
  border-color: rgb(var(--color-accent-primary-rgb) / 22%);
  background: rgb(var(--color-accent-primary-rgb) / 14%);
  color: var(--color-text-primary);
}

.pricing-button--primary:hover:not(:disabled) {
  background: rgb(var(--color-accent-primary-rgb) / 19%);
}

.pricing-button--secondary {
  border-color: rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 58%);
  color: var(--color-text-secondary);
}

.pricing-button--secondary:hover:not(:disabled) {
  color: var(--color-text-primary);
}

.pricing-button--danger,
.pricing-button--ghost-danger {
  border-color: rgb(185 93 75 / 25%);
  color: rgb(150 62 46);
}

.pricing-button--danger {
  background: rgb(185 93 75 / 14%);
}

.pricing-button--ghost-danger {
  background: rgb(185 93 75 / 7%);
}

:global(.dark) .pricing-button--danger,
:global(.dark) .pricing-button--ghost-danger {
  color: rgb(244 173 153);
}

.pricing-button--compact {
  min-height: 2rem;
  padding: 0 0.68rem;
  font-size: 0.76rem;
}

.pricing-button__spinner,
.pricing-loading__spinner {
  width: 0.9rem;
  height: 0.9rem;
  border: 2px solid currentcolor;
  border-top-color: transparent;
  border-radius: 9999px;
  opacity: 0;
}

.pricing-button__spinner--active,
.pricing-loading__spinner {
  animation: pricing-spin 0.8s linear infinite;
  opacity: 0.8;
}

.pricing-boundary {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.75rem;
  border-radius: 1.2rem;
  padding: 0.78rem;
}

.pricing-boundary__item {
  display: grid;
  gap: 0.34rem;
  border-radius: 1rem;
  background: rgb(var(--color-bg-surface-rgb) / 48%);
  padding: 0.82rem;
}

.pricing-boundary__item strong {
  color: var(--color-text-primary);
  font-size: 0.98rem;
}

.pricing-content {
  display: grid;
  gap: 1rem;
}

.pricing-card,
.pricing-confirm,
.pricing-status,
.pricing-loading {
  border-radius: 1.2rem;
  padding: 1rem;
}

.pricing-card {
  display: grid;
  gap: 0.95rem;
}

.pricing-section-heading {
  display: grid;
  gap: 0.32rem;
}

.pricing-section-heading--split {
  justify-content: space-between;
}

.pricing-section-heading h2,
.pricing-confirm h2 {
  color: var(--color-text-primary);
  font-size: 1.05rem;
  font-weight: 760;
  letter-spacing: -0.02em;
}

.pricing-metric-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.65rem;
}

.pricing-metric,
.pricing-row {
  border: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  border-radius: 1rem;
  background: rgb(var(--color-bg-surface-rgb) / 52%);
}

.pricing-metric {
  display: grid;
  gap: 0.2rem;
  padding: 0.78rem;
}

.pricing-metric span,
.pricing-row__metrics small {
  color: var(--color-text-muted);
  font-size: 0.72rem;
  font-weight: 700;
}

.pricing-metric strong,
.pricing-row__metrics strong {
  color: var(--color-text-primary);
  font-size: 1rem;
}

.pricing-metric small,
.pricing-row__metrics em {
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-style: normal;
}

.pricing-list {
  display: grid;
  gap: 0.62rem;
}

.pricing-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.8rem;
  padding: 0.82rem;
}

.pricing-row__main {
  display: grid;
  gap: 0.58rem;
  min-width: 0;
}

.pricing-row__title-line {
  flex-wrap: wrap;
}

.pricing-row h3 {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 0.96rem;
  font-weight: 760;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pricing-row__metrics,
.pricing-form__grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.55rem;
}

.pricing-row__metrics span {
  display: grid;
  gap: 0.15rem;
  min-width: 0;
}

.pricing-empty {
  display: grid;
  gap: 0.25rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 22%);
  border-radius: 1rem;
  padding: 1.2rem;
  text-align: center;
}

.pricing-empty strong {
  color: var(--color-text-primary);
}

.pricing-form {
  display: grid;
  gap: 0.78rem;
}

.pricing-field {
  display: grid;
  gap: 0.32rem;
}

.pricing-input {
  min-height: 2.55rem;
  width: 100%;
  border: 1px solid rgb(var(--color-border-default-rgb) / 17%);
  border-radius: 0.82rem;
  background: rgb(var(--color-bg-surface-rgb) / 68%);
  color: var(--color-text-primary);
  font-size: 0.9rem;
  outline: none;
  padding: 0 0.78rem;
}

.pricing-input:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
  box-shadow: 0 0 0 3px rgb(var(--color-accent-primary-rgb) / 9%);
}

.pricing-input:disabled {
  opacity: 0.62;
}

.pricing-status {
  display: flex;
  gap: 0.45rem;
  align-items: center;
  font-size: 0.86rem;
}

.pricing-status--success {
  border-color: rgb(86 128 94 / 18%);
  background: rgb(86 128 94 / 10%);
  color: rgb(59 104 69);
}

.pricing-status--error {
  border-color: rgb(185 93 75 / 24%);
  background: rgb(185 93 75 / 10%);
  color: rgb(139 56 43);
}

:global(.dark) .pricing-status--success {
  color: rgb(174 218 181);
}

:global(.dark) .pricing-status--error {
  color: rgb(244 173 153);
}

.pricing-loading {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.55rem;
  min-height: 8rem;
  color: var(--color-text-secondary);
}

.pricing-loading__spinner {
  opacity: 0.75;
}

.pricing-confirm {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  border-color: rgb(185 93 75 / 20%);
  background: rgb(185 93 75 / 8%);
}

@keyframes pricing-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (width < 1080px) {
  .pricing-hero,
  .pricing-confirm,
  .pricing-row,
  .pricing-section-heading--split {
    align-items: flex-start;
    grid-template-columns: 1fr;
    flex-direction: column;
  }

  .pricing-boundary,
  .pricing-metric-grid,
  .pricing-row__metrics,
  .pricing-form__grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .pricing-hero__actions,
  .pricing-row__actions,
  .pricing-confirm__actions {
    justify-content: flex-start;
  }
}

@media (width < 680px) {
  .pricing-view {
    padding: 0.75rem;
  }

  .pricing-boundary,
  .pricing-metric-grid,
  .pricing-row__metrics,
  .pricing-form__grid {
    grid-template-columns: 1fr;
  }

  .pricing-hero__actions,
  .pricing-form__actions,
  .pricing-confirm__actions,
  .pricing-button {
    width: 100%;
  }
}
</style>
