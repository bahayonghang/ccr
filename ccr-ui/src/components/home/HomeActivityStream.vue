<template>
  <section
    class="home-activity"
    data-home-activity
  >
    <header class="home-activity__header">
      <div class="home-activity__lede">
        <p class="home-activity__eyebrow">
          {{ t('home.activityEyebrow') }}
        </p>
        <h2 class="home-activity__title">
          {{ t('home.activityTitle') }}
        </h2>
      </div>
      <div
        class="home-activity__filters"
        role="group"
        :aria-label="t('home.activityTitle')"
      >
        <button
          v-for="option in filterOptions"
          :key="option.id"
          type="button"
          class="home-activity-filter"
          :data-active="filter === option.id ? 'true' : 'false'"
          :aria-pressed="filter === option.id"
          @click="filter = option.id"
        >
          {{ option.label }}
          <span class="home-activity-filter__count">{{ option.count }}</span>
        </button>
      </div>
    </header>

    <ol
      v-if="visibleEntries.length > 0"
      class="home-activity__list"
    >
      <li
        v-for="entry in visibleEntries"
        :key="entry.id"
        class="home-activity-entry"
        :data-level="entry.level"
      >
        <span class="home-activity-entry__time">{{ formatTime(entry.timestamp) }}</span>
        <span
          class="home-activity-entry__dot"
          :aria-label="entry.level"
        />
        <span class="home-activity-entry__channel">{{ entry.channel }}</span>
        <p
          class="home-activity-entry__message"
          :title="entry.message"
        >
          {{ entry.message }}
        </p>
      </li>
    </ol>

    <div
      v-else
      class="home-activity__empty"
    >
      <span class="home-activity__empty-icon">
        <SIcon
          name="History"
          size="w-5 h-5"
        />
      </span>
      <div class="home-activity__empty-copy">
        <h3>{{ t('home.activityEmptyTitle') }}</h3>
        <p>{{ t('home.activityEmptyDescription') }}</p>
      </div>
      <RouterLink
        to="/monitoring"
        class="home-activity__empty-cta"
      >
        {{ t('home.activityViewAll') }}
        <SIcon
          name="ArrowRight"
          size="w-4 h-4"
        />
      </RouterLink>
    </div>

    <footer
      v-if="visibleEntries.length > 0"
      class="home-activity__footer"
    >
      <RouterLink to="/monitoring">
        {{ t('home.activityOpenMonitoring') }}
        <SIcon
          name="ArrowRight"
          size="w-4 h-4"
        />
      </RouterLink>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
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

type FilterId = 'all' | 'warn' | 'error'

const filter = ref<FilterId>('all')

const sortedEntries = computed(() => {
  return [...props.entries].sort((left, right) => new Date(right.timestamp).getTime() - new Date(left.timestamp).getTime())
})

const matchesFilter = (entry: MonitoringEntry, id: FilterId) => {
  if (id === 'all') return true
  if (id === 'warn') return entry.level === 'warn' || entry.level === 'error'
  return entry.level === 'error'
}

const filteredEntries = computed(() => sortedEntries.value.filter((entry) => matchesFilter(entry, filter.value)))

const visibleEntries = computed(() => filteredEntries.value.slice(0, props.limit))

const filterOptions = computed(() => ([
  { id: 'all' as const, label: t('home.activityFilterAll'), count: sortedEntries.value.length },
  { id: 'warn' as const, label: t('home.activityFilterWarn'), count: sortedEntries.value.filter((entry) => matchesFilter(entry, 'warn')).length },
  { id: 'error' as const, label: t('home.activityFilterError'), count: sortedEntries.value.filter((entry) => matchesFilter(entry, 'error')).length },
]))

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
  height: 100%;
  gap: 0.75rem;
  padding: var(--home-card-pad);
  border: 1px solid var(--home-border-card);
  border-radius: var(--home-card-radius);
  background: var(--home-surface-card);
  box-shadow: var(--home-elevation-raised);
}

.home-activity__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
}

.home-activity__lede {
  display: grid;
  gap: 0.18rem;
}

