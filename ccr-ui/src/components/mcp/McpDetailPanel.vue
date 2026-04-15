<template>
  <div class="mcp-detail-panel">
    <div
      v-if="!group"
      class="mcp-detail-panel__empty"
    >
      <SIcon
        name="Server"
        size="w-8 h-8"
        class="text-text-muted/40"
      />
      <p>Select a server to view details</p>
    </div>

    <template v-else-if="primaryServer">
      <!-- 头部 -->
      <div class="detail-header">
        <div class="detail-header__info">
          <div class="detail-header__icon">
            <SIcon
              :name="group.transportType === 'stdio' ? 'Terminal' : 'Globe'"
              size="w-5 h-5"
            />
          </div>
          <div>
            <h2 class="detail-header__title">
              {{ group.name }}
            </h2>
            <p class="detail-header__sub">
              {{ group.transportType.toUpperCase() }} · {{ group.items.length }} instance(s)
            </p>
          </div>
        </div>
        <div class="detail-header__actions">
          <button
            type="button"
            class="detail-btn"
            @click="$emit('edit', group.name)"
          >
            <SIcon
              name="Pencil"
              size="w-4 h-4"
            />
            <span>Edit</span>
          </button>
          <button
            type="button"
            class="detail-btn detail-btn--danger"
            @click="$emit('delete', group)"
          >
            <SIcon
              name="Trash2"
              size="w-4 h-4"
            />
            <span>Delete</span>
          </button>
        </div>
      </div>

      <!-- Transport 信息 -->
      <section class="detail-section">
        <h3 class="detail-section__title">
          Transport
        </h3>
        <div class="detail-grid">
          <div class="detail-field">
            <span class="detail-field__label">Type</span>
            <span class="detail-field__value">{{ group.transportType }}</span>
          </div>
          <div class="detail-field">
            <span class="detail-field__label">{{ group.transportType === 'stdio' ? 'Command' : 'URL' }}</span>
            <span class="detail-field__value detail-field__value--mono">{{ group.transportLabel }}</span>
          </div>
          <div
            v-if="primaryServer.args?.length"
            class="detail-field"
          >
            <span class="detail-field__label">Args</span>
            <span class="detail-field__value detail-field__value--mono">{{ primaryServer.args.join(' ') }}</span>
          </div>
          <div
            v-if="primaryServer.timeout"
            class="detail-field"
          >
            <span class="detail-field__label">Timeout</span>
            <span class="detail-field__value">{{ primaryServer.timeout }}ms</span>
          </div>
          <div
            v-if="primaryServer.cwd"
            class="detail-field"
          >
            <span class="detail-field__label">CWD</span>
            <span class="detail-field__value detail-field__value--mono">{{ primaryServer.cwd }}</span>
          </div>
        </div>
      </section>

      <!-- 环境变量 -->
      <section
        v-if="hasEnvVars"
        class="detail-section"
      >
        <h3 class="detail-section__title">
          Environment Variables
        </h3>
        <div class="detail-kv-list">
          <div
            v-for="(value, key) in primaryServer.env"
            :key="key"
            class="detail-kv"
          >
            <span class="detail-kv__key">{{ key }}</span>
            <span class="detail-kv__value">{{ maskValue(String(value)) }}</span>
          </div>
        </div>
      </section>

      <!-- Headers -->
      <section
        v-if="hasHeaders"
        class="detail-section"
      >
        <h3 class="detail-section__title">
          Headers
        </h3>
        <div class="detail-kv-list">
          <div
            v-for="(value, key) in primaryServer.headers"
            :key="key"
            class="detail-kv"
          >
            <span class="detail-kv__key">{{ key }}</span>
            <span class="detail-kv__value">{{ maskValue(String(value)) }}</span>
          </div>
        </div>
      </section>

      <!-- Agent 实例列表 -->
      <section class="detail-section">
        <h3 class="detail-section__title">
          Agents
        </h3>
        <div class="detail-agent-list">
          <div
            v-for="item in group.items"
            :key="`${item.platform}-${item.name}`"
            class="detail-agent-row"
          >
            <AgentIcons
              :agents="[item.platform]"
              :compact="false"
            />
            <span
              class="detail-agent-status"
              :class="item.disabled ? 'detail-agent-status--disabled' : 'detail-agent-status--active'"
            >
              {{ item.disabled ? 'Disabled' : 'Active' }}
            </span>
            <button
              type="button"
              class="detail-btn detail-btn--sm"
              @click="$emit('toggle', item)"
            >
              <SIcon
                :name="item.disabled ? 'ToggleLeft' : 'ToggleRight'"
                size="w-4 h-4"
              />
            </button>
          </div>
        </div>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import AgentIcons from '@/components/common/AgentIcons.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { McpGroup } from '@/types/mcpManager'
