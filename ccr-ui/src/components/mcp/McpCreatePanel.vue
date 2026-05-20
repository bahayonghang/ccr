<template>
  <div class="mcp-form-panel">
    <div class="form-panel-header">
      <h2 class="form-panel-header__title">
        {{ isEditing ? $t('mcp.manager.form.editTitle') : $t('mcp.manager.form.addTitle') }}
      </h2>
      <button
        type="button"
        class="detail-btn"
        @click="$emit('cancel')"
      >
        <SIcon
          name="X"
          size="w-4 h-4"
        />
      </button>
    </div>

    <form
      class="form-panel-body"
      @submit.prevent="$emit('submit')"
    >
      <!-- 目标平台 -->
      <div class="form-field">
        <label class="form-field__label">{{ $t('mcp.manager.form.targetPlatform') }}</label>
        <select
          :value="formData.platform"
          class="form-field__input"
          :disabled="isEditing"
          @change="updateField('platform', ($event.target as HTMLSelectElement).value)"
        >
          <option
            v-for="p in platforms"
            :key="p"
            :value="p"
          >
            {{ platformMeta[p]?.label ?? p }}
          </option>
        </select>
      </div>

      <div
        v-if="formData.platform === 'claude'"
        class="form-field"
      >
        <label class="form-field__label">{{ $t('mcp.manager.form.claudeScope') }}</label>
        <select
          :value="formData.scope ?? 'user'"
          class="form-field__input"
          @change="updateField('scope', ($event.target as HTMLSelectElement).value)"
        >
          <option value="user">
            {{ $t('mcp.manager.scopes.user') }} — ~/.claude.json
          </option>
          <option value="local">
            {{ $t('mcp.manager.scopes.local') }} — {{ $t('mcp.manager.form.currentProjectEntry') }}
          </option>
          <option value="project">
            {{ $t('mcp.manager.scopes.project') }} — {{ $t('mcp.manager.form.repositoryMcpJson') }}
          </option>
        </select>
        <p
          v-if="formData.scope === 'project'"
          class="form-field__hint form-field__hint--warning"
        >
          {{ $t('mcp.manager.form.projectScopeWarningPrefix') }}
          <code>.mcp.json</code>
          {{ $t('mcp.manager.form.projectScopeWarningSuffix') }}
        </p>
      </div>

      <!-- 名称 -->
      <div class="form-field">
        <label class="form-field__label">{{ $t('mcp.manager.form.nameLabel') }} <span class="text-red-400">*</span></label>
        <input
          :value="formData.name ?? ''"
          type="text"
          class="form-field__input"
          placeholder="my-mcp-server"
          :disabled="isEditing"
          @input="updateField('name', ($event.target as HTMLInputElement).value)"
        >
      </div>

      <!-- 协议切换 -->
      <div class="form-field">
        <label class="form-field__label">{{ $t('mcp.manager.form.protocolLabel') }}</label>
        <div class="protocol-toggle">
          <button
            type="button"
            class="protocol-toggle__btn"
            :class="{ 'protocol-toggle__btn--active': !isHttpMode }"
            @click="$emit('update:isHttpMode', false)"
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
            @click="$emit('update:isHttpMode', true)"
          >
            <SIcon
              name="Globe"
              size="w-4 h-4"
            />
            HTTP
          </button>
        </div>
      </div>

      <!-- STDIO: Command -->
      <div
        v-if="!isHttpMode"
        class="form-field"
      >
        <label class="form-field__label">{{ $t('mcp.manager.detail.commandLabel') }} <span class="text-red-400">*</span></label>
        <input
          :value="formData.command ?? ''"
          type="text"
          class="form-field__input form-field__input--mono"
          placeholder="npx -y @example/mcp-server"
          @input="updateField('command', ($event.target as HTMLInputElement).value)"
        >
      </div>

      <!-- HTTP: URL -->
      <div
        v-if="isHttpMode"
        class="form-field"
      >
        <label class="form-field__label">{{ $t('mcp.manager.detail.urlLabel') }} <span class="text-red-400">*</span></label>
        <input
          :value="formData.url ?? ''"
          type="text"
          class="form-field__input form-field__input--mono"
          placeholder="http://localhost:3000/mcp"
          @input="updateField('url', ($event.target as HTMLInputElement).value)"
        >
      </div>

      <!-- STDIO: Args -->
      <div
        v-if="!isHttpMode"
        class="form-field"
      >
        <label class="form-field__label">{{ $t('mcp.manager.form.argsLabel') }}</label>
        <input
          :value="argInput"
          type="text"
          class="form-field__input form-field__input--mono"
          placeholder="--port 3000 --verbose"
          @input="$emit('update:argInput', ($event.target as HTMLInputElement).value)"
        >
      </div>

      <!-- 环境变量 -->
      <div
        v-if="!isHttpMode"
        class="form-field"
      >
        <label class="form-field__label">{{ $t('mcp.manager.form.envLabel') }}</label>
        <div class="kv-editor">
          <div
            v-for="(value, key) in (formData.env ?? {})"
            :key="key"
            class="kv-editor__row"
          >
            <span class="kv-editor__key">{{ key }}</span>
            <span class="kv-editor__value">{{ maskValue(String(value)) }}</span>
            <button
              type="button"
              class="kv-editor__remove"
              @click="$emit('removeEnv', String(key))"
            >
              <SIcon
                name="X"
                size="w-3 h-3"
              />
            </button>
          </div>
          <div class="kv-editor__add">
            <input
              :value="envKey"
              type="text"
              class="form-field__input"
              :placeholder="$t('mcp.manager.form.envKeyPlaceholder')"
              @input="$emit('update:envKey', ($event.target as HTMLInputElement).value)"
            >
            <input
              :value="envValue"
              type="text"
              class="form-field__input"
              :placeholder="$t('mcp.manager.form.envValuePlaceholder')"
              @input="$emit('update:envValue', ($event.target as HTMLInputElement).value)"
            >
            <button
              type="button"
              class="detail-btn"
              :disabled="!envKey || !envValue"
              @click="$emit('addEnv')"
            >
              <SIcon
                name="Plus"
                size="w-4 h-4"
              />
            </button>
          </div>
        </div>
      </div>

      <!-- HTTP: Headers -->
      <div
        v-if="isHttpMode"
        class="form-field"
      >
        <label class="form-field__label">{{ $t('mcp.manager.form.headersLabel') }}</label>
        <div class="kv-editor">
          <div
            v-for="(value, key) in (formData.headers ?? {})"
            :key="key"
            class="kv-editor__row"
          >
            <span class="kv-editor__key">{{ key }}</span>
            <span class="kv-editor__value">{{ maskValue(String(value)) }}</span>
            <button
              type="button"
              class="kv-editor__remove"
              @click="$emit('removeHeader', String(key))"
            >
              <SIcon
                name="X"
                size="w-3 h-3"
              />
            </button>
          </div>
          <div class="kv-editor__add">
            <input
              :value="headerKey"
              type="text"
              class="form-field__input"
              placeholder="Header-Name"
              @input="$emit('update:headerKey', ($event.target as HTMLInputElement).value)"
            >
            <input
              :value="headerValue"
              type="text"
              class="form-field__input"
              placeholder="header-value"
              @input="$emit('update:headerValue', ($event.target as HTMLInputElement).value)"
            >
            <button
              type="button"
              class="detail-btn"
              :disabled="!headerKey || !headerValue"
              @click="$emit('addHeader')"
            >
              <SIcon
                name="Plus"
                size="w-4 h-4"
              />
            </button>
          </div>
        </div>
      </div>

      <!-- 提交 -->
      <div class="form-panel-footer">
        <button
          type="button"
          class="detail-btn"
          @click="$emit('cancel')"
        >
          {{ $t('common.cancel') }}
        </button>
        <button
          type="submit"
          class="detail-btn detail-btn--primary"
        >
          <SIcon
            name="Check"
            size="w-4 h-4"
          />
          <span>{{ isEditing ? $t('common.save') : $t('mcp.manager.form.create') }}</span>
        </button>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { UnifiedMcpRequest, PlatformMeta, UnifiedMcpPlatform } from '@/types/unifiedMcp'

