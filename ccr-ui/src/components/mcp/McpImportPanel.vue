<template>
  <div class="mcp-import-panel">
    <div class="form-panel-header">
      <h2 class="form-panel-header__title">
        {{ t('mcp.manager.import.title') }}
      </h2>
      <button
        type="button"
        class="import-close-btn"
        @click="$emit('cancel')"
      >
        <SIcon
          name="X"
          size="w-4 h-4"
        />
      </button>
    </div>

    <div class="import-body">
      <p class="import-hint">
        {{ t('mcp.manager.import.hintPrefix') }}
        <code>mcpServers</code>
        {{ t('mcp.manager.import.hintSuffix') }}
      </p>

      <div class="form-field">
        <label class="form-field__label">{{ t('mcp.manager.form.targetPlatform') }}</label>
        <select
          v-model="targetPlatform"
          class="form-field__input"
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
        v-if="targetPlatform === 'claude'"
        class="form-field"
      >
        <label class="form-field__label">{{ t('mcp.manager.form.claudeScope') }}</label>
        <select
          v-model="targetScope"
          class="form-field__input"
        >
          <option value="user">
            {{ t('mcp.manager.scopes.user') }} — {{ claudeUserScopePath }}
          </option>
          <option value="local">
            {{ t('mcp.manager.scopes.local') }} — {{ t('mcp.manager.form.currentProjectEntry') }}
          </option>
          <option value="project">
            {{ t('mcp.manager.scopes.project') }} — {{ t('mcp.manager.form.repositoryMcpJson') }}
          </option>
        </select>
      </div>

      <div class="form-field">
        <label class="form-field__label">{{ t('mcp.manager.import.jsonLabel') }}</label>
        <textarea
          v-model="jsonInput"
          class="import-textarea"
          placeholder="{ &quot;mcpServers&quot;: { &quot;my-server&quot;: { &quot;command&quot;: &quot;npx&quot;, &quot;args&quot;: [&quot;-y&quot;, &quot;my-mcp&quot;] } } }"
          rows="12"
        />
      </div>

      <div
        v-if="parseError"
        class="import-error"
      >
        <SIcon
          name="AlertCircle"
          size="w-4 h-4"
        />
        <span>{{ parseError }}</span>
      </div>

      <div
        v-if="parsedServers.length > 0"
        class="import-preview"
      >
        <h3 class="form-field__label">
          {{ t('mcp.manager.import.previewTitle', { count: parsedServers.length }) }}
        </h3>
        <div
          v-for="server in parsedServers"
          :key="server.name"
          class="import-preview__item"
        >
          <SIcon
            :name="server.type === 'stdio' ? 'Terminal' : 'Globe'"
            size="w-4 h-4"
            class="text-text-muted"
          />
          <span class="import-preview__name">{{ server.name }}</span>
          <span class="import-preview__type">{{ server.type }}</span>
        </div>
      </div>
    </div>

    <div class="import-footer">
      <button
        type="button"
        class="import-btn"
        @click="$emit('cancel')"
      >
        {{ t('common.cancel') }}
      </button>
      <button
        type="button"
        class="import-btn import-btn--primary"
        :disabled="parsedServers.length === 0"
        @click="handleImport"
      >
        <SIcon
          name="Download"
          size="w-4 h-4"
        />
        <span>{{ t('mcp.manager.import.submit', { count: parsedServers.length }) }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { PlatformMeta, UnifiedMcpPlatform } from '@/types/unifiedMcp'

interface ParsedServer {
  name: string
  type: 'stdio' | 'http'
  command?: string
  args?: string[]
  url?: string
  env?: Record<string, string>
  headers?: Record<string, string>
}

const props = defineProps<{
  platforms: UnifiedMcpPlatform[]
  platformMeta: Record<string, PlatformMeta>
}>()

const { t } = useI18n({ useScope: 'global' })

const emit = defineEmits<{
  cancel: []
  import: [servers: ParsedServer[], platform: string, scope?: string]
}>()

const jsonInput = ref('')
const targetPlatform = ref<UnifiedMcpPlatform>(props.platforms[0] ?? 'claude')
const targetScope = ref('user')
const parseError = ref('')
const parsedServers = ref<ParsedServer[]>([])
const claudeUserScopePath = '~/.claude.json'

watch(jsonInput, (value) => {
  parseError.value = ''
  parsedServers.value = []

  if (!value.trim()) return

  try {
    const parsed = JSON.parse(value)
    const mcpServers = parsed.mcpServers ?? parsed

    if (typeof mcpServers !== 'object' || mcpServers === null) {
      parseError.value = t('mcp.manager.import.errors.invalidFormat')
      return
    }

    const servers: ParsedServer[] = []
    for (const [name, config] of Object.entries(mcpServers)) {
      const cfg = config as Record<string, unknown>
      const hasCommand = typeof cfg.command === 'string'
      const hasUrl = typeof cfg.url === 'string'

      if (!hasCommand && !hasUrl) {
        parseError.value = t('mcp.manager.import.errors.missingCommandOrUrl', { name })
        return
      }

      servers.push({
        name,
        type: hasCommand ? 'stdio' : 'http',
        command: hasCommand ? String(cfg.command) : undefined,
        args: Array.isArray(cfg.args) ? cfg.args.map(String) : undefined,
        url: hasUrl ? String(cfg.url) : undefined,
        env: typeof cfg.env === 'object' && cfg.env !== null
          ? Object.fromEntries(Object.entries(cfg.env).map(([k, v]) => [k, String(v)]))
          : undefined,
        headers: typeof cfg.headers === 'object' && cfg.headers !== null
          ? Object.fromEntries(Object.entries(cfg.headers).map(([k, v]) => [k, String(v)]))
          : undefined,
      })
    }

    parsedServers.value = servers
  } catch {
    parseError.value = t('mcp.manager.import.errors.invalidJson')
  }
})

function handleImport() {
  if (parsedServers.value.length > 0) {
    emit(
      'import',
      parsedServers.value,
      targetPlatform.value,
      targetPlatform.value === 'claude' ? targetScope.value : undefined,
    )
  }
}
</script>

<style scoped>
.mcp-import-panel {
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

.import-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: 0.5rem;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--motion-subtle-duration) var(--motion-subtle-ease);

}