import type { UnifiedMcpServer } from '@/types/unifiedMcp'

const props = defineProps<{
  /** 当前选中的 MCP 分组 */
  group: McpGroup | null
}>()

defineEmits<{
  edit: [groupName: string]
  delete: [group: McpGroup]
  toggle: [server: UnifiedMcpServer]
}>()

const primaryServer = computed(() => props.group?.items[0] ?? null)

const hasEnvVars = computed(() => {
  const env = primaryServer.value?.env
  return env && Object.keys(env).length > 0
})

const hasHeaders = computed(() => {
  const headers = primaryServer.value?.headers
  return headers && Object.keys(headers).length > 0
})

/** 敏感值脱敏 (保留前4后2) */
function maskValue(value: string): string {
  if (value.length <= 8) return '••••••'
  return value.slice(0, 4) + '••••' + value.slice(-2)
}
</script>

<style scoped>
.mcp-detail-panel {
  height: 100%;
  overflow-y: auto;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.mcp-detail-panel__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  height: 100%;
  font-size: 0.875rem;
  color: var(--color-text-muted);

}

/* 头部 */
.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.detail-header__info {
  display: flex;
  align-items: center;
  gap: 0.75rem;

}

.detail-header__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 0.75rem;
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: rgb(var(--color-accent-primary-rgb));
  flex-shrink: 0;

}

.detail-header__title {
  font-size: 1.125rem;
  font-weight: 700;
  color: var(--color-text-primary);

}

.detail-header__sub {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin-top: 0.125rem;

}

.detail-header__actions {
  display: flex;
  gap: 0.5rem;
  flex-shrink: 0;

}

/* 按钮 */
.detail-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  border-radius: 0.625rem;
  font-size: 0.8125rem;
  font-weight: 500;
  border: 1px solid var(--surface-status-border, rgb(var(--color-border-default-rgb) / 55%));
  background: var(--surface-status-bg);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.detail-btn:hover {
  color: var(--color-text-primary);
  background: var(--surface-card-bg);

}
.detail-btn--danger { color: rgb(239 68 68 / 85%); }

.detail-btn--danger:hover {
  color: rgb(239 68 68);
  background: rgb(239 68 68 / 8%);
}
.detail-btn--sm { padding: 0.25rem 0.5rem; }

/* 区块 */
.detail-section {
  padding: 1rem;
  border-radius: 1rem;
  border: 1px solid var(--surface-card-border, rgb(var(--color-border-default-rgb) / 45%));
  background: var(--surface-card-bg);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-1);
}

.detail-section__title {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--color-text-muted);
  margin-bottom: 0.75rem;

}

/* 字段网格 */
.detail-grid {
  display: grid;
  gap: 0.75rem;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
}

.detail-field__label {
  display: block;
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--color-text-muted);
  margin-bottom: 0.25rem;

}

.detail-field__value {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  word-break: break-all;

}

.detail-field__value--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;

}

/* KV 列表 */
.detail-kv-list {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.detail-kv {
  display: flex;
  align-items: baseline;
  gap: 0.75rem;
  padding: 0.375rem 0.5rem;
  border-radius: 0.5rem;
  background: rgb(var(--color-bg-base-rgb) / 42%);

}

.detail-kv__key {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-primary);
  flex-shrink: 0;

}

.detail-kv__value {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

}

/* Agent 列表 */
.detail-agent-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.detail-agent-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0.625rem;
  border-radius: 0.625rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 35%);
  background: rgb(var(--color-bg-base-rgb) / 42%);

}

.detail-agent-status {
  margin-left: auto;
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;

}
.detail-agent-status--active { color: rgb(34 197 94); }
.detail-agent-status--disabled { color: var(--color-text-muted); }
</style>
