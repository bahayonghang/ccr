<template>
  <div
    v-if="loading && servers.length === 0"
    class="loading-state"
  >
    <SIcon
      name="RefreshCw"
      size="w-6 h-6"
      class="animate-spin text-[var(--color-accent-primary)]"
    />
    <p>加载中...</p>
  </div>

  <div
    v-else-if="error && servers.length === 0"
    class="error-state"
  >
    <SIcon
      name="AlertCircle"
      size="w-8 h-8"
      class="text-[var(--color-danger)]"
    />
    <p>{{ error }}</p>
    <button
      class="btn-retry"
      @click="emit('retry')"
    >
      重试
    </button>
  </div>

  <div
    v-else-if="filteredServers.length === 0"
    class="empty-state"
  >
    <SIcon
      name="Server"
      size="w-10 h-10"
      class="text-[var(--color-text-muted)]"
    />
    <p v-if="hasActiveFilters">
      没有匹配的服务器
    </p>
    <p v-else>
      暂无 MCP 服务器，点击上方按钮添加
    </p>
  </div>

  <div
    v-else
    class="server-grid"
  >
    <div
      v-for="server in filteredServers"
      :key="`${server.platform}-${server.name}`"
      class="server-card"
      :class="{ 'server-card--disabled': server.disabled }"
    >
      <div class="server-card__header">
        <div class="server-card__name-row">
          <span
            class="server-card__platform-dot"
            :style="{ background: getPlatformColor(server.platform) }"
            :title="getPlatformLabel(server.platform)"
          />
          <span class="server-card__name">{{ server.name }}</span>
          <span
            v-if="server.disabled"
            class="server-card__disabled-badge"
          >已禁用</span>
        </div>
        <span class="server-card__platform-label">
          {{ getPlatformLabel(server.platform) }}
        </span>
      </div>

      <div class="server-card__body">
        <div
          v-if="server.command"
          class="server-card__field"
        >
          <SIcon
            name="Terminal"
            size="w-3.5 h-3.5"
            class="shrink-0"
          />
          <code class="server-card__code">{{ server.command }}</code>
        </div>
        <div
          v-if="server.url"
          class="server-card__field"
        >
          <SIcon
            name="Globe"
            size="w-3.5 h-3.5"
            class="shrink-0"
          />
          <code class="server-card__code">{{ server.url }}</code>
        </div>
        <div
          v-if="server.args && server.args.length > 0"
          class="server-card__field"
        >
          <SIcon
            name="ChevronRight"
            size="w-3.5 h-3.5"
            class="shrink-0"
          />
          <span class="server-card__args">{{ server.args.join(' ') }}</span>
        </div>
        <div
          v-if="server.env && Object.keys(server.env).length > 0"
          class="server-card__tags"
        >
          <span
            v-for="key in Object.keys(server.env)"
            :key="key"
            class="env-tag"
            :title="`${key}=${server.env[key]}`"
          >
            {{ key }}
          </span>
        </div>
      </div>

      <div class="server-card__actions">
        <button
          v-if="supportsFeature(server.platform, 'supports_toggle')"
          class="action-btn"
          :title="server.disabled ? '启用' : '禁用'"
          @click="emit('toggle', server)"
        >
          <SIcon
            :name="server.disabled ? 'ToggleLeft' : 'ToggleRight'"
            size="w-4 h-4"
          />
        </button>
        <button
          class="action-btn"
          title="编辑"
          @click="emit('edit', server)"
        >
          <SIcon
            name="Pencil"
            size="w-4 h-4"
          />
        </button>
        <button
          class="action-btn action-btn--danger"
          title="删除"
          @click="emit('delete', server)"
        >
          <SIcon
            name="Trash2"
            size="w-4 h-4"
          />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { PlatformMcpCapability, UnifiedMcpServer } from '@/types/unifiedMcp'

type SupportedFeature = Exclude<keyof PlatformMcpCapability, 'platform'>

defineProps<{
  error: string | null
  filteredServers: UnifiedMcpServer[]
  getPlatformColor: (platform: string) => string
  getPlatformLabel: (platform: string) => string
  hasActiveFilters: boolean
  loading: boolean
  servers: UnifiedMcpServer[]
  supportsFeature: (platform: string, feature: SupportedFeature) => boolean
}>()

const emit = defineEmits<{
  delete: [server: UnifiedMcpServer]
  edit: [server: UnifiedMcpServer]
  retry: []
  toggle: [server: UnifiedMcpServer]
}>()
</script>

<style scoped>
.loading-state,
.error-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-12) 0;
  color: var(--color-text-muted);
}

.btn-retry {
  padding: 6px 16px;
  font-size: 0.8125rem;
  border-radius: var(--radius-md);
  background: var(--color-accent-primary);
  color: white;
  border: none;
  cursor: pointer;
}

.server-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
  gap: var(--space-4);
}

.server-card {
  background: var(--glass-bg-light);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  transition: border-color var(--duration-fast), box-shadow var(--duration-fast);
}

.server-card:hover {
  border-color: var(--color-border-accent);
  box-shadow: var(--shadow-sm);
}

.server-card--disabled {
  opacity: 0.55;
}

.server-card__header {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.server-card__name-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.server-card__platform-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.server-card__name {
  font-weight: 500;
  font-size: 0.9375rem;
  color: var(--color-text-primary);
  word-break: break-all;
}

.server-card__disabled-badge {
  font-size: 0.625rem;
  font-weight: 500;
  text-transform: uppercase;
  padding: 1px 6px;
  border-radius: var(--radius-full);
  background: var(--color-danger);
  color: white;
}

.server-card__platform-label {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  padding-left: 18px;
}

.server-card__body {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}

.server-card__field {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  color: var(--color-text-secondary);
  font-size: 0.8125rem;
}

.server-card__code {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  word-break: break-all;
  background: var(--glass-bg-medium);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
}

.server-card__args {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.server-card__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 2px;
}

.env-tag {
  font-size: 0.6875rem;
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  background: var(--glass-bg-medium);
  color: var(--color-text-muted);
  font-family: var(--font-mono);
}

.server-card__actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-1);
  padding-top: var(--space-2);
  border-top: 1px solid var(--color-border-default);
}

.action-btn {
  display: inline-flex;
  align-items: center;
  padding: 6px;
  border-radius: var(--radius-sm);
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: color var(--duration-fast), background var(--duration-fast);
}

.action-btn:hover {
  color: var(--color-text-primary);
  background: var(--glass-bg-medium);
}

.action-btn--danger:hover {
  color: var(--color-danger);
}

@media (width >= 1280px) {
  .server-grid {
    grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  }
}

@media (width <= 768px) {
  .server-grid {
    grid-template-columns: 1fr;
  }
}
</style>
