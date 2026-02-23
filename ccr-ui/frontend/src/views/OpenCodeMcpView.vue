<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <AnimatedBackground variant="minimal" />

    <div class="max-w-5xl mx-auto space-y-5">
      <!-- 页面标题 -->
      <div class="flex items-center justify-between animate-slide-up">
        <div class="flex items-center gap-3">
          <RouterLink
            to="/opencode"
            class="p-2 rounded-lg text-text-muted hover:text-text-primary transition-colors"
          >
            <ChevronLeft class="w-5 h-5" />
          </RouterLink>
          <div>
            <h1 class="text-2xl font-bold text-text-primary">
              MCP 服务器
            </h1>
            <p class="text-text-muted text-sm">
              管理 OpenCode 原生格式 MCP 服务器（local / remote）
            </p>
          </div>
        </div>
        <button
          class="flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-all hover:scale-105"
          style="background: var(--accent-primary); color: white;"
          @click="showAddDialog = true"
        >
          <Plus class="w-4 h-4" />
          添加服务器
        </button>
      </div>

      <!-- 加载状态 -->
      <div
        v-if="loading"
        class="flex items-center justify-center py-16"
      >
        <div class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
      </div>

      <!-- 错误状态 -->
      <Card
        v-else-if="error"
        variant="elevated"
        class="p-6 text-center"
      >
        <p class="text-red-400 mb-3">
          {{ error }}
        </p>
        <button
          class="text-sm text-accent-primary hover:underline"
          @click="loadServers"
        >
          重新加载
        </button>
      </Card>

      <!-- 空状态 -->
      <Card
        v-else-if="servers.length === 0"
        variant="glass"
        class="p-10 text-center"
      >
        <Server class="w-12 h-12 text-text-muted mx-auto mb-4" />
        <h3 class="text-lg font-bold text-text-primary mb-2">
          暂无 MCP 服务器
        </h3>
        <p class="text-text-muted text-sm mb-4">
          添加 local（本地命令）或 remote（HTTP/SSE）MCP 服务器
        </p>
        <button
          class="px-4 py-2 rounded-lg font-medium text-sm transition-all hover:scale-105"
          style="background: var(--accent-primary); color: white;"
          @click="showAddDialog = true"
        >
          添加第一个服务器
        </button>
      </Card>

      <!-- 服务器列表 -->
      <div
        v-else
        class="space-y-3"
      >
        <Card
          v-for="server in servers"
          :key="server.id"
          variant="elevated"
          class="p-4 animate-slide-up"
        >
          <div class="flex items-start justify-between gap-4">
            <div class="flex items-start gap-3 min-w-0">
              <!-- 类型图标 -->
              <div
                class="w-10 h-10 rounded-lg flex items-center justify-center shrink-0"
                :class="server.type === 'local' ? 'bg-green-500/10' : 'bg-blue-500/10'"
              >
                <component
                  :is="server.type === 'local' ? Terminal : Globe"
                  class="w-5 h-5"
                  :class="server.type === 'local' ? 'text-green-500' : 'text-blue-500'"
                />
              </div>

              <!-- 服务器信息 -->
              <div class="min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <h3 class="font-bold text-text-primary truncate">
                    {{ server.id }}
                  </h3>
                  <span
                    class="px-2 py-0.5 rounded text-xs font-bold uppercase shrink-0"
                    :class="server.type === 'local'
                      ? 'bg-green-500/10 text-green-400'
                      : 'bg-blue-500/10 text-blue-400'"
                  >
                    {{ server.type }}
                  </span>
                </div>

                <!-- local 类型：显示命令 -->
                <div
                  v-if="server.type === 'local' && server.command?.length"
                  class="text-xs text-text-muted font-mono truncate"
                >
                  {{ server.command.join(' ') }}
                </div>

                <!-- remote 类型：显示 URL -->
                <div
                  v-else-if="server.type === 'remote' && server.url"
                  class="text-xs text-text-muted font-mono truncate"
                >
                  {{ server.url }}
                </div>

                <!-- 环境变量数量 -->
                <div
                  v-if="server.environment && Object.keys(server.environment).length > 0"
                  class="text-xs text-text-muted mt-1"
                >
                  {{ Object.keys(server.environment).length }} 个环境变量
                </div>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="flex items-center gap-2 shrink-0">
              <button
                class="p-2 rounded-lg text-text-muted hover:text-blue-400 hover:bg-blue-500/10 transition-colors"
                title="编辑"
                @click="editServer(server)"
              >
                <Pencil class="w-4 h-4" />
              </button>
              <button
                class="p-2 rounded-lg text-text-muted hover:text-red-400 hover:bg-red-500/10 transition-colors"
                title="删除"
                @click="confirmDelete(server)"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <!-- 添加/编辑弹窗 -->
    <div
      v-if="showAddDialog || editingServer"
      class="fixed inset-0 flex items-center justify-center z-50 p-4"
      style="background: rgba(0,0,0,0.5); backdrop-filter: blur(4px);"
      @click.self="closeDialog"
    >
      <Card
        variant="glass"
        class="w-full max-w-lg p-6 space-y-4 max-h-[90vh] overflow-y-auto"
      >
        <div class="flex items-center justify-between">
          <h2 class="text-lg font-bold text-text-primary">
            {{ editingServer ? '编辑 MCP 服务器' : '添加 MCP 服务器' }}
          </h2>
          <button
            class="p-1 rounded text-text-muted hover:text-text-primary"
            @click="closeDialog"
          >
            <X class="w-5 h-5" />
          </button>
        </div>

        <!-- 服务器 ID -->
        <div>
          <label class="block text-xs font-bold text-text-muted uppercase tracking-wider mb-1">服务器 ID *</label>
          <input
            v-model="form.id"
            :disabled="!!editingServer"
            type="text"
            placeholder="例：my-mcp-server"
            class="w-full px-3 py-2 rounded-lg text-sm bg-bg-elevated border border-border-default text-text-primary placeholder:text-text-muted focus:outline-none focus:border-blue-500 disabled:opacity-50"
          />
        </div>

        <!-- 类型选择 -->
        <div>
          <label class="block text-xs font-bold text-text-muted uppercase tracking-wider mb-1">服务器类型 *</label>
          <div class="flex gap-3">
            <button
              class="flex-1 py-2 rounded-lg text-sm font-medium border transition-all"
              :class="form.type === 'local'
                ? 'bg-green-500/20 border-green-500 text-green-400'
                : 'bg-bg-elevated border-border-default text-text-muted hover:border-green-500/50'"
              @click="form.type = 'local'"
            >
              <Terminal class="w-4 h-4 mx-auto mb-1" />
              local（本地命令）
            </button>
            <button
              class="flex-1 py-2 rounded-lg text-sm font-medium border transition-all"
              :class="form.type === 'remote'
                ? 'bg-blue-500/20 border-blue-500 text-blue-400'
                : 'bg-bg-elevated border-border-default text-text-muted hover:border-blue-500/50'"
              @click="form.type = 'remote'"
            >
              <Globe class="w-4 h-4 mx-auto mb-1" />
              remote（HTTP/SSE）
            </button>
          </div>
        </div>

        <!-- local: 命令输入 -->
        <div v-if="form.type === 'local'">
          <label class="block text-xs font-bold text-text-muted uppercase tracking-wider mb-1">命令（空格分隔）*</label>
          <input
            v-model="form.commandStr"
            type="text"
            placeholder="例：npx -y @modelcontextprotocol/server-everything"
            class="w-full px-3 py-2 rounded-lg text-sm bg-bg-elevated border border-border-default text-text-primary placeholder:text-text-muted focus:outline-none focus:border-blue-500"
          />
          <p class="text-xs text-text-muted mt-1">
            命令将被拆分为数组：["npx", "-y", "@modelcontextprotocol/server-everything"]
          </p>
        </div>

        <!-- remote: URL 输入 -->
        <div v-if="form.type === 'remote'">
          <label class="block text-xs font-bold text-text-muted uppercase tracking-wider mb-1">URL *</label>
          <input
            v-model="form.url"
            type="text"
            placeholder="例：https://mcp.example.com/sse"
            class="w-full px-3 py-2 rounded-lg text-sm bg-bg-elevated border border-border-default text-text-primary placeholder:text-text-muted focus:outline-none focus:border-blue-500"
          />
        </div>

        <!-- 操作按钮 -->
        <div class="flex justify-end gap-3 pt-2">
          <button
            class="px-4 py-2 rounded-lg text-sm text-text-muted hover:text-text-primary"
            @click="closeDialog"
          >
            取消
          </button>
          <button
            :disabled="!isFormValid || saving"
            class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all hover:scale-105 disabled:opacity-50 disabled:hover:scale-100"
            style="background: var(--accent-primary); color: white;"
            @click="saveServer"
          >
            <Loader2
              v-if="saving"
              class="w-4 h-4 animate-spin"
            />
            {{ editingServer ? '更新' : '添加' }}
          </button>
        </div>
      </Card>
    </div>

    <!-- 删除确认弹窗 -->
    <div
      v-if="deletingServer"
      class="fixed inset-0 flex items-center justify-center z-50 p-4"
      style="background: rgba(0,0,0,0.5); backdrop-filter: blur(4px);"
      @click.self="deletingServer = null"
    >
      <Card
        variant="glass"
        class="w-full max-w-sm p-6 space-y-4"
      >
        <h2 class="text-lg font-bold text-text-primary">
          确认删除
        </h2>
        <p class="text-text-secondary text-sm">
          确定要删除 MCP 服务器 <strong>{{ deletingServer.id }}</strong>（{{ deletingServer.type }}）吗？
        </p>
        <div class="flex justify-end gap-3">
          <button
            class="px-4 py-2 rounded-lg text-sm text-text-muted hover:text-text-primary"
            @click="deletingServer = null"
          >
            取消
          </button>
          <button
            :disabled="saving"
            class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-red-500 text-white hover:bg-red-600 disabled:opacity-50"
            @click="doDelete"
          >
            <Loader2
              v-if="saving"
              class="w-4 h-4 animate-spin"
            />
            删除
          </button>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { ChevronLeft, Plus, Server, Terminal, Globe, Pencil, Trash2, X, Loader2 } from 'lucide-vue-next'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import {
  listOpenCodeMcpServers,
  addOpenCodeMcpServer,
  updateOpenCodeMcpServer,
  deleteOpenCodeMcpServer,
} from '@/api/modules/opencode'
import type { OpenCodeMcpServer } from '@/types/opencode'

