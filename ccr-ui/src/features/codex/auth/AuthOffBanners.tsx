import { SIcon, buttonClass } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

interface AuthOffBannersProps {
  t: TranslateFunction
  canAuthOff: boolean
  canOff: boolean
  loading: boolean
  onAuthOff: () => void
  onProfileOff: () => void
}

export function AuthOffBanners({ t, canAuthOff, canOff, loading, onAuthOff, onProfileOff }: AuthOffBannersProps) {
  return (
    <>
      {canAuthOff ? (
        <section className="codex-auth-view__off-banner mb-4 flex items-center gap-3 rounded-2xl border border-accent-warning/30 bg-bg-elevated p-4" data-testid="codex-auth-off">
          <div className="min-w-0 flex-1">
            <strong>{t('auth.off')}</strong>
            <p className="text-sm text-text-muted">{t('auth.offDescription')}</p>
          </div>
          <button type="button" className={buttonClass({ variant: 'ghost' })} disabled={loading} onClick={onAuthOff}>
            <SIcon name="LogOut" size="w-4 h-4" />{t('auth.off')}
          </button>
        </section>
      ) : null}
      {canOff ? (
        <section className="codex-auth-view__off-banner mb-4 flex items-center gap-3 rounded-2xl border border-accent-warning/30 bg-bg-elevated p-4" data-testid="codex-auth-profile-off">
          <div className="min-w-0 flex-1">
            <strong>{t('codex.auth.off.title')}</strong>
            <p className="text-sm text-text-muted">{t('codex.auth.off.description')}</p>
          </div>
          <button type="button" className={buttonClass({ variant: 'ghost' })} disabled={loading} onClick={onProfileOff}>
            <SIcon name="Power" size="w-4 h-4" />{t('codex.auth.off.action')}
          </button>
        </section>
      ) : null}
    </>
  )
}
