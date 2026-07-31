<!-- 三行字段 diff（label / 当前值 → 目标值）：Inspector 预览与 Apply 确认框共用。
     相同行弱化、不同行用陶土 accent 强调；缺失值渲染统一占位符。
     token 走 --cp-* 并带回退（确认框经 Teleport 脱离 --cp-* 作用域时回退到同值全局 token）。 -->
<template>
  <ul class="cp-diff-rows">
    <li
      v-for="row in rows"
      :key="row.key"
      class="cp-diff-row"
      :class="{ 'cp-diff-row--changed': row.changed }"
    >
      <span class="cp-diff-row__label">{{ row.label }}</span>
      <span class="cp-diff-row__values">
        <span
          class="cp-diff-row__value cp-diff-row__value--from"
          :class="{ 'cp-diff-row__value--empty': row.from === null }"
        >{{ row.from ?? placeholder }}</span>
        <span
          class="cp-diff-row__arrow"
          aria-hidden="true"
        >→</span>
        <span
          class="cp-diff-row__value cp-diff-row__value--to"
          :class="{ 'cp-diff-row__value--empty': row.to === null }"
        >{{ row.to ?? placeholder }}</span>
      </span>
    </li>
  </ul>
</template>

<script setup lang="ts">
import type { ProfileDiffRow } from '@/utils/profileDiff'

interface Props {
  rows: ProfileDiffRow[]
  /** 缺失值占位符（'—' / '未设置' 由调用方按语境决定） */
  placeholder?: string
}

withDefaults(defineProps<Props>(), {
  placeholder: '—',
})
</script>

<style scoped>
.cp-diff-rows {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.cp-diff-row {
  display: grid;
  grid-template-columns: minmax(4.5rem, auto) minmax(0, 1fr);
  align-items: baseline;
  gap: 0.5rem;
  padding: 0.3125rem 0.5rem;
  border-radius: 6px;
  border: 1px solid transparent;
}

.cp-diff-row--changed {
  background: var(--cp-accent-soft, rgb(var(--color-accent-primary-rgb) / 14%));
  border-color: var(--cp-accent-line, rgb(var(--color-accent-primary-rgb) / 35%));
}

.cp-diff-row__label {
  font-family: var(--cp-mono, var(--font-mono));
  font-size: 0.75rem;
  letter-spacing: 0.0625rem;
  text-transform: uppercase;
  color: var(--cp-ink-3, var(--color-text-ghost));
  white-space: nowrap;
}

.cp-diff-row__values {
  display: flex;
  align-items: baseline;
  gap: 0.375rem;
  min-width: 0;
  font-family: var(--cp-mono, var(--font-mono));
  font-size: 0.75rem;
}

.cp-diff-row__value {
  min-width: 0;
  overflow-wrap: anywhere;
  color: var(--cp-ink-2, var(--color-text-muted));
}

.cp-diff-row__value--to {
  color: var(--cp-ink-0, var(--color-text-primary));
}

.cp-diff-row__value--empty {
  color: var(--cp-ink-4, var(--color-text-disabled));
}

.cp-diff-row--changed .cp-diff-row__value--to {
  color: var(--cp-accent, var(--color-accent-primary));
  font-weight: 600;
}

.cp-diff-row__arrow {
  flex-shrink: 0;
  color: var(--cp-ink-4, var(--color-text-disabled));
}

.cp-diff-row--changed .cp-diff-row__arrow {
  color: var(--cp-accent, var(--color-accent-primary));
}
</style>
