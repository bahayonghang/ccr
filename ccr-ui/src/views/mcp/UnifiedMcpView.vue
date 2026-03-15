<template>
  <div class="unified-mcp-view">
    <!-- Command Bar (合并 Header + Platform Stats + Filter Bar) -->
    <div class="command-bar">
      <!-- Row 1: 标题 + 搜索 + 操作按钮 -->
      <div class="command-bar__row">
        <div class="command-bar__title">
          MCP 服务器
          <span
            v-if="servers.length > 0"
            class="command-bar__badge"
          >
            {{ servers.length }}
          </span>
        </div>
        <div class="command-bar__search">
          <SIcon
            name="Search"
            size="w-4 h-4"
            class="text-[var(--color-text-muted)]"
          />
          <input
            v-model="filterKeyword"
            type="text"
            placeholder="搜索服务器名称、命令或 URL..."
            class="command-bar__search-input"
          >
          <button
            v-if="filterKeyword"
            class="command-bar__search-clear"
            @click="filterKeyword = ''"
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
            @click="openAddForm()"
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
            @click="loadServers()"
          >
            <SIcon
              name="RefreshCw"
              size="w-4 h-4"
              :class="{ 'animate-spin': loading }"
            />
          </button>
        </div>
      </div>
      <!-- Row 2: 平台芯片 + 协议切换 -->
      <div class="command-bar__row">
        <div class="command-bar__platforms">
          <button
            class="stat-chip"
            :class="{ 'stat-chip--active': filterPlatform === '' }"
            @click="filterPlatform = ''"
          >
            <span class="stat-chip__label">全部</span>
            <span class="stat-chip__count">{{ servers.length }}</span>
          </button>
          <button
            v-for="p in ALL_PLATFORMS"
            :key="p"
            class="stat-chip"
            :class="{ 'stat-chip--active': filterPlatform === p }"
            :style="{ '--chip-color': PLATFORM_META[p].color }"
            @click="filterPlatform = filterPlatform === p ? '' : p"
          >
            <span
              class="stat-chip__dot"
              :style="{ background: PLATFORM_META[p].color }"
            />
            <span class="stat-chip__label">{{ PLATFORM_META[p].label }}</span>
            <span class="stat-chip__count">{{ platformCounts[p] || 0 }}</span>
          </button>
        </div>
        <div class="command-bar__protocol">
          <button
            v-for="opt in protocolOptions"
            :key="opt.value"
            class="protocol-btn"
            :class="{ 'protocol-btn--active': filterProtocol === opt.value }"
            @click="filterProtocol = opt.value"
          >
            {{ opt.label }}
          </button>
        </div>
      </div>
    </div>

    <!-- Server List -->
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
        @click="loadServers()"
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
        <!-- Card Header -->
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

        <!-- Card Body -->
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

        <!-- Card Actions -->
        <div class="server-card__actions">
          <button
            v-if="supportsFeature(server.platform, 'supports_toggle')"
            class="action-btn"
            :title="server.disabled ? '启用' : '禁用'"
            @click="toggleServer(server)"
          >
            <SIcon
              :name="server.disabled ? 'ToggleLeft' : 'ToggleRight'"
              size="w-4 h-4"
            />
          </button>
          <button
            class="action-btn"
            title="编辑"
            @click="openEditForm(server)"
          >
            <SIcon
              name="Pencil"
              size="w-4 h-4"
            />
          </button>
          <button
            class="action-btn action-btn--danger"
            title="删除"
            @click="handleDelete(server)"
          >
            <SIcon
              name="Trash2"
              size="w-4 h-4"
            />
          </button>
        </div>
      </div>
    </div>

    <!-- Add/Edit Modal -->
    <Teleport to="body">
      <div
        v-if="showForm"
        class="modal-overlay"
        @click.self="closeForm"
      >
        <div class="modal-content">
          <div class="modal-header">
            <h2>{{ editingServer ? '编辑 MCP 服务器' : '添加 MCP 服务器' }}</h2>
            <button
              class="modal-close"
              @click="closeForm"
            >
              <SIcon
                name="X"
                size="w-5 h-5"
              />
            </button>
          </div>

          <form
            class="modal-form"
            @submit.prevent="submitForm"
          >
            <!-- Platform Select -->
            <div class="form-group">
              <label class="form-label">目标平台</label>
              <select
                v-model="formData.platform"
                class="form-select"
                :disabled="!!editingServer"
              >
                <option
                  v-for="p in ALL_PLATFORMS"
                  :key="p"
                  :value="p"
                >
                  {{ PLATFORM_META[p].label }}
                </option>
              </select>
            </div>

            <!-- Name -->
            <div class="form-group">
              <label class="form-label">名称 <span class="text-red-400">*</span></label>
              <input
                v-model="formData.name"
                type="text"
                class="form-input"
                placeholder="my-mcp-server"
                :disabled="!!editingServer"
              >
            </div>

            <!-- Protocol Toggle -->
            <div class="form-group">
              <label class="form-label">协议类型</label>
              <div class="protocol-toggle">
                <button
                  type="button"
                  class="protocol-toggle__btn"
                  :class="{ 'protocol-toggle__btn--active': !isHttpMode }"
                  @click="isHttpMode = false"
                >
                  <SIcon
                    name="Terminal"
                    size="w-4 h-4"
                  />
                  STDIO
                </button>
                <button
                  type="button"
                  class="protocol-toggle__btn"
                  :class="{ 'protocol-toggle__btn--active': isHttpMode }"
                  @click="isHttpMode = true"
                >
                  <SIcon
                    name="Globe"
                    size="w-4 h-4"
                  />
                  HTTP
                </button>
              </div>
            </div>

            <!-- Command (STDIO) -->
            <div
              v-if="!isHttpMode"
              class="form-group"
            >
              <label class="form-label">Command <span class="text-red-400">*</span></label>
              <input
                v-model="formData.command"
                type="text"
                class="form-input"
                placeholder="npx -y @example/mcp-server"
              >
            </div>

            <!-- URL (HTTP) -->
            <div
              v-if="isHttpMode"
              class="form-group"
            >
              <label class="form-label">URL <span class="text-red-400">*</span></label>
              <input
                v-model="formData.url"
                type="text"
                class="form-input"
                placeholder="http://localhost:3000/mcp"
              >
            </div>

            <!-- Args -->
            <div
              v-if="!isHttpMode"
              class="form-group"
            >
              <label class="form-label">参数 (空格分隔)</label>
              <input
                v-model="argInput"
                type="text"
                class="form-input"
                placeholder="--port 3000 --verbose"
              >
            </div>

            <!-- Env -->
            <div class="form-group">
              <label class="form-label">环境变量</label>
              <div class="kv-list">
                <div
                  v-for="(val, key) in formData.env"
                  :key="key"
                  class="kv-item"
                >
                  <code>{{ key }}={{ val }}</code>
                  <button
                    type="button"
                    class="kv-remove"
                    @click="removeEnvVar(String(key))"
                  >
                    <SIcon
                      name="X"
                      size="w-3 h-3"
                    />
                  </button>
                </div>
              </div>
              <div class="kv-add">
                <input
                  v-model="envKey"
                  type="text"
                  class="form-input form-input--sm"
                  placeholder="KEY"
                >
                <input
                  v-model="envValue"
                  type="text"
                  class="form-input form-input--sm"
                  placeholder="VALUE"
                >
                <button
                  type="button"
                  class="btn-kv-add"
                  @click="addEnvVar"
                >
                  <SIcon
                    name="Plus"
                    size="w-3.5 h-3.5"
                  />
                </button>
              </div>
            </div>

            <!-- Headers (按能力矩阵显示) -->
            <div
              v-if="currentCapability?.supports_headers"
              class="form-group"
            >
              <label class="form-label">Headers</label>
              <div class="kv-list">
                <div
                  v-for="(val, key) in formData.headers"
                  :key="key"
                  class="kv-item"
                >
                  <code>{{ key }}: {{ val }}</code>
                  <button
                    type="button"
                    class="kv-remove"
                    @click="removeHeader(String(key))"
                  >
                    <SIcon
                      name="X"
                      size="w-3 h-3"
                    />
                  </button>
                </div>
              </div>
              <div class="kv-add">
                <input
                  v-model="headerKey"
                  type="text"
                  class="form-input form-input--sm"
                  placeholder="Header"
                >
                <input
                  v-model="headerValue"
                  type="text"
                  class="form-input form-input--sm"
                  placeholder="Value"
                >
                <button
                  type="button"
                  class="btn-kv-add"
                  @click="addHeader"
                >
                  <SIcon
                    name="Plus"
                    size="w-3.5 h-3.5"
                  />
                </button>
              </div>
            </div>

            <!-- Timeout -->
            <div
              v-if="currentCapability?.supports_timeout"
              class="form-group"
            >
              <label class="form-label">超时 (秒)</label>
              <input
                v-model.number="formData.timeout"
                type="number"
                class="form-input"
                placeholder="30"
                min="0"
              >
            </div>

            <!-- CWD -->
            <div
              v-if="currentCapability?.supports_cwd"
              class="form-group"
            >
              <label class="form-label">工作目录</label>
              <input
                v-model="formData.cwd"
                type="text"
                class="form-input"
                placeholder="/path/to/project"
              >
            </div>

            <!-- Trust -->
            <div
              v-if="currentCapability?.supports_trust"
              class="form-group"
            >
              <label class="form-label-inline">
                <input
                  v-model="formData.trust"
                  type="checkbox"
                  class="form-checkbox"
                >
                信任此服务器 (trust)
              </label>
            </div>

            <!-- Include Tools -->
            <div
              v-if="currentCapability?.supports_include_tools"
              class="form-group"
            >
              <label class="form-label">包含的工具 (逗号分隔)</label>
              <input
                v-model="includeToolInput"
                type="text"
                class="form-input"
                placeholder="tool1, tool2, tool3"
              >
            </div>

            <!-- Submit -->
            <div class="form-actions">
              <button
                type="button"
                class="btn-cancel"
                @click="closeForm"
              >
                取消
              </button>
              <button
                type="submit"
                class="btn-submit"
              >
                {{ editingServer ? '保存' : '添加' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>

    <!-- Delete Confirm Dialog -->
    <Teleport to="body">
      <div
        v-if="showDeleteConfirm"
        class="modal-overlay"
        @click.self="showDeleteConfirm = false"
      >
        <div class="modal-content modal-content--sm">
          <div class="modal-header">
            <h2>确认删除</h2>
            <button
              class="modal-close"
              @click="showDeleteConfirm = false"
            >
              <SIcon
                name="X"
                size="w-5 h-5"
              />
            </button>
          </div>
          <div class="delete-confirm-body">
            <SIcon
              name="AlertCircle"
              size="w-10 h-10"
              class="text-[var(--color-danger)]"
            />
            <p>
              确定要从 <strong>{{ getPlatformLabel(deletingServer?.platform ?? '') }}</strong>
              删除服务器 <strong>{{ deletingServer?.name }}</strong> 吗？
            </p>
            <p class="text-sm text-[var(--color-text-muted)]">
              此操作不可撤销
            </p>
          </div>
          <div class="form-actions">
            <button
              class="btn-cancel"
              @click="showDeleteConfirm = false"
            >
              取消
            </button>
            <button
              class="btn-danger"
              @click="confirmDelete"
            >
              删除
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import { useUnifiedMcp } from '@/composables/useUnifiedMcp'
import type { UnifiedMcpServer, UnifiedMcpPlatform } from '@/types/unifiedMcp'

const {
  PLATFORM_META,
  ALL_PLATFORMS,
  servers,
  loading,
  error,
  filterPlatform,
  filterKeyword,
  filterProtocol,
  filteredServers,
  platformCounts,
  hasActiveFilters,
  showForm,
  editingServer,
  isHttpMode,
  formData,
  argInput,
  envKey,
  envValue,
  headerKey,
  headerValue,
  includeToolInput,
  currentCapability,
  loadServers,
  toggleServer,
  deleteServer,
  openAddForm,
  openEditForm,
  closeForm,
  submitForm,
  addEnvVar,
  removeEnvVar,
  addHeader,
  removeHeader,
  supportsFeature,
} = useUnifiedMcp()

// 删除确认
const showDeleteConfirm = ref(false)
const deletingServer = ref<UnifiedMcpServer | null>(null)

function handleDelete(server: UnifiedMcpServer) {
  deletingServer.value = server
  showDeleteConfirm.value = true
}

async function confirmDelete() {
  if (deletingServer.value) {
    await deleteServer(deletingServer.value)
    showDeleteConfirm.value = false
    deletingServer.value = null
  }
}

// 协议过滤选项
const protocolOptions = [
  { value: 'all' as const, label: '全部' },
  { value: 'stdio' as const, label: 'STDIO' },
  { value: 'http' as const, label: 'HTTP' },
]

// 辅助函数
function getPlatformColor(platform: string): string {
  return PLATFORM_META[platform as UnifiedMcpPlatform]?.color ?? '#6b7280'
}

function getPlatformLabel(platform: string): string {
  return PLATFORM_META[platform as UnifiedMcpPlatform]?.label ?? platform
}

onMounted(() => {
  loadServers()
})
</script>

<style scoped>
/* ============ Layout ============ */
.unified-mcp-view {
  padding: var(--space-5) var(--space-6);
  max-width: 1600px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

/* ============ Command Bar (Glass) ============ */
.command-bar {
  position: sticky;
  top: 0;
  z-index: 20;
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

/* ============ Buttons ============ */
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

/* ============ Platform Chips ============ */
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

/* ============ Protocol Buttons ============ */
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

/* ============ State Screens ============ */
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

/* ============ Server Grid ============ */
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
  font-family: var(--font-mono, monospace);
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
  font-family: var(--font-mono, monospace);
}

/* ============ Card Actions ============ */
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

/* ============ Modal ============ */
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(0 0 0 / 50%);
  backdrop-filter: blur(4px);
  padding: var(--space-4);
}

.modal-content {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-xl);
  width: 100%;
  max-width: 540px;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: var(--shadow-2xl);
}

.modal-content--sm {
  max-width: 420px;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--color-border-default);
}

