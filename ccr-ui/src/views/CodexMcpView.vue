<template>
  <div class="stage-page p-6 lg:p-10">
    <div class="mx-auto max-w-7xl space-y-5">
      <ModuleSubnav module="codex" />

      <section class="grid gap-4 xl:grid-cols-2">
        <Card
          variant="glass"
          class="p-5"
        >
          <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
            <div class="space-y-3">
              <div class="flex items-center gap-3">
                <div class="flex h-12 w-12 items-center justify-center rounded-2xl border border-pink-500/20 bg-pink-500/10 shadow-lg backdrop-blur-md">
                  <SIcon
                    name="Server"
                    size="w-6 h-6"
                    class="text-pink-300"
                  />
                </div>
                <div>
                  <p class="eyebrow">
                    {{ tt('Codex / MCP control plane', 'Codex / MCP control plane') }}
                  </p>
                  <h1 class="title">
                    {{ tt('Codex MCP 控制台', 'Codex MCP Console') }}
                  </h1>
                  <p class="lead">
                    {{ tt('按官方 mcp_servers 配置面管理 transport、tool scope、auth 注入与启动策略。', 'Manage transport, tool scope, auth injection, and startup policy through the official mcp_servers surface.') }}
                  </p>
                </div>
              </div>

              <div class="flex flex-wrap gap-2">
                <span class="chip chip-pink">control plane</span>
                <span class="chip chip-neutral">official mcp_servers</span>
                <span class="chip chip-emerald">power-user density</span>
              </div>
            </div>

            <div class="flex flex-wrap gap-2">
              <RouterLink to="/codex">
                <Button
                  variant="glass"
                  size="sm"
                >
                  <SIcon
                    name="ArrowLeft"
                    size="w-4 h-4"
                    class="mr-2"
                  />
                  {{ tt('返回概览', 'Back') }}
                </Button>
              </RouterLink>
              <Button
                variant="glass"
                size="sm"
                @click="startCreate('stdio')"
              >
                <SIcon
                  name="Terminal"
                  size="w-4 h-4"
                  class="mr-2"
                />
                {{ tt('新建 STDIO', 'New STDIO') }}
              </Button>
              <Button
                variant="glass"
                size="sm"
                @click="startCreate('http')"
              >
                <SIcon
                  name="Globe"
                  size="w-4 h-4"
                  class="mr-2"
                />
                {{ tt('新建 HTTP', 'New HTTP') }}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                :loading="loading"
                @click="void loadServers(true)"
              >
                <SIcon
                  name="RefreshCw"
                  size="w-4 h-4"
                  class="mr-2"
                />
                {{ tt('刷新', 'Refresh') }}
              </Button>
            </div>
          </div>

          <div class="mt-5 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
            <div class="stat-card">
              <p class="stat-label">
                {{ tt('Server 总数', 'Total servers') }}
              </p>
              <p class="stat-value">
                {{ servers.length }}
              </p>
              <p class="stat-detail">
                {{ activeCount }} live
              </p>
            </div>
            <div class="stat-card">
              <p class="stat-label">
                HTTP transport
              </p>
              <p class="stat-value">
                {{ httpCount }}
              </p>
              <p class="stat-detail">
                {{ authAwareCount }} auth-aware
              </p>
            </div>
            <div class="stat-card">
              <p class="stat-label">
                Scoped tools
              </p>
              <p class="stat-value">
                {{ scopedCount }}
              </p>
              <p class="stat-detail">
                {{ requiredCount }} required
              </p>
            </div>
            <div class="stat-card">
              <p class="stat-label">
                Legacy
              </p>
              <p class="stat-value">
                {{ legacyCount }}
              </p>
              <p class="stat-detail">
                startup_timeout_ms / bearer_token
              </p>
            </div>
          </div>
        </Card>

        <Card
          variant="glass"
          class="p-5"
        >
          <div class="space-y-4">
            <div>
              <p class="eyebrow">
                {{ tt('一手资料', 'First-party docs') }}
              </p>
              <h2 class="panel-title">
                {{ tt('官方能力面', 'Official surface') }}
              </h2>
              <p class="lead">
                {{ tt('这页按官方 Codex 与 MCP 文档重建，不再停留在旧 CRUD 壳层。', 'This page is rebuilt from official Codex and MCP docs instead of the old CRUD shell.') }}
              </p>
            </div>

            <div class="space-y-3">
              <div class="note-row">
                <SIcon
                  name="ArrowRight"
                  size="w-4 h-4"
                  class="text-pink-300"
                /><span>{{ tt('STDIO 更适合本地工具链，HTTP 更适合远程或托管 MCP 服务。', 'Use STDIO for local toolchains and HTTP for remote or hosted MCP services.') }}</span>
              </div>
              <div class="note-row">
                <SIcon
                  name="ArrowRight"
                  size="w-4 h-4"
                  class="text-pink-300"
                /><span>{{ tt('enabled_tools / disabled_tools 用来缩小 Codex 可见的工具面。', 'enabled_tools / disabled_tools shrink what Codex can see.') }}</span>
              </div>
              <div class="note-row">
                <SIcon
                  name="ArrowRight"
                  size="w-4 h-4"
                  class="text-pink-300"
                /><span>{{ tt('bearer_token_env_var 与 env_http_headers 比明文 token 更可控。', 'bearer_token_env_var and env_http_headers are safer than literal tokens.') }}</span>
              </div>
              <div class="note-row">
                <SIcon
                  name="ArrowRight"
                  size="w-4 h-4"
                  class="text-pink-300"
                /><span>{{ tt('required=true 会把 server 升级成启动时必须可用的依赖。', 'required=true upgrades a server into a startup dependency.') }}</span>
              </div>
            </div>

            <div class="grid gap-3 md:grid-cols-2">
              <a
                href="https://developers.openai.com/codex/config-reference#mcp_servers"
                target="_blank"
                rel="noreferrer"
                class="doc-link"
              ><span>Codex Config Reference</span><SIcon
                name="ArrowUpRight"
                size="w-4 h-4"
              /></a>
              <a
                href="https://platform.openai.com/docs/docs-mcp"
                target="_blank"
                rel="noreferrer"
                class="doc-link"
              ><span>OpenAI Docs MCP</span><SIcon
                name="ArrowUpRight"
                size="w-4 h-4"
              /></a>
            </div>

            <button
              type="button"
              class="doc-link text-left"
              @click="prefillDocsPreset"
            >
              <span>{{ tt('填入 OpenAI Docs MCP 模板', 'Prefill OpenAI Docs MCP preset') }}</span>
              <SIcon
                name="Sparkles"
                size="w-4 h-4"
                class="text-pink-300"
              />
            </button>
          </div>
        </Card>
      </section>

      <Card
        variant="glass"
        class="p-5"
      >
        <div class="grid gap-4 xl:grid-cols-4">
          <label class="search-box xl:col-span-1">
            <SIcon
              name="Search"
              size="w-4 h-4"
            />
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="tt('搜索 name、command、url 或 tool scope', 'Search by name, command, URL, or tool scope')"
            >
          </label>

          <div>
            <p class="filter-label">
              Transport
            </p>
            <div class="filter-row">
              <button
                v-for="item in transportOptions"
                :key="item.value"
                type="button"
                class="filter-pill"
                :class="{ 'filter-pill--active': transportFilter === item.value }"
                @click="transportFilter = item.value"
              >
                {{ item.label }}
              </button>
            </div>
          </div>

          <div>
            <p class="filter-label">
              State
            </p>
            <div class="filter-row">
              <button
                v-for="item in stateOptions"
                :key="item.value"
                type="button"
                class="filter-pill"
                :class="{ 'filter-pill--active': stateFilter === item.value }"
                @click="stateFilter = item.value"
              >
                {{ item.label }}
              </button>
            </div>
          </div>

          <div>
            <p class="filter-label">
              Focus
            </p>
            <div class="filter-row">
              <button
                v-for="item in focusOptions"
                :key="item.value"
                type="button"
                class="filter-pill"
                :class="{ 'filter-pill--active': focusFilter === item.value }"
                @click="focusFilter = item.value"
              >
                {{ item.label }}
              </button>
            </div>
          </div>
        </div>
      </Card>

      <section class="workspace-grid">
        <Card
          variant="glass"
          class="p-5"
        >
          <div class="mb-4">
            <p class="eyebrow">
              Inventory
            </p>
            <h2 class="panel-title">
              Servers
            </h2>
            <p class="lead">
              {{ filteredServers.length }} / {{ servers.length }}
            </p>
          </div>
          <div
            v-if="error && !servers.length"
            class="alert-box"
          >
            {{ tt('加载 Codex MCP 服务器失败', 'Failed to load Codex MCP servers') }}: {{ error }}
          </div>

          <div
            v-else-if="!servers.length && !loading"
            class="empty-box"
          >
            <SIcon
              name="Server"
              size="w-8 h-8"
            />
            <h3>{{ tt('还没有 Codex MCP 服务器', 'No Codex MCP servers yet') }}</h3>
            <p>{{ tt('可以先从空白模板开始，或直接填入 OpenAI Docs MCP。', 'Start from a blank template or prefill OpenAI Docs MCP.') }}</p>
            <div class="flex flex-wrap gap-2">
              <Button
                variant="glass"
                size="sm"
                @click="startCreate('stdio')"
              >
                {{ tt('新建 STDIO', 'New STDIO') }}
              </Button>
              <Button
                variant="glass"
                size="sm"
                @click="prefillDocsPreset"
              >
                Docs MCP
              </Button>
            </div>
          </div>

          <div
            v-else-if="!filteredServers.length && !loading"
            class="empty-box"
          >
            <SIcon
              name="SearchX"
              size="w-8 h-8"
            />
            <h3>{{ tt('没有匹配的服务器', 'No matching servers') }}</h3>
            <p>{{ tt('换个关键词或清空筛选再试一次。', 'Try another keyword or clear the filters.') }}</p>
            <Button
              variant="glass"
              size="sm"
              @click="clearFilters"
            >
              {{ tt('清空筛选', 'Clear filters') }}
            </Button>
          </div>

          <div
            v-else
            class="space-y-3"
          >
            <article
              v-for="server in filteredServers"
              :key="server.name"
              class="server-card"
              :class="{ 'server-card--active': selectedServerName === server.name }"
              @click="selectServer(server)"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 space-y-2">
                  <div class="flex flex-wrap items-center gap-2">
                    <span
                      class="state-dot"
                      :class="server.enabled ? 'state-dot--live' : 'state-dot--paused'"
                    />
                    <h3 class="server-name">
                      {{ server.name }}
                    </h3>
                    <span class="mini-chip mini-chip--neutral">{{ server.transport.toUpperCase() }}</span>
                    <span
                      v-if="!server.enabled"
                      class="mini-chip mini-chip--amber"
                    >paused</span>
                    <span
                      v-if="server.required"
                      class="mini-chip mini-chip--rose"
                    >required</span>
                    <span
                      v-if="hasScopedTools(server)"
                      class="mini-chip mini-chip--emerald"
                    >scoped</span>
                    <span
                      v-if="hasLegacyCompatibility(server)"
                      class="mini-chip mini-chip--amber"
                    >legacy</span>
                  </div>
                  <p class="endpoint-text">
                    {{ server.transport === 'http' ? server.url : server.command }}
                  </p>
                </div>

                <button
                  type="button"
                  class="icon-button"
                  @click.stop="void toggleServer(server)"
                >
                  <SIcon
                    :name="server.enabled ? 'ToggleRight' : 'ToggleLeft'"
                    size="w-4 h-4"
                  />
                </button>
              </div>

              <div class="mt-4 grid grid-cols-2 gap-3 xl:grid-cols-4">
                <div class="metric-card">
                  <span>Args</span><strong>{{ server.args.length }}</strong>
                </div>
                <div class="metric-card">
                  <span>Env</span><strong>{{ countEnvInputs(server) }}</strong>
                </div>
                <div class="metric-card">
                  <span>Headers</span><strong>{{ countHeaderInputs(server) }}</strong>
                </div>
                <div class="metric-card">
                  <span>Timeout</span><strong>{{ timeoutLabel(server) }}</strong>
                </div>
              </div>

              <p class="card-note mt-4">
                {{ describeServer(server) }}
              </p>
            </article>
          </div>
        </Card>

        <Card
          variant="glass"
          class="p-5 xl:sticky xl:top-6"
        >
          <div class="mb-4 flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
            <div>
              <p class="eyebrow">
                {{ editorMode === 'edit' ? tt('Edit', 'Edit') : tt('Create', 'Create') }}
              </p>
              <h2 class="panel-title">
                {{ editorMode === 'edit' ? tt('编辑当前 server', 'Edit current server') : tt('新建 Codex MCP server', 'Create a Codex MCP server') }}
              </h2>
              <p class="lead">
                {{ editorMode ? tt('这里直接编辑 Codex 的 transport、tool scope、headers、env 和 timeout。', 'Edit transport, tool scope, headers, env injection, and timeout policy in one place.') : tt('选中一个 server，或者先创建新配置。', 'Select a server or create a new one.') }}
              </p>
            </div>
          </div>

          <div
            v-if="!editorMode"
            class="empty-box"
          >
            <SIcon
              name="Sparkles"
              size="w-8 h-8"
            />
            <h3>{{ tt('选中一个 server，或者先创建新配置', 'Select a server or create a new one') }}</h3>
            <p>{{ tt('旧版页面只覆盖少量字段；这里已经扩到官方 Codex MCP 配置面。', 'The old page only covered a narrow slice; this editor maps the official Codex MCP surface.') }}</p>
          </div>

          <form
            v-else
            class="space-y-4"
            @submit.prevent="void saveDraft()"
          >
            <div class="form-section">
              <div class="grid gap-4 xl:grid-cols-2">
                <label class="field-block">
                  <span>Server name</span>
                  <input
                    v-model="draft.name"
                    class="field-input"
                    type="text"
                    :disabled="editorMode === 'edit'"
                    placeholder="openaiDeveloperDocs"
                  >
                  <small
                    v-if="editorMode === 'edit'"
                    class="field-hint"
                  >{{ tt('现有 server 名称保持锁定；如需改名，建议新建后删除旧项。', 'Existing server keys stay locked. Create a new one and remove the old entry if you need a rename.') }}</small>
                </label>

                <div class="field-block">
                  <span>Transport</span>
                  <div class="flex flex-wrap gap-2">
                    <button
                      type="button"
                      class="toggle-pill"
                      :class="{ 'toggle-pill--active': draft.transport === 'stdio' }"
                      @click="draft.transport = 'stdio'"
                    >
                      STDIO
                    </button>
                    <button
                      type="button"
                      class="toggle-pill"
                      :class="{ 'toggle-pill--active': draft.transport === 'http' }"
                      @click="draft.transport = 'http'"
                    >
                      HTTP
                    </button>
                  </div>
                </div>
              </div>

              <div class="mt-4 flex flex-wrap gap-2">
                <label class="switch-pill"><input
                  v-model="draft.enabled"
                  type="checkbox"
                ><span>Enabled</span></label>
                <label class="switch-pill"><input
                  v-model="draft.required"
                  type="checkbox"
                ><span>Required</span></label>
              </div>
            </div>

            <div class="form-section">
              <div class="grid gap-4 xl:grid-cols-2">
                <label class="field-block">
                  <span>enabled_tools</span>
                  <textarea
                    v-model="draft.enabledToolsText"
                    class="field-textarea"
                    rows="3"
                    placeholder="search&#10;read"
                  />
                </label>
                <label class="field-block">
                  <span>disabled_tools</span>
                  <textarea
                    v-model="draft.disabledToolsText"
                    class="field-textarea"
                    rows="3"
                    placeholder="write"
                  />
                </label>
              </div>
            </div>

            <div class="form-section">
              <div class="grid gap-4 xl:grid-cols-2">
                <label
                  v-if="draft.transport === 'http'"
                  class="field-block"
                ><span>URL</span><input
                  v-model="draft.url"
                  class="field-input"
                  type="text"
                  placeholder="https://api.openai.com/mcp"
                ></label>
                <label
                  v-else
                  class="field-block"
                ><span>Command</span><input
                  v-model="draft.command"
                  class="field-input"
                  type="text"
                  placeholder="npx"
                ></label>
                <label
                  v-if="draft.transport === 'stdio'"
                  class="field-block"
                ><span>cwd</span><input
                  v-model="draft.cwd"
                  class="field-input"
                  type="text"
                  placeholder="/workspace/project"
                ></label>
                <label
                  v-else
                  class="field-block"
                ><span>bearer_token_env_var</span><input
                  v-model="draft.bearer_token_env_var"
                  class="field-input"
                  type="text"
                  placeholder="OPENAI_API_KEY"
                ></label>
              </div>

              <label
                v-if="draft.transport === 'stdio'"
                class="field-block mt-4"
              >
                <span>Args</span>
                <textarea
                  v-model="draft.argsText"
                  class="field-textarea"
                  rows="4"
                  placeholder="-y&#10;@modelcontextprotocol/server-filesystem&#10;/workspace/project"
                />
                <small class="field-hint">{{ tt('建议一行一个参数；如果只写一行，保存时会按空格拆分。', 'Prefer one arg per line. A single-line entry is split on whitespace when saved.') }}</small>
              </label>
            </div>

            <div class="flex flex-wrap gap-2">
              <Button
                variant="primary"
                type="submit"
                :loading="submitting"
              >
                <SIcon
                  name="Save"
                  size="w-4 h-4"
                  class="mr-2"
                />{{ editorMode === 'edit' ? tt('保存变更', 'Save changes') : tt('创建服务器', 'Create server') }}
              </Button>
              <Button
                variant="glass"
                type="button"
                @click="resetDraftFromBaseline"
              >
                {{ tt('恢复表单', 'Reset form') }}
              </Button>
              <Button
                variant="ghost"
                type="button"
                @click="cancelEditor"
              >
                {{ tt('取消编辑', 'Cancel') }}
              </Button>
              <Button
                v-if="editorMode === 'edit'"
                :variant="deleteArmed ? 'danger' : 'outline'"
                type="button"
                @click="armOrDeleteSelected"
              >
                <SIcon
                  name="Trash2"
                  size="w-4 h-4"
                  class="mr-2"
                />{{ deleteArmed ? tt('确认删除', 'Confirm delete') : tt('准备删除', 'Arm delete') }}
              </Button>
            </div>
          </form>
        </Card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onActivated, onMounted, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { addCodexMcpServer, deleteCodexMcpServer, listCodexMcpServers, updateCodexMcpServer } from '@/api'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'
