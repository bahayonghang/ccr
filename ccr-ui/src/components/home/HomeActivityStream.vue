<template>
  <section
    class="home-activity"
    data-home-activity
  >
    <header class="home-activity__header">
      <div>
        <p class="home-section-kicker">
          {{ t('home.activityEyebrow') }}
        </p>
        <h2>{{ t('home.activityTitle') }}</h2>
      </div>
      <RouterLink
        to="/monitoring"
        class="home-activity__link"
      >
        {{ t('home.activityOpenMonitoring') }}
        <SIcon
          name="ArrowRight"
          size="w-4 h-4"
        />
      </RouterLink>
    </header>

    <ol
      v-if="recentEntries.length > 0"
      class="home-activity__list"
    >
      <li
        v-for="entry in recentEntries"
        :key="entry.id"
        class="home-activity-entry"
        :data-level="entry.level"
      >
        <span class="home-activity-entry__dot" />
        <div class="home-activity-entry__body">
          <div class="home-activity-entry__meta">
            <span>{{ entry.channel }}</span>
            <span>{{ formatTime(entry.timestamp) }}</span>
          </div>
          <p>{{ entry.message }}</p>
        </div>
      </li>
    </ol>

    <div
      v-else
      class="home-activity__empty"
    >
      <SIcon
        name="History"
        size="w-5 h-5"
      />
      <div>
        <h3>{{ t('home.activityEmptyTitle') }}</h3>
        <p>{{ t('home.activityEmptyDescription') }}</p>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { MonitoringEntry } from '@/composables/useMonitoringFeed'

const props = withDefaults(defineProps<{
  entries: MonitoringEntry[]
  limit?: number
}>(), {
  limit: 6,
})

const { t } = useI18n()

const recentEntries = computed(() => {
  return [...props.entries]
    .sort((left, right) => new Date(right.timestamp).getTime() - new Date(left.timestamp).getTime())
    .slice(0, props.limit)
})

const formatTime = (timestamp: string) => {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return t('home.activityUnknownTime')

  return new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}
</script>

<style scoped>
.home-activity {
  display: flex;
  flex-direction: column;
  min-height: 100%;
  gap: 0.9rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 15%);
  border-radius: 14px;
  background: rgb(var(--color-bg-elevated-rgb) / 86%);
  padding: 1rem;
}

.home-activity__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.home-section-kicker {
  color: var(--color-text-muted);
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.home-activity h2,
.home-activity h3 {
  margin: 0;
  color: var(--color-text-primary);
  font-weight: 650;
  letter-spacing: 0;
}

.home-activity h2 {
  margin-top: 0.25rem;
  font-size: 1rem;
}

.home-activity__link {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--color-text-secondary);
  font-size: 0.76rem;
  font-weight: 650;
  white-space: nowrap;
}

.home-activity__link:hover,
.home-activity__link:focus-visible {
  color: var(--color-accent-primary);
}

.home-activity__list {
  display: grid;
  gap: 0.55rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

.home-activity-entry {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 0.65rem;
  align-items: flex-start;
  border: 1px solid rgb(var(--color-border-default-rgb) / 10%);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 64%);
  padding: 0.65rem;
}

.home-activity-entry__dot {
  width: 0.5rem;
  height: 0.5rem;
  margin-top: 0.35rem;
  border-radius: 999px;
  background: var(--color-accent-info);
}

.home-activity-entry[data-level='warn'] .home-activity-entry__dot {
  background: var(--accent-warning);
}

.home-activity-entry[data-level='error'] .home-activity-entry__dot {
  background: var(--color-danger);
}

.home-activity-entry[data-level='debug'] .home-activity-entry__dot {
  background: var(--color-text-muted);
}

.home-activity-entry__body {
  min-width: 0;
}

.home-activity-entry__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.7rem;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-size: 0.68rem;
  text-transform: uppercase;
}

.home-activity-entry p {
  display: -webkit-box;
  overflow: hidden;
  margin: 0.18rem 0 0;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  line-height: 1.5;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.home-activity__empty {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 22%);
  border-radius: 10px;
  color: var(--color-text-muted);
  padding: 0.9rem;
}

.home-activity__empty p {
  margin: 0.18rem 0 0;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  line-height: 1.5;
}
</style>
