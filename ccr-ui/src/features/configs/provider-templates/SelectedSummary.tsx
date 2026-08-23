interface SelectedSummaryProps {
  sourceLabel: string
  name: string
  endpoint: string
}

export function SelectedSummary({ sourceLabel, name, endpoint }: SelectedSummaryProps) {
  const extra = endpoint.trim() ? ` · ${endpoint.trim()}` : ''
  return (
    <div className="provider-template-selector__summary" data-testid="provider-template-selected-summary">
      <span className="provider-template-selector__summary-badge">{sourceLabel}</span>
      <span className="provider-template-selector__summary-text">
        {name}
        {extra}
      </span>
    </div>
  )
}
