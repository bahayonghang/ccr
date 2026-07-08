<template>
  <BaseModal
    :model-value="modelValue"
    :title="$t('usage.dashboard.ops.drawerTitle')"
    :description="presentation.summaryLine"
    size="2xl"
    scrollable
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="usage-diag">
      <p class="usage-diag__scope">
        {{ presentation.summaryLine }}
      </p>

      <section class="usage-diag__section">
        <div class="usage-diag__section-head">
          <span>{{ $t('usage.dashboard.ops.healthGridTitle') }}</span>
        </div>
        <div class="usage-diag__health-grid">
          <article
            v-for="item in presentation.healthItems"
            :key="item.id"
            class="usage-diag-health"
            :class="`usage-diag-health--${item.tone}`"
          >
            <span class="usage-diag-health__axis" />
            <div>
              <p class="usage-diag-health__label">
                {{ item.label }}
              </p>
              <strong>{{ item.value }}</strong>
              <p>{{ item.detail }}</p>
            </div>
          </article>
        </div>
      </section>

      <section class="usage-diag__section">
        <div class="usage-diag__section-head">
          <span>{{ $t('usage.dashboard.ops.sourcesTitle') }}</span>
          <small>{{ $t('usage.dashboard.ops.sourcesHint') }}</small>
        </div>
        <div
          v-if="presentation.sourceItems.length > 0"
          class="usage-diag-sources"
        >
          <article
            v-for="source in presentation.sourceItems"
            :key="source.id"
            class="usage-diag-source"
            :class="`usage-diag-source--${source.tone}`"
          >
            <div class="usage-diag-source__head">
              <strong>{{ source.label }}</strong>
              <Badge
                :variant="badgeVariant(source.tone)"
                size="xs"
                pill
              >
                {{ source.statusLabel }}
              </Badge>
            </div>
            <p>{{ source.detail }}</p>
            <p
              v-if="source.hint"
              class="usage-diag-source__hint"
            >
              {{ source.hint }}
            </p>
            <button
              v-if="source.tone !== 'success'"
              type="button"
              class="usage-diag-source__refresh"
              @click="emit('refresh')"
            >
              <SIcon
                name="RefreshCw"
                size="w-3 h-3"
              />
              {{ $t('usage.dashboard.ops.actions.refresh_usage') }}
            </button>
          </article>
        </div>
        <p
          v-else
          class="usage-diag__empty"
        >
          {{ $t('usage.dashboard.ops.noSources') }}
        </p>
      </section>

      <section
        v-if="presentation.alerts.length > 0"
        class="usage-diag__section"
      >
        <div class="usage-diag__section-head">
          <span>{{ $t('usage.dashboard.ops.alertsTitle') }}</span>
          <small>{{ $t('usage.dashboard.ops.alertsHint') }}</small>
        </div>
        <article
          v-for="alert in presentation.alerts"
          :key="alert.id"
          class="usage-diag-alert"
          :class="`usage-diag-alert--${alert.tone}`"
        >
          <strong>{{ alert.title }}</strong>
          <p
            v-for="detail in alert.details"
            :key="detail"
          >
            {{ detail }}
          </p>
        </article>
      </section>
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import BaseModal from '@/components/common/BaseModal.vue'
import Badge from '@/components/ui/Badge.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { UsageOpsCockpitPresentation, UsageOpsTone } from '@/views/usage/usageOpsCockpit'

defineProps<{
  modelValue: boolean
  presentation: UsageOpsCockpitPresentation
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  refresh: []
}>()

const badgeVariant = (tone: UsageOpsTone) => (tone === 'muted' ? 'default' : tone)
</script>

<style scoped>
.usage-diag {
  display: grid;
  gap: 1.1rem;
}

.usage-diag__scope {
  color: var(--color-text-secondary);
  font-size: 0.8rem;
}

.usage-diag__section {
  display: grid;
  gap: 0.62rem;
}

.usage-diag__section-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.8rem;
}

.usage-diag__section-head span {
  color: var(--color-text-primary);
  font-size: 0.86rem;
  font-weight: 740;
}