.modal-header h2 {
  font-size: 1.125rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.modal-close {
  display: flex;
  padding: 4px;
  border: none;
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
}

.modal-close:hover {
  color: var(--color-text-primary);
  background: var(--glass-bg-medium);
}

/* ============ Form ============ */
.modal-form {
  padding: var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.form-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.form-label-inline {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.form-input,
.form-select {
  padding: 8px 12px;
  font-size: 0.8125rem;
  border-radius: var(--radius-md);
  background: var(--glass-bg-light);
  border: 1px solid var(--color-border-default);
  color: var(--color-text-primary);
  outline: none;
  transition: border-color var(--duration-fast);
}

.form-input:focus,
.form-select:focus {
  border-color: var(--color-accent-primary);
}

.form-input:disabled,
.form-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.form-input--sm {
  padding: 6px 8px;
  font-size: 0.75rem;
}

.form-checkbox {
  accent-color: var(--color-accent-primary);
}

.protocol-toggle {
  display: flex;
  border-radius: var(--radius-md);
  overflow: hidden;
  border: 1px solid var(--color-border-default);
}

.protocol-toggle__btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px;
  font-size: 0.8125rem;
  font-weight: 500;
  background: var(--glass-bg-light);
  border: none;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.protocol-toggle__btn:first-child {
  border-right: 1px solid var(--color-border-default);
}

.protocol-toggle__btn--active {
  background: var(--color-accent-primary);
  color: white;
}

/* ============ Key-Value List ============ */
.kv-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.kv-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  padding: 4px 8px;
  background: var(--glass-bg-light);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
}

.kv-item code {
  font-family: var(--font-mono, monospace);
  word-break: break-all;
  color: var(--color-text-secondary);
}

.kv-remove {
  display: flex;
  padding: 2px;
  border: none;
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  flex-shrink: 0;
}
.kv-remove:hover { color: var(--color-danger); }

.kv-add {
  display: flex;
  gap: var(--space-1);
  margin-top: 4px;
}

.btn-kv-add {
  display: flex;
  align-items: center;
  padding: 6px;
  border-radius: var(--radius-sm);
  background: var(--glass-bg-medium);
  border: 1px solid var(--color-border-default);
  color: var(--color-text-secondary);
  cursor: pointer;
  flex-shrink: 0;
}
.btn-kv-add:hover { color: var(--color-accent-primary); }

/* ============ Form Actions ============ */
.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-5) var(--space-4);
}