.import-close-btn:hover {
  color: var(--color-text-primary);
  background: var(--surface-status-bg);
}

.import-body {
  flex: 1;
  overflow-y: auto;
  padding: 1.25rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;

}

.import-hint {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  line-height: 1.5;

}

.import-hint code {
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  background: rgb(var(--color-bg-base-rgb) / 55%);
  font-family: var(--font-mono);
  font-size: 0.75rem;

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

}

.form-field__input:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: var(--elevation-2);
}

.import-textarea {
  width: 100%;
  padding: 0.75rem;
  border-radius: 0.75rem;
  border: 1px solid var(--surface-status-border, rgb(var(--color-border-default-rgb) / 55%));
  background: var(--surface-status-bg);
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--color-text-primary);
  outline: none;
  resize: vertical;

}

.import-textarea:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: var(--elevation-2);
}

.import-error {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 0.75rem;
  border-radius: 0.625rem;
  background: rgb(239 68 68 / 8%);
  border: 1px solid rgb(239 68 68 / 20%);
  font-size: 0.8125rem;
  color: rgb(239 68 68);

}

.import-preview {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;

}

.import-preview__item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.625rem;
  border-radius: 0.5rem;
  background: rgb(var(--color-bg-base-rgb) / 42%);

}

.import-preview__name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
  flex: 1;
}

.import-preview__type {
  font-size: 0.6875rem;
  color: var(--color-text-muted);
  text-transform: uppercase;
}

.import-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid rgb(var(--color-border-default-rgb) / 45%);

}

.import-btn {
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

.import-btn:hover {
  color: var(--color-text-primary);
  background: var(--surface-card-bg);
}

.import-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.import-btn--primary {
  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 18%), rgb(var(--color-accent-secondary-rgb) / 10%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  color: var(--color-text-primary);

}
</style>
