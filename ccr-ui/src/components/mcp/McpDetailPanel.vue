<template>
  <div class="mcp-detail-panel">
    <div
      v-if="!group"
      class="mcp-detail-panel__empty"
    >
      <SIcon
        name="Server"
        size="w-8 h-8"
      />
      <p>No MCP server selected</p>
      <span>Use the left rail to inspect effective configuration and precedence.</span>
    </div>

    <template v-else-if="primaryServer">
      <div class="detail-header">
        <div class="detail-header__info">
          <div class="detail-header__icon">
            <SIcon
              :name="group.transportType === 'stdio' ? 'Terminal' : 'Globe'"
              size="w-5 h-5"
            />
          </div>
          <div>
            <p class="detail-header__eyebrow">
              {{ primaryServer.scope ?? 'global' }} scope · {{ primaryServer.effective === false ? 'not effective' : 'effective' }}
            </p>
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

      <section class="detail-section detail-section--effective">
        <div class="detail-section__heading">
          <h3 class="detail-section__title">
            Effective config
          </h3>
          <span
            class="state-pill"
            :class="statePillClass(primaryServer)"
          >
            {{ stateLabel(primaryServer) }}
          </span>
        </div>
        <div class="detail-grid">
          <div class="detail-field">
            <span class="detail-field__label">Type</span>
            <span class="detail-field__value">{{ group.transportType }}</span>
          </div>
          <div class="detail-field detail-field--wide">
            <span class="detail-field__label">{{ group.transportType === 'stdio' ? 'Command' : 'URL' }}</span>
            <span class="detail-field__value detail-field__value--mono">{{ group.transportLabel || '—' }}</span>
          </div>
          <div
            v-if="primaryServer.args?.length"
            class="detail-field detail-field--wide"
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
            class="detail-field detail-field--wide"
          >
            <span class="detail-field__label">CWD</span>
            <span class="detail-field__value detail-field__value--mono">{{ primaryServer.cwd }}</span>
          </div>
        </div>
      </section>

      <section class="detail-section">
        <h3 class="detail-section__title">
          Source & precedence
        </h3>
        <div class="precedence-stack">
          <div
            v-for="item in precedenceItems"
            :key="`${item.platform}-${item.scope ?? 'global'}-${item.name}`"
            class="precedence-row"
            :class="{ 'precedence-row--active': item.effective !== false && !item.hidden_by }"
          >
            <span class="precedence-row__dot" />
            <div class="precedence-row__body">
              <strong>{{ item.scope ?? item.platform }}</strong>
              <span>{{ item.source_path ?? 'source path unavailable' }}</span>
              <em v-if="item.hidden_by">Hidden by {{ item.hidden_by }}</em>
              <em v-else-if="item.approval_state">Approval: {{ item.approval_state }}</em>
            </div>
            <span
              class="state-pill"
              :class="statePillClass(item)"
            >
              {{ stateLabel(item) }}
            </span>
          </div>
        </div>
      </section>

      <section
        v-if="hasEnvVars || hasHeaders"
        class="detail-section"
      >
        <h3 class="detail-section__title">
          Env / Headers
        </h3>
        <div
          v-if="hasEnvVars"
          class="detail-kv-list"
        >
          <div
            v-for="(value, key) in primaryServer.env"
            :key="key"
            class="detail-kv"
          >
            <span class="detail-kv__key">{{ key }}</span>
            <span class="detail-kv__value">{{ maskValue(String(value)) }}</span>
          </div>
        </div>
        <div
          v-if="hasHeaders"
          class="detail-kv-list"
        >
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

      <section class="detail-section">
        <h3 class="detail-section__title">
          Raw config preview
        </h3>
        <pre class="raw-config-preview">{{ rawConfigPreview }}</pre>
      </section>

      <section class="detail-section">
        <h3 class="detail-section__title">
          Instances
        </h3>
        <div class="detail-agent-list">
          <div
            v-for="item in group.items"
            :key="`${item.platform}-${item.scope ?? 'global'}-${item.name}-instance`"
            class="detail-agent-row"
          >
            <AgentIcons
              :agents="[item.platform]"
              :compact="false"
            />
            <span class="detail-agent-scope">{{ item.scope ?? 'global' }}</span>
            <span
              class="detail-agent-status"
              :class="item.disabled ? 'detail-agent-status--disabled' : 'detail-agent-status--active'"
            >
              {{ item.disabled ? 'Disabled' : 'Enabled' }}
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

      <section
        v-if="diagnostics.length"
        class="detail-section detail-section--diagnostics"
      >
        <h3 class="detail-section__title">
          Source diagnostics
        </h3>
        <div class="diagnostic-list">
          <div
            v-for="(diagnostic, index) in diagnostics"
            :key="`${diagnostic.source_path ?? 'diagnostic'}-${index}`"
            class="diagnostic-row"
          >
            <span class="diagnostic-row__level">{{ diagnostic.level }}</span>
            <span>{{ diagnostic.message }}</span>
          </div>
        </div>
      </section>
    </template>

    <div
      v-else
      class="mcp-detail-panel__empty"
    >
      <SIcon
        name="AlertCircle"
        size="w-8 h-8"
      />
      <p>Configuration exists but no readable instance was returned</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import AgentIcons from '@/components/common/AgentIcons.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { McpGroup } from '@/types/mcpManager'