.home-activity__eyebrow {
  margin: 0;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.home-activity__title {
  margin: 0;
  color: var(--color-text-primary);
  font-family: var(--font-brand);
  font-size: var(--home-text-section);
  font-weight: 620;
  letter-spacing: var(--home-tracking-display);
}

.home-activity__filters {
  display: inline-flex;
  gap: 0.25rem;
  padding: 0.18rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 999px;
  background: var(--home-surface-sunk);
  box-shadow: var(--home-elevation-sunk);
}

.home-activity-filter {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.22rem 0.65rem;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
  transition:
    background-color var(--home-motion-duration) var(--home-motion-ease),
    color var(--home-motion-duration) var(--home-motion-ease);
}

.home-activity-filter:hover {
  color: var(--color-text-primary);
}

.home-activity-filter:focus-visible {
  outline: 0;
  box-shadow: var(--home-focus-ring);
}

.home-activity-filter[data-active='true'] {
  background: rgb(var(--color-accent-primary-rgb) / 14%);
  color: var(--color-accent-primary);
}

.home-activity-filter__count {
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-meta);
  font-weight: 700;
}

.home-activity-filter[data-active='true'] .home-activity-filter__count {
  color: var(--color-accent-primary);
}

.home-activity__list {
  display: grid;
  gap: 0.2rem;
  margin: 0;
  padding: 0;
  list-style: none;
  flex: 1;
  min-height: 0;
}

.home-activity-entry {
  display: grid;
  grid-template-columns: auto auto auto minmax(0, 1fr);
  align-items: center;
  gap: 0.55rem;
  padding: 0.4rem 0.55rem;
  border-radius: 8px;
  transition: background-color var(--home-motion-duration) var(--home-motion-ease);
}

.home-activity-entry:hover {
  background: rgb(var(--color-border-default-rgb) / 6%);
}

.home-activity-entry__time {
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: 0.01em;
  min-width: 2.8rem;
}

.home-activity-entry__dot {
  width: 0.42rem;
  height: 0.42rem;
  border-radius: 999px;
  background: var(--color-info);
}

.home-activity-entry[data-level='warn'] .home-activity-entry__dot {
  background: var(--color-warning);
}

.home-activity-entry[data-level='error'] .home-activity-entry__dot {
  background: var(--color-danger);
}

.home-activity-entry[data-level='debug'] .home-activity-entry__dot {
  background: var(--color-text-disabled);
}

.home-activity-entry__channel {
  padding: 0.1rem 0.4rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 4px;
  background: rgb(var(--color-bg-surface-rgb) / 68%);
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: 0.625rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.home-activity-entry__message {
  overflow: hidden;
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-body);
  letter-spacing: var(--home-tracking-body);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.home-activity__empty {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.85rem;
  flex: 1;
  min-height: 7rem;
  padding: 1rem;
  border: 1px dashed var(--home-border-hairline);
  border-radius: 10px;
}

.home-activity__empty-icon {
  display: grid;
  place-items: center;
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 999px;
  background: rgb(var(--color-bg-surface-rgb) / 70%);
  color: var(--color-text-muted);
}

.home-activity__empty-copy h3 {
  margin: 0;
  color: var(--color-text-primary);
  font-size: var(--home-text-body);
  font-weight: 600;
}

.home-activity__empty-copy p {
  margin: 0.15rem 0 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
  line-height: var(--home-leading-body);
}

.home-activity__empty-cta {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.4rem 0.7rem;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 36%);
  border-radius: 999px;
  background: rgb(var(--color-accent-primary-rgb) / 8%);
  color: var(--color-accent-primary);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-decoration: none;
  text-transform: uppercase;
  transition:
    background-color var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.home-activity__empty-cta:hover {
  background: rgb(var(--color-accent-primary-rgb) / 16%);
  transform: translateX(2px);
}

.home-activity__empty-cta:focus-visible {
  outline: 0;
  box-shadow: var(--home-focus-ring);
}

.home-activity__footer {
  display: flex;
  justify-content: flex-end;
  padding-top: 0.4rem;
  border-top: 1px solid var(--home-border-hairline);
}

.home-activity__footer a {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-decoration: none;
  text-transform: uppercase;
  transition: color var(--home-motion-duration) var(--home-motion-ease);
}

.home-activity__footer a:hover,
.home-activity__footer a:focus-visible {
  color: var(--color-accent-primary);
  outline: 0;
}

@media (width <= 720px) {
  .home-activity__header {
    flex-direction: column;
  }

  .home-activity__empty {
    grid-template-columns: 1fr;
    text-align: left;
  }
}

@media (prefers-reduced-motion: reduce) {
  .home-activity-entry,
  .home-activity-filter,
  .home-activity__empty-cta,
  .home-activity__footer a {
    transition: none;
  }
}
</style>
