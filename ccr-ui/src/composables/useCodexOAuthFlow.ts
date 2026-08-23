import { useCallback, useRef, useState } from 'react'
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
import { createTf, type TranslateFunction } from '@/utils/tf'
// 过渡期接线（批次 4）：Pinia 已删，Zustand 单例经 getState() 提供同名 API。
import { useUIStore } from '@/shell/stores/ui'
import { extractErrorMessage } from '@/utils/errorHandler'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { logger } from '@/utils/logger'

type UnlistenFn = () => void | Promise<void>

/**
 * Codex OAuth 授权子流程的 React 迁移（08-22-state-logic-port 批次 5b-ii）。
 * - 流程瞬态（loginId/authUrl/callbackUrl/pending/portBusy/busy/timeoutMessage）→ useState；
 * - OAuth 命令为多步编排（端口检查 + 启动 + 打开浏览器 / 提交回调 + 收尾提示），
 *   沿用批次 5b-i 的先例（useCodexAgentSources）：编排类命令保持 async 回调而非拆
 *   useMutation，语义逐行等价；
 * - Tauri 事件监听（completed/timeout）保留视图手动 install/cleanup 的调用时序，
 *   数组累积改写为取消协议：cleanup 已跑过时迟到的 unlisten 立即调用
 *   （原 oauthUnlisteners.push 写法见 mutation-rewrite.md 对应行）。事件回调经
 *   latestDepsRef 读取最新 loginId 与注入依赖，避免闭包捕获陈旧值。
 *
 * 签名变化（消费方 AddCodexAccountModal.vue 待迁移）：
 * - useI18n → t 参数传入；
 * - ComputedRef/Ref 注入 → 普通值 + setState 写入器（共享弹窗态归调用方持有）。
 */
