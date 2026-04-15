<template>
  <div
    class="agent-icons"
    :class="{ 'agent-icons--compact': compact }"
  >
    <span
      v-for="agent in visibleAgents"
      :key="agent.id"
      class="agent-icons__chip"
      :style="{ '--agent-color': agent.color }"
      :title="agent.label"
    >
      <SIcon
        :name="agent.icon"
        size="w-3 h-3"
      />
      <span
        v-if="!compact"
        class="agent-icons__label"
      >{{ agent.label }}</span>
    </span>
    <span
      v-if="overflowCount > 0"
      class="agent-icons__overflow"
      :title="`${overflowCount} more agent(s)`"
    >
      +{{ overflowCount }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'

/** agent 元数据定义 */
const AGENT_META: Record<string, { label: string; icon: string; color: string }> = {
  claude: { label: 'Claude', icon: 'Code2', color: 'var(--color-platform-claude, #c96442)' },
  codex: { label: 'Codex', icon: 'Settings', color: 'var(--color-platform-codex, #10a37f)' },
  gemini: { label: 'Gemini', icon: 'Sparkles', color: 'var(--color-platform-gemini, #4285f4)' },
  droid: { label: 'Droid', icon: 'Bot', color: 'var(--color-platform-droid, #8b5cf6)' },
  opencode: { label: 'OpenCode', icon: 'TerminalSquare', color: 'var(--color-platform-opencode, #6b7280)' },
}

const props = withDefaults(defineProps<{
  /** agent ID 列表，例如 ['claude', 'codex'] */
  agents: string[]
  /** 紧凑模式：仅显示图标，不显示文字 */
  compact?: boolean
  /** 最多显示数量，超出部分用 +N 表示 */
  maxVisible?: number
}>(), {
  compact: true,
  maxVisible: 4,
})

interface ResolvedAgent {
  id: string
  label: string
  icon: string
  color: string
}

const resolvedAgents = computed<ResolvedAgent[]>(() =>
  props.agents
    .map((id) => {
      const meta = AGENT_META[id]
      if (!meta) return null
      return { id, ...meta }
    })
    .filter((a): a is ResolvedAgent => a !== null),
)

const visibleAgents = computed(() =>
  resolvedAgents.value.slice(0, props.maxVisible),
)

const overflowCount = computed(() =>
  Math.max(0, resolvedAgents.value.length - props.maxVisible),
)
</script>

<style scoped>
.agent-icons {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  flex-shrink: 0;
}

.agent-icons__chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.125rem 0.375rem;
  border-radius: 0.375rem;
  font-size: 0.6875rem;
  font-weight: 500;
  color: var(--agent-color);
  background: color-mix(in srgb, var(--agent-color) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--agent-color) 20%, transparent);
  line-height: 1.4;
  transition:
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.agent-icons--compact .agent-icons__chip {
  padding: 0.1875rem;
  border-radius: 0.3125rem;
}

.agent-icons__label {
  white-space: nowrap;
}

.agent-icons__overflow {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.25rem;
  height: 1.25rem;
  padding: 0 0.25rem;
  border-radius: 0.3125rem;
  font-size: 0.625rem;
  font-weight: 600;
  color: var(--color-text-muted);
  background: rgb(var(--color-bg-base-rgb) / 68%);
}
</style>