import type { CodexMcpServer, CodexMcpServerRequest, CodexMcpServersResponse } from '@/types'

type Transport = 'stdio' | 'http'
type TransportFilter = 'all' | Transport
type StateFilter = 'all' | 'enabled' | 'disabled'
type FocusFilter = 'all' | 'scoped' | 'required'
type EditorMode = 'create' | 'edit' | null

interface ServerRecord {
  name: string
  enabled: boolean
  transport: Transport
  command: string | null
  args: string[]
  env: Record<string, string>
  env_vars: string[]
  cwd: string | null
  startup_timeout_ms: number | null
  startup_timeout_sec: number | null
  tool_timeout_sec: number | null
  url: string | null
  http_headers: Record<string, string>
  env_http_headers: Record<string, string>
  bearer_token: string | null
  bearer_token_env_var: string | null
  oauth_resource: string | null
  scopes: string[]
  enabled_tools: string[]
  disabled_tools: string[]
  required: boolean
}

interface Draft {
  name: string
  enabled: boolean
  required: boolean
  transport: Transport
  command: string
  url: string
  cwd: string
  argsText: string
  enabledToolsText: string
  disabledToolsText: string
  bearer_token_env_var: string
}

const REFRESH_TTL_MS = 30_000

const { locale } = useI18n()
const uiStore = useUIStore()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)

