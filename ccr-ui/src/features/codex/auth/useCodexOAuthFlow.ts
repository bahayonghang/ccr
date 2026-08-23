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
import { surfaceNotify } from '@/configs/surfaceNotify'
import type { CodexAuthMutationResponse } from '@/types'
import { extractErrorMessage } from '@/utils/errorHandler'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { logger } from '@/utils/logger'
import { createTf, type TranslateFunction } from '@/utils/tf'

type UnlistenFn = () => void | Promise<void>

export function useCodexOAuthFlow(deps: {
  t: TranslateFunction
  effectivePreferredAccountName: string | null
  ensurePreferredAccountNameIsValid: () => boolean
  applyMutationSuccess: (result: CodexAuthMutationResponse, successMessage: string) => Promise<void>
  setAddAccountError: (value: string | null) => void
  setAddAccountNotice: (value: string | null) => void
  setShowAddAccountModal: (value: boolean) => void
}) {
  const tf = createTf(deps.t)
  const [oauthLoginId, setOauthLoginId] = useState('')
  const [oauthAuthUrl, setOauthAuthUrl] = useState('')
  const [oauthCallbackUrl, setOauthCallbackUrl] = useState('')
  const [oauthPending, setOauthPending] = useState(false)
  const [oauthPortBusy, setOauthPortBusy] = useState(false)
  const [oauthBusy, setOauthBusy] = useState(false)
  const [oauthTimeoutMessage, setOauthTimeoutMessage] = useState<string | null>(null)
  const oauthLoginIdRef = useRef(oauthLoginId)
  oauthLoginIdRef.current = oauthLoginId
  const latestDepsRef = useRef(deps)
  latestDepsRef.current = deps
  const oauthUnlistenersRef = useRef<UnlistenFn[]>([])
  const oauthDisposedRef = useRef(false)
  const oauthPortBusyRef = useRef(oauthPortBusy)
  oauthPortBusyRef.current = oauthPortBusy
  const oauthPendingRef = useRef(oauthPending)
  oauthPendingRef.current = oauthPending

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

  const handleReleaseOauthPort = useCallback(async () => {
    try {
      setOauthBusy(true)
      const report = await codexReleaseOAuthPort()
      await refreshOauthPortStatus()
      if (report.unknownPids.length > 0) {
        latestDepsRef.current.setAddAccountError(
          tf('codex.auth.oauth.portOwnedByOtherProcess', 'Port 1455 is owned by another process (PID: {pids}). Close that process manually.', {
            pids: report.unknownPids.join(', '),
          }),
        )
        return
      }
      surfaceNotify.success(tf('codex.auth.oauth.releasePortSuccess', 'Released the callback port ({count} process(es)).', { count: report.cancelRequested }))
    } catch (error) {
      latestDepsRef.current.setAddAccountError(extractErrorMessage(error) || tf('codex.auth.oauth.releasePortFailed', 'Failed to release port 1455.'))
    } finally {
      setOauthBusy(false)
    }
  }, [refreshOauthPortStatus, tf])

  const handleStartOauth = useCallback(async () => {
    latestDepsRef.current.setAddAccountError(null)
    latestDepsRef.current.setAddAccountNotice(null)
    setOauthTimeoutMessage(null)
    if (!latestDepsRef.current.ensurePreferredAccountNameIsValid()) return
    try {
      setOauthBusy(true)
      await refreshOauthPortStatus()
      if (oauthPortBusyRef.current && !oauthPendingRef.current) {
        latestDepsRef.current.setAddAccountError(tf('codex.auth.oauth.portBusyError', 'Port 1455 is busy. Release it first, then retry the OAuth flow.'))
        return
      }
      const result = await codexOAuthLoginStart()
      setOauthLoginId(result.loginId)
      setOauthAuthUrl(result.authUrl)
      setOauthPending(true)
      await codexOpenExternalUrl(result.authUrl)
      latestDepsRef.current.setAddAccountNotice(
        tf('codex.auth.oauth.started', 'Browser authorization started. After the callback arrives, CCR will finish the login automatically.'),
      )
    } catch (error) {
      latestDepsRef.current.setAddAccountError(extractErrorMessage(error) || tf('codex.auth.oauth.startFailed', 'Failed to start OAuth authorization.'))
    } finally {
      setOauthBusy(false)
    }
  }, [refreshOauthPortStatus, tf])

  const handleSubmitOauthCallback = useCallback(async () => {
    latestDepsRef.current.setAddAccountError(null)
    if (!oauthLoginIdRef.current || !oauthCallbackUrl.trim()) {
      latestDepsRef.current.setAddAccountError(tf('codex.auth.oauth.callbackRequired', 'Paste the callback URL before submitting it.'))
      return
    }
    try {
      setOauthBusy(true)
      await codexOAuthSubmitCallbackUrl(oauthLoginIdRef.current, oauthCallbackUrl.trim())
      latestDepsRef.current.setAddAccountNotice(tf('codex.auth.oauth.callbackSubmitted', 'Callback received. Finalizing the OAuth account now...'))
    } catch (error) {
      latestDepsRef.current.setAddAccountError(extractErrorMessage(error) || tf('codex.auth.oauth.callbackSubmitFailed', 'Failed to submit the callback URL.'))
    } finally {
      setOauthBusy(false)
    }
  }, [oauthCallbackUrl, tf])

  const finalizeOauthLoginById = useCallback(
    async (loginId: string) => {
      if (!latestDepsRef.current.ensurePreferredAccountNameIsValid()) return
      try {
        setOauthBusy(true)
        const result = await codexOAuthLoginCompleted(loginId, latestDepsRef.current.effectivePreferredAccountName)
        await latestDepsRef.current.applyMutationSuccess(result, tf('codex.auth.oauth.success', 'OAuth account added successfully.'))
        latestDepsRef.current.setShowAddAccountModal(false)
      } catch (error) {
        latestDepsRef.current.setAddAccountError(extractErrorMessage(error) || tf('codex.auth.oauth.completeFailed', 'Failed to complete the OAuth login.'))
      } finally {
        setOauthBusy(false)
      }
    },
    [tf],
  )

  const handleFinalizeOauth = useCallback(async () => {
    if (!oauthLoginIdRef.current) {
      latestDepsRef.current.setAddAccountError(tf('codex.auth.oauth.notStarted', 'Start the OAuth flow before finalizing it.'))
      return
    }
    await finalizeOauthLoginById(oauthLoginIdRef.current)
  }, [finalizeOauthLoginById, tf])

  const cancelOauthFlow = useCallback(async () => {
    try {
      setOauthBusy(true)
      if (oauthLoginIdRef.current) await codexOAuthLoginCancel(oauthLoginIdRef.current)
      resetOauthState()
      await refreshOauthPortStatus()
    } catch (error) {
      latestDepsRef.current.setAddAccountError(extractErrorMessage(error) || tf('codex.auth.oauth.cancelFailed', 'Failed to cancel the OAuth flow.'))
    } finally {
      setOauthBusy(false)
    }
  }, [refreshOauthPortStatus, resetOauthState, tf])

  const trackOauthListener = useCallback((pending: Promise<UnlistenFn>) => {
    void pending.then((unlisten) => {
      if (oauthDisposedRef.current) void unlisten()
      else oauthUnlistenersRef.current.push(unlisten)
    })
  }, [])

  const installOauthListeners = useCallback(async () => {
    if (!isTauriRuntime()) return
    try {
      const { listen } = await import('@tauri-apps/api/event')
      oauthDisposedRef.current = false
      trackOauthListener(
        listen<{ loginId?: string }>('codex-oauth-login-completed', async (event) => {
          const loginId = event.payload?.loginId
          if (!loginId || loginId !== oauthLoginIdRef.current) return
          await finalizeOauthLoginById(loginId)
        }),
      )
      trackOauthListener(
        listen<{ loginId?: string; timeoutSeconds?: number }>('codex-oauth-login-timeout', async (event) => {
          const loginId = event.payload?.loginId
          if (!loginId || loginId !== oauthLoginIdRef.current) return
          setOauthTimeoutMessage(
            tf('codex.auth.oauth.timeoutMessage', 'No callback arrived within {seconds} seconds. You can restart the flow or paste the manual callback URL.', {
              seconds: event.payload?.timeoutSeconds ?? 300,
            }),
          )
          resetOauthState()
          await refreshOauthPortStatus()
        }),
      )
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
    setOauthTimeoutMessage,
    resetOauthState,
    refreshOauthPortStatus,
    handleReleaseOauthPort,
    handleStartOauth,
    handleSubmitOauthCallback,
    handleFinalizeOauth,
    cancelOauthFlow,
    installOauthListeners,
    cleanupOauthListeners,
  }
}