import type { UnifiedMcpDiagnostic, UnifiedMcpServer } from '@/types/unifiedMcp'

const props = defineProps<{
  group: McpGroup | null
  diagnostics?: UnifiedMcpDiagnostic[]
}>()

defineEmits<{
  edit: [groupName: string]
  delete: [group: McpGroup]
  toggle: [server: UnifiedMcpServer]
}>()

const primaryServer = computed(() =>
  props.group?.items.find(item => item.effective !== false && !item.hidden_by)
  ?? props.group?.items[0]
  ?? null,
)

const diagnostics = computed(() => props.diagnostics ?? [])

const precedenceItems = computed(() => {
  const items = props.group?.items ?? []
  const order = { local: 0, project: 1, user: 2 } as Record<string, number>
  return [...items].sort((a, b) => {
    const aOrder = order[String(a.scope)] ?? 9
    const bOrder = order[String(b.scope)] ?? 9
    if (aOrder !== bOrder) return aOrder - bOrder
    return String(a.platform).localeCompare(String(b.platform))
  })
})

const rawConfigPreview = computed(() => {
  const raw = primaryServer.value?.raw_config
  if (raw && typeof raw === 'object') {
    return JSON.stringify(raw, null, 2)
  }

  return JSON.stringify(
    {
      command: primaryServer.value?.command ?? undefined,
      url: primaryServer.value?.url ?? undefined,
      args: primaryServer.value?.args,
      env: primaryServer.value?.env,
      headers: primaryServer.value?.headers,
      disabled: primaryServer.value?.disabled,
    },
    null,
    2,
  )
})

const hasEnvVars = computed(() => {
  const env = primaryServer.value?.env
  return env && Object.keys(env).length > 0
})

const hasHeaders = computed(() => {
  const headers = primaryServer.value?.headers
  return headers && Object.keys(headers).length > 0
})

function stateLabel(server: UnifiedMcpServer): string {
  if (server.hidden_by) return 'Overridden'
  if (server.disabled) return 'Disabled'
  if (server.approval_state === 'pending') return 'Pending'
  if (server.approval_state === 'disabled') return 'Not approved'
  if (server.effective === false) return 'Hidden'
  return 'Effective'
}

function statePillClass(server: UnifiedMcpServer): string {
  if (server.hidden_by || server.effective === false) return 'state-pill--muted'
  if (server.disabled || server.approval_state === 'disabled') return 'state-pill--danger'
  if (server.approval_state === 'pending') return 'state-pill--warning'
  return 'state-pill--ok'
}

function maskValue(value: string): string {
  if (!value) return ''
  if (value.includes('•')) return value
  if (value.length <= 8) return '••••••'
  return `${value.slice(0, 4)}••••${value.slice(-2)}`
}
</script>

<style scoped>
.mcp-detail-panel {
  height: 100%;
  overflow-y: auto;
  padding: 1.6rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.mcp-detail-panel__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  height: 100%;
  color: var(--color-text-muted);
  text-align: center;
}

.mcp-detail-panel__empty p {
  color: var(--color-text-primary);
  font-weight: 700;
}

.mcp-detail-panel__empty span {
  font-size: 0.8rem;
}

.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding-bottom: 0.45rem;
}

.detail-header__info {
  display: flex;
  align-items: center;
  gap: 0.9rem;
}

.detail-header__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.8rem;
  height: 2.8rem;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 22%);
  border-radius: 0.9rem;
  background: rgb(var(--color-accent-primary-rgb) / 9%);
  color: rgb(var(--color-accent-primary-rgb));
  flex-shrink: 0;
}

.detail-header__eyebrow {
  color: rgb(var(--color-accent-primary-rgb) / 88%);
  font-size: 0.64rem;
  font-weight: 750;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.detail-header__title {
  margin-top: 0.1rem;
  color: var(--color-text-primary);
  font-family: Georgia, 'Times New Roman', var(--font-sans);
  font-size: 1.55rem;
  line-height: 1.1;
  letter-spacing: -0.035em;
}

.detail-header__sub {
  color: var(--color-text-muted);
  font-size: 0.76rem;
  margin-top: 0.25rem;
}

.detail-header__actions {
  display: flex;
  gap: 0.5rem;
  flex-shrink: 0;
}

.detail-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.42rem 0.78rem;
  border-radius: 999px;
  font-size: 0.78rem;
  font-weight: 650;
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

.detail-btn--danger {
  color: rgb(var(--color-danger-rgb, 239 68 68) / 85%);
}

.detail-btn--danger:hover {
  border-color: rgb(var(--color-danger-rgb, 239 68 68) / 22%);
  background: rgb(var(--color-danger-rgb, 239 68 68) / 8%);
  color: rgb(var(--color-danger-rgb, 239 68 68));
}

.detail-btn--sm {
  padding: 0.28rem 0.55rem;
}

.detail-section {
  padding: 1rem;
  border: 1px solid var(--surface-card-border, rgb(var(--color-border-default-rgb) / 45%));
  border-radius: 1rem;
  background: rgb(var(--color-bg-surface-rgb) / 54%);
  box-shadow: var(--elevation-1);
}

.detail-section--effective {
  background:
    linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 9%), transparent 46%),
    rgb(var(--color-bg-surface-rgb) / 62%);
}

