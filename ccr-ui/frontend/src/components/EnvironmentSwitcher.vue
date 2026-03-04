<script setup lang="ts">
/**
 * 环境切换器 — 显示当前执行环境，支持切换 Local/WSL/SSH
 */
import { ref, onMounted, computed } from 'vue'
import { Monitor, ChevronDown, RefreshCw, Server, Terminal } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'

interface EnvironmentInfo {
  id: string
  name: string
  env_type: string
  is_active: boolean
  description: string
}

const environments = ref<EnvironmentInfo[]>([])
const isOpen = ref(false)
const isLoading = ref(false)
const isRefreshing = ref(false)

const currentEnv = computed(() => environments.value.find(e => e.is_active))

const envIcon = (envType: string) => {
  switch (envType) {
    case 'local': return Monitor
    case 'wsl': return Terminal
    case 'ssh': return Server
    default: return Monitor
  }
}

const envColor = (envType: string) => {
  switch (envType) {
    case 'local': return 'text-emerald-400'
    case 'wsl': return 'text-orange-400'
    case 'ssh': return 'text-sky-400'
    default: return 'text-slate-400'
  }
}

const fetchEnvironments = async () => {
  try {
    environments.value = await invoke<EnvironmentInfo[]>('list_environments')
  } catch (e) {
    console.error('[EnvironmentSwitcher] Failed to list environments:', e)
  }
}

const switchEnv = async (envId: string) => {
  if (currentEnv.value?.id === envId) {
    isOpen.value = false
    return
  }

  isLoading.value = true
  try {
    await invoke('switch_environment', { envId })
    await fetchEnvironments()
  } catch (e) {
    console.error('[EnvironmentSwitcher] Failed to switch:', e)
  } finally {
    isLoading.value = false
    isOpen.value = false
  }
}

const refreshEnvs = async () => {
  isRefreshing.value = true
  try {
    environments.value = await invoke<EnvironmentInfo[]>('refresh_environments')
  } catch (e) {
    console.error('[EnvironmentSwitcher] Failed to refresh:', e)
  } finally {
    isRefreshing.value = false
  }
}

const handleClickOutside = (e: MouseEvent) => {
  const el = (e.target as HTMLElement).closest('.env-switcher')
  if (!el) isOpen.value = false
}

onMounted(async () => {
  await fetchEnvironments()
  document.addEventListener('click', handleClickOutside)
})
</script>

<template>
  <div class="env-switcher relative">
    <!-- 触发按钮 -->
    <button
      class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors duration-200 border bg-bg-surface border-border-default text-text-secondary hover:text-text-primary hover:border-accent-primary/30 hover:bg-bg-elevated"
      @click.stop="isOpen = !isOpen"
    >
      <component
        :is="envIcon(currentEnv?.env_type || 'local')"
        class="w-3.5 h-3.5"
        :class="envColor(currentEnv?.env_type || 'local')"
      />
      <span class="max-w-[120px] truncate">
        {{ currentEnv?.name || 'Local' }}
      </span>
      <ChevronDown
        class="w-3 h-3 transition-transform duration-200"
        :class="{ 'rotate-180': isOpen }"
      />
    </button>

    <!-- 下拉面板 -->
    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0 scale-95 -translate-y-1"
      enter-to-class="opacity-100 scale-100 translate-y-0"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="opacity-100 scale-100 translate-y-0"
      leave-to-class="opacity-0 scale-95 -translate-y-1"
    >
      <div
        v-if="isOpen"
        class="absolute right-0 top-full mt-1 w-64 rounded-xl border border-border-default bg-bg-elevated/95 backdrop-blur-lg shadow-xl z-50 overflow-hidden"
      >
        <!-- 标题栏 -->
        <div class="flex items-center justify-between px-3 py-2 border-b border-border-subtle">
          <span class="text-[10px] font-bold uppercase tracking-wider text-text-muted">
            执行环境
          </span>
          <button
            class="p-1 rounded-md hover:bg-bg-surface transition-colors"
            :disabled="isRefreshing"
            title="刷新环境列表"
            @click.stop="refreshEnvs"
          >
            <RefreshCw
              class="w-3 h-3 text-text-muted"
              :class="{ 'animate-spin': isRefreshing }"
            />
          </button>
        </div>

        <!-- 环境列表 -->
        <div class="py-1 max-h-60 overflow-y-auto">
          <button
            v-for="env in environments"
            :key="env.id"
            class="w-full flex items-center gap-3 px-3 py-2 text-left text-sm transition-colors duration-150"
            :class="[
              env.is_active
                ? 'bg-accent-primary/10 text-accent-primary'
                : 'text-text-secondary hover:bg-bg-surface hover:text-text-primary'
            ]"
            :disabled="isLoading"
            @click.stop="switchEnv(env.id)"
          >
            <component
              :is="envIcon(env.env_type)"
              class="w-4 h-4 flex-shrink-0"
              :class="env.is_active ? 'text-accent-primary' : envColor(env.env_type)"
            />
            <div class="flex-1 min-w-0">
              <div class="font-medium truncate">
                {{ env.name }}
              </div>
              <div class="text-[10px] text-text-muted truncate">
                {{ env.description }}
              </div>
            </div>
            <!-- 活跃指示器 -->
            <div
              v-if="env.is_active"
              class="w-2 h-2 rounded-full bg-accent-primary shadow-[0_0_6px_rgba(var(--color-accent-primary-rgb),0.6)]"
            />
          </button>

          <div
            v-if="environments.length === 0"
            class="px-3 py-4 text-center text-xs text-text-muted"
          >
            未检测到可用环境
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>
