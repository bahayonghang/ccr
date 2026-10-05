import { BaseModal } from '@/ui'
import type { UsageOpsCockpitPresentation } from '@/views/usage/usageOpsCockpit'
import { useUsageT } from '../translate'
import '../styles/usage-diagnostics-drawer.css'

interface UsageDiagnosticsDrawerProps {
  open: boolean
  presentation: UsageOpsCockpitPresentation
  onOpenChange: (open: boolean) => void
  onRefresh: () => void
}

export function UsageDiagnosticsDrawer({
  open,
  presentation,
  onOpenChange,
  onRefresh,
}: UsageDiagnosticsDrawerProps) {
  const t = useUsageT()

  return (
    <BaseModal
      modelValue={open}
      title={t('usage.dashboard.ops.drawerTitle')}
      description={presentation.summaryLine}
      size="2xl"
      scrollable
      onUpdateModelValue={onOpenChange}
    >
      <div className="usage-diag">
        <p className="usage-diag__scope">{presentation.summaryLine}</p>
        <section className="usage-diag__section">
          <div className="usage-diag__health-grid">
            {presentation.healthItems.map((item) => (
              <article key={item.id} className={`usage-diag-health usage-diag-health--${item.tone}`}>
                <div>
                  <p className="usage-diag-health__label">{item.label}</p>
                  <strong>{item.value}</strong>
                  <p>{item.detail}</p>
                </div>
              </article>
            ))}
          </div>
        </section>
        <button type="button" className="usage-diag-source__refresh" onClick={onRefresh}>
          {t('usage.dashboard.ops.actions.refresh_usage')}
        </button>
      </div>
    </BaseModal>
  )
}