.usage-diag__section-head small {
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 650;
}

.usage-diag__health-grid {
  display: grid;
  gap: 0.6rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.usage-diag-health {
  --usage-diag-item-rgb: var(--color-info-rgb);

  position: relative;
  overflow: hidden;
  display: flex;
  min-width: 0;
  gap: 0.6rem;
  border-radius: 0.95rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  background: rgb(var(--color-bg-surface-rgb) / 54%);
  padding: 0.65rem 0.7rem;
}

.usage-diag-health--success {
  --usage-diag-item-rgb: var(--color-success-rgb);
}

.usage-diag-health--warning {
  --usage-diag-item-rgb: var(--color-warning-rgb);
}

.usage-diag-health--danger {
  --usage-diag-item-rgb: var(--color-danger-rgb);
}

.usage-diag-health--info {
  --usage-diag-item-rgb: var(--color-info-rgb);
}

.usage-diag-health--muted {
  --usage-diag-item-rgb: var(--color-text-muted-rgb);
}

.usage-diag-health__axis {
  width: 3px;
  flex: none;
  border-radius: 999px;
  background: rgb(var(--usage-diag-item-rgb) / 74%);
}

.usage-diag-health__label {
  color: var(--color-text-muted);
  font-size: 0.64rem;
  font-weight: 760;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.usage-diag-health strong {
  display: block;
  overflow: hidden;
  margin-top: 0.16rem;
  color: var(--color-text-primary);
  font-size: 0.92rem;
  font-weight: 730;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.usage-diag-health p {
  margin-top: 0.2rem;
  color: var(--color-text-secondary);
  font-size: 0.74rem;
  line-height: 1.4;
}

.usage-diag-sources {
  display: grid;
  gap: 0.55rem;
}

.usage-diag-source,
.usage-diag-alert {
  --usage-diag-item-rgb: var(--color-info-rgb);

  border-radius: 0.9rem;
  border: 1px solid rgb(var(--usage-diag-item-rgb) / 14%);
  background: rgb(var(--usage-diag-item-rgb) / 6%);
  padding: 0.62rem 0.7rem;
}

.usage-diag-source--success,
.usage-diag-alert--success {
  --usage-diag-item-rgb: var(--color-success-rgb);
}

.usage-diag-source--warning,
.usage-diag-alert--warning {
  --usage-diag-item-rgb: var(--color-warning-rgb);
}

.usage-diag-source--danger,
.usage-diag-alert--danger {
  --usage-diag-item-rgb: var(--color-danger-rgb);
}

.usage-diag-source--info,
.usage-diag-alert--info {
  --usage-diag-item-rgb: var(--color-info-rgb);
}

.usage-diag-source--muted,
.usage-diag-alert--muted {
  --usage-diag-item-rgb: var(--color-text-muted-rgb);
}

.usage-diag-source__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
}

.usage-diag-source strong,
.usage-diag-alert strong {
  color: var(--color-text-primary);
  font-size: 0.82rem;
  font-weight: 740;
}

.usage-diag-source p,
.usage-diag-alert p {
  margin-top: 0.34rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  line-height: 1.5;
}

.usage-diag-source__hint {
  color: var(--color-text-muted);
  font-style: italic;
}

.usage-diag-source__refresh {
  display: inline-flex;
  align-items: center;
  gap: 0.32rem;
  margin-top: 0.5rem;
  border-radius: 999px;
  border: 1px solid rgb(var(--usage-diag-item-rgb) / 22%);
  background: rgb(var(--usage-diag-item-rgb) / 10%);
  padding: 0.26rem 0.62rem;
  color: rgb(var(--usage-diag-item-rgb));
  font-size: 0.7rem;
  font-weight: 720;
  transition:
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-diag-source__refresh:hover {
  transform: translateY(-1px);
  background: rgb(var(--usage-diag-item-rgb) / 16%);
}

.usage-diag__empty {
  border-radius: 0.9rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 14%);
  padding: 0.72rem;
  color: var(--color-text-secondary);
  font-size: 0.8rem;
}

@media (width < 640px) {
  .usage-diag__health-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
