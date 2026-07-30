<!-- 4 列统计条：当前 profile / 配置总数 / 平台特定列 / Health。
     平台差异通过 labels + secondary 列注入；样式靠视图的 --cp-* 继承换肤。 -->
<template>
  <div class="cp-stats">
    <div class="cp-stat surface-status">
      <div class="cp-stat__head">
        <span
          class="cp-stat__dot"
          :class="{ 'cp-stat__dot--good': Boolean(current) }"
        />
        {{ labels.current }}
      </div>
      <div class="cp-stat__value cp-stat__value--mono">
        {{ current || labels.notSet }}
      </div>
      <div class="cp-stat__hint">
        {{ labels.currentHint }}
      </div>
    </div>

    <div class="cp-stat surface-status">
      <div class="cp-stat__head">
        <SIcon
          name="Folder"
          size="w-3 h-3"
        />
        {{ labels.total }}
      </div>
      <div class="cp-stat__value cp-stat__value--mono">
        {{ total }}
      </div>
      <div class="cp-stat__hint">
        {{ labels.totalHint }}
      </div>
    </div>

    <div class="cp-stat surface-status">
      <div class="cp-stat__head">
        <SIcon
          :name="secondary.icon"
          size="w-3 h-3"
        />
        {{ secondary.title }}
      </div>
      <div
        class="cp-stat__value"
        :class="{ 'cp-stat__value--mono': secondary.mono }"
      >
        {{ secondary.value }}
      </div>
      <div class="cp-stat__hint">
        {{ secondary.hint }}
      </div>
    </div>

    <!-- 第四槽：Health（点击滚动到 Inspector Health 区） -->
    <button
      type="button"
      class="cp-stat cp-stat--clickable surface-status"
      :class="{ 'cp-stat--warn': health.warn }"
      @click="emit('healthClick')"
    >
      <div class="cp-stat__head">
        <SIcon
          :name="health.icon ?? 'ShieldCheck'"
          size="w-3 h-3"
        />
        {{ health.title }}
      </div>
      <div class="cp-stat__value cp-stat__value--mono">
        {{ health.value }}
      </div>
      <div class="cp-stat__hint">
        {{ health.hint }}
      </div>
    </button>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

export interface ProfilesStatStripLabels {
  current: string
  notSet: string
  currentHint: string
  total: string
  totalHint: string
}

/** 第三列（平台特定）：Claude=认证分布；Codex=配置模式 */
export interface ProfilesStatStripSecondary {
  icon: string
  title: string
  value: string
  hint: string
  /** value 是否用等宽字体（Claude 计数用 mono，Codex 文案不用） */
  mono?: boolean
}

/** 第四槽：Health（问题数） */
export interface ProfilesStatStripHealth {
  title: string
  value: string
  hint: string
  /** 缺省 'ShieldCheck' */
  icon?: string
  /** 有问题时 warn 高亮 */
  warn?: boolean
}

interface Props {
  current: string | null
  total: number
  labels: ProfilesStatStripLabels
  secondary: ProfilesStatStripSecondary
  health: ProfilesStatStripHealth
}

defineProps<Props>()

const emit = defineEmits<{
  (e: 'healthClick'): void
}>()
</script>

<style scoped>
.cp-stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  margin-bottom: 14px;
}

.cp-stat {
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 13px 16px;

  /* 背景/边框由 surface-status 工具类提供 */
  border-radius: 10px;
}

.cp-stat__head {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--cp-ink-3);
  font-size: 0.75rem;
  letter-spacing: 0.8px;
  text-transform: uppercase;
}

.cp-stat__dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: var(--cp-ink-4);
}

.cp-stat__dot--good { background: var(--cp-good); }

.cp-stat__value {
  color: var(--cp-ink-0);
  font-size: 1.1875rem;
  font-weight: 600;
  letter-spacing: -0.3px;
  word-break: break-all;
}

.cp-stat__value--mono { font-family: var(--cp-mono); }

.cp-stat__hint {
  color: var(--cp-ink-3);
  font-size: 0.75rem;
}

.cp-stat--clickable {
  font: inherit;
  text-align: left;
  cursor: pointer;
  transition: border-color 120ms ease;
}

.cp-stat--clickable:hover {
  border-color: var(--cp-accent-line);
}

.cp-stat--warn .cp-stat__head,
.cp-stat--warn .cp-stat__value {
  color: var(--cp-warn);
}

@media (width <= 1024px) {
  .cp-stats { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}

@media (prefers-reduced-motion: reduce) {
  .cp-stat--clickable { transition: none; }
}
</style>