export function useCodexOAuthFlow(deps: {
  /** i18n 翻译函数 */
  t: TranslateFunction
  /** 用户自定义的目标账号名（经校验后的有效值，null 表示自动命名） */
  effectivePreferredAccountName: string | null
  /** 校验命名草稿；非法时写入 addAccountError 并返回 false */
  ensurePreferredAccountNameIsValid: () => boolean
  /** 成功添加账号后的统一收尾（刷新列表 + 成功提示 + 重置 oauth 态） */
  applyMutationSuccess: (result: CodexAuthMutationResponse, successMessage: string) => Promise<void>
  /** 添加账号弹窗的错误写入器（与其他添加方式共享） */
  setAddAccountError: (value: string | null) => void
  /** 添加账号弹窗的提示写入器 */
  setAddAccountNotice: (value: string | null) => void
  /** 添加账号弹窗开关写入器 */
  setShowAddAccountModal: (value: boolean) => void
}) {
  const { t } = deps

  const tf = createTf(t)
  const uiStore = useUIStore.getState()

  const [oauthLoginId, setOauthLoginId] = useState('')
  const [oauthAuthUrl, setOauthAuthUrl] = useState('')
  const [oauthCallbackUrl, setOauthCallbackUrl] = useState('')
  const [oauthPending, setOauthPending] = useState(false)
  const [oauthPortBusy, setOauthPortBusy] = useState(false)
  const [oauthBusy, setOauthBusy] = useState(false)
  const [oauthTimeoutMessage, setOauthTimeoutMessage] = useState<string | null>(null)

  // 监听回调读取的最新值镜像（原 Vue 闭包读 .value 的等价物）。
  const oauthLoginIdRef = useRef(oauthLoginId)
  oauthLoginIdRef.current = oauthLoginId
  const latestDepsRef = useRef(deps)
  latestDepsRef.current = deps

  // 取消协议状态：cleanup 已跑过时迟到的 unlisten 立即调用，不入数组。
  const oauthUnlistenersRef = useRef<UnlistenFn[]>([])
  const oauthDisposedRef = useRef(false)

  const resetOauthState = useCallback(() => {
    setOauthLoginId('')
    setOauthAuthUrl('')
    setOauthCallbackUrl('')
    setOauthPending(false)
  }, [])

  const refreshOauthPortStatus = useCallback(async () => {
    if (!isTauriRuntime()) {
      setOauthPortBusy(false)
      return
    }
    try {
      setOauthPortBusy(await codexIsOAuthPortInUse())
    } catch (error) {
      logger.error('Failed to check oauth port:', error)
      setOauthPortBusy(false)
    }
  }, [])

  // 端口占用读数的最新值镜像（原闭包直接读 oauthPortBusy.value）。
  const oauthPortBusyRef = useRef(oauthPortBusy)
  oauthPortBusyRef.current = oauthPortBusy
  // pending 读数镜像（handleStartOauth 内的守卫判断）。
  const oauthPendingRef = useRef(oauthPending)
  oauthPendingRef.current = oauthPending

  const handleReleaseOauthPort = useCallback(async () => {
    try {
      setOauthBusy(true)
      const report = await codexReleaseOAuthPort()
      await refreshOauthPortStatus()
      if (report.unknownPids.length > 0) {
        latestDepsRef.current.setAddAccountError(
          tf(
            'codex.auth.oauth.portOwnedByOtherProcess',
            'Port 1455 is owned by another process (PID: {pids}). Close that process manually.',
            { pids: report.unknownPids.join(', ') }
          )
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
      latestDepsRef.current.setAddAccountError(
        extractErrorMessage(error) ||
          tf('codex.auth.oauth.releasePortFailed', 'Failed to release port 1455.')
      )
    } finally {
      setOauthBusy(false)
    }
  }, [refreshOauthPortStatus, tf, uiStore])

  const handleStartOauth = useCallback(async () => {
    latestDepsRef.current.setAddAccountError(null)
    latestDepsRef.current.setAddAccountNotice(null)
    setOauthTimeoutMessage(null)
    if (!latestDepsRef.current.ensurePreferredAccountNameIsValid()) {
      return
    }
    try {
      setOauthBusy(true)
      await refreshOauthPortStatus()
      if (oauthPortBusyRef.current && !oauthPendingRef.current) {
        latestDepsRef.current.setAddAccountError(
          tf(
            'codex.auth.oauth.portBusyError',
            'Port 1455 is busy. Release it first, then retry the OAuth flow.'
          )
        )
        return
      }

      const result = await codexOAuthLoginStart()
      setOauthLoginId(result.loginId)
      setOauthAuthUrl(result.authUrl)
      setOauthPending(true)
      await codexOpenExternalUrl(result.authUrl)
      latestDepsRef.current.setAddAccountNotice(
        tf(
          'codex.auth.oauth.started',
          'Browser authorization started. After the callback arrives, CCR will finish the login automatically.'
        )
      )
    } catch (error) {
      latestDepsRef.current.setAddAccountError(
        extractErrorMessage(error) ||
          tf('codex.auth.oauth.startFailed', 'Failed to start OAuth authorization.')
      )
    } finally {
      setOauthBusy(false)
    }
  }, [refreshOauthPortStatus, tf])

  const handleSubmitOauthCallback = useCallback(async () => {
    latestDepsRef.current.setAddAccountError(null)
    if (!oauthLoginIdRef.current || !oauthCallbackUrl.trim()) {
      latestDepsRef.current.setAddAccountError(
        tf('codex.auth.oauth.callbackRequired', 'Paste the callback URL before submitting it.')
      )
      return
    }

    try {
      setOauthBusy(true)
      await codexOAuthSubmitCallbackUrl(oauthLoginIdRef.current, oauthCallbackUrl.trim())
      latestDepsRef.current.setAddAccountNotice(
        tf(
          'codex.auth.oauth.callbackSubmitted',
          'Callback received. Finalizing the OAuth account now...'
        )
      )
    } catch (error) {
      latestDepsRef.current.setAddAccountError(
        extractErrorMessage(error) ||
          tf('codex.auth.oauth.callbackSubmitFailed', 'Failed to submit the callback URL.')
      )
    } finally {
      setOauthBusy(false)
    }
  }, [oauthCallbackUrl, tf])

  const finalizeOauthLoginById = useCallback(
    async (loginId: string) => {
      if (!latestDepsRef.current.ensurePreferredAccountNameIsValid()) {
        return
      }
      try {
        setOauthBusy(true)
        const result = await codexOAuthLoginCompleted(
          loginId,
          latestDepsRef.current.effectivePreferredAccountName
        )
        await latestDepsRef.current.applyMutationSuccess(
          result,
          tf('codex.auth.oauth.success', 'OAuth account added successfully.')
        )
        latestDepsRef.current.setShowAddAccountModal(false)
      } catch (error) {
        latestDepsRef.current.setAddAccountError(
          extractErrorMessage(error) ||
            tf('codex.auth.oauth.completeFailed', 'Failed to complete the OAuth login.')
        )
      } finally {
        setOauthBusy(false)
      }
    },
    [tf]
  )

  const handleFinalizeOauth = useCallback(async () => {
    if (!oauthLoginIdRef.current) {
      latestDepsRef.current.setAddAccountError(
        tf('codex.auth.oauth.notStarted', 'Start the OAuth flow before finalizing it.')
      )
      return
    }
    await finalizeOauthLoginById(oauthLoginIdRef.current)
  }, [finalizeOauthLoginById, tf])

  const cancelOauthFlow = useCallback(async () => {
    try {
      setOauthBusy(true)
      if (oauthLoginIdRef.current) {
        await codexOAuthLoginCancel(oauthLoginIdRef.current)
      }
      resetOauthState()
      await refreshOauthPortStatus()
    } catch (error) {
      latestDepsRef.current.setAddAccountError(
        extractErrorMessage(error) ||
          tf('codex.auth.oauth.cancelFailed', 'Failed to cancel the OAuth flow.')
      )
    } finally {
      setOauthBusy(false)
    }
  }, [refreshOauthPortStatus, resetOauthState, tf])

  const trackOauthListener = useCallback((pending: Promise<UnlistenFn>) => {
    void pending.then((unlisten) => {
      if (oauthDisposedRef.current) {
        void unlisten()
      } else {
        oauthUnlistenersRef.current.push(unlisten)
      }
    })
  }, [])

  const installOauthListeners = useCallback(async () => {
    if (!isTauriRuntime()) return
    try {
      const { listen } = await import('@tauri-apps/api/event')
      oauthDisposedRef.current = false

      const completed = listen<{ loginId?: string }>(
        'codex-oauth-login-completed',
        async (event) => {
          const loginId = event.payload?.loginId
          if (!loginId || loginId !== oauthLoginIdRef.current) return
          await finalizeOauthLoginById(loginId)
        }
      )
      const timeout = listen<{ loginId?: string; timeoutSeconds?: number }>(
        'codex-oauth-login-timeout',
        async (event) => {
          const loginId = event.payload?.loginId
          if (!loginId || loginId !== oauthLoginIdRef.current) return
          setOauthTimeoutMessage(
            tf(
              'codex.auth.oauth.timeoutMessage',
              'No callback arrived within {seconds} seconds. You can restart the flow or paste the manual callback URL.',
              { seconds: event.payload?.timeoutSeconds ?? 300 }
            )
          )
          resetOauthState()
          await refreshOauthPortStatus()
        }
      )

      trackOauthListener(completed)
      trackOauthListener(timeout)
    } catch (error) {
      logger.error('Failed to install oauth listeners:', error)
    }
  }, [finalizeOauthLoginById, refreshOauthPortStatus, resetOauthState, tf, trackOauthListener])

  const cleanupOauthListeners = useCallback(async () => {
    oauthDisposedRef.current = true
    const pending = [...oauthUnlistenersRef.current]
    oauthUnlistenersRef.current = []
    await Promise.allSettled(pending.map((unlisten) => Promise.resolve(unlisten())))
  }, [])

  return {
    oauthLoginId,
    oauthAuthUrl,
    oauthCallbackUrl,
    oauthPending,
    oauthPortBusy,
    oauthBusy,
    oauthTimeoutMessage,
    setOauthCallbackUrl,
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