const servers = ref<ServerRecord[]>([])
const loading = ref(false)
const submitting = ref(false)
const error = ref<string | null>(null)
const searchQuery = ref('')
const transportFilter = ref<TransportFilter>('all')
const stateFilter = ref<StateFilter>('all')
const focusFilter = ref<FocusFilter>('all')
const editorMode = ref<EditorMode>(null)
const selectedServerName = ref<string | null>(null)
const deleteArmed = ref(false)
const lastLoadedAt = ref(0)
const draft = reactive(createDraft())
const draftBaseline = ref(createDraft())

const transportOptions = [
  { value: 'all' as const, label: 'All' },
  { value: 'stdio' as const, label: 'STDIO' },
  { value: 'http' as const, label: 'HTTP' },
]

const stateOptions = [
  { value: 'all' as const, label: 'All' },
  { value: 'enabled' as const, label: 'enabled' },
  { value: 'disabled' as const, label: 'paused' },
]

const focusOptions = [
  { value: 'all' as const, label: 'All' },
  { value: 'scoped' as const, label: 'scoped' },
  { value: 'required' as const, label: 'required' },
]

const filteredServers = computed(() => {
  const keyword = searchQuery.value.trim().toLowerCase()
  return servers.value.filter((server) => {
    if (transportFilter.value !== 'all' && server.transport !== transportFilter.value) return false
    if (stateFilter.value === 'enabled' && !server.enabled) return false
    if (stateFilter.value === 'disabled' && server.enabled) return false
    if (focusFilter.value === 'scoped' && !hasScopedTools(server)) return false
    if (focusFilter.value === 'required' && !server.required) return false
    if (!keyword) return true
    return [server.name, server.command ?? '', server.url ?? '', server.args.join(' '), server.enabled_tools.join(' '), server.disabled_tools.join(' '), server.scopes.join(' ')]
      .join(' ')
      .toLowerCase()
      .includes(keyword)
  })
})

