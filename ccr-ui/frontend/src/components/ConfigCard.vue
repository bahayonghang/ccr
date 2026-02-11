<template>
  <div
    v-bind="{ 'data-config-name': config.name }"
    class="config-row group relative flex items-stretch transition-all duration-300 rounded-xl overflow-hidden"
    :class="{
      'config-row--current': config.is_current,
      'config-row--disabled': config.enabled === false,
    }"
    @click="$emit('edit', config.name)"
  >
    <!-- Left Accent Bar -->
    <div
      class="accent-bar w-[3px] shrink-0 transition-all duration-300"
      :class="accentBarClass"
    />

    <!-- Main Content -->
    <div class="flex-1 flex items-center gap-4 px-5 py-4 min-w-0">
      <!-- Provider Avatar -->
      <div
        class="avatar-wrapper relative shrink-0 w-10 h-10 rounded-xl flex items-center justify-center text-sm font-bold text-white shadow-md transition-transform duration-300 group-hover:scale-105"
        :class="avatarClass"
      >
        {{ config.provider?.[0]?.toUpperCase() || 'C' }}
        <!-- Active Indicator Dot -->
        <span
          v-if="config.is_current"
          class="absolute -top-1 -right-1 w-3 h-3 rounded-full bg-emerald-400 border-2 border-bg-elevated shadow-sm"
        >
          <span class="absolute inset-0 rounded-full bg-emerald-400 animate-ping opacity-60" />
        </span>
      </div>

      <!-- Info Block -->
      <div class="flex-1 min-w-0 space-y-1.5">
        <!-- Top Row: Name + Status -->
        <div class="flex items-center gap-2.5 min-w-0">
          <h3
            class="text-sm font-bold font-display truncate transition-colors duration-300"
            :class="nameColorClass"
          >
            {{ config.name }}
          </h3>

          <!-- Status Badges -->
          <div class="flex gap-1.5 shrink-0">
            <span
              v-if="config.is_current"
              class="inline-flex items-center gap-1 text-[10px] font-bold uppercase tracking-wider text-emerald-400 bg-emerald-400/10 px-2 py-0.5 rounded-full ring-1 ring-emerald-400/20"
            >
              Active
            </span>
            <span
              v-if="config.is_default"
              class="inline-flex items-center text-[10px] font-bold uppercase tracking-wider text-amber-400 bg-amber-400/10 px-2 py-0.5 rounded-full ring-1 ring-amber-400/20"
            >
              Default
            </span>
          </div>
        </div>

        <!-- Description -->
        <p class="text-xs text-text-secondary truncate leading-relaxed">
          {{ config.description || 'No description provided.' }}
        </p>
      </div>

      <!-- Meta Chips -->
      <div class="hidden md:flex items-center gap-2 shrink-0">
        <!-- Model -->
        <div
          v-if="config.model"
          class="meta-chip"
          :title="config.model"
        >
          <Sparkles class="w-3 h-3 text-accent-primary opacity-70" />
          <span class="truncate max-w-[120px]">{{ config.model }}</span>
        </div>

        <!-- Provider -->
        <div
          v-if="config.provider"
          class="meta-chip"
        >
          <Building2 class="w-3 h-3 opacity-50" />
          <span>{{ config.provider }}</span>
        </div>

        <!-- Calls Count -->
        <div
          v-if="(config.usage_count ?? 0) > 0"
          class="meta-chip"
        >
          <TrendingUp class="w-3 h-3 text-accent-secondary opacity-70" />
          <span class="font-bold text-accent-secondary">{{ config.usage_count }}</span>
        </div>
      </div>
    </div>

    <!-- Right Action Area -->
    <div class="flex items-center gap-2 pr-4 shrink-0">
      <!-- Switch Button -->
      <button
        v-if="!config.is_current"
        class="switch-btn px-3.5 py-1.5 rounded-lg text-xs font-bold transition-all duration-200 opacity-0 group-hover:opacity-100 focus:opacity-100"
        @click.stop="$emit('switch', config.name)"
      >
        <ArrowRightLeft class="w-3.5 h-3.5 mr-1 inline-block" />
        Switch
      </button>
      <span
        v-else
        class="inline-flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-bold text-emerald-400/70 bg-emerald-400/5 cursor-default"
      >
        <CheckCircle class="w-3.5 h-3.5" />
        In Use
      </span>

      <!-- Edit Button -->
      <button
        class="edit-btn p-2 rounded-lg transition-all duration-200 opacity-0 group-hover:opacity-100 focus:opacity-100"
        :class="config.is_current ? 'opacity-60' : ''"
        @click.stop="$emit('edit', config.name)"
      >
        <Settings class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  Settings, Building2, ArrowRightLeft, CheckCircle,
  Sparkles, TrendingUp
} from 'lucide-vue-next'
import type { ConfigItem } from '@/types'

