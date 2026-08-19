<template>
  <OpenCodePageShell
    :title="tt('MCP 服务器', 'MCP servers')"
    :description="tt('管理 local / remote MCP 定义，并提供官方 CLI auth / debug / logout 动作。', 'Manage local / remote MCP definitions and keep the official CLI auth / debug / logout actions close by.')"
    icon="Server"
    tone="cyan"
    badge="mcp"
  >
    <template #actions>
      <Button
        variant="primary"
        surface="card"
        density="compact"
        motion="standard"
        @click="openCreate('local')"
      >
        <template #leading>
          <SIcon
            name="Plus"
            size="w-4 h-4"
          />
        </template>
        {{ tt('添加服务器', 'Add server') }}
      </Button>
    </template>

    <div class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_320px]">
      <div class="space-y-4">
        <Card
          v-if="loading"
          variant="glass"
          class="p-8 text-center"
        >
          <div class="mx-auto h-8 w-8 rounded-full border-2 border-cyan-300/25 border-t-cyan-300 animate-spin" />
        </Card>

        <Card
          v-else-if="servers.length === 0"
          variant="glass"
          class="p-8 text-center"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            {{ tt('暂无 MCP 服务器', 'No MCP servers yet') }}
          </h2>
          <p class="mt-2 text-sm text-text-secondary">
            {{ tt('可添加本地命令型 server，或远程 HTTP/SSE server。', 'Add a local command-based server or a remote HTTP/SSE server.') }}
          </p>
        </Card>

        <Card
          v-for="server in servers"
          :key="server.id"
          variant="glass"
          class="p-5"
        >
          <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div class="min-w-0">
              <div class="mb-3 flex flex-wrap items-center gap-2">
                <span class="rounded-full border border-cyan-300/20 bg-cyan-300/10 px-3 py-1 text-xs font-semibold text-cyan-200">
                  {{ server.id }}
                </span>
                <span class="rounded-full bg-bg-base px-3 py-1 text-xs font-semibold text-text-secondary">
                  {{ server.type }}
                </span>
                <span
                  class="rounded-full px-3 py-1 text-xs font-semibold"
                  :class="server.enabled === false ? 'bg-amber-300/10 text-amber-200' : 'bg-emerald-300/10 text-emerald-200'"
                >
                  {{ server.enabled === false ? tt('已禁用', 'Disabled') : tt('已启用', 'Enabled') }}
                </span>
              </div>

              <div class="grid gap-3 md:grid-cols-3">
                <div class="rounded-2xl border border-border-default/55 bg-bg-base p-3 md:col-span-2">
                  <span class="text-[11px] font-semibold text-text-muted">{{ tt('入口', 'Entrypoint') }}</span>
                  <p class="mt-2 break-all font-mono text-sm text-text-primary">
                    {{ server.type === 'local' ? stringifyCommandInput(server.command) || tt('缺少 command', 'Missing command') : server.url || tt('缺少 URL', 'Missing URL') }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border-default/55 bg-bg-base p-3">
                  <span class="text-[11px] font-semibold text-text-muted">{{ tt('附加项', 'Extras') }}</span>
                  <p class="mt-2 text-sm text-text-primary">
                    {{ `${tt('环境变量', 'Env')} ${Object.keys(server.environment || {}).length} · ${tt('请求头', 'Headers')} ${Object.keys(server.headers || {}).length}` }}
                  </p>
                </div>
              </div>

              <div
                v-if="server.type === 'remote'"
                class="mt-4 flex flex-wrap gap-2"
              >
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="copyCli(`opencode mcp auth ${server.id}`)"
                >
                  {{ tt('授权', 'Auth') }}
                </Button>
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="copyCli(`opencode mcp debug ${server.id}`)"
                >
                  {{ tt('调试', 'Debug') }}
                </Button>
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="copyCli(`opencode mcp logout ${server.id}`)"
                >
                  {{ tt('登出', 'Logout') }}
                </Button>
              </div>
            </div>

            <div class="flex flex-wrap gap-2">
              <Button
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
                @click="openEdit(server)"
              >
                <template #leading>
                  <SIcon
                    name="Pencil"
                    size="w-4 h-4"
                  />
                </template>
                {{ tt('编辑', 'Edit') }}
              </Button>
              <Button
                variant="danger"
                surface="status"
                density="compact"
                motion="subtle"
                @click="removeServer(server.id)"
              >
                <template #leading>
                  <SIcon
                    name="Trash2"
                    size="w-4 h-4"
                  />
                </template>
                {{ tt('删除', 'Delete') }}
              </Button>
            </div>
          </div>
        </Card>
      </div>

      <Card
        variant="glass"
        class="p-5"
      >
        <h2 class="text-lg font-semibold text-text-primary">
          {{ tt('CLI handoff', 'CLI handoff') }}
        </h2>
        <p class="mt-2 text-sm text-text-secondary">
          {{ tt('OpenCode 的 MCP OAuth 与调试动作本质上还是 CLI 能力，这里直接给你可执行命令。', 'OpenCode still handles MCP OAuth and debugging through the CLI, so this panel gives you the exact commands.') }}
        </p>
        <div class="mt-4 space-y-3">
          <div class="rounded-2xl border border-border-default/55 bg-bg-base p-3">
            <strong class="font-mono text-sm text-text-primary">{{ opencodeMcpAddCommand }}</strong>
            <p class="mt-2 text-sm text-text-secondary">
              {{ tt('交互式添加 local 或 remote server。', 'Interactively add a local or remote server.') }}
            </p>
          </div>
          <div class="rounded-2xl border border-border-default/55 bg-bg-base p-3">
            <strong class="font-mono text-sm text-text-primary">{{ opencodeMcpAuthCommand }}</strong>
            <p class="mt-2 text-sm text-text-secondary">
              {{ tt('OAuth-enabled remote server 登录。', 'Sign in to an OAuth-enabled remote server.') }}
            </p>
          </div>
          <div class="rounded-2xl border border-border-default/55 bg-bg-base p-3">
            <strong class="font-mono text-sm text-text-primary">{{ opencodeMcpDebugCommand }}</strong>
            <p class="mt-2 text-sm text-text-secondary">
              {{ tt('排查 OAuth / transport 连接问题。', 'Troubleshoot OAuth or transport connection issues.') }}
            </p>
          </div>
        </div>
      </Card>
    </div>

    <BaseModal
      v-model="showModal"
      :title="editingId ? tt('编辑 MCP 服务器', 'Edit MCP server') : tt('添加 MCP 服务器', 'Add MCP server')"
      :description="tt('OpenCode `mcp` 配置使用 `local / remote` 两种形态。', 'OpenCode `mcp` settings use `local / remote` server shapes.')"
      size="lg"
      content-class="max-w-2xl"
    >
      <div class="space-y-4">
        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('服务器 ID *', 'Server ID *') }}</label>
            <input
              v-model="form.id"
              :disabled="Boolean(editingId)"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              placeholder="github"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('类型', 'Type') }}</label>
            <select
              v-model="form.type"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
            >
              <option value="local">
                {{ tt('本地', 'local') }}
              </option>
              <option value="remote">
                {{ tt('远程', 'remote') }}
              </option>
            </select>
          </div>
        </div>

        <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary">
          <input
            v-model="form.enabled"
            type="checkbox"
          >
          {{ tt('启用该 MCP server', 'Enable this MCP server') }}
        </label>

        <div v-if="form.type === 'local'">
          <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('命令', 'Command') }}</label>
          <input
            v-model="form.command"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 font-mono text-sm text-text-primary"
            placeholder="npx -y @modelcontextprotocol/server-github"
          >
        </div>

        <div v-else>
          <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('URL', 'URL') }}</label>
          <input
            v-model="form.url"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 font-mono text-sm text-text-primary"
            placeholder="https://mcp.example.com/sse"
          >
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('环境变量 JSON', 'Environment JSON') }}</label>
            <textarea
              v-model="form.environmentJson"
              rows="6"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 font-mono text-sm text-text-primary"
            />
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('请求头 JSON', 'Headers JSON') }}</label>
            <textarea
              v-model="form.headersJson"
              rows="6"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 font-mono text-sm text-text-primary"
            />
          </div>
        </div>

        <div class="flex justify-end gap-3 border-t border-border-default/55 pt-4">
          <Button
            variant="secondary"
            surface="status"
            density="compact"
            motion="subtle"
            @click="showModal = false"
          >
            {{ tt('取消', 'Cancel') }}
          </Button>
          <Button
            variant="primary"
            surface="card"
            density="compact"
            motion="standard"
            :disabled="saving"
            @click="saveServer"
          >
            <template #leading>
              <SIcon
                v-if="saving"
                name="Loader2"
                size="w-4 h-4"
                class="animate-spin"
              />
            </template>
            {{ tt('保存', 'Save') }}
          </Button>
        </div>
      </div>
    </BaseModal>
  </OpenCodePageShell>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getErrorMessage } from '@/utils/errorHandler'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import OpenCodePageShell from '@/components/opencode/OpenCodePageShell.vue'