const activeCount = computed(() => servers.value.filter((server) => server.enabled).length)
const httpCount = computed(() => servers.value.filter((server) => server.transport === 'http').length)
const scopedCount = computed(() => servers.value.filter((server) => hasScopedTools(server)).length)
const requiredCount = computed(() => servers.value.filter((server) => server.required).length)
const legacyCount = computed(() => servers.value.filter((server) => hasLegacyCompatibility(server)).length)
const authAwareCount = computed(() => servers.value.filter((server) => hasHttpAuth(server)).length)

function createDraft(transport: Transport = 'stdio'): Draft {
  return { name: '', enabled: true, required: false, transport, command: '', url: '', cwd: '', argsText: '', enabledToolsText: '', disabledToolsText: '', bearer_token_env_var: '' }
}

function resetDraft(source: Draft) {
  Object.assign(draft, createDraft(source.transport), source)
}

function normalizeServer(server: CodexMcpServer): ServerRecord {
  return {
    name: server.name,
    enabled: server.enabled ?? true,
    transport: (server.transport ?? (server.url ? 'http' : 'stdio')) as Transport,
    command: server.command ?? null,
    args: server.args ?? [],
    env: { ...(server.env ?? {}) },
    env_vars: server.env_vars ?? [],
    cwd: server.cwd ?? null,
    startup_timeout_ms: server.startup_timeout_ms ?? null,
    startup_timeout_sec: server.startup_timeout_sec ?? null,
    tool_timeout_sec: server.tool_timeout_sec ?? null,
    url: server.url ?? null,
    http_headers: { ...(server.http_headers ?? {}) },
    env_http_headers: { ...(server.env_http_headers ?? {}) },
    bearer_token: server.bearer_token ?? null,
    bearer_token_env_var: server.bearer_token_env_var ?? null,
    oauth_resource: server.oauth_resource ?? null,
    scopes: server.scopes ?? [],
    enabled_tools: server.enabled_tools ?? [],
    disabled_tools: server.disabled_tools ?? [],
    required: Boolean(server.required),
  }
}

