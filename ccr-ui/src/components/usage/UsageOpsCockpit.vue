<template>
  <section
    class="usage-ops"
    :class="`usage-ops--${presentation.tone}`"
    data-usage-ops-cockpit
  >
    <div class="usage-ops__hero">
      <div class="usage-ops__copy">
        <p class="usage-ops__eyebrow">
          {{ presentation.eyebrow }}
        </p>
        <h2>{{ presentation.title }}</h2>
        <p>{{ presentation.detail }}</p>
        <div class="usage-ops__scope">
          <span>{{ presentation.summaryLine }}</span>
        </div>
      </div>

      <div class="usage-ops__actions">
        <button
          v-if="presentation.primaryActionLabel"
          type="button"
          class="usage-ops__action usage-ops__action--primary"
          @click="emit('primary-action', presentation.primaryAction)"
        >
          {{ presentation.primaryActionLabel }}
        </button>
        <button
          v-if="presentation.secondaryActionLabel"
          type="button"
          class="usage-ops__action"
          @click="emit('secondary-action')"
        >
          {{ presentation.secondaryActionLabel }}
        </button>
      </div>
    </div>

    <div class="usage-ops__health-grid">
      <article
        v-for="item in presentation.healthItems"
        :key="item.id"
        class="usage-ops-health"
        :class="`usage-ops-health--${item.tone}`"
      >
        <span class="usage-ops-health__axis" />
        <div>
          <p class="usage-ops-health__label">
            {{ item.label }}
          </p>
          <strong>{{ item.value }}</strong>
          <p>{{ item.detail }}</p>
        </div>
      </article>
    </div>

    <div class="usage-ops__lower">
      <div class="usage-ops__source-strip">
        <div class="usage-ops__section-head">
          <span>{{ $t('usage.dashboard.ops.sourcesTitle') }}</span>
          <small>{{ $t('usage.dashboard.ops.sourcesHint') }}</small>
        </div>
        <div
          v-if="presentation.sourceItems.length > 0"
          class="usage-ops-sources"
        >
          <article
            v-for="source in presentation.sourceItems"
            :key="source.id"
            class="usage-ops-source"
            :class="`usage-ops-source--${source.tone}`"
          >
            <span>
              <strong>{{ source.label }}</strong>
              <em>{{ source.statusLabel }}</em>
            </span>
            <p>{{ source.detail }}</p>
          </article>
        </div>
        <p
          v-else
          class="usage-ops__empty"
        >
          {{ $t('usage.dashboard.ops.noSources') }}
        </p>
      </div>

      <div class="usage-ops__alerts">
        <div class="usage-ops__section-head">
          <span>{{ $t('usage.dashboard.ops.alertsTitle') }}</span>
          <small>{{ $t('usage.dashboard.ops.alertsHint') }}</small>
        </div>
        <article
          v-for="alert in presentation.alerts"
          :key="alert.id"
          class="usage-ops-alert"
          :class="`usage-ops-alert--${alert.tone}`"
        >
          <strong>{{ alert.title }}</strong>
          <p
            v-for="detail in alert.details"
            :key="detail"
          >
            {{ detail }}
          </p>
        </article>
        <p
          v-if="presentation.alerts.length === 0"
          class="usage-ops__empty"
        >
          {{ $t('usage.dashboard.ops.noAlerts') }}
        </p>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { UsageOpsAction, UsageOpsCockpitPresentation } from '@/views/usage/usageOpsCockpit'

defineProps<{
  presentation: UsageOpsCockpitPresentation
}>()

const emit = defineEmits<{
  'primary-action': [action: UsageOpsAction]
  'secondary-action': []
}>()
</script>

<style scoped>
.usage-ops {
  --usage-ops-rgb: var(--color-info-rgb);

  position: relative;
  isolation: isolate;
  overflow: hidden;
  display: grid;
  gap: 1rem;
  border-radius: 1.55rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 16%);
  padding: 1rem;
  background:
    linear-gradient(135deg, rgb(var(--usage-ops-rgb) / 10%), transparent 34%),
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 94%), rgb(var(--color-bg-surface-rgb) / 78%));
  box-shadow: var(--elevation-1), inset 0 1px 0 rgb(255 255 255 / 8%);
}

.usage-ops--success {
  --usage-ops-rgb: var(--color-success-rgb);
}

.usage-ops--warning {
  --usage-ops-rgb: var(--color-warning-rgb);
}

.usage-ops--danger {
  --usage-ops-rgb: var(--color-danger-rgb);
}

.usage-ops--info {
  --usage-ops-rgb: var(--color-info-rgb);
}

.usage-ops--muted {
  --usage-ops-rgb: var(--color-text-muted-rgb);
}

.usage-ops::before {
  content: '';
  position: absolute;
  inset: 0 auto 0 0;
  z-index: -1;
  width: 4px;
  background: linear-gradient(180deg, rgb(var(--usage-ops-rgb) / 86%), transparent);
}

.usage-ops__hero,
.usage-ops__lower,
.usage-ops__section-head,
.usage-ops-source span {
  display: flex;
}