import { useUIStore } from '@/stores/ui'
import { addOpenCodeMcpServer, deleteOpenCodeMcpServer, listOpenCodeMcpServers, updateOpenCodeMcpServer } from '@/api'
import type { OpenCodeMcpServer } from '@/types'
import { formatJsonInput, parseJsonInput, splitCommandInput, stringifyCommandInput } from '@/utils/opencode'
import { copyText } from '@/utils/clipboard'

const { locale } = useI18n()
const uiStore = useUIStore()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)
const loading = ref(false)
const saving = ref(false)
const showModal = ref(false)
const editingId = ref('')
const servers = ref<OpenCodeMcpServer[]>([])
const opencodeMcpAddCommand = 'opencode mcp add'
const opencodeMcpAuthCommand = 'opencode mcp auth <name>'
const opencodeMcpDebugCommand = 'opencode mcp debug <name>'

const form = reactive({
  id: '',
  type: 'local' as 'local' | 'remote',
  enabled: true,
  command: '',
  url: '',
  environmentJson: '{}',
  headersJson: '{}',
})

async function loadServers() {
  loading.value = true
  try {
    servers.value = await listOpenCodeMcpServers()
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  } finally {
    loading.value = false
  }
}

function openCreate(type: 'local' | 'remote') {
  editingId.value = ''
  form.id = ''
  form.type = type
  form.enabled = true
  form.command = ''
  form.url = ''
  form.environmentJson = '{}'
  form.headersJson = '{}'
  showModal.value = true
}

