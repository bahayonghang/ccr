<template>
  <OpenCodePageShell
    title="MCP servers"
    description="管理 local / remote MCP 定义，并提供官方 CLI auth / debug / logout 动作。"
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
        添加服务器
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
            暂无 MCP 服务器
          </h2>
          <p class="mt-2 text-sm text-text-secondary">
            可添加本地命令型 server，或远程 HTTP/SSE server。
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
                <span class="rounded-full border border-cyan-300/20 bg-cyan-300/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-cyan-200">
                  {{ server.id }}
                </span>
                <span class="rounded-full bg-bg-base/45 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-text-secondary">
                  {{ server.type }}
                </span>
                <span
                  class="rounded-full px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em]"
                  :class="server.enabled === false ? 'bg-amber-300/10 text-amber-200' : 'bg-emerald-300/10 text-emerald-200'"
                >
                  {{ server.enabled === false ? 'disabled' : 'enabled' }}
                </span>
              </div>

              <div class="grid gap-3 md:grid-cols-3">
                <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3 md:col-span-2">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted">entrypoint</span>
                  <p class="mt-2 break-all font-mono text-sm text-text-primary">
                    {{ server.type === 'local' ? stringifyCommandInput(server.command) || 'missing command' : server.url || 'missing url' }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted">extras</span>
                  <p class="mt-2 text-sm text-text-primary">
                    env {{ Object.keys(server.environment || {}).length }} · headers {{ Object.keys(server.headers || {}).length }}
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
                  Auth
                </Button>
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="copyCli(`opencode mcp debug ${server.id}`)"
                >
                  Debug
                </Button>
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="copyCli(`opencode mcp logout ${server.id}`)"
                >
                  Logout
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
                编辑
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
                删除
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
          CLI handoff
        </h2>
        <p class="mt-2 text-sm text-text-secondary">
          OpenCode 的 MCP OAuth 与调试动作本质上还是 CLI 能力，这里直接给你可执行命令。
        </p>
        <div class="mt-4 space-y-3">
          <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3">
            <strong class="font-mono text-sm text-text-primary">opencode mcp add</strong>
            <p class="mt-2 text-sm text-text-secondary">
              交互式添加 local 或 remote server。
            </p>
          </div>
          <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3">
            <strong class="font-mono text-sm text-text-primary">opencode mcp auth &lt;name&gt;</strong>
            <p class="mt-2 text-sm text-text-secondary">
              OAuth-enabled remote server 登录。
            </p>
          </div>
          <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3">
            <strong class="font-mono text-sm text-text-primary">opencode mcp debug &lt;name&gt;</strong>
            <p class="mt-2 text-sm text-text-secondary">
              排查 OAuth / transport 连接问题。
            </p>
          </div>
        </div>
      </Card>
    </div>

    <BaseModal
      v-model="showModal"
      :title="editingId ? '编辑 MCP 服务器' : '添加 MCP 服务器'"
      description="OpenCode `mcp` 配置使用 `local / remote` 两种形态。"
      size="lg"
      content-class="max-w-2xl"
    >
      <div class="space-y-4">
        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">server id *</label>
            <input
              v-model="form.id"
              :disabled="Boolean(editingId)"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="github"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">type</label>
            <select
              v-model="form.type"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
            >
              <option value="local">
                local
              </option>
              <option value="remote">
                remote
              </option>
            </select>
          </div>
        </div>

        <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base/35 px-4 py-3 text-sm text-text-primary">
          <input
            v-model="form.enabled"
            type="checkbox"
          >
          启用该 MCP server
        </label>

        <div v-if="form.type === 'local'">
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">command</label>
          <input
            v-model="form.command"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
            placeholder="npx -y @modelcontextprotocol/server-github"
          >
        </div>

        <div v-else>
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">url</label>
          <input
            v-model="form.url"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
            placeholder="https://mcp.example.com/sse"
          >
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">environment JSON</label>
            <textarea
              v-model="form.environmentJson"
              rows="6"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
            />
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">headers JSON</label>
            <textarea
              v-model="form.headersJson"
              rows="6"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
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
            取消
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
            保存
          </Button>
        </div>
      </div>
    </BaseModal>
  </OpenCodePageShell>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import OpenCodePageShell from '@/components/opencode/OpenCodePageShell.vue'
import { useUIStore } from '@/stores/ui'
import { addOpenCodeMcpServer, deleteOpenCodeMcpServer, listOpenCodeMcpServers, updateOpenCodeMcpServer } from '@/api'
import type { OpenCodeMcpServer } from '@/types'
import { copyText, formatJsonInput, parseJsonInput, splitCommandInput, stringifyCommandInput } from '@/utils/opencode'

const uiStore = useUIStore()
const loading = ref(false)
const saving = ref(false)
const showModal = ref(false)
const editingId = ref('')
const servers = ref<OpenCodeMcpServer[]>([])

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
    servers.value = await listOpenCodeMcpServers<OpenCodeMcpServer[]>()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
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
    uiStore.showError('Server id 不能为空')
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

    uiStore.showSuccess(editingId.value ? 'MCP 服务器已更新' : 'MCP 服务器已创建')
    showModal.value = false
    await loadServers()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  } finally {
    saving.value = false
  }
}

async function removeServer(id: string) {
  try {
    await deleteOpenCodeMcpServer(id)
    uiStore.showSuccess('MCP 服务器已删除')
    await loadServers()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function copyCli(command: string) {
  try {
    await copyText(command)
    uiStore.showSuccess(`已复制: ${command}`)
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

onMounted(() => {
  void loadServers()
})
</script>
