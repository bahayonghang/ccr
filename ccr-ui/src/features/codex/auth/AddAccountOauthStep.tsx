import { useCallback, type ChangeEvent } from 'react'
import type { CodexTf } from '../useCodexLocale'
import { SIcon, buttonClass } from '@/ui'

interface AddAccountOauthStepProps {
  tf: CodexTf
  oauthPortBusy: boolean
  oauthPending: boolean
  oauthBusy: boolean
  oauthTimeoutMessage: string | null
  oauthAuthUrl: string
  oauthCallbackUrl: string
  nameError: string | null
  onReleasePort: () => void
  onStart: () => void
  onFinalize: () => void
  onCancel: () => void
  onCallbackChange: (value: string) => void
  onSubmitCallback: () => void
}

export function AddAccountOauthStep({
  tf,
  oauthPortBusy,
  oauthPending,
  oauthBusy,
  oauthTimeoutMessage,
  oauthAuthUrl,
  oauthCallbackUrl,
  nameError,
  onReleasePort,
  onStart,
  onFinalize,
  onCancel,
  onCallbackChange,
  onSubmitCallback,
}: AddAccountOauthStepProps) {
  const handleCallback = useCallback(
    (event: ChangeEvent<HTMLTextAreaElement>) => onCallbackChange(event.target.value),
    [onCallbackChange],
  )
  return (
    <section className="rounded-2xl border border-border-default/15 bg-bg-surface p-5">
      <div className="codex-auth-view__title-inline">
        <SIcon name="Globe" size="w-5 h-5" className="codex-auth-view__section-icon" />
        <div>
          <h3 className="codex-auth-view__section-title">{tf('codex.auth.oauth.title', 'OpenAI OAuth authorization')}</h3>
          <p className="codex-auth-view__section-copy">
            {tf('codex.auth.oauth.hint', 'CCR listens on http://localhost:1455/auth/callback. After the browser flow completes, the account will be imported and switched automatically.')}
          </p>
        </div>
      </div>
      {oauthPortBusy && !oauthPending ? (
        <div className="codex-auth-view__warning-panel">
          <div>
            <p className="font-medium text-text-primary">{tf('codex.auth.oauth.portBusyTitle', 'Port 1455 is occupied')}</p>
            <p className="mt-1 text-sm text-text-muted">{tf('codex.auth.oauth.portBusyHint', 'Release the callback port before starting OAuth, otherwise the browser redirect cannot be captured.')}</p>
          </div>
          <button type="button" className={buttonClass({ variant: 'secondary' })} disabled={oauthBusy} onClick={onReleasePort}>
            {tf('codex.auth.oauth.releasePort', 'Release port')}
          </button>
        </div>
      ) : null}
      {oauthTimeoutMessage ? (
        <div className="codex-auth-view__warning-panel">
          <div>
            <p className="font-medium text-text-primary">{tf('codex.auth.oauth.timeoutTitle', 'Authorization timed out')}</p>
            <p className="mt-1 text-sm text-text-muted">{oauthTimeoutMessage}</p>
          </div>
        </div>
      ) : null}
      <div className="codex-auth-view__oauth-grid">
        <div className="codex-auth-view__oauth-actions">
          <button type="button" className={buttonClass({ variant: 'primary' })} disabled={oauthBusy || (oauthPortBusy && !oauthPending) || Boolean(nameError)} onClick={onStart}>
            <SIcon name={oauthPending ? 'ExternalLink' : 'PlayCircle'} size="w-4 h-4" />
            {oauthPending ? tf('codex.auth.oauth.openBrowser', 'Open browser again') : tf('codex.auth.oauth.start', 'Start OAuth authorization')}
          </button>
          {oauthPending ? (
            <button type="button" className={buttonClass({ variant: 'secondary' })} disabled={oauthBusy} onClick={onFinalize}>
              {tf('codex.auth.oauth.finish', 'Finish login')}
            </button>
          ) : null}
          {oauthPending ? (
            <button type="button" className={buttonClass({ variant: 'ghost' })} disabled={oauthBusy} onClick={onCancel}>
              {tf('codex.auth.oauth.cancel', 'Cancel OAuth')}
            </button>
          ) : null}
        </div>
        <label className="codex-auth-view__input-group codex-auth-view__input-group--full">
          <span className="codex-auth-view__input-label">{tf('codex.auth.oauth.authUrl', 'Authorization URL')}</span>
          <textarea rows={3} className="codex-auth-view__textarea" readOnly value={oauthAuthUrl} />
        </label>
        <label className="codex-auth-view__input-group codex-auth-view__input-group--full">
          <span className="codex-auth-view__input-label">{tf('codex.auth.oauth.callbackUrl', 'Manual callback URL')}</span>
          <textarea
            rows={4}
            className="codex-auth-view__textarea"
            placeholder={tf('codex.auth.oauth.callbackPlaceholder', 'If the browser could not return to CCR, paste the final localhost callback URL here.')}
            value={oauthCallbackUrl}
            onChange={handleCallback}
          />
        </label>
        <div className="codex-auth-view__oauth-actions">
          <button type="button" className={buttonClass({ variant: 'secondary' })} disabled={!oauthPending || oauthBusy || !oauthCallbackUrl.trim()} onClick={onSubmitCallback}>
            {tf('codex.auth.oauth.submitCallback', 'Submit callback URL')}
          </button>
        </div>
      </div>
    </section>
  )
}