function serverToDraft(server: ServerRecord): Draft {
  return {
    name: server.name,
    enabled: server.enabled,
    required: server.required,
    transport: server.transport,
    command: server.command ?? '',
    url: server.url ?? '',
    cwd: server.cwd ?? '',
    argsText: server.args.join('\n'),
    enabledToolsText: server.enabled_tools.join('\n'),
    disabledToolsText: server.disabled_tools.join('\n'),
    bearer_token_env_var: server.bearer_token_env_var ?? '',
  }
}

function parseArgs(value: string) {
  const trimmed = value.trim()
  if (!trimmed) return null
  const lines = trimmed.split(/\r?\n/).map((item) => item.trim()).filter(Boolean)
  if (lines.length === 1) {
    const split = lines[0].split(/\s+/).filter(Boolean)
    return split.length > 0 ? split : null
  }
  return lines
}

function nullableText(value: string) {
  const trimmed = value.trim()
  return trimmed ? trimmed : null
}

function buildRequestFromDraft(): CodexMcpServerRequest {
  return {
    name: draft.name.trim(),
    enabled: draft.enabled,
    required: draft.required,
    command: draft.transport === 'stdio' ? nullableText(draft.command) : null,
    args: draft.transport === 'stdio' ? parseArgs(draft.argsText) : null,
    enabled_tools: parseArgs(draft.enabledToolsText),
    disabled_tools: parseArgs(draft.disabledToolsText),
    cwd: draft.transport === 'stdio' ? nullableText(draft.cwd) : null,
    url: draft.transport === 'http' ? nullableText(draft.url) : null,
    bearer_token_env_var: draft.transport === 'http' ? nullableText(draft.bearer_token_env_var) : null,
  }
}

