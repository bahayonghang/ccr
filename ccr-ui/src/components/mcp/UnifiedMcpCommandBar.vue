<template>
  <div class="command-bar">
    <div class="command-bar__row">
      <div class="command-bar__title">
        MCP 服务器
        <span
          v-if="serverCount > 0"
          class="command-bar__badge"
        >
          {{ serverCount }}
        </span>
      </div>
      <div class="command-bar__search">
        <SIcon
          name="Search"
          size="w-4 h-4"
          class="text-[var(--color-text-muted)]"
        />
        <input
          :value="filterKeyword"
          type="text"
          placeholder="搜索服务器名称、命令或 URL..."
          class="command-bar__search-input"
          @input="emit('update:filterKeyword', ($event.target as HTMLInputElement).value)"
        >
        <button
          v-if="filterKeyword"
          class="command-bar__search-clear"
          @click="emit('update:filterKeyword', '')"
        >
          <SIcon
            name="X"
            size="w-3.5 h-3.5"
          />
        </button>
      </div>
      <div class="command-bar__actions">
        <button
          class="btn-add"
          @click="emit('open-add')"
        >
          <SIcon
            name="Plus"
            size="w-4 h-4"
          />
          <span class="hidden sm:inline">添加</span>
        </button>
        <button
          class="btn-refresh"
          :disabled="loading"
          @click="emit('refresh')"
        >
          <SIcon
            name="RefreshCw"
            size="w-4 h-4"
            :class="{ 'animate-spin': loading }"
          />
        </button>
      </div>
    </div>

    <div class="command-bar__row">
      <div class="command-bar__platforms">
        <button
          class="stat-chip"
          :class="{ 'stat-chip--active': filterPlatform === '' }"
          @click="emit('update:filterPlatform', '')"
        >
          <span class="stat-chip__label">全部</span>
          <span class="stat-chip__count">{{ serverCount }}</span>
        </button>
        <button
          v-for="platform in allPlatforms"
          :key="platform"
          class="stat-chip"
          :class="{ 'stat-chip--active': filterPlatform === platform }"
          :style="{ '--chip-color': platformMeta[platform].color }"
          @click="emit('update:filterPlatform', filterPlatform === platform ? '' : platform)"
        >
          <span
            class="stat-chip__dot"
            :style="{ background: platformMeta[platform].color }"
          />
          <span class="stat-chip__label">{{ platformMeta[platform].label }}</span>
          <span class="stat-chip__count">{{ platformCounts[platform] || 0 }}</span>
        </button>
      </div>
      <div class="command-bar__protocol">
        <button
          v-for="option in protocolOptions"
          :key="option.value"
          class="protocol-btn"
          :class="{ 'protocol-btn--active': filterProtocol === option.value }"
          @click="emit('update:filterProtocol', option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { PlatformMeta, UnifiedMcpPlatform } from '@/types/unifiedMcp'

interface ProtocolOption {
  value: 'all' | 'stdio' | 'http'
  label: string
}

defineProps<{
  allPlatforms: UnifiedMcpPlatform[]
  filterKeyword: string
  filterPlatform: UnifiedMcpPlatform | ''
  filterProtocol: 'all' | 'stdio' | 'http'
  loading: boolean
  platformCounts: Record<string, number>
  platformMeta: Record<UnifiedMcpPlatform, PlatformMeta>
  protocolOptions: ProtocolOption[]
  serverCount: number
}>()

const emit = defineEmits<{
  'open-add': []
  refresh: []
  'update:filterKeyword': [value: string]
  'update:filterPlatform': [value: UnifiedMcpPlatform | '']
  'update:filterProtocol': [value: 'all' | 'stdio' | 'http']
}>()
</script>