const servers = ref<OpenCodeMcpServer[]>([])
const loading = ref(true)
const error = ref('')
const saving = ref(false)
const showAddDialog = ref(false)
const editingServer = ref<OpenCodeMcpServer | null>(null)
const deletingServer = ref<OpenCodeMcpServer | null>(null)

const form = reactive({
  id: '',
  type: 'local' as 'local' | 'remote',
  commandStr: '',
  url: '',
})

const isFormValid = computed(() => {
  if (!form.id) return false
  if (form.type === 'local') return !!form.commandStr.trim()
  if (form.type === 'remote') return !!form.url.trim()
  return false
})

const loadServers = async () => {
  loading.value = true
  error.value = ''
  try {
    servers.value = await listOpenCodeMcpServers()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '加载失败'
  } finally {
    loading.value = false
  }
}

const editServer = (server: OpenCodeMcpServer) => {
  editingServer.value = server
  form.id = server.id
  form.type = server.type
  form.commandStr = server.command?.join(' ') || ''
  form.url = server.url || ''
}

const confirmDelete = (server: OpenCodeMcpServer) => {
  deletingServer.value = server
}

const closeDialog = () => {
  showAddDialog.value = false
  editingServer.value = null
  form.id = ''
  form.type = 'local'
  form.commandStr = ''
  form.url = ''
}

const saveServer = async () => {
  if (!isFormValid.value) return
  saving.value = true
  try {
    const command = form.type === 'local'
      ? form.commandStr.trim().split(/\s+/)
      : undefined
    const url = form.type === 'remote' ? form.url.trim() : undefined

    const request = {
      id: form.id,
      type: form.type,
      command,
      url,
    }

    if (editingServer.value) {
      await updateOpenCodeMcpServer(form.id, request)
    } else {
      await addOpenCodeMcpServer(request)
    }
    closeDialog()
    await loadServers()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '保存失败'
  } finally {
    saving.value = false
  }
}

const doDelete = async () => {
  if (!deletingServer.value) return
  saving.value = true
  try {
    await deleteOpenCodeMcpServer(deletingServer.value.id)
    deletingServer.value = null
    await loadServers()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '删除失败'
  } finally {
    saving.value = false
  }
}

onMounted(loadServers)
</script>
