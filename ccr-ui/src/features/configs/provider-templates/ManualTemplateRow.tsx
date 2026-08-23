import { memo, useCallback } from 'react'
import { SIcon } from '@/ui'
import { tt } from '../locale'

interface ManualTemplateRowProps {
  active: boolean
  optionId: string
  onHover: () => void
  onSelect: () => void
}

export const ManualTemplateRow = memo(function ManualTemplateRow({
  active,
  optionId,
  onHover,
  onSelect,
}: ManualTemplateRowProps) {
  const handleHover = useCallback(() => {
    onHover()
  }, [onHover])
  const className = active
    ? 'provider-template-modal__row provider-template-modal__row--manual provider-template-modal__row--active'
    : 'provider-template-modal__row provider-template-modal__row--manual'
  return (
    <button
      id={optionId}
      type="button"
      role="option"
      className={className}
      data-testid="provider-template-manual-row"
      aria-selected={active}
      data-index={0}
      onMouseEnter={handleHover}
      onClick={onSelect}
    >
      <span className="provider-template-modal__icon-box">
        <SIcon name="Pencil" size="w-3.5 h-3.5" />
      </span>
      <span className="provider-template-modal__row-main">
        <span className="provider-template-modal__row-title">{tt('手动填写 provider', 'Manual provider')}</span>
        <span className="provider-template-modal__row-sub">
          {tt('不套用模板，继续直接编辑字段。', 'Keep editing fields without applying a template.')}
        </span>
      </span>
      <span className="provider-template-modal__pill">{tt('手动', 'Manual')}</span>
    </button>
  )
})
