import { computed, onMounted, onUnmounted, ref } from 'vue'
import { getCodexTraySnapshot, switchCodexAuth } from '@/api/tauri'
import {
  shellRequestQuit,
  shellShowMainWindow,
} from '@/api/runtime/environment'
import type { CodexTraySnapshot } from '@/types'
import { logger } from '@/utils/logger'
import { getCurrentWindowSafe } from '@/utils/tauriWindow'

export function useCodexTrayPanel() {
  const snapshot = ref<CodexTraySnapshot | null>(null)
  const screen = ref<'overview' | 'switch'>('overview')
  const loading = ref(false)
  const busyAccount = ref<string | null>(null)
  const error = ref<string | null>(null)

  let stopRefreshEvent: (() => void) | null = null

  const currentAccount = computed(() => snapshot.value?.current_account ?? null)
  const accounts = computed(() => snapshot.value?.accounts ?? [])
  const canManageAccounts = computed(() => snapshot.value?.can_manage_accounts ?? false)
  const canOpenSwitchScreen = computed(() => canManageAccounts.value)

  const loadSnapshot = async (force = false) => {
    loading.value = true
    error.value = null

    try {
      snapshot.value = await getCodexTraySnapshot<CodexTraySnapshot>(force)
    } catch (loadError) {
      logger.error('Failed to load Codex tray snapshot:', loadError)
      error.value = loadError instanceof Error ? loadError.message : String(loadError)
    } finally {
      loading.value = false
    }
  }

  const switchAccount = async (name: string) => {
    busyAccount.value = name
    error.value = null

    try {
      await switchCodexAuth(name)
      await loadSnapshot(true)
      screen.value = 'overview'
    } catch (switchError) {
      logger.error('Failed to switch Codex tray account:', switchError)
      error.value = switchError instanceof Error ? switchError.message : String(switchError)
    } finally {
      busyAccount.value = null
    }
  }

  const openMain = async (targetRoute?: string) => {
    await shellShowMainWindow(targetRoute)
  }

  const openUsage = async () => {
    await shellShowMainWindow('/usage')
  }

  const openAuth = async () => {
    await shellShowMainWindow('/codex/auth')
  }

  const quit = async () => {
    await shellRequestQuit()
  }

  const goToSwitchScreen = () => {
    if (!canOpenSwitchScreen.value) {
      return
    }
    screen.value = 'switch'
  }

  const goToOverview = () => {
    screen.value = 'overview'
  }

  onMounted(async () => {
    await loadSnapshot(false)

    const win = await getCurrentWindowSafe()
    if (!win) {
      return
    }

    stopRefreshEvent = await win.listen<CodexTraySnapshot>('codex-tray:refresh', (event) => {
      snapshot.value = event.payload
      error.value = null
    })
  })

  onUnmounted(() => {
    stopRefreshEvent?.()
    stopRefreshEvent = null
  })

  return {
    accounts,
    busyAccount,
    canManageAccounts,
    canOpenSwitchScreen,
    currentAccount,
    error,
    goToOverview,
    goToSwitchScreen,
    loading,
    openAuth,
    openMain,
    openUsage,
    quit,
    screen,
    snapshot,
    switchAccount,
    loadSnapshot,
  }
}