function buildRequestFromServer(server: ServerRecord): CodexMcpServerRequest {
  return {
    name: server.name,
    enabled: server.enabled,
    required: server.required,
    command: server.command,
    args: server.args.length > 0 ? [...server.args] : null,
    env: Object.keys(server.env).length > 0 ? { ...server.env } : null,
    env_vars: server.env_vars.length > 0 ? [...server.env_vars] : null,
    cwd: server.cwd,
    startup_timeout_ms: server.startup_timeout_ms,
    startup_timeout_sec: server.startup_timeout_sec,
    tool_timeout_sec: server.tool_timeout_sec,
    url: server.url,
    http_headers: Object.keys(server.http_headers).length > 0 ? { ...server.http_headers } : null,
    env_http_headers: Object.keys(server.env_http_headers).length > 0 ? { ...server.env_http_headers } : null,
    bearer_token: server.bearer_token,
    bearer_token_env_var: server.bearer_token_env_var,
    oauth_resource: server.oauth_resource,
    scopes: server.scopes.length > 0 ? [...server.scopes] : null,
    enabled_tools: server.enabled_tools.length > 0 ? [...server.enabled_tools] : null,
    disabled_tools: server.disabled_tools.length > 0 ? [...server.disabled_tools] : null,
  }
}

function hasScopedTools(server: ServerRecord) {
  return server.enabled_tools.length > 0 || server.disabled_tools.length > 0
}

function hasHttpAuth(server: ServerRecord) {
  return Boolean(server.bearer_token || server.bearer_token_env_var || Object.keys(server.http_headers).length > 0 || Object.keys(server.env_http_headers).length > 0 || server.oauth_resource || server.scopes.length > 0)
}

function hasLegacyCompatibility(server: ServerRecord) {
  return Boolean(server.bearer_token || server.startup_timeout_ms)
}

function countEnvInputs(server: ServerRecord) {
  return Object.keys(server.env).length + server.env_vars.length
}

function countHeaderInputs(server: ServerRecord) {
  return Object.keys(server.http_headers).length + Object.keys(server.env_http_headers).length
}

function timeoutLabel(server: ServerRecord) {
  if (server.tool_timeout_sec) return `${server.tool_timeout_sec}s`
  if (server.startup_timeout_sec) return `start ${server.startup_timeout_sec}s`
  if (server.startup_timeout_ms) return `start ${server.startup_timeout_ms}ms`
  return '—'
}

function describeServer(server: ServerRecord) {
  const parts: string[] = []
  if (hasScopedTools(server)) parts.push(`${server.enabled_tools.length + server.disabled_tools.length} tool rules`)
  if (countEnvInputs(server) > 0) parts.push(`${countEnvInputs(server)} env inputs`)
  if (countHeaderInputs(server) > 0) parts.push(`${countHeaderInputs(server)} header injectors`)
  if (server.required) parts.push('required')
  if (!server.enabled) parts.push('paused')
  if (hasLegacyCompatibility(server)) parts.push('legacy')
  return parts.join(' · ') || 'No extra policy'
}

function selectServer(server: ServerRecord) {
  editorMode.value = 'edit'
  selectedServerName.value = server.name
  deleteArmed.value = false
  const nextDraft = serverToDraft(server)
  draftBaseline.value = { ...nextDraft }
  resetDraft(nextDraft)
}