<style scoped>
.command-bar {
  position: sticky;
  top: 0;
  z-index: var(--layer-dropdown);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-5);
  border-radius: var(--radius-xl);
  background: var(--glass-bg-light);
  backdrop-filter: blur(16px) saturate(180%);
  border: 1px solid var(--color-border-default);
  box-shadow: var(--shadow-sm);
}

.command-bar__row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-wrap: wrap;
}

.command-bar__title {
  font-size: 1.125rem;
  font-weight: 500;
  color: var(--color-text-primary);
  display: flex;
  align-items: center;
  gap: var(--space-2);
  white-space: nowrap;
  flex-shrink: 0;
}

.command-bar__badge {
  font-size: 0.6875rem;
  font-weight: 500;
  background: var(--color-accent-primary);
  color: white;
  padding: 1px 7px;
  border-radius: var(--radius-full);
  line-height: 1.4;
}

.command-bar__search {
  flex: 1;
  min-width: 180px;
  max-width: 480px;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 6px 12px;
  border-radius: var(--radius-md);
  background: var(--glass-bg-light);
  border: 1px solid var(--color-border-default);
  transition: border-color var(--duration-fast);
}

.command-bar__search:focus-within {
  border-color: var(--color-accent-primary);
}

.command-bar__search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 0.8125rem;
  color: var(--color-text-primary);
}

.command-bar__search-input:focus-visible {
  outline: 2px solid rgb(var(--color-accent-primary-rgb) / 50%);
  outline-offset: 2px;
}

.command-bar__search-input::placeholder {
  color: var(--color-text-muted);
}

.command-bar__search-clear {
  display: flex;
  padding: 2px;
  border: none;
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
}

.command-bar__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-left: auto;
  flex-shrink: 0;
}

.command-bar__platforms {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  flex: 1;
  min-width: 0;
}

.command-bar__protocol {
  display: flex;
  border-radius: var(--radius-md);
  overflow: hidden;
  border: 1px solid var(--color-border-default);
  flex-shrink: 0;
}

.btn-add {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 8px 16px;
  font-size: 0.8125rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  background: var(--color-accent-primary);
  color: white;
  border: none;
  cursor: pointer;
  transition: opacity var(--duration-fast);
}

.btn-add:hover { opacity: 0.85; }

.btn-refresh {
  display: inline-flex;
  align-items: center;
  padding: 8px;
  border-radius: var(--radius-md);
  background: var(--glass-bg-light);
  border: 1px solid var(--color-border-default);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background var(--duration-fast);
}

.btn-refresh:hover { background: var(--glass-bg-medium); }

.btn-refresh:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.stat-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  border-radius: var(--radius-full);
  font-size: 0.8125rem;
  font-weight: 500;
  background: var(--glass-bg-light);
  border: 1px solid var(--color-border-default);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast);
  white-space: nowrap;
}

.stat-chip:hover { background: var(--glass-bg-medium); }

.stat-chip--active {
  background: var(--color-accent-primary);
  color: white;
  border-color: var(--color-accent-primary);
}

.stat-chip__dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.stat-chip__count {
  font-size: 0.75rem;
  opacity: 0.7;
}

.protocol-btn {
  padding: 5px 12px;
  font-size: 0.75rem;
  font-weight: 500;
  background: var(--glass-bg-light);
  border: none;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.protocol-btn:not(:last-child) {
  border-right: 1px solid var(--color-border-default);
}

.protocol-btn--active {
  background: var(--color-accent-primary);
  color: white;
}

@media (width <= 768px) {
  .command-bar__row {
    flex-wrap: wrap;
  }

  .command-bar__search {
    order: 3;
    min-width: 100%;
    max-width: none;
  }

  .command-bar__platforms {
    overflow-x: auto;
    flex-wrap: nowrap;
    scrollbar-width: none;
  }

  .command-bar__platforms::-webkit-scrollbar {
    display: none;
  }
}

@media (width <= 640px) {
  .command-bar {
    padding: var(--space-3);
    border-radius: var(--radius-lg);
  }
}
</style>