function openEdit(server: OpenCodeMcpServer) {
  editingId.value = server.id
  form.id = server.id
  form.type = server.type
  form.enabled = server.enabled !== false
  form.command = stringifyCommandInput(server.command)
  form.url = server.url || ''
  form.environmentJson = formatJsonInput(server.environment || {})
  form.headersJson = formatJsonInput(server.headers || {})
  showModal.value = true
}

async function saveServer() {
  if (!form.id.trim()) {
    uiStore.showError(tt('Server id 不能为空', 'Server ID is required'))
    return
  }

  saving.value = true
  try {
    const request: OpenCodeMcpServer = {
      id: form.id.trim(),
      type: form.type,
      enabled: form.enabled,
      command: form.type === 'local' ? splitCommandInput(form.command) : undefined,
      url: form.type === 'remote' ? form.url.trim() : undefined,
      environment: parseJsonInput<Record<string, string>>(form.environmentJson, {}),
      headers: parseJsonInput<Record<string, string>>(form.headersJson, {}),
    }

    if (editingId.value) {
      await updateOpenCodeMcpServer(request.id, request)
    } else {
      await addOpenCodeMcpServer(request)
    }

    uiStore.showSuccess(editingId.value ? tt('MCP 服务器已更新', 'MCP server updated') : tt('MCP 服务器已创建', 'MCP server created'))
    showModal.value = false
    await loadServers()
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  } finally {
    saving.value = false
  }
}

async function removeServer(id: string) {
  try {
    await deleteOpenCodeMcpServer(id)
    uiStore.showSuccess(tt('MCP 服务器已删除', 'MCP server deleted'))
    await loadServers()
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function copyCli(command: string) {
  try {
    await copyText(command)
    uiStore.showSuccess(`${tt('已复制', 'Copied')}: ${command}`)
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

onMounted(() => {
  void loadServers()
})
</script>
