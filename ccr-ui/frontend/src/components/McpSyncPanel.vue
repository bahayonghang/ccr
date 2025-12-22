<template>
  <div class="glass-effect rounded-3xl p-6 border border-white/20">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-xl bg-gradient-to-br from-guofeng-emerald/20 to-guofeng-cyan/20 text-guofeng-emerald">
          <RefreshCw class="w-5 h-5" />
        </div>
        <div>
          <h2 class="text-lg font-bold text-guofeng-text-primary">
            {{ $t('mcp.sync.title') }}
          </h2>
          <p class="text-xs text-guofeng-text-muted">
            {{ $t('mcp.sync.subtitle') }}
          </p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="text-xs px-3 py-1.5 rounded-lg bg-guofeng-bg-tertiary hover:bg-guofeng-emerald/10 text-guofeng-text-secondary hover:text-guofeng-emerald transition-all flex items-center gap-1.5"
          :disabled="loading"
          @click="loadSourceServers"
        >
          <RefreshCw
            class="w-3.5 h-3.5"
            :class="{ 'animate-spin': loading }"
          />
          {{ $t('common.refresh') }}
        </button>
        <button
          class="px-4 py-2 rounded-xl font-bold text-sm text-white flex items-center gap-2 transition-all hover:scale-105 bg-guofeng-emerald shadow-lg shadow-guofeng-emerald/20"
          :disabled="syncing || sourceServers.length === 0"
          @click="handleSyncAll"
        >
          <Loader2
            v-if="syncing"
            class="w-4 h-4 animate-spin"
          />
          <Zap
            v-else
            class="w-4 h-4"
          />
          {{ $t('mcp.sync.syncAll') }}
        </button>
      </div>
    </div>

    <!-- Platform Selection -->
    <div class="mb-6">
      <label class="block text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider mb-3">
        {{ $t('mcp.sync.targetPlatforms') }}
      </label>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="platform in platforms"
          :key="platform.id"
          class="px-3 py-2 rounded-xl text-xs font-medium flex items-center gap-2 transition-all border"
          :class="selectedPlatforms.includes(platform.id)
            ? 'bg-guofeng-emerald/20 text-guofeng-emerald border-guofeng-emerald/30'
            : 'bg-guofeng-bg-tertiary text-guofeng-text-muted border-transparent hover:border-guofeng-border'"
          @click="togglePlatform(platform.id)"
        >
          <span>{{ platform.icon }}</span>
          <span>{{ platform.name }}</span>
        </button>
      </div>
    </div>

    <!-- Source Servers List -->
    <div>
      <div class="flex items-center justify-between mb-3">
        <label class="text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider">
          {{ $t('mcp.sync.sourceServers') }} (Claude)
        </label>
        <span class="text-xs text-guofeng-text-muted">
          {{ sourceServers.length }} {{ $t('mcp.sync.servers') }}
        </span>
      </div>

      <!-- Loading -->
      <div
        v-if="loading"
        class="flex justify-center py-8"
      >
        <div class="w-8 h-8 rounded-full border-3 border-guofeng-emerald/30 border-t-guofeng-emerald animate-spin" />
      </div>

      <!-- Empty State -->
      <div
        v-else-if="sourceServers.length === 0"
        class="text-center py-8 bg-guofeng-bg-tertiary/50 rounded-2xl border border-dashed border-guofeng-border"
      >
        <Server class="w-10 h-10 mx-auto mb-2 text-guofeng-text-muted opacity-50" />
        <p class="text-sm text-guofeng-text-muted">
          {{ $t('mcp.sync.noServers') }}
        </p>
        <p class="text-xs text-guofeng-text-muted mt-1">
          {{ $t('mcp.sync.noServersHint') }}
        </p>
      </div>

      <!-- Server Cards -->
      <div
        v-else
        class="space-y-3"
      >
        <div
          v-for="server in sourceServers"
          :key="server.name"
          class="group p-4 rounded-2xl bg-guofeng-bg-tertiary/50 border border-guofeng-border/50 hover:border-guofeng-emerald/30 transition-all"
        >
          <div class="flex items-center justify-between">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <h4 class="font-bold text-sm text-guofeng-text-primary truncate">
                  {{ server.name }}
                </h4>
                <span class="px-2 py-0.5 rounded-full text-[10px] font-medium bg-guofeng-cyan/10 text-guofeng-cyan">
                  Claude
                </span>
              </div>
              <div class="flex items-center gap-1.5 text-xs font-mono text-guofeng-text-muted bg-guofeng-bg-tertiary rounded-lg px-2 py-1 overflow-hidden">
                <Terminal class="w-3 h-3 flex-shrink-0" />
                <span class="truncate">{{ server.command }} {{ server.args.join(' ') }}</span>
              </div>
            </div>
            <button
              class="ml-4 px-3 py-2 rounded-xl text-xs font-medium bg-guofeng-emerald/10 text-guofeng-emerald hover:bg-guofeng-emerald/20 transition-all flex items-center gap-1.5"
              :disabled="syncing"
              @click="handleSyncServer(server.name)"
            >
              <RefreshCw
                class="w-3.5 h-3.5"
                :class="{ 'animate-spin': syncingServer === server.name }"
              />
              {{ $t('mcp.sync.sync') }}
            </button>
          </div>

          <!-- Sync Results (if any) -->
          <div
            v-if="syncResults[server.name]"
            class="mt-3 pt-3 border-t border-guofeng-border/30"
          >
            <div class="flex flex-wrap gap-2">
              <span
                v-for="result in syncResults[server.name]"
                :key="result.platform"
                class="inline-flex items-center gap-1 px-2 py-1 rounded-lg text-[10px] font-medium"
                :class="result.success
                  ? 'bg-guofeng-emerald/10 text-guofeng-emerald'
                  : 'bg-guofeng-red/10 text-guofeng-red'"
              >
                <component
                  :is="result.success ? Check : X"
                  class="w-3 h-3"
                />
                {{ result.platform }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  RefreshCw,
  Zap,
  Terminal,
  Server,
  Check,
  X,
  Loader2
} from 'lucide-vue-next'
import {
  listSourceMcpServers,
  syncMcpServer,
  syncAllMcpServers,
  type McpServerInfo,
  type SyncResult
} from '@/api/client'