defineProps<{
  /** 是否为编辑模式 */
  isEditing: boolean
  /** 表单数据 */
  formData: UnifiedMcpRequest
  /** 协议模式 */
  isHttpMode: boolean
  /** 参数输入 */
  argInput: string
  /** 环境变量 key */
  envKey: string
  /** 环境变量 value */
  envValue: string
  /** header key */
  headerKey: string
  /** header value */
  headerValue: string
  /** 支持的平台列表 */
  platforms: UnifiedMcpPlatform[]
  /** 平台元信息 */
  platformMeta: Record<string, PlatformMeta>
}>()

const emit = defineEmits<{
  submit: []
  cancel: []
  'update:isHttpMode': [value: boolean]
  'update:argInput': [value: string]
  'update:envKey': [value: string]
  'update:envValue': [value: string]
  'update:headerKey': [value: string]
  'update:headerValue': [value: string]
  'updateField': [field: keyof UnifiedMcpRequest, value: unknown]
  addEnv: []
  removeEnv: [key: string]
  addHeader: []
  removeHeader: [key: string]
}>()

function updateField(field: keyof UnifiedMcpRequest, value: unknown) {
  emit('updateField', field, value)
}

function maskValue(value: string): string {
  if (!value) return ''
  if (value.includes('•')) return value
  if (value.length <= 8) return '••••••'
  return `${value.slice(0, 4)}••••${value.slice(-2)}`
}
</script>

