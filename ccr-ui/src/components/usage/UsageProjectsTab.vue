<template>
  <section class="projects-tab glass-panel rounded-xl p-4">
    <div class="projects-tab__table-head">
      <div>
        <h3 class="projects-tab__title">
          {{ $t('usage.dashboard.projects.title') }}
        </h3>
        <p class="projects-tab__subtitle">
          {{ $t('usage.dashboard.projects.subtitle') }}
        </p>
      </div>
    </div>

    <div
      v-if="sortedProjects.length > 0"
      class="projects-tab__list"
    >
      <article
        v-for="(project, index) in sortedProjects"
        :key="project.project_path"
        class="projects-tab__item"
      >
        <span class="projects-tab__rank-cell">{{ index + 1 }}</span>

        <div class="projects-tab__project-copy">
          <div class="projects-tab__project-line">
            <strong
              class="projects-tab__project-name"
              :title="project.project_path"
            >
              {{ projectName(project.project_path) }}
            </strong>
            <span class="projects-tab__project-cost">
              {{ ctx.formatCost(project.total_cost) }}
            </span>
          </div>

          <div class="projects-tab__project-path">
            {{ project.project_path }}
          </div>

          <div class="projects-tab__metrics">
            <span>{{ ctx.formatTokens(project.total_tokens) }} {{ $t('usage.dashboard.table.tokens') }}</span>
            <span>{{ project.request_count.toLocaleString() }} {{ $t('usage.dashboard.table.requests') }}</span>
            <span>{{ formatShare(project.total_cost) }} {{ $t('usage.dashboard.table.share') }}</span>
          </div>

          <div
            class="projects-tab__progress"
            :aria-label="$t('usage.dashboard.projects.shareProgress')"
          >
            <span :style="{ width: `${Math.max(costShare(project.total_cost) * 100, 4)}%` }" />
          </div>
        </div>
      </article>
    </div>

    <div
      v-else
      class="projects-tab__empty"
    >
      {{ $t('usage.dashboard.table.noData') }}
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useUsageDashboardContext } from '@/views/usage/usageDashboardContext'

const ctx = useUsageDashboardContext()

const totalCost = computed(() =>
  ctx.projectStats.reduce((sum, item) => sum + item.total_cost, 0),
)

const sortedProjects = computed(() =>
  [...ctx.projectStats].sort((left, right) =>
    right.total_cost - left.total_cost ||
    right.total_tokens - left.total_tokens ||
    right.request_count - left.request_count,
  ),
)

const formatShare = (value: number) => {
  if (totalCost.value <= 0) return '0%'
  return `${Math.round((value / totalCost.value) * 100)}%`
}

const costShare = (value: number) => (totalCost.value > 0 ? value / totalCost.value : 0)

const projectName = (path: string) => {
  const normalized = path.replace(/\\/g, '/')
  const parts = normalized.split('/').filter(Boolean)
  return parts.at(-1) ?? ctx.shortenPath(path)
}
</script>

<style scoped>
.projects-tab {
  display: grid;
  gap: 1rem;
  overflow: hidden;
}

.projects-tab__table-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.projects-tab__title {
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 650;
}

.projects-tab__subtitle {
  margin-top: 0.35rem;
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  line-height: 1.6;
}

.projects-tab__list {
  display: grid;
  gap: 0.7rem;
  max-height: 38rem;
  overflow: auto;
  padding-right: 0.15rem;
}

.projects-tab__item {
  display: grid;
  grid-template-columns: 2.4rem minmax(0, 1fr);
  gap: 0.78rem;
  align-items: start;
  border-radius: 1.18rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 50%), rgb(var(--color-bg-surface-rgb) / 26%)),
    radial-gradient(circle at 100% 0%, rgb(var(--color-accent-secondary-rgb) / 8%), transparent 42%);
  padding: 0.82rem 0.9rem;
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.projects-tab__item:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
}

.projects-tab__rank-cell {
  display: inline-flex;
  width: 2rem;
  height: 2rem;
  align-items: center;
  justify-content: center;
  border-radius: 0.82rem;
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-text-primary);
  font-size: 0.82rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.projects-tab__project-copy {
  display: grid;
  min-width: 0;
  gap: 0.34rem;
}

.projects-tab__project-line {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.projects-tab__project-name {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 0.95rem;
  font-weight: 680;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.projects-tab__project-cost {
  flex-shrink: 0;
  color: var(--color-text-primary);
  font-size: 0.98rem;
  font-weight: 720;
  font-variant-numeric: tabular-nums;
}

.projects-tab__project-path {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 0.76rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.projects-tab__metrics {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem 0.8rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  font-variant-numeric: tabular-nums;
}

.projects-tab__progress {
  height: 0.42rem;
  overflow: hidden;
  border-radius: 9999px;
  background: rgb(var(--color-border-default-rgb) / 18%);
}

.projects-tab__progress span {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, rgb(var(--color-accent-primary-rgb) / 86%), rgb(var(--color-info-rgb) / 82%));
}

.projects-tab__empty {
  display: flex;
  min-height: 16rem;
  align-items: center;
  justify-content: center;
  border-radius: 1.2rem;
  border: 1px dashed rgb(var(--color-accent-primary-rgb) / 16%);
  color: var(--color-text-muted);
}

@media (width < 720px) {
  .projects-tab__item {
    grid-template-columns: minmax(0, 1fr);
  }

  .projects-tab__project-line {
    align-items: flex-start;
    flex-direction: column;
    gap: 0.3rem;
  }
}
</style>
