<template>
  <Teleport to="body">
    <div
      v-if="show"
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
          <div class="form-group">
            <label class="form-label">目标平台</label>
            <select
              :value="formData.platform"
              class="form-select"
              :disabled="!!editingServer"
              @change="updateFormField('platform', ($event.target as HTMLSelectElement).value)"
            >
              <option
                v-for="platform in allPlatforms"
                :key="platform"
                :value="platform"
              >
                {{ platformMeta[platform].label }}
              </option>
            </select>
          </div>

          <div class="form-group">
            <label class="form-label">名称 <span class="text-red-400">*</span></label>
            <input
              :value="formData.name ?? ''"
              type="text"
              class="form-input"
              placeholder="my-mcp-server"
              :disabled="!!editingServer"
              @input="updateFormField('name', ($event.target as HTMLInputElement).value)"
            >
          </div>

          <div class="form-group">
            <label class="form-label">协议类型</label>
            <div class="protocol-toggle">
              <button
                type="button"
                class="protocol-toggle__btn"
                :class="{ 'protocol-toggle__btn--active': !isHttpMode }"
                @click="setHttpMode(false)"
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
                @click="setHttpMode(true)"
              >
                <SIcon
                  name="Globe"
                  size="w-4 h-4"
                />
                HTTP
              </button>
            </div>
          </div>

          <div
            v-if="!isHttpMode"
            class="form-group"
          >
            <label class="form-label">Command <span class="text-red-400">*</span></label>
            <input
              :value="formData.command ?? ''"
              type="text"
              class="form-input"
              placeholder="npx -y @example/mcp-server"
              @input="updateFormField('command', ($event.target as HTMLInputElement).value)"
            >
          </div>

          <div
            v-if="isHttpMode"
            class="form-group"
          >
            <label class="form-label">URL <span class="text-red-400">*</span></label>
            <input
              :value="formData.url ?? ''"
              type="text"
              class="form-input"
              placeholder="http://localhost:3000/mcp"
              @input="updateFormField('url', ($event.target as HTMLInputElement).value)"
            >
          </div>

          <div
            v-if="!isHttpMode"
            class="form-group"
          >
            <label class="form-label">参数 (空格分隔)</label>
            <input
              :value="argInput"
              type="text"
              class="form-input"
              placeholder="--port 3000 --verbose"
              @input="updateArgInput(($event.target as HTMLInputElement).value)"
            >
          </div>

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
                :value="envKey"
                type="text"
                class="form-input form-input--sm"
                placeholder="KEY"
                @input="updateEnvKey(($event.target as HTMLInputElement).value)"
              >
              <input
                :value="envValue"
                type="text"
                class="form-input form-input--sm"
                placeholder="VALUE"
                @input="updateEnvValue(($event.target as HTMLInputElement).value)"
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
                :value="headerKey"
                type="text"
                class="form-input form-input--sm"
                placeholder="Header"
                @input="updateHeaderKey(($event.target as HTMLInputElement).value)"
              >
              <input
                :value="headerValue"
                type="text"
                class="form-input form-input--sm"
                placeholder="Value"
                @input="updateHeaderValue(($event.target as HTMLInputElement).value)"
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

          <div
            v-if="currentCapability?.supports_timeout"
            class="form-group"
          >
            <label class="form-label">超时 (秒)</label>
            <input
              :value="formData.timeout ?? ''"
              type="number"
              class="form-input"
              placeholder="30"
              min="0"
              @input="updateTimeout(($event.target as HTMLInputElement).value)"
            >
          </div>

          <div
            v-if="currentCapability?.supports_cwd"
            class="form-group"
          >
            <label class="form-label">工作目录</label>
            <input
              :value="formData.cwd ?? ''"
              type="text"
              class="form-input"
              placeholder="/path/to/project"
              @input="updateFormField('cwd', ($event.target as HTMLInputElement).value)"
            >
          </div>

          <div
            v-if="currentCapability?.supports_trust"
            class="form-group"
          >
            <label class="form-label-inline">
              <input
                :checked="!!formData.trust"
                type="checkbox"
                class="form-checkbox"
                @change="updateFormField('trust', ($event.target as HTMLInputElement).checked)"
              >
              信任此服务器 (trust)
            </label>
          </div>

          <div
            v-if="currentCapability?.supports_include_tools"
            class="form-group"
          >
            <label class="form-label">包含的工具 (逗号分隔)</label>
            <input
              :value="includeToolInput"
              type="text"
              class="form-input"
              placeholder="tool1, tool2, tool3"
              @input="updateIncludeToolInput(($event.target as HTMLInputElement).value)"
            >
          </div>

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
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { PlatformMcpCapability, PlatformMeta, UnifiedMcpPlatform, UnifiedMcpRequest, UnifiedMcpServer } from '@/types/unifiedMcp'

interface Props {
  show: boolean
  editingServer: UnifiedMcpServer | null
  allPlatforms: UnifiedMcpPlatform[]
  platformMeta: Record<UnifiedMcpPlatform, PlatformMeta>
  isHttpMode: boolean
  formData: UnifiedMcpRequest
  argInput: string
  envKey: string
  envValue: string
  headerKey: string
  headerValue: string
  includeToolInput: string
  currentCapability: PlatformMcpCapability | null
  closeForm: () => void
  submitForm: () => void
  setHttpMode: (value: boolean) => void
  updateFormField: (field: keyof UnifiedMcpRequest, value: unknown) => void
  updateArgInput: (value: string) => void
  updateEnvKey: (value: string) => void
  updateEnvValue: (value: string) => void
  updateHeaderKey: (value: string) => void
  updateHeaderValue: (value: string) => void
  updateIncludeToolInput: (value: string) => void
  addEnvVar: () => void
  removeEnvVar: (key: string) => void
  addHeader: () => void
  removeHeader: (key: string) => void
}

const props = defineProps<Props>()

const updateTimeout = (value: string) => {
  if (value === '') {
    props.updateFormField('timeout', null)
    return
  }

  const parsed = Number(value)
  props.updateFormField('timeout', Number.isFinite(parsed) ? parsed : null)
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--layer-popover);
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

.form-input:focus-visible,
.form-select:focus-visible {
  outline: 2px solid rgb(var(--color-accent-primary-rgb) / 50%);
  outline-offset: 2px;
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
  font-family: var(--font-mono);
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
</style>
