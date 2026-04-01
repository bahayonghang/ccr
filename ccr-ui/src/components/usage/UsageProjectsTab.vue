<template>
  <section class="projects-tab glass-panel rounded-[26px] p-4">
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
      class="projects-tab__table-shell"
    >
      <table class="projects-tab__table">
        <colgroup>
          <col class="projects-tab__col projects-tab__col--rank">
          <col class="projects-tab__col projects-tab__col--project">
          <col class="projects-tab__col projects-tab__col--requests">
          <col class="projects-tab__col projects-tab__col--tokens">
          <col class="projects-tab__col projects-tab__col--cost">
          <col class="projects-tab__col projects-tab__col--share">
        </colgroup>
        <thead>
          <tr>
            <th>#</th>
            <th>{{ $t('usage.dashboard.table.project') }}</th>
            <th class="is-right">
              {{ $t('usage.dashboard.table.requests') }}
            </th>
            <th class="is-right">
              {{ $t('usage.dashboard.table.tokens') }}
            </th>
            <th class="is-right">
              {{ $t('usage.dashboard.table.cost') }}
            </th>
            <th class="is-right">
              {{ $t('usage.dashboard.table.share') }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(project, index) in sortedProjects"
            :key="project.project_path"
          >
            <td class="projects-tab__rank-cell">
              {{ index + 1 }}
            </td>
            <td>
              <div
                class="projects-tab__project-name"
                :title="project.project_path"
              >
                {{ shortenPath(project.project_path) }}
              </div>
              <div class="projects-tab__project-path">
                {{ project.project_path }}
              </div>
            </td>
            <td class="is-right">
              {{ project.request_count.toLocaleString() }}
            </td>
            <td class="is-right">
              {{ formatTokens(project.total_tokens) }}
            </td>
            <td class="is-right">
              {{ formatCost(project.total_cost) }}
            </td>
            <td class="is-right">
              {{ formatShare(project.total_cost) }}
            </td>
          </tr>
        </tbody>
      </table>
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
import type { ProjectStat } from '@/types/usage'

interface Props {
  projectStats: ProjectStat[]
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
  shortenPath: (path: string) => string
}

const props = defineProps<Props>()

const totalCost = computed(() =>
  props.projectStats.reduce((sum, item) => sum + item.total_cost, 0),
)

const sortedProjects = computed(() =>
  [...props.projectStats].sort((left, right) =>
    right.total_cost - left.total_cost ||
    right.total_tokens - left.total_tokens ||
    right.request_count - left.request_count,
  ),
)

const formatShare = (value: number) => {
  if (totalCost.value <= 0) return '0%'
  return `${Math.round((value / totalCost.value) * 100)}%`
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

.projects-tab__table-shell {
  max-height: 38rem;
  overflow: auto;
  border-radius: 1.15rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 44%);
}

.projects-tab__table {
  min-width: 64rem;
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
}

.projects-tab__table thead th {
  position: sticky;
  top: 0;
  z-index: 1;
  padding: 0.85rem 1rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 94%);
  color: var(--color-text-muted);
  font-size: 0.74rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-align: left;
  text-transform: uppercase;
}

.projects-tab__table tbody td {
  padding: 0.92rem 1rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  color: var(--color-text-secondary);
  font-size: 0.9rem;
  font-variant-numeric: tabular-nums;
  vertical-align: top;
}

.projects-tab__table tbody tr:hover {
  background: rgb(var(--color-accent-primary-rgb) / 6%);
}

.projects-tab__rank-cell {
  color: var(--color-text-primary);
  font-weight: 700;
}

.projects-tab__project-name {
  overflow: hidden;
  color: var(--color-text-primary);
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.projects-tab__project-path {
  margin-top: 0.2rem;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 0.76rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.projects-tab__table .is-right {
  text-align: right;
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

.projects-tab__col--rank {
  width: 4rem;
}

.projects-tab__col--project {
  width: 32rem;
}

.projects-tab__col--requests,
.projects-tab__col--tokens,
.projects-tab__col--cost,
.projects-tab__col--share {
  width: 9rem;
}
</style>
