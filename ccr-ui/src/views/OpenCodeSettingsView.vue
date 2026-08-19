<template>
  <OpenCodePageShell
    :title="tt('设置', 'Settings')"
    :description="tt('拆分管理 `opencode.json` 与 `tui.json`，把 server/runtime、tools/permissions、theme/keybinds 放回各自语义层。', 'Manage `opencode.json` and `tui.json` separately so server/runtime, tools/permissions, and theme/keybinds stay in the right layer.')"
    icon="SlidersHorizontal"
    tone="lime"
    badge="settings"
  >
    <template #actions>
      <Button
        variant="primary"
        surface="card"
        density="compact"
        motion="standard"
        :disabled="saving"
        @click="saveAll"
      >
        <template #leading>
          <SIcon
            v-if="saving"
            name="Loader2"
            size="w-4 h-4"
            class="animate-spin"
          />
          <SIcon
            v-else
            name="Save"
            size="w-4 h-4"
          />
        </template>
        {{ tt('保存全部', 'Save all') }}
      </Button>
    </template>

    <div class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_360px]">
      <div class="space-y-5">
        <Card
          variant="glass"
          class="p-5"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            {{ tt('Runtime config · opencode.json', 'Runtime config · opencode.json') }}
          </h2>
          <div class="mt-4 grid gap-4 md:grid-cols-2">
            <div>
              <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('模型', 'model') }}</label>
              <input
                v-model="form.model"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              >
            </div>
            <div>
              <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('small_model', 'small_model') }}</label>
              <input
                v-model="form.smallModel"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              >
            </div>
            <div>
              <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('default_agent', 'default_agent') }}</label>
              <input
                v-model="form.defaultAgent"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              >
            </div>
            <div>
              <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('share', 'share') }}</label>
              <select
                v-model="form.share"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              >
                <option value="manual">
                  {{ tt('手动', 'manual') }}
                </option>
                <option value="auto">
                  {{ tt('自动', 'auto') }}
                </option>
                <option value="disabled">
                  {{ tt('禁用', 'disabled') }}
                </option>
              </select>
            </div>
          </div>

          <div class="mt-4 grid gap-4 md:grid-cols-2">
            <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary">
              <input
                v-model="form.snapshot"
                type="checkbox"
              >
              {{ tt('启用 snapshot', 'Enable snapshot') }}
            </label>
            <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary">
              <input
                v-model="form.autoupdate"
                type="checkbox"
              >
              {{ tt('启用 autoupdate', 'Enable autoupdate') }}
            </label>
          </div>
        </Card>

        <Card
          variant="glass"
          class="p-5"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            {{ tt('Server / tools / permissions', 'Server / tools / permissions') }}
          </h2>
          <div class="mt-4 grid gap-4 md:grid-cols-3">
            <div>
              <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('端口', 'port') }}</label>
              <input
                v-model="form.serverPort"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              >
            </div>
            <div>
              <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('主机名', 'hostname') }}</label>
              <input
                v-model="form.serverHostname"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              >
            </div>
            <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary">
              <input
                v-model="form.serverMdns"
                type="checkbox"
              >
              {{ tt('mDNS', 'mDNS') }}
            </label>
          </div>

          <div class="mt-4 grid gap-4 md:grid-cols-2">
            <div>
              <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('tools JSON', 'tools JSON') }}</label>
              <textarea
                v-model="form.toolsJson"
                rows="8"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 font-mono text-sm text-text-primary"
              />
            </div>
            <div>
              <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('permission JSON', 'permission JSON') }}</label>
              <textarea
                v-model="form.permissionJson"
                rows="8"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 font-mono text-sm text-text-primary"
              />
            </div>
          </div>
        </Card>

        <Card
          variant="glass"
          class="p-5"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            {{ tt('TUI config · tui.json', 'TUI config · tui.json') }}
          </h2>
          <div class="mt-4 grid gap-4 md:grid-cols-2">
            <div>
              <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('主题', 'theme') }}</label>
              <select
                v-model="form.theme"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              >
                <option
                  v-for="theme in themes"
                  :key="theme.id"
                  :value="theme.id"
                >
                  {{ theme.name }}
                </option>
              </select>
            </div>
            <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary">
              <input
                v-model="form.mouse"
                type="checkbox"
              >
              {{ tt('启用 mouse', 'Enable mouse') }}
            </label>
          </div>

          <div class="mt-4">
            <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('keybinds JSON', 'keybinds JSON') }}</label>
            <textarea
              v-model="form.keybindsJson"
              rows="10"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 font-mono text-sm text-text-primary"
            />
          </div>
        </Card>
      </div>

      <div class="space-y-5">
        <Card
          variant="glass"
          class="p-5"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            {{ tt('Instructions', 'Instructions') }}
          </h2>
          <p class="mt-2 text-sm text-text-secondary">
            {{ tt('每行一个路径或 glob，会进入 `instructions` 数组。', 'Add one path or glob per line. Each entry goes into the `instructions` array.') }}
          </p>
          <textarea
            v-model="form.instructionsText"
            rows="10"
            class="mt-4 w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 font-mono text-sm text-text-primary"
          />
        </Card>
      </div>
    </div>
  </OpenCodePageShell>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getErrorMessage } from '@/utils/errorHandler'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import OpenCodePageShell from '@/components/opencode/OpenCodePageShell.vue'
