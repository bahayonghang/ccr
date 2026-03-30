<script setup lang="ts">
/**
 * 环境切换器 — 显示当前执行环境，支持切换 Local/WSL/SSH
 */
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { logger } from '@/utils/logger'

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
const { t } = useI18n()

const currentEnv = computed(() => environments.value.find(e => e.is_active))

const envIcon = (envType: string) => {
  switch (envType) {
    case 'local': return 'Monitor'
    case 'wsl': return 'Terminal'
    case 'ssh': return 'Server'
    default: return 'Monitor'
  }
}

const envColor = (envType: string) => {
  switch (envType) {
    case 'local': return 'text-emerald-400'
    case 'wsl': return 'text-orange-400'
    case 'ssh': return 'text-sky-400'
    default: return 'text-text-muted'
  }
}

const fetchEnvironments = async () => {
  try {
    environments.value = await invoke<EnvironmentInfo[]>('list_environments')
  } catch (e) {
    logger.error('[EnvironmentSwitcher] Failed to list environments:', e)
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
    logger.error('[EnvironmentSwitcher] Failed to switch:', e)
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
    logger.error('[EnvironmentSwitcher] Failed to refresh:', e)
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

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div class="env-switcher shrink-0 relative">
    <!-- 触发按钮 -->
    <button
      type="button"
      class="glass-surface flex items-center gap-2 rounded-lg border border-border-default/60 px-3 py-1.5 text-xs font-medium text-text-secondary transition-interactive duration-200 hover:border-accent-primary/35 hover:bg-bg-elevated/80 hover:text-text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/20"
      :aria-expanded="isOpen"
      aria-haspopup="listbox"
      @click.stop="isOpen = !isOpen"
    >
      <SIcon
        :name="envIcon(currentEnv?.env_type || 'local')"
        size="w-3.5 h-3.5"
        :class="envColor(currentEnv?.env_type || 'local')"
      />
      <span class="max-w-[120px] truncate">
        {{ currentEnv?.name || t('common.environment.local') }}
      </span>
      <SIcon
        name="ChevronDown"
        size="w-3 h-3"
        class="transition-transform duration-200"
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
        class="env-switcher__menu glass-surface absolute right-0 top-full mt-1 w-64 overflow-hidden rounded-xl border border-border-default/70 shadow-xl"
        role="listbox"
      >
        <!-- 标题栏 -->
        <div class="flex items-center justify-between border-b border-border-default/50 px-3 py-2">
          <span class="text-[10px] font-bold uppercase tracking-wider text-text-muted">
            {{ t('common.environment.title') }}
          </span>
          <button
            type="button"
            class="rounded-md p-1 text-text-muted transition-colors hover:bg-bg-surface/70 hover:text-text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/20"
            :disabled="isRefreshing"
            :title="t('common.environment.refresh')"
            @click.stop="refreshEnvs"
          >
            <SIcon
              name="RefreshCw"
              size="w-3 h-3"
              :class="{ 'animate-spin': isRefreshing }"
            />
          </button>
        </div>

        <!-- 环境列表 -->
        <div class="py-1 max-h-60 overflow-y-auto">
          <button
            v-for="env in environments"
            :key="env.id"
            type="button"
            class="w-full flex items-center gap-3 px-3 py-2 text-left text-sm transition-colors duration-150"
            :class="[
              env.is_active
                ? 'bg-accent-primary/10 text-accent-primary'
                : 'text-text-secondary hover:bg-bg-surface/70 hover:text-text-primary'
            ]"
            :disabled="isLoading"
            role="option"
            :aria-selected="env.is_active"
            @click.stop="switchEnv(env.id)"
          >
            <SIcon
              :name="envIcon(env.env_type)"
              size="w-4 h-4"
              class="flex-shrink-0"
              :class="env.is_active ? 'text-accent-primary' : envColor(env.env_type)"
            />
            <div class="flex-1 min-w-0">
              <div class="font-medium truncate">
                {{ env.name }}
              </div>
              <div class="truncate text-[10px] text-text-muted">
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
            {{ t('common.environment.empty') }}
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.env-switcher__menu {
  z-index: var(--layer-dropdown);
}
</style>
