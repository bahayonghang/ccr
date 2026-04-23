<template>
  <OpenCodePageShell
    title="Settings"
    description="拆分管理 `opencode.json` 与 `tui.json`，把 server/runtime、tools/permissions、theme/keybinds 放回各自语义层。"
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
        保存全部
      </Button>
    </template>

    <div class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_360px]">
      <div class="space-y-5">
        <Card
          variant="glass"
          class="p-5"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            Runtime config · opencode.json
          </h2>
          <div class="mt-4 grid gap-4 md:grid-cols-2">
            <div>
              <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">model</label>
              <input
                v-model="form.model"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              >
            </div>
            <div>
              <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">small_model</label>
              <input
                v-model="form.smallModel"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              >
            </div>
            <div>
              <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">default_agent</label>
              <input
                v-model="form.defaultAgent"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              >
            </div>
            <div>
              <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">share</label>
              <select
                v-model="form.share"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              >
                <option value="manual">
                  manual
                </option>
                <option value="auto">
                  auto
                </option>
                <option value="disabled">
                  disabled
                </option>
              </select>
            </div>
          </div>

          <div class="mt-4 grid gap-4 md:grid-cols-2">
            <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base/35 px-4 py-3 text-sm text-text-primary">
              <input
                v-model="form.snapshot"
                type="checkbox"
              >
              启用 snapshot
            </label>
            <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base/35 px-4 py-3 text-sm text-text-primary">
              <input
                v-model="form.autoupdate"
                type="checkbox"
              >
              启用 autoupdate
            </label>
          </div>
        </Card>

        <Card
          variant="glass"
          class="p-5"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            Server / tools / permissions
          </h2>
          <div class="mt-4 grid gap-4 md:grid-cols-3">
            <div>
              <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">port</label>
              <input
                v-model="form.serverPort"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              >
            </div>
            <div>
              <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">hostname</label>
              <input
                v-model="form.serverHostname"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              >
            </div>
            <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base/35 px-4 py-3 text-sm text-text-primary">
              <input
                v-model="form.serverMdns"
                type="checkbox"
              >
              mDNS
            </label>
          </div>

          <div class="mt-4 grid gap-4 md:grid-cols-2">
            <div>
              <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">tools JSON</label>
              <textarea
                v-model="form.toolsJson"
                rows="8"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
              />
            </div>
            <div>
              <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">permission JSON</label>
              <textarea
                v-model="form.permissionJson"
                rows="8"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
              />
            </div>
          </div>
        </Card>

        <Card
          variant="glass"
          class="p-5"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            TUI config · tui.json
          </h2>
          <div class="mt-4 grid gap-4 md:grid-cols-2">
            <div>
              <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">theme</label>
              <select
                v-model="form.theme"
                class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
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
            <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base/35 px-4 py-3 text-sm text-text-primary">
              <input
                v-model="form.mouse"
                type="checkbox"
              >
              启用 mouse
            </label>
          </div>

          <div class="mt-4">
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">keybinds JSON</label>
            <textarea
              v-model="form.keybindsJson"
              rows="10"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
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
            Instructions
          </h2>
          <p class="mt-2 text-sm text-text-secondary">
            每行一个路径或 glob，会进入 `instructions` 数组。
          </p>
          <textarea
            v-model="form.instructionsText"
            rows="10"
            class="mt-4 w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
          />
        </Card>
      </div>
    </div>
  </OpenCodePageShell>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import OpenCodePageShell from '@/components/opencode/OpenCodePageShell.vue'
import { useUIStore } from '@/stores/ui'
import { getOpenCodeConfig, getOpenCodeTuiSettings, listOpenCodeThemes, updateOpenCodeConfig, updateOpenCodeTuiSettings } from '@/api'
import type { OpenCodeConfig, OpenCodeTheme, OpenCodeTuiConfig } from '@/types'
import { formatJsonInput, normalizeStringListInput, parseJsonInput } from '@/utils/opencode'

const uiStore = useUIStore()
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
      getOpenCodeConfig<OpenCodeConfig>(),
      getOpenCodeTuiSettings<OpenCodeTuiConfig>(),
      listOpenCodeThemes<OpenCodeTheme[]>(),
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
    uiStore.showError(error instanceof Error ? error.message : String(error))
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
    uiStore.showSuccess('OpenCode 设置已保存')
    await loadSettings()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  void loadSettings()
})
</script>