.detail-section__heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.detail-section__title {
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 750;
  letter-spacing: 0.13em;
  text-transform: uppercase;
}

.detail-grid {
  display: grid;
  gap: 0.85rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.detail-field--wide {
  grid-column: 1 / -1;
}

.detail-field__label {
  display: block;
  margin-bottom: 0.24rem;
  color: var(--color-text-muted);
  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.detail-field__value {
  color: var(--color-text-primary);
  font-size: 0.82rem;
  word-break: break-all;
}

.detail-field__value--mono,
.detail-kv__key,
.detail-kv__value {
  font-family: var(--font-mono);
  font-size: 0.74rem;
}

.precedence-stack,
.detail-kv-list,
.detail-agent-list,
.diagnostic-list {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
}

.precedence-row,
.detail-agent-row,
.detail-kv,
.diagnostic-row {
  display: flex;
  align-items: center;
  gap: 0.7rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 28%);
  border-radius: 0.75rem;
  background: rgb(var(--color-bg-base-rgb) / 38%);
  padding: 0.55rem 0.65rem;
}

.precedence-row--active {
  border-color: rgb(var(--color-success-rgb, 34 197 94) / 24%);
  background: rgb(var(--color-success-rgb, 34 197 94) / 6%);
}

.precedence-row__dot {
  width: 0.46rem;
  height: 0.46rem;
  border-radius: 999px;
  background: rgb(var(--color-border-default-rgb) / 80%);
}

.precedence-row--active .precedence-row__dot {
  background: rgb(var(--color-success-rgb, 34 197 94));
}

.precedence-row__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.12rem;
}

.precedence-row__body strong,
.detail-agent-scope {
  color: var(--color-text-primary);
  font-size: 0.78rem;
  font-weight: 720;
  text-transform: capitalize;
}

.precedence-row__body span,
.precedence-row__body em {
  overflow: hidden;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-style: normal;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.state-pill {
  flex-shrink: 0;
  border: 1px solid transparent;
  border-radius: 999px;
  padding: 0.15rem 0.48rem;
  font-size: 0.62rem;
  font-weight: 800;
  letter-spacing: 0.07em;
  text-transform: uppercase;
}

.state-pill--ok {
  border-color: rgb(var(--color-success-rgb, 34 197 94) / 24%);
  background: rgb(var(--color-success-rgb, 34 197 94) / 10%);
  color: rgb(var(--color-success-rgb, 34 197 94));
}

.state-pill--warning {
  border-color: rgb(var(--color-warning-rgb, 245 158 11) / 26%);
  background: rgb(var(--color-warning-rgb, 245 158 11) / 10%);
  color: rgb(var(--color-warning-rgb, 245 158 11));
}

.state-pill--danger {
  border-color: rgb(var(--color-danger-rgb, 239 68 68) / 24%);
  background: rgb(var(--color-danger-rgb, 239 68 68) / 9%);
  color: rgb(var(--color-danger-rgb, 239 68 68));
}

.state-pill--muted {
  border-color: rgb(var(--color-border-default-rgb) / 34%);
  background: rgb(var(--color-bg-elevated-rgb) / 44%);
  color: var(--color-text-muted);
}

.detail-kv {
  justify-content: space-between;
}

.detail-kv__key {
  color: var(--color-text-primary);
  font-weight: 700;
}

.detail-kv__value {
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail-agent-status {
  margin-left: auto;
  font-size: 0.68rem;
  font-weight: 750;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.detail-agent-status--active {
  color: rgb(var(--color-success-rgb, 34 197 94));
}

.detail-agent-status--disabled {
  color: var(--color-text-muted);
}

.diagnostic-row {
  align-items: baseline;
  color: var(--color-text-secondary);
  font-size: 0.76rem;
}

.diagnostic-row__level {
  color: rgb(var(--color-accent-primary-rgb) / 90%);
  font-size: 0.64rem;
  font-weight: 800;
  letter-spacing: 0.09em;
  text-transform: uppercase;
}

.raw-config-preview {
  overflow-x: auto;
  border: 1px solid rgb(var(--color-border-default-rgb) / 28%);
  border-radius: 0.75rem;
  background: rgb(var(--color-bg-base-rgb) / 42%);
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  font-size: 0.72rem;
  line-height: 1.55;
  padding: 0.75rem;
  white-space: pre-wrap;
}

@media (width <= 860px) {
  .detail-header {
    flex-direction: column;
  }

  .detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>
