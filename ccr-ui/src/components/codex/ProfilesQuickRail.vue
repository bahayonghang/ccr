<!-- 快速切换条：仅展示已启用 profile，附带 ⌘+数字键提示 -->
<template>
  <div
    v-if="enabledProfiles.length > 0"
    class="cp-rail surface-workspace"
  >
    <div class="cp-rail__head">
      <SIcon
        name="Sparkles"
        size="w-3.5 h-3.5"
        class="cp-rail__head-icon"
      />
      {{ $t('codex.profiles.quickSwitch') }}
    </div>
    <div class="cp-rail__list">
      <button
        v-for="(profile, index) in enabledProfiles"
        :key="profile.name"
        type="button"
        class="cp-chip"
        :class="{
          'cp-chip--active': profile.name === currentName,
          'cp-chip--busy': busyName === profile.name,
        }"
        :disabled="disabled"
        :aria-pressed="profile.name === currentName"
        :title="profile.description || profile.name"
        @click="emit('apply', profile.name)"
      >
        <span
          class="cp-chip__dot"
          :class="{ 'cp-chip__dot--off': profile.name !== currentName }"
        />
        <span class="cp-chip__name">{{ profile.name }}</span>
        <span
          v-if="index < 9"
          class="cp-chip__kbd"
        >{{ index + 1 }}</span>
      </button>
    </div>
    <div class="cp-rail__hint">
      {{ $t('codex.profiles.quickRailHint') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { CodexProfile } from '@/types'

interface Props {
  profiles: CodexProfile[]
  currentName: string | null
  disabled?: boolean
  busyName?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  busyName: null,
})

const emit = defineEmits<{
  (e: 'apply', name: string): void
}>()

const enabledProfiles = computed(() =>
  props.profiles.filter(p => p.enabled !== false),
)
</script>

<style scoped>
.cp-rail {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  margin-bottom: 14px;

  /* 背景/边框由 surface-workspace 工具类提供 */
  border-radius: 12px;
}

.cp-rail__head {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  color: var(--cp-ink-2);
  font-size: 12px;
  font-weight: 600;
}

.cp-rail__head-icon { color: var(--cp-accent); }

.cp-rail__list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.cp-rail__hint {
  flex-shrink: 0;
  color: var(--cp-ink-4);
  font-size: 10.5px;
  font-family: var(--cp-mono);
}

.cp-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  border-radius: 6px;
  border: 1px solid var(--cp-line-2);
  background: var(--cp-bg-2);
  color: var(--cp-ink-1);
  font-family: var(--cp-mono);
  font-size: 12px;
  cursor: pointer;
  transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
}

.cp-chip:hover:not(:disabled, .cp-chip--active) {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-chip:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.cp-chip--active {
  background: var(--cp-accent);
  border-color: var(--cp-accent);
  color: var(--cp-on-accent);
  font-weight: 600;
}

.cp-chip--busy { opacity: 0.7; }

.cp-chip__dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--cp-good);
  flex-shrink: 0;
}

.cp-chip__dot--off { background: var(--cp-ink-4); }

.cp-chip--active .cp-chip__dot { background: var(--cp-on-accent); }

.cp-chip__name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.cp-chip__kbd {
  margin-left: 2px;
  padding: 0 4px;
  border-radius: 3px;
  background: rgb(0 0 0 / 25%);
  color: inherit;
  font-size: 10px;
  opacity: 0.65;
}

.cp-chip--active .cp-chip__kbd {
  background: rgb(0 0 0 / 18%);
  opacity: 0.8;
}

@media (prefers-reduced-motion: reduce) {
  .cp-chip { transition: none; }
}
</style>
