<template>
  <div class="glass-effect rounded-3xl p-6 border border-white/20">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-xl bg-gradient-to-br from-accent-success/20 to-accent-primary/20 text-accent-success">
          <SIcon
            name="RefreshCw"
            size="w-5 h-5"
          />
        </div>
        <div>
          <h2 class="text-lg font-bold text-text-primary">
            {{ $t('mcp.sync.title') }}
          </h2>
          <p class="text-xs text-text-muted">
            {{ $t('mcp.sync.subtitle') }}
          </p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          class="text-xs px-3 py-1.5 rounded-lg bg-bg-surface hover:bg-accent-success/10 text-text-secondary hover:text-accent-success transition-colors flex items-center gap-1.5"
          :disabled="loading"
          @click="loadSourceServers"
        >
          <SIcon
            name="RefreshCw"
            size="w-3.5 h-3.5"
            :class="{ 'animate-spin': loading }"
          />
          {{ $t('common.refresh') }}
        </button>
        <button
          class="px-4 py-2 rounded-xl font-bold text-sm text-white flex items-center gap-2 transition-transform hover:scale-105 bg-accent-success shadow-lg shadow-accent-success/20"
          :disabled="syncing || sourceServers.length === 0"
          @click="handleSyncAll"
        >
          <SIcon
            v-if="syncing"
            name="Loader2"
            size="w-4 h-4"
            class="animate-spin"
          />
          <SIcon
            v-else
            name="Zap"
            size="w-4 h-4"
          />
          {{ $t('mcp.sync.syncAll') }}
        </button>
      </div>
    </div>

    <!-- Platform Selection -->
    <div class="mb-6">
      <label class="block text-xs font-bold text-text-secondary uppercase tracking-wider mb-3">
        {{ $t('mcp.sync.targetPlatforms') }}
      </label>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="platform in platforms"
          :key="platform.id"
          class="px-3 py-2 rounded-xl text-xs font-medium flex items-center gap-2 transition-colors border"
          :class="selectedPlatforms.includes(platform.id)
            ? 'bg-accent-success/20 text-accent-success border-accent-success/30'
            : 'bg-bg-surface text-text-muted border-transparent hover:border-border-default'"
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
        <label class="text-xs font-bold text-text-secondary uppercase tracking-wider">
          {{ $t('mcp.sync.sourceServers') }} (Claude)
        </label>
        <span class="text-xs text-text-muted">
          {{ sourceServers.length }} {{ $t('mcp.sync.servers') }}
        </span>
      </div>

      <!-- Loading -->
      <div
        v-if="loading"
        class="flex justify-center py-8"
      >
        <div class="w-8 h-8 rounded-full border-3 border-accent-success/30 border-t-accent-success animate-spin" />
      </div>

      <!-- Empty State -->
      <div
        v-else-if="sourceServers.length === 0"
        class="text-center py-8 bg-bg-surface/50 rounded-2xl border border-dashed border-border-default"
      >
        <SIcon
          name="Server"
          size="w-10 h-10"
          class="mx-auto mb-2 text-text-muted opacity-50"
        />
        <p class="text-sm text-text-muted">
          {{ $t('mcp.sync.noServers') }}
        </p>
        <p class="text-xs text-text-muted mt-1">
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
          class="group p-4 rounded-2xl bg-bg-surface/50 border border-border-default/50 hover:border-accent-success/30 transition-colors"
        >
          <div class="flex items-center justify-between">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <h4 class="font-bold text-sm text-text-primary truncate">
                  {{ server.name }}
                </h4>
                <span class="px-2 py-0.5 rounded-full text-[10px] font-medium bg-accent-primary/10 text-accent-primary">
                  Claude
                </span>
              </div>
              <div class="flex items-center gap-1.5 text-xs font-mono text-text-muted bg-bg-surface rounded-lg px-2 py-1 overflow-hidden">
                <SIcon
                  name="Terminal"
                  size="w-3 h-3"
                  class="flex-shrink-0"
                />
                <span class="truncate">{{ server.command }} {{ server.args.join(' ') }}</span>
              </div>
            </div>
            <button
              class="ml-4 px-3 py-2 rounded-xl text-xs font-medium bg-accent-success/10 text-accent-success hover:bg-accent-success/20 transition-colors flex items-center gap-1.5"
              :disabled="syncing"
              @click="handleSyncServer(server.name)"
            >
              <SIcon
                name="RefreshCw"
                size="w-3.5 h-3.5"
                :class="{ 'animate-spin': syncingServer === server.name }"
              />
              {{ $t('mcp.sync.sync') }}
            </button>
          </div>

          <!-- Sync Results (if any) -->
          <div
            v-if="syncResults[server.name]"
            class="mt-3 pt-3 border-t border-border-default/30"
          >
            <div class="flex flex-wrap gap-2">
              <span
                v-for="result in syncResults[server.name]"
                :key="result.platform"
                class="inline-flex items-center gap-1 px-2 py-1 rounded-lg text-[10px] font-medium"
                :class="result.success
                  ? 'bg-accent-success/10 text-accent-success'
                  : 'bg-accent-danger/10 text-accent-danger'"
              >
                <SIcon
                  :name="result.success ? 'Check' : 'X'"
                  size="w-3 h-3"
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
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  listSourceMcpServers,
  syncMcpServer,
  syncAllMcpServers,
  type McpServerInfo,
  type SyncResult
} from '@/api'
import { logger } from '@/utils/logger'

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
    logger.error('Failed to load source MCP servers:', err)
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
    const response = await syncMcpServer(serverName, selectedPlatforms.value) as { results: SyncResult[] }
    syncResults.value[serverName] = response.results
    emit('synced')
  } catch (err) {
    logger.error('Failed to sync server:', err)
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
    const response = await syncAllMcpServers(selectedPlatforms.value) as {
      servers: Record<string, SyncResult[]>
    }

    // Update results for each server
    for (const [serverName, results] of Object.entries(response.servers)) {
      syncResults.value[serverName] = results
    }

    emit('synced')
    alert(t('mcp.sync.syncAllSuccess'))
  } catch (err) {
    logger.error('Failed to sync all servers:', err)
    alert(`${t('mcp.sync.syncFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`)
  } finally {
    syncing.value = false
  }
}

onMounted(() => {
  loadSourceServers()
})
</script>
