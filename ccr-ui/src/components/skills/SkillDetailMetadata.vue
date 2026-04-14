<template>
  <div class="metadata-section">
    <p
      v-if="description"
      class="text-sm leading-relaxed text-text-primary"
    >
      {{ description }}
    </p>

    <div class="meta-grid">
      <div
        v-for="item in items"
        :key="item.id"
        class="meta-item"
      >
        <SIcon
          :name="item.icon"
          size="w-3.5 h-3.5"
          class="shrink-0"
          :class="item.iconColor ? undefined : 'text-text-muted'"
          :style="item.iconColor ? { color: item.iconColor } : undefined"
        />
        <span class="meta-label">{{ item.label }}</span>
        <span
          class="meta-value"
          :class="{ 'font-mono text-[11px]': item.monospace }"
          :title="item.valueTitle"
        >
          {{ item.value }}
          <a
            v-if="item.linkUrl"
            :href="item.linkUrl"
            target="_blank"
            rel="noopener noreferrer"
            class="ml-1 inline-flex text-accent-primary hover:underline"
            @click.stop
          >
            <SIcon
              name="ExternalLink"
              size="w-3 h-3"
            />
          </a>
        </span>
      </div>
    </div>

    <div
      v-if="tags.length > 0"
      class="mt-2 flex flex-wrap gap-1"
    >
      <span
        v-for="tag in tags"
        :key="tag"
        class="tag-badge"
      >
        #{{ tag }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { SkillDetailMetaItem } from '@/types/skillDetailModal'

defineProps<{
  description?: string
  items: SkillDetailMetaItem[]
  tags: string[]
}>()
</script>

<style scoped>
.metadata-section {
  @apply rounded-xl border border-border-default/10 p-4 space-y-3;

  background: rgb(0 0 0 / 20%);
}

.meta-grid {
  @apply grid grid-cols-1 gap-2 sm:grid-cols-3;
}

.meta-item {
  @apply flex items-center gap-2 text-sm;
}

.meta-label {
  @apply text-xs text-text-muted;
}

.meta-value {
  @apply text-xs font-medium text-white;
}

.tag-badge {
  @apply px-2 py-0.5 rounded-md text-[10px] font-medium glass-surface text-text-muted;
}
</style>

