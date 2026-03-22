<template>
  <div class="glass-panel overflow-hidden rounded-xl">
    <table class="w-full text-sm">
      <thead>
        <tr class="border-b border-border-subtle text-left text-text-muted">
          <th class="p-3">
            {{ $t('usage.dashboard.table.project') }}
          </th>
          <th class="p-3 text-right">
            {{ $t('usage.dashboard.table.requests') }}
          </th>
          <th class="p-3 text-right">
            {{ $t('usage.dashboard.table.tokens') }}
          </th>
          <th class="p-3 text-right">
            {{ $t('usage.dashboard.table.cost') }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="project in projectStats"
          :key="project.project_path"
          class="border-b border-border-subtle/50 transition-colors hover:bg-accent-primary/5"
        >
          <td
            class="max-w-xs truncate p-3 font-medium text-text-primary"
            :title="project.project_path"
          >
            {{ shortenPath(project.project_path) }}
          </td>
          <td class="p-3 text-right text-text-secondary">
            {{ project.request_count }}
          </td>
          <td class="p-3 text-right text-text-secondary">
            {{ formatTokens(project.total_tokens) }}
          </td>
          <td class="p-3 text-right text-text-secondary">
            {{ formatCost(project.total_cost) }}
          </td>
        </tr>
      </tbody>
    </table>
    <div
      v-if="!projectStats.length"
      class="p-6 text-center text-sm text-text-muted"
    >
      {{ $t('usage.dashboard.table.noData') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ProjectStat } from '@/types/usage'

interface Props {
  projectStats: ProjectStat[]
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
  shortenPath: (path: string) => string
}

defineProps<Props>()
</script>