function startCreate(transport: Transport) {
  editorMode.value = 'create'
  selectedServerName.value = null
  deleteArmed.value = false
  const nextDraft = createDraft(transport)
  draftBaseline.value = { ...nextDraft }
  resetDraft(nextDraft)
}

function prefillDocsPreset() {
  const nextDraft = createDraft('http')
  nextDraft.name = 'openaiDeveloperDocs'
  nextDraft.url = 'https://api.openai.com/mcp'
  nextDraft.bearer_token_env_var = 'OPENAI_API_KEY'
  editorMode.value = 'create'
  selectedServerName.value = null
  deleteArmed.value = false
  draftBaseline.value = { ...nextDraft }
  resetDraft(nextDraft)
}

function clearFilters() {
  searchQuery.value = ''
  transportFilter.value = 'all'
  stateFilter.value = 'all'
  focusFilter.value = 'all'
}

function resetDraftFromBaseline() {
  deleteArmed.value = false
  resetDraft(draftBaseline.value)
}

function cancelEditor() {
  editorMode.value = null
  selectedServerName.value = null
  deleteArmed.value = false
  draftBaseline.value = createDraft()
  resetDraft(createDraft())
}

async function loadServers(force = false) {
  if (!force && loading.value) return
  if (!force && lastLoadedAt.value > 0 && Date.now() - lastLoadedAt.value < REFRESH_TTL_MS) return
  loading.value = true
  error.value = null
  try {
    const data = await listCodexMcpServers<CodexMcpServersResponse>()
    servers.value = Array.isArray(data.servers) ? data.servers.map(normalizeServer) : []
    lastLoadedAt.value = Date.now()
  } catch (reason) {
    const message = reason instanceof Error ? reason.message : String(reason)
    logger.error('Failed to load Codex MCP servers', reason)
    error.value = message
    uiStore.showError(`${tt('加载 Codex MCP 服务器失败', 'Failed to load Codex MCP servers')}: ${message}`)
  } finally {
    loading.value = false
  }
}

function validateDraft() {
  if (!draft.name.trim()) {
    uiStore.showWarning(tt('请填写 server name', 'Server name is required'))
    return false
  }
  if (draft.transport === 'stdio' && !draft.command.trim()) {
    uiStore.showWarning(tt('STDIO 模式需要 command', 'STDIO mode needs a command'))
    return false
  }
  if (draft.transport === 'http' && !draft.url.trim()) {
    uiStore.showWarning(tt('HTTP 模式需要 URL', 'HTTP mode needs a URL'))
    return false
  }
  return true
}

async function saveDraft() {
  if (!validateDraft()) return
  try {
    submitting.value = true
    const currentServer = selectedServerName.value
      ? servers.value.find((server) => server.name === selectedServerName.value)
      : null
    const request = editorMode.value === 'edit' && currentServer
      ? { ...buildRequestFromServer(currentServer), ...buildRequestFromDraft() }
      : buildRequestFromDraft()
    if (editorMode.value === 'edit' && selectedServerName.value) {
      await updateCodexMcpServer(selectedServerName.value, request)
      uiStore.showSuccess(tt('Codex MCP 服务器已更新', 'Codex MCP server updated'))
    } else {
      await addCodexMcpServer(request)
      uiStore.showSuccess(tt('Codex MCP 服务器已创建', 'Codex MCP server created'))
      selectedServerName.value = request.name ?? draft.name.trim()
      editorMode.value = 'edit'
    }
    await loadServers(true)
    if (selectedServerName.value) {
      const refreshed = servers.value.find((server) => server.name === selectedServerName.value)
      if (refreshed) selectServer(refreshed)
    }
  } catch (reason) {
    uiStore.showError(reason instanceof Error ? reason.message : String(reason))
  } finally {
    submitting.value = false
  }
}

async function toggleServer(server: ServerRecord) {
  try {
    const request = buildRequestFromServer(server)
    request.enabled = !server.enabled
    await updateCodexMcpServer(server.name, request)
    uiStore.showSuccess(server.enabled ? tt('服务器已停用', 'Server paused') : tt('服务器已启用', 'Server enabled'))
    await loadServers(true)
    if (selectedServerName.value === server.name) {
      const refreshed = servers.value.find((item) => item.name === server.name)
      if (refreshed) selectServer(refreshed)
    }
  } catch (reason) {
    uiStore.showError(reason instanceof Error ? reason.message : String(reason))
  }
}

async function armOrDeleteSelected() {
  if (editorMode.value !== 'edit' || !selectedServerName.value) return
  if (!deleteArmed.value) {
    deleteArmed.value = true
    return
  }
  try {
    submitting.value = true
    await deleteCodexMcpServer(selectedServerName.value)
    uiStore.showSuccess(tt('Codex MCP 服务器已删除', 'Codex MCP server deleted'))
    cancelEditor()
    await loadServers(true)
  } catch (reason) {
    uiStore.showError(reason instanceof Error ? reason.message : String(reason))
  } finally {
    submitting.value = false
    deleteArmed.value = false
  }
}

