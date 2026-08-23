import { ref, type ComputedRef, type Ref } from 'vue'
import {
  codexIsOAuthPortInUse,
  codexOAuthLoginCancel,
  codexOAuthLoginCompleted,
  codexOAuthLoginStart,
  codexOAuthSubmitCallbackUrl,
  codexOpenExternalUrl,
  codexReleaseOAuthPort,
} from '@/api'
import type { CodexAuthMutationResponse } from '@/types'
import { useTf } from '@/composables/useTf'
// 过渡期接线（批次 4）：Pinia 已删，Zustand 单例经 getState() 提供同名 API；
// 本文件在批次 5 转换为 React hook 时整体重写。
import { useUIStore } from '@/shell/stores/ui'
import { extractErrorMessage } from '@/utils/errorHandler'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { logger } from '@/utils/logger'

type UnlistenFn = () => void | Promise<void>

/**
 * Codex OAuth 授权子流程：端口探测/释放、启动浏览器授权、手工回调提交、完成/取消，
 * 以及 tauri 事件监听（completed / timeout）的安装与清理。
 * 与「添加账号弹窗」深度耦合，故由该弹窗调用并注入其命名草稿与成功回调。
 */
export function useCodexOAuthFlow(deps: {
  /** 用户自定义的目标账号名（经校验后的有效值，null 表示自动命名） */
  effectivePreferredAccountName: ComputedRef<string | null>
  /** 校验命名草稿；非法时写入 addAccountError 并返回 false */
  ensurePreferredAccountNameIsValid: () => boolean
  /** 成功添加账号后的统一收尾（刷新列表 + 成功提示 + 重置 oauth 态） */
  applyMutationSuccess: (result: CodexAuthMutationResponse, successMessage: string) => Promise<void>
  /** 添加账号弹窗的错误/提示/开关状态（与其他添加方式共享） */
  addAccountError: Ref<string | null>
  addAccountNotice: Ref<string | null>
  showAddAccountModal: Ref<boolean>
}) {
  const {
    effectivePreferredAccountName,
    ensurePreferredAccountNameIsValid,
    applyMutationSuccess,
    addAccountError,
    addAccountNotice,
    showAddAccountModal,
  } = deps

  const tf = useTf()
  const uiStore = useUIStore.getState()

  const oauthLoginId = ref('')
  const oauthAuthUrl = ref('')
  const oauthCallbackUrl = ref('')
  const oauthPending = ref(false)
  const oauthPortBusy = ref(false)
  const oauthBusy = ref(false)
  const oauthTimeoutMessage = ref<string | null>(null)

  let oauthUnlisteners: UnlistenFn[] = []

  const resetOauthState = () => {
    oauthLoginId.value = ''
    oauthAuthUrl.value = ''
    oauthCallbackUrl.value = ''
    oauthPending.value = false
  }

  const refreshOauthPortStatus = async () => {
    if (!isTauriRuntime()) {
      oauthPortBusy.value = false
      return
    }
    try {
      oauthPortBusy.value = await codexIsOAuthPortInUse()
    } catch (error) {
      logger.error('Failed to check oauth port:', error)
      oauthPortBusy.value = false
    }
  }

  const handleReleaseOauthPort = async () => {
    try {
      oauthBusy.value = true
      const report = await codexReleaseOAuthPort()
      await refreshOauthPortStatus()
      if (report.unknownPids.length > 0) {
        addAccountError.value = tf(
          'codex.auth.oauth.portOwnedByOtherProcess',
          'Port 1455 is owned by another process (PID: {pids}). Close that process manually.',
          { pids: report.unknownPids.join(', ') }
        )
        return
      }
      uiStore.showSuccess(
        tf(
          'codex.auth.oauth.releasePortSuccess',
          'Released the callback port ({count} process(es)).',
          { count: report.cancelRequested }
        )
      )
    } catch (error) {
      addAccountError.value =
        extractErrorMessage(error) ||
        tf('codex.auth.oauth.releasePortFailed', 'Failed to release port 1455.')
    } finally {
      oauthBusy.value = false
    }
  }

  const handleStartOauth = async () => {
    addAccountError.value = null
    addAccountNotice.value = null
    oauthTimeoutMessage.value = null
    if (!ensurePreferredAccountNameIsValid()) {
      return
    }
    try {
      oauthBusy.value = true
      await refreshOauthPortStatus()
      if (oauthPortBusy.value && !oauthPending.value) {
        addAccountError.value = tf(
          'codex.auth.oauth.portBusyError',
          'Port 1455 is busy. Release it first, then retry the OAuth flow.'
        )
        return
      }

      const result = await codexOAuthLoginStart()
      oauthLoginId.value = result.loginId
      oauthAuthUrl.value = result.authUrl
      oauthPending.value = true
      await codexOpenExternalUrl(result.authUrl)
      addAccountNotice.value = tf(
        'codex.auth.oauth.started',
        'Browser authorization started. After the callback arrives, CCR will finish the login automatically.'
      )
    } catch (error) {
      addAccountError.value =
        extractErrorMessage(error) ||
        tf('codex.auth.oauth.startFailed', 'Failed to start OAuth authorization.')
    } finally {
      oauthBusy.value = false
    }
  }

  const handleSubmitOauthCallback = async () => {
    addAccountError.value = null
    if (!oauthLoginId.value || !oauthCallbackUrl.value.trim()) {
      addAccountError.value = tf(
        'codex.auth.oauth.callbackRequired',
        'Paste the callback URL before submitting it.'
      )
      return
    }

    try {
      oauthBusy.value = true
      await codexOAuthSubmitCallbackUrl(oauthLoginId.value, oauthCallbackUrl.value.trim())
      addAccountNotice.value = tf(
        'codex.auth.oauth.callbackSubmitted',
        'Callback received. Finalizing the OAuth account now...'
      )
    } catch (error) {
      addAccountError.value =
        extractErrorMessage(error) ||
        tf('codex.auth.oauth.callbackSubmitFailed', 'Failed to submit the callback URL.')
    } finally {
      oauthBusy.value = false
    }
  }

  const finalizeOauthLoginById = async (loginId: string) => {
    if (!ensurePreferredAccountNameIsValid()) {
      return
    }
    try {
      oauthBusy.value = true
      const result = await codexOAuthLoginCompleted(
        loginId,
        effectivePreferredAccountName.value
      )
      await applyMutationSuccess(
        result,
        tf('codex.auth.oauth.success', 'OAuth account added successfully.')
      )
      showAddAccountModal.value = false
    } catch (error) {
      addAccountError.value =
        extractErrorMessage(error) ||
        tf('codex.auth.oauth.completeFailed', 'Failed to complete the OAuth login.')
    } finally {
      oauthBusy.value = false
    }
  }

  const handleFinalizeOauth = async () => {
    if (!oauthLoginId.value) {
      addAccountError.value = tf(
        'codex.auth.oauth.notStarted',
        'Start the OAuth flow before finalizing it.'
      )
      return
    }
    await finalizeOauthLoginById(oauthLoginId.value)
  }

  const cancelOauthFlow = async () => {
    try {
      oauthBusy.value = true
      if (oauthLoginId.value) {
        await codexOAuthLoginCancel(oauthLoginId.value)
      }
      resetOauthState()
      await refreshOauthPortStatus()
    } catch (error) {
      addAccountError.value =
        extractErrorMessage(error) ||
        tf('codex.auth.oauth.cancelFailed', 'Failed to cancel the OAuth flow.')
    } finally {
      oauthBusy.value = false
    }
  }

  const installOauthListeners = async () => {
    if (!isTauriRuntime()) return
    try {
      const { listen } = await import('@tauri-apps/api/event')
      const completed = await listen<{ loginId?: string }>(
        'codex-oauth-login-completed',
        async (event) => {
          const loginId = event.payload?.loginId
          if (!loginId || loginId !== oauthLoginId.value) return
          await finalizeOauthLoginById(loginId)
        }
      )
      const timeout = await listen<{ loginId?: string; timeoutSeconds?: number }>(
        'codex-oauth-login-timeout',
        async (event) => {
          const loginId = event.payload?.loginId
          if (!loginId || loginId !== oauthLoginId.value) return
          oauthTimeoutMessage.value = tf(
            'codex.auth.oauth.timeoutMessage',
            'No callback arrived within {seconds} seconds. You can restart the flow or paste the manual callback URL.',
            { seconds: event.payload?.timeoutSeconds ?? 300 }
          )
          resetOauthState()
          await refreshOauthPortStatus()
        }
      )
      oauthUnlisteners.push(completed, timeout)
    } catch (error) {
      logger.error('Failed to install oauth listeners:', error)
    }
  }

  const cleanupOauthListeners = async () => {
    const pending = [...oauthUnlisteners]
    oauthUnlisteners = []
    await Promise.allSettled(pending.map((unlisten) => Promise.resolve(unlisten())))
  }

  return {
    oauthLoginId,
    oauthAuthUrl,
    oauthCallbackUrl,
    oauthPending,
    oauthPortBusy,
    oauthBusy,
    oauthTimeoutMessage,
    resetOauthState,
    refreshOauthPortStatus,
    handleReleaseOauthPort,
    handleStartOauth,
    handleSubmitOauthCallback,
    finalizeOauthLoginById,
    handleFinalizeOauth,
    cancelOauthFlow,
    installOauthListeners,
    cleanupOauthListeners,
  }
}