interface Props {
  config: ConfigItem
}

const props = defineProps<Props>()

defineEmits<{
  switch: [name: string]
  edit: [name: string]
  delete: [name: string]
  enable: [name: string]
  disable: [name: string]
}>()

// Provider type detection
const providerType = computed(() => {
  const type = props.config.provider_type?.toLowerCase() || ''
  if (type.includes('official')) return 'official'
  if (type.includes('third')) return 'third'
  return 'uncategorized'
})

// Left accent bar color by provider type
const accentBarClass = computed(() => {
  const map = {
    official: 'bg-cyan-400 group-hover:bg-cyan-300',
    third: 'bg-violet-400 group-hover:bg-violet-300',
    uncategorized: 'bg-amber-400 group-hover:bg-amber-300',
  }
  return map[providerType.value]
})

// Avatar gradient by provider type
const avatarClass = computed(() => {
  const map = {
    official: 'bg-gradient-to-br from-cyan-500 to-cyan-700',
    third: 'bg-gradient-to-br from-violet-500 to-violet-700',
    uncategorized: 'bg-gradient-to-br from-amber-500 to-amber-700',
  }
  return map[providerType.value]
})

// Name color by provider type
const nameColorClass = computed(() => {
  const map = {
    official: 'text-cyan-400 group-hover:text-cyan-300',
    third: 'text-violet-400 group-hover:text-violet-300',
    uncategorized: 'text-amber-400 group-hover:text-amber-300',
  }
  return map[providerType.value]
})
</script>

<style scoped>
/* === Row Base - 液态玻璃 === */
.config-row {
  background: var(--liquid-glass-bg);
  backdrop-filter: var(--liquid-glass-blur);
  border: 1px solid var(--liquid-glass-border);
  box-shadow: var(--liquid-glass-highlight);
  cursor: pointer;
}

.config-row:hover {
  background: var(--glass-bg-medium);
  border-color: var(--glass-border-medium);
  box-shadow: var(--liquid-glass-shadow);
  transform: translateX(2px);
}

/* === Current Config Highlight === */
.config-row--current {
  background: linear-gradient(135deg,
    rgb(6 182 212 / 6%) 0%,
    rgb(139 92 246 / 4%) 100%
  );
  border-color: rgb(6 182 212 / 15%);
  box-shadow: var(--glow-primary);
}

.config-row--current:hover {
  border-color: rgb(6 182 212 / 25%);
  box-shadow:
    var(--glow-primary),
    var(--shadow-md);
}

/* === Disabled Config === */
.config-row--disabled {
  opacity: 0.5;
}

/* === Meta Chips === */
.meta-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.25rem 0.625rem;
  border-radius: 9999px;
  font-size: 0.6875rem;
  font-family: var(--font-mono);
  color: var(--color-text-secondary);
  background: var(--glass-bg-light);
  border: 1px solid var(--color-border-subtle);
  white-space: nowrap;
  transition: all 0.2s;
}

.config-row:hover .meta-chip {
  background: var(--glass-bg-medium);
  border-color: var(--color-border-default);
}

/* === Switch Button === */
.switch-btn {
  background: linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary));
  color: white;
  box-shadow: 0 2px 8px rgb(var(--color-accent-primary-rgb) / 25%);
}

.switch-btn:hover {
  box-shadow: 0 4px 16px rgb(var(--color-accent-primary-rgb) / 40%);
  transform: translateY(-1px);
}

/* === Edit Button === */
.edit-btn {
  color: var(--color-text-muted);
  background: transparent;
}

.edit-btn:hover {
  color: var(--color-accent-primary);
  background: var(--glass-bg-medium);
}

/* === Highlight Pulse (scroll-to) === */
.config-row.highlight-pulse {
  animation: row-highlight-pulse 1.5s ease-out;
}

@keyframes row-highlight-pulse {
  0% {
    box-shadow: 0 0 0 0 rgb(6 182 212 / 40%);
  }

  50% {
    box-shadow: 0 0 0 6px rgb(6 182 212 / 10%);
  }

  100% {
    box-shadow: 0 0 0 0 rgb(6 182 212 / 0%);
  }
}

/* === Accent bar glow for current === */
.config-row--current .accent-bar {
  box-shadow: 2px 0 8px rgb(6 182 212 / 30%);
}
</style>
