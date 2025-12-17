<template>
  <div class="glass-effect rounded-3xl p-6 border border-white/20 mb-6">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-xl bg-gradient-to-br from-guofeng-indigo/20 to-guofeng-purple/20 text-guofeng-indigo">
          <Sparkles class="w-5 h-5" />
        </div>
        <div>
          <h2 class="text-lg font-bold text-guofeng-text-primary">
            {{ $t('mcp.presets.title') }}
          </h2>
          <p class="text-xs text-guofeng-text-muted">
            {{ $t('mcp.presets.subtitle') }}
          </p>
        </div>
      </div>
      <button
        class="text-xs px-3 py-1.5 rounded-lg bg-guofeng-bg-tertiary hover:bg-guofeng-indigo/10 text-guofeng-text-secondary hover:text-guofeng-indigo transition-all flex items-center gap-1.5"
        @click="showPresetsPanel = !showPresetsPanel"
      >
        <component
          :is="showPresetsPanel ? ChevronUp : ChevronDown"
          class="w-4 h-4"
        />
        {{ showPresetsPanel ? $t('mcp.presets.collapse') : $t('mcp.presets.expand') }}
      </button>
    </div>

    <!-- Presets Grid -->
    <div
      v-show="showPresetsPanel"
      class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4"
    >
      <!-- Loading State -->
      <div
        v-if="loading"
        class="col-span-full flex justify-center py-8"
      >
        <div class="w-8 h-8 rounded-full border-3 border-guofeng-indigo/30 border-t-guofeng-indigo animate-spin" />
      </div>

      <!-- Preset Cards -->
      <div
        v-for="preset in presets"
        v-else
        :key="preset.id"
        class="group relative rounded-2xl p-4 border border-white/10 bg-gradient-to-br from-white/5 to-white/10 hover:border-guofeng-indigo/30 hover:shadow-lg transition-all duration-300 cursor-pointer"
        @click="handlePresetClick(preset)"
      >
        <!-- Tags Badge -->
        <div class="flex flex-wrap gap-1.5 mb-3">
          <span
            v-for="tag in preset.tags.slice(0, 2)"
            :key="tag"
            class="px-2 py-0.5 rounded-full text-[10px] font-medium bg-guofeng-indigo/10 text-guofeng-indigo"
          >
            {{ tag }}
          </span>
          <span
            v-if="preset.requires_api_key"
            class="px-2 py-0.5 rounded-full text-[10px] font-medium bg-guofeng-amber/10 text-guofeng-amber"
          >
            🔑 API Key
          </span>
        </div>

        <!-- Name -->
        <h3 class="font-bold text-sm text-guofeng-text-primary mb-1 truncate group-hover:text-guofeng-indigo transition-colors">
          {{ preset.name }}
        </h3>

        <!-- Description -->
        <p class="text-xs text-guofeng-text-muted line-clamp-2 mb-3">
          {{ preset.description }}
        </p>

        <!-- Command Preview -->
        <div class="flex items-center gap-1.5 text-[10px] font-mono text-guofeng-text-muted bg-guofeng-bg-tertiary/50 rounded-lg px-2 py-1.5 overflow-hidden">
          <Terminal class="w-3 h-3 flex-shrink-0" />
          <span class="truncate">{{ preset.command }} {{ preset.args.join(' ') }}</span>
        </div>

        <!-- Hover Install Button -->
        <div class="absolute inset-0 bg-gradient-to-t from-guofeng-ink/80 to-transparent opacity-0 group-hover:opacity-100 transition-opacity rounded-2xl flex items-end justify-center pb-4">
          <button
            class="px-4 py-2 rounded-lg bg-guofeng-indigo text-white text-xs font-bold flex items-center gap-2 hover:bg-guofeng-indigo/90 transition-colors shadow-lg"
            @click.stop="handleInstall(preset)"
          >
            <Download class="w-3.5 h-3.5" />
            {{ $t('mcp.presets.install') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Install Modal -->
    <div
      v-if="showInstallModal && selectedPreset"
      class="fixed inset-0 bg-guofeng-ink/40 backdrop-blur-sm flex items-center justify-center p-4 z-50"
      @click="closeInstallModal"
    >
      <div
        class="glass-effect rounded-3xl p-8 max-w-lg w-full shadow-2xl border border-white/30"
        @click.stop
      >
        <!-- Header -->
        <div class="flex items-center gap-4 mb-6">
          <div class="w-12 h-12 rounded-2xl bg-gradient-to-br from-guofeng-indigo to-guofeng-purple flex items-center justify-center text-white">
            <Sparkles class="w-6 h-6" />
          </div>
          <div>
            <h3 class="text-xl font-bold text-guofeng-text-primary">
              {{ selectedPreset.name }}
            </h3>
            <p class="text-sm text-guofeng-text-muted">
              {{ selectedPreset.description }}
            </p>
          </div>
        </div>

        <!-- Platform Selection -->
        <div class="mb-6">
          <label class="block text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider mb-3">
            {{ $t('mcp.presets.selectPlatforms') }}
          </label>
          <div class="grid grid-cols-2 sm:grid-cols-5 gap-2">
            <button
              v-for="platform in platforms"
              :key="platform.id"
              class="px-3 py-2 rounded-xl text-xs font-medium flex items-center justify-center gap-2 transition-all border"
              :class="selectedPlatforms.includes(platform.id)
                ? 'bg-guofeng-indigo/20 text-guofeng-indigo border-guofeng-indigo/30'
                : 'bg-guofeng-bg-tertiary text-guofeng-text-muted border-transparent hover:border-guofeng-border'"
              @click="togglePlatform(platform.id)"
            >
              <span>{{ platform.icon }}</span>
              <span>{{ platform.name }}</span>
            </button>
          </div>
        </div>

        <!-- API Key Input (if required) -->
        <div
          v-if="selectedPreset.requires_api_key && selectedPreset.api_key_env"
          class="mb-6"
        >
          <label class="block text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider mb-2">
            {{ selectedPreset.api_key_env }}
            <span class="text-guofeng-red">*</span>
          </label>
          <input
            v-model="apiKeyValue"
            type="password"
            class="w-full px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-indigo focus:ring-4 focus:ring-guofeng-indigo/10 outline-none transition-all font-mono text-sm"
            :placeholder="`${$t('mcp.presets.enterApiKey')} ${selectedPreset.api_key_env}`"
          >
          <p class="text-xs text-guofeng-text-muted mt-2">
            {{ $t('mcp.presets.apiKeyHint') }}
          </p>
        </div>

        <!-- Command Preview -->
        <div class="mb-6 p-4 rounded-xl bg-guofeng-bg-tertiary/50 border border-guofeng-border/50">
          <div class="text-xs font-medium text-guofeng-text-muted mb-2">
            {{ $t('mcp.presets.commandPreview') }}
          </div>
          <code class="text-sm font-mono text-guofeng-indigo break-all">
            {{ selectedPreset.command }} {{ selectedPreset.args.join(' ') }}
          </code>
        </div>

        <!-- Links -->
        <div
          v-if="selectedPreset.homepage || selectedPreset.docs"
          class="flex gap-3 mb-6"
        >
          <a
            v-if="selectedPreset.homepage"
            :href="selectedPreset.homepage"
            target="_blank"
            class="text-xs text-guofeng-indigo hover:underline flex items-center gap-1"
          >
            <ExternalLink class="w-3 h-3" />
            {{ $t('mcp.presets.homepage') }}
          </a>
          <a
            v-if="selectedPreset.docs"
            :href="selectedPreset.docs"
            target="_blank"
            class="text-xs text-guofeng-indigo hover:underline flex items-center gap-1"
          >
            <Book class="w-3 h-3" />
            {{ $t('mcp.presets.documentation') }}
          </a>
        </div>

        <!-- Actions -->
        <div class="flex gap-4">
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-white text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary border border-guofeng-border"
            @click="closeInstallModal"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-guofeng-indigo text-white shadow-lg shadow-guofeng-indigo/20 hover:shadow-xl hover:shadow-guofeng-indigo/30 hover:-translate-y-0.5 flex items-center justify-center gap-2"
            :disabled="installing || selectedPlatforms.length === 0"
            @click="confirmInstall"
          >
            <Loader2
              v-if="installing"
              class="w-4 h-4 animate-spin"
            />
            <Download
              v-else
              class="w-4 h-4"
            />
            {{ installing ? $t('mcp.presets.installing') : $t('mcp.presets.confirmInstall') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Sparkles,
  ChevronUp,
  ChevronDown,
  Terminal,
  Download,
  ExternalLink,
  Book,
  Loader2
} from 'lucide-vue-next'
import {
  listMcpPresets,
  installMcpPreset,
  type McpPreset
} from '@/api/client'

const { t } = useI18n({ useScope: 'global' })

const emit = defineEmits<{
  (e: 'installed'): void
}>()

// State
const showPresetsPanel = ref(true)
const loading = ref(true)
const presets = ref<McpPreset[]>([])
const showInstallModal = ref(false)
const selectedPreset = ref<McpPreset | null>(null)
const selectedPlatforms = ref<string[]>(['claude'])
const apiKeyValue = ref('')
const installing = ref(false)

// Available platforms
const platforms = [
  { id: 'claude', name: 'Claude', icon: '🤖' },
  { id: 'codex', name: 'Codex', icon: '💻' },
  { id: 'gemini', name: 'Gemini', icon: '✨' },
  { id: 'qwen', name: 'Qwen', icon: '🌟' },
  { id: 'iflow', name: 'iFlow', icon: '🌊' }
]

// Load presets
const loadPresets = async () => {
  try {
    loading.value = true
    presets.value = await listMcpPresets()
  } catch (err) {
    console.error('Failed to load MCP presets:', err)
  } finally {
    loading.value = false
  }
}

// Handle preset click
const handlePresetClick = (preset: McpPreset) => {
  selectedPreset.value = preset
  selectedPlatforms.value = ['claude']
  apiKeyValue.value = ''
  showInstallModal.value = true
}

// Handle install button
const handleInstall = (preset: McpPreset) => {
  handlePresetClick(preset)
}

// Toggle platform selection
const togglePlatform = (platformId: string) => {
  const index = selectedPlatforms.value.indexOf(platformId)
  if (index === -1) {
    selectedPlatforms.value.push(platformId)
  } else {
    selectedPlatforms.value.splice(index, 1)
  }
}

// Close modal
const closeInstallModal = () => {
  showInstallModal.value = false
  selectedPreset.value = null
  apiKeyValue.value = ''
}

// Confirm installation
const confirmInstall = async () => {
  if (!selectedPreset.value || selectedPlatforms.value.length === 0) return

  // Check API key requirement
  if (selectedPreset.value.requires_api_key && selectedPreset.value.api_key_env && !apiKeyValue.value) {
    alert(t('mcp.presets.apiKeyRequired'))
    return
  }

  installing.value = true

  try {
    const env: Record<string, string> = {}
    if (selectedPreset.value.api_key_env && apiKeyValue.value) {
      env[selectedPreset.value.api_key_env] = apiKeyValue.value
    }

    const result = await installMcpPreset(
      selectedPreset.value.id,
      selectedPlatforms.value,
      env
    )

    // Check results
    const failed = result.results.filter(r => !r.success)
    if (failed.length > 0) {
      const failedPlatforms = failed.map(f => `${f.platform}: ${f.message}`).join('\n')
      alert(`${t('mcp.presets.installPartialFailed')}\n\n${failedPlatforms}`)
    } else {
      alert(t('mcp.presets.installSuccess'))
    }

    closeInstallModal()
    emit('installed')
  } catch (err) {
    console.error('Failed to install preset:', err)
    alert(`${t('mcp.presets.installFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`)
  } finally {
    installing.value = false
  }
}

onMounted(() => {
  loadPresets()
})
</script>