.btn-cancel {
  padding: 8px 16px;
  font-size: 0.8125rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  background: var(--glass-bg-light);
  border: 1px solid var(--color-border-default);
  color: var(--color-text-secondary);
  cursor: pointer;
}
.btn-cancel:hover { background: var(--glass-bg-medium); }

.btn-submit {
  padding: 8px 20px;
  font-size: 0.8125rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  background: var(--color-accent-primary);
  color: white;
  border: none;
  cursor: pointer;
}
.btn-submit:hover { opacity: 0.85; }

.btn-danger {
  padding: 8px 20px;
  font-size: 0.8125rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  background: var(--color-danger);
  color: white;
  border: none;
  cursor: pointer;
}
.btn-danger:hover { opacity: 0.85; }

/* ============ Delete Confirm ============ */
.delete-confirm-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-6) var(--space-5) var(--space-2);
  text-align: center;
  color: var(--color-text-primary);
}

/* ============ Responsive ============ */

/* >= 1280px: 4列网格 */
@media (width >= 1280px) {
  .server-grid {
    grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  }
}

/* < 768px: 命令栏堆叠 */
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

  .server-grid {
    grid-template-columns: 1fr;
  }
}

/* < 640px: 更紧凑内边距 */
@media (width <= 640px) {
  .unified-mcp-view {
    padding: var(--space-3);
  }

  .command-bar {
    padding: var(--space-3);
    border-radius: var(--radius-lg);
  }
}
</style>