.usage-ops__hero {
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.usage-ops__copy {
  max-width: 52rem;
}

.usage-ops__eyebrow,
.usage-ops-health__label {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 780;
  letter-spacing: 0.11em;
  text-transform: uppercase;
}

.usage-ops h2 {
  margin-top: 0.25rem;
  color: var(--color-text-primary);
  font-size: clamp(1.35rem, 1vw + 1rem, 2rem);
  font-weight: 760;
  letter-spacing: -0.035em;
}

.usage-ops__copy > p:not(.usage-ops__eyebrow),
.usage-ops-health p,
.usage-ops-source p,
.usage-ops-alert p,
.usage-ops__empty {
  color: var(--color-text-secondary);
  font-size: 0.8rem;
  line-height: 1.5;
}

.usage-ops__scope {
  display: inline-flex;
  margin-top: 0.75rem;
  border-radius: 999px;
  border: 1px solid rgb(var(--usage-ops-rgb) / 14%);
  background: rgb(var(--usage-ops-rgb) / 8%);
  padding: 0.32rem 0.72rem;
  color: var(--color-text-primary);
  font-size: 0.78rem;
  font-weight: 700;
}

.usage-ops__actions {
  display: flex;
  flex: none;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.5rem;
}

.usage-ops__action {
  border-radius: 999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 16%);
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
  padding: 0.56rem 0.85rem;
  color: var(--color-text-primary);
  font-size: 0.78rem;
  font-weight: 730;
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-ops__action:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--usage-ops-rgb) / 24%);
  background: rgb(var(--usage-ops-rgb) / 9%);
}

.usage-ops__action--primary {
  border-color: rgb(var(--usage-ops-rgb) / 24%);
  background: rgb(var(--usage-ops-rgb) / 14%);
}

.usage-ops__health-grid {
  display: grid;
  gap: 0.7rem;
  grid-template-columns: repeat(6, minmax(0, 1fr));
}

.usage-ops-health {
  --usage-ops-item-rgb: var(--color-info-rgb);

  position: relative;
  overflow: hidden;
  display: flex;
  min-width: 0;
  gap: 0.65rem;
  border-radius: 1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  background: rgb(var(--color-bg-surface-rgb) / 54%);
  padding: 0.72rem;
}

.usage-ops-health--success,
.usage-ops-source--success {
  --usage-ops-item-rgb: var(--color-success-rgb);
}

.usage-ops-health--warning,
.usage-ops-source--warning,
.usage-ops-alert--warning {
  --usage-ops-item-rgb: var(--color-warning-rgb);
}

.usage-ops-health--danger,
.usage-ops-source--danger {
  --usage-ops-item-rgb: var(--color-danger-rgb);
}

.usage-ops-health--info,
.usage-ops-source--info,
.usage-ops-alert--info {
  --usage-ops-item-rgb: var(--color-info-rgb);
}

.usage-ops-health--muted,
.usage-ops-source--muted {
  --usage-ops-item-rgb: var(--color-text-muted-rgb);
}

.usage-ops-health__axis {
  width: 3px;
  flex: none;
  border-radius: 999px;
  background: rgb(var(--usage-ops-item-rgb) / 74%);
}

.usage-ops-health strong {
  display: block;
  overflow: hidden;
  margin-top: 0.18rem;
  color: var(--color-text-primary);
  font-size: 0.98rem;
  font-weight: 740;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.usage-ops__lower {
  align-items: stretch;
  gap: 0.8rem;
}

.usage-ops__source-strip,
.usage-ops__alerts {
  min-width: 0;
  flex: 1;
  border-radius: 1.1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 11%);
  background: rgb(var(--color-bg-elevated-rgb) / 42%);
  padding: 0.8rem;
}

.usage-ops__section-head {
  align-items: baseline;
  justify-content: space-between;
  gap: 0.8rem;
  margin-bottom: 0.62rem;
}

.usage-ops__section-head span {
  color: var(--color-text-primary);
  font-size: 0.88rem;
  font-weight: 740;
}

.usage-ops__section-head small {
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 650;
}

.usage-ops-sources {
  display: grid;
  gap: 0.52rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.usage-ops-source,
.usage-ops-alert {
  --usage-ops-item-rgb: var(--color-info-rgb);

  border-radius: 0.9rem;
  border: 1px solid rgb(var(--usage-ops-item-rgb) / 14%);
  background: rgb(var(--usage-ops-item-rgb) / 7%);
  padding: 0.62rem 0.7rem;
}

.usage-ops-source span {
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
}

.usage-ops-source strong,
.usage-ops-alert strong {
  color: var(--color-text-primary);
  font-size: 0.82rem;
  font-weight: 740;
}

.usage-ops-source em {
  flex: none;
  border-radius: 999px;
  background: rgb(var(--usage-ops-item-rgb) / 12%);
  padding: 0.16rem 0.44rem;
  color: rgb(var(--usage-ops-item-rgb));
  font-size: 0.66rem;
  font-style: normal;
  font-weight: 780;
}

.usage-ops-source p,
.usage-ops-alert p {
  margin-top: 0.34rem;
}

.usage-ops__alerts {
  display: grid;
  gap: 0.52rem;
}

.usage-ops-alert {
  background: rgb(var(--usage-ops-item-rgb) / 8%);
}

.usage-ops__empty {
  border-radius: 0.9rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 14%);
  padding: 0.72rem;
}

@media (width < 1280px) {
  .usage-ops__health-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

@media (width < 980px) {
  .usage-ops__hero,
  .usage-ops__lower {
    flex-direction: column;
  }

  .usage-ops__actions {
    justify-content: flex-start;
  }
}

@media (width < 760px) {
  .usage-ops__health-grid,
  .usage-ops-sources {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
