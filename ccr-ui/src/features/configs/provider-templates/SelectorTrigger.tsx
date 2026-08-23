import { SIcon } from '@/ui'

interface SelectorTriggerProps {
  label: string
  helper: string
  disabled: boolean
  title: string
  subtitle: string
  onOpen: () => void
}

export function SelectorTrigger({ label, helper, disabled, title, subtitle, onOpen }: SelectorTriggerProps) {
  return (
    <div className="provider-template-selector__head">
      <div className="provider-template-selector__copy">
        <span className="provider-template-selector__label">{label}</span>
        {helper ? <span className="provider-template-selector__helper">{helper}</span> : null}
      </div>
      <button
        type="button"
        className="provider-template-selector__trigger"
        data-testid="provider-template-trigger"
        disabled={disabled}
        onClick={onOpen}
      >
        <span className="provider-template-selector__trigger-icon">
          <SIcon name="Search" size="w-4 h-4" />
        </span>
        <span className="provider-template-selector__trigger-main">
          <span className="provider-template-selector__trigger-label">{title}</span>
          <span className="provider-template-selector__trigger-sub">{subtitle}</span>
        </span>
        <SIcon name="ChevronDown" size="w-4 h-4" className="provider-template-selector__chevron" />
      </button>
    </div>
  )
}