onMounted(() => {
  void loadServers(true)
})

onActivated(() => {
  void loadServers(false)
})
</script>

<style scoped>
.eyebrow,
.stat-label,
.filter-label {
  color: var(--stage-text-quiet);
  letter-spacing: 0.18em;
  text-transform: uppercase;
  font-size: 0.75rem;
}

.title,
.panel-title,
.server-name {
  color: var(--stage-text-primary);
}

.title {
  font-family: MapleBright, 'Microsoft YaHei UI', system-ui, sans-serif;
  font-size: 1.875rem;
  font-weight: 700;
  line-height: 1.1;
}

.lead,
.endpoint-text,
.card-note {
  color: var(--stage-text-secondary);
  font-size: 0.875rem;
  line-height: 1.6;
}

.chip,
.mini-chip,
.filter-pill,
.toggle-pill,
.switch-pill {
  border: 1px solid var(--stage-border-soft);
  border-radius: 9999px;
  padding: 0.5rem 0.75rem;
  font-size: 0.75rem;
}

.chip-pink,
.filter-pill--active,
.toggle-pill--active {
  border-color: rgb(236 72 153 / 20%);
  background: rgb(236 72 153 / 10%);
  color: rgb(249 168 212);
}

.chip-neutral,
.mini-chip--neutral,
.filter-pill,
.toggle-pill,
.switch-pill,
.search-box,
.field-input,
.field-textarea,
.stat-card,
.note-row,
.doc-link,
.server-card,
.form-section,
.metric-card {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);
}

.chip-neutral,
.mini-chip--neutral,
.filter-pill,
.toggle-pill,
.switch-pill {
  color: var(--stage-text-secondary);
}

.chip-emerald,
.mini-chip--emerald {
  border-color: rgb(16 185 129 / 20%);
  background: rgb(16 185 129 / 10%);
  color: rgb(110 231 183);
}

.mini-chip--amber {
  border-color: rgb(245 158 11 / 20%);
  background: rgb(245 158 11 / 10%);
  color: rgb(252 211 77);
}

.mini-chip--rose {
  border-color: rgb(244 63 94 / 20%);
  background: rgb(244 63 94 / 10%);
  color: rgb(253 164 175);
}

.stat-card,
.note-row,
.doc-link,
.server-card,
.form-section {
  border-radius: 1rem;
  padding: 1rem;
}

.stat-value {
  color: var(--stage-text-primary);
  margin-top: 0.25rem;
  font-size: 1.125rem;
  font-weight: 600;
}

.stat-detail,
.field-hint {
  color: var(--stage-text-muted);
  margin-top: 0.25rem;
  font-size: 0.75rem;
  line-height: 1.5;
}

.search-box,
.field-input,
.field-textarea {
  width: 100%;
  border-radius: 1rem;
  padding: 0.75rem 1rem;
  color: var(--stage-text-primary);
  outline: none;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.search-box input {
  min-width: 0;
  flex: 1;
  background: transparent;
  outline: none;
}

.field-textarea {
  min-height: 7rem;
  resize: vertical;
}

.alert-box,
.empty-box {
  display: flex;
  min-height: 16rem;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  border-radius: 1rem;
  border: 1px dashed rgb(255 255 255 / 10%);
  padding: 2rem;
  text-align: center;
}

.alert-box {
  min-height: 7rem;
  color: rgb(255 225 233 / 92%);
  background: rgb(120 27 56 / 22%);
  border-color: rgb(234 143 170 / 28%);
}

.server-card {
  cursor: pointer;
  transition: border-color 0.2s ease;
}

.server-card:hover,
.server-card--active {
  border-color: rgb(236 72 153 / 30%);
}

.state-dot {
  height: 0.625rem;
  width: 0.625rem;
  border-radius: 9999px;
}

.state-dot--live {
  background: rgb(16 185 129 / 80%);
}

.state-dot--paused {
  background: rgb(245 158 11 / 80%);
}

.icon-button {
  display: flex;
  height: 2.25rem;
  width: 2.25rem;
  align-items: center;
  justify-content: center;
  border-radius: 0.75rem;
  color: var(--stage-text-secondary);
}

.metric-card {
  border-radius: 1rem;
  padding: 0.75rem;
}

.metric-card span {
  display: block;
  color: var(--stage-text-quiet);
  font-size: 0.75rem;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.metric-card strong {
  display: block;
  margin-top: 0.25rem;
  color: var(--stage-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field-block span {
  color: var(--stage-text-primary);
  font-size: 0.875rem;
  font-weight: 500;
}

@media (width <= 1024px) {
  .workspace-grid {
    grid-template-columns: 1fr;
  }
}
</style>