const { t } = useI18n({ useScope: 'global' })

const emit = defineEmits<{
  (e: 'synced'): void
}>()

// State
const loading = ref(false)
const syncing = ref(false)
const syncingServer = ref<string | null>(null)
const sourceServers = ref<McpServerInfo[]>([])
const selectedPlatforms = ref<string[]>(['codex', 'gemini', 'qwen', 'iflow'])
const syncResults = ref<Record<string, SyncResult[]>>({})

// Available platforms (excluding Claude as it's the source)
const platforms = [
  { id: 'codex', name: 'Codex', icon: '💻' },
  { id: 'gemini', name: 'Gemini', icon: '✨' },
  { id: 'qwen', name: 'Qwen', icon: '🌟' },
  { id: 'iflow', name: 'iFlow', icon: '🌊' }
]

// Toggle platform selection
const togglePlatform = (platformId: string) => {
  const index = selectedPlatforms.value.indexOf(platformId)
  if (index === -1) {
    selectedPlatforms.value.push(platformId)
  } else {
    selectedPlatforms.value.splice(index, 1)
  }
}

// Load source servers
const loadSourceServers = async () => {
  try {
    loading.value = true
    sourceServers.value = await listSourceMcpServers()
  } catch (err) {
    console.error('Failed to load source MCP servers:', err)
  } finally {
    loading.value = false
  }
}

// Sync single server
const handleSyncServer = async (serverName: string) => {
  if (selectedPlatforms.value.length === 0) {
    alert(t('mcp.sync.selectPlatformFirst'))
    return
  }

  try {
    syncingServer.value = serverName
    const response = await syncMcpServer(serverName, selectedPlatforms.value)
    syncResults.value[serverName] = response.results
    emit('synced')
  } catch (err) {
    console.error('Failed to sync server:', err)
    alert(`${t('mcp.sync.syncFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`)
  } finally {
    syncingServer.value = null
  }
}

// Sync all servers
const handleSyncAll = async () => {
  if (selectedPlatforms.value.length === 0) {
    alert(t('mcp.sync.selectPlatformFirst'))
    return
  }

  try {
    syncing.value = true
    const response = await syncAllMcpServers(selectedPlatforms.value)
    
    // Update results for each server
    for (const [serverName, results] of Object.entries(response.servers)) {
      syncResults.value[serverName] = results
    }
    
    emit('synced')
    alert(t('mcp.sync.syncAllSuccess'))
  } catch (err) {
    console.error('Failed to sync all servers:', err)
    alert(`${t('mcp.sync.syncFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`)
  } finally {
    syncing.value = false
  }
}

onMounted(() => {
  loadSourceServers()
})
</script>