import { useUIStore } from '@/stores/ui'
import { getOpenCodeConfig, getOpenCodeTuiSettings, listOpenCodeThemes, updateOpenCodeConfig, updateOpenCodeTuiSettings } from '@/api'
import type { OpenCodeConfig, OpenCodeTheme, OpenCodeTuiConfig } from '@/types'
import { formatJsonInput, normalizeStringListInput, parseJsonInput } from '@/utils/opencode'

const { locale } = useI18n()
const uiStore = useUIStore()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)
const saving = ref(false)
const themes = ref<OpenCodeTheme[]>([])

const form = reactive({
  model: '',
  smallModel: '',
  defaultAgent: '',
  share: 'manual' as 'manual' | 'auto' | 'disabled',
  snapshot: true,
  autoupdate: true,
  serverPort: '',
  serverHostname: '',
  serverMdns: false,
  toolsJson: '{}',
  permissionJson: '{}',
  theme: 'system',
  mouse: true,
  keybindsJson: '{}',
  instructionsText: '',
})

async function loadSettings() {
  try {
    const [config, tui, themeList] = await Promise.all([
      getOpenCodeConfig(),
      getOpenCodeTuiSettings(),
      listOpenCodeThemes(),
    ])

    form.model = config.model || ''
    form.smallModel = config.small_model || ''
    form.defaultAgent = config.default_agent || ''
    form.share = config.share || 'manual'
    form.snapshot = config.snapshot !== false
    form.autoupdate = config.autoupdate !== false
    form.serverPort = config.server?.port != null ? String(config.server.port) : ''
    form.serverHostname = config.server?.hostname || ''
    form.serverMdns = Boolean(config.server?.mdns)
    form.toolsJson = formatJsonInput(config.tools || {})
    form.permissionJson = formatJsonInput(config.permission || {})
    form.instructionsText = (config.instructions || []).join('\n')

    form.theme = tui.theme || 'system'
    form.mouse = tui.mouse !== false
    form.keybindsJson = formatJsonInput(tui.keybinds || {})

    themes.value = themeList
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function saveAll() {
  saving.value = true
  try {
    const runtimePatch: OpenCodeConfig = {
      model: form.model.trim() || undefined,
      small_model: form.smallModel.trim() || undefined,
      default_agent: form.defaultAgent.trim() || undefined,
      share: form.share,
      snapshot: form.snapshot,
      autoupdate: form.autoupdate,
      tools: parseJsonInput<Record<string, unknown>>(form.toolsJson, {}),
      permission: parseJsonInput<Record<string, unknown>>(form.permissionJson, {}),
      instructions: normalizeStringListInput(form.instructionsText),
      server: {
        port: form.serverPort.trim() ? Number(form.serverPort.trim()) : undefined,
        hostname: form.serverHostname.trim() || undefined,
        mdns: form.serverMdns,
      },
    }

    const tuiPatch: OpenCodeTuiConfig = {
      theme: form.theme,
      mouse: form.mouse,
      keybinds: parseJsonInput<Record<string, unknown>>(form.keybindsJson, {}),
    }

    await Promise.all([
      updateOpenCodeConfig(runtimePatch),
      updateOpenCodeTuiSettings(tuiPatch),
    ])
    uiStore.showSuccess(tt('OpenCode 设置已保存', 'OpenCode settings saved'))
    await loadSettings()
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  void loadSettings()
})
</script>
