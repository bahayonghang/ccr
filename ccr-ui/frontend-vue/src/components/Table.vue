<template>
  <div class="overflow-x-auto rounded-xl glass-effect border border-border-color">
    <table class="min-w-full divide-y divide-border-color">
      <thead>
        <tr>
          <th
            v-for="column in columns"
            :key="column.key"
            class="px-6 py-4 text-left text-xs font-medium uppercase tracking-wider"
            :style="{ color: 'var(--text-secondary)' }"
          >
            {{ column.title }}
          </th>
          <th v-if="hasActions" class="px-6 py-4 text-right text-xs font-medium uppercase tracking-wider" :style="{ color: 'var(--text-secondary)' }">
            操作
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-border-color">
        <tr
          v-for="(item, index) in data"
          :key="item[keyField] || index"
          class="transition-colors hover:bg-bg-secondary"
        >
          <td
            v-for="column in columns"
            :key="column.key"
            class="px-6 py-4 whitespace-nowrap text-sm"
            :style="{ color: 'var(--text-primary)' }"
          >
            <slot
              :name="column.key"
              :item="item"
              :value="item[column.key]"
            >
              {{ item[column.key] }}
            </slot>
          </td>
          <td v-if="hasActions" class="px-6 py-4 whitespace-nowrap text-right text-sm">
            <slot name="actions" :item="item" />
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
interface Column {
  key: string;
  title: string;
  render?: (value: any, item: any) => string;
}

interface Props {
  columns: Column[];
  data: any[];
  keyField?: string;
  hasActions?: boolean;
}

withDefaults(defineProps<Props>(), {
  keyField: 'id',
  hasActions: false,
})
</script>