<style scoped>
.mcp-form-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.form-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 45%);

}

.form-panel-header__title {
  font-size: 1rem;
  font-weight: 700;
  color: var(--color-text-primary);

}

.form-panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 1.25rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;

}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;

}

.form-field__label {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--color-text-muted);

}

.form-field__input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 0.75rem;
  border: 1px solid var(--surface-status-border, rgb(var(--color-border-default-rgb) / 55%));
  background: var(--surface-status-bg);
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  outline: none;
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease);

}
.form-field__input::placeholder { color: rgb(var(--color-text-muted-rgb) / 60%); }

.form-field__input:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: var(--elevation-2);
}

.form-field__input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-field__hint {
  color: var(--color-text-muted);
  font-size: 0.72rem;
  line-height: 1.45;
}

.form-field__hint code {
  font-family: var(--font-mono);
}

.form-field__hint--warning {
  color: rgb(var(--color-warning-rgb, 245 158 11) / 92%);
}

.form-field__input--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

/* 协议切换 */
.protocol-toggle {
  display: flex;
  gap: 0.25rem;
  padding: 0.25rem;
  border-radius: 0.75rem;
  background: rgb(var(--color-bg-base-rgb) / 55%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 35%);
}

.protocol-toggle__btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  flex: 1;
  justify-content: center;
  padding: 0.4375rem 0.75rem;
  border-radius: 0.5rem;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--motion-subtle-duration) var(--motion-subtle-ease);

}

.protocol-toggle__btn--active {
  background: var(--surface-card-bg);
  color: var(--color-text-primary);
  box-shadow: var(--elevation-1);

}

/* KV 编辑器 */
.kv-editor {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.kv-editor__row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.5rem;
  border-radius: 0.5rem;
  background: rgb(var(--color-bg-base-rgb) / 42%);

}

.kv-editor__key {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-primary);

}

.kv-editor__value {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--color-text-muted);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}

.kv-editor__remove {
  color: var(--color-text-muted);
  cursor: pointer;
  flex-shrink: 0;
}
.kv-editor__remove:hover { color: rgb(239 68 68); }

.kv-editor__add {
  display: flex;
  gap: 0.375rem;
  align-items: center;
}

/* Footer */
.form-panel-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid rgb(var(--color-border-default-rgb) / 45%);
}

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
  transition: all var(--motion-subtle-duration) var(--motion-subtle-ease);

}

.detail-btn:hover {
  color: var(--color-text-primary);
  background: var(--surface-card-bg);

}

.detail-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.detail-btn--primary {
  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 18%), rgb(var(--color-accent-secondary-rgb) / 10%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  color: var(--color-text-primary);

}
</style>
