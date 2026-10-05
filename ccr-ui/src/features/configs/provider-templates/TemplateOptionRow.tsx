import { memo, useCallback } from 'react'
import type { MouseEvent } from 'react'
import type { ProviderTemplate, ProviderTemplateOption } from '@/types/providerTemplates'
import { SIcon } from '@/ui'
import { tt } from '../locale'

interface TemplateOptionRowProps {
  option: ProviderTemplateOption
  index: number
  active: boolean
  optionId: string
  onHover: (index: number) => void
  onSelect: (option: ProviderTemplateOption) => void
  onEdit: (template: ProviderTemplate) => void
  onDelete: (id: string) => void
}

export const TemplateOptionRow = memo(function TemplateOptionRow({
  option,
  index,
  active,
  optionId,
  onHover,
  onSelect,
  onEdit,
  onDelete,
}: TemplateOptionRowProps) {
  const handleHover = useCallback(() => {
    onHover(index)
  }, [index, onHover])
  const handleSelect = useCallback(() => {
    onSelect(option)
  }, [onSelect, option])
  const handleEdit = useCallback(
    (event: MouseEvent<HTMLButtonElement>) => {
      event.stopPropagation()
      onEdit(option.template)
    },
    [onEdit, option.template],
  )
  const handleDelete = useCallback(
    (event: MouseEvent<HTMLButtonElement>) => {
      event.stopPropagation()
      onDelete(option.template.id)
    },
    [onDelete, option.template.id],
  )
  const className = active
    ? 'provider-template-modal__row provider-template-modal__row--active'
    : 'provider-template-modal__row'

  return (
    <button
      id={optionId}
      type="button"
      role="option"
      className={className}
      data-testid="provider-template-option"
      aria-selected={active}
      data-index={index}
      onMouseEnter={handleHover}
      onClick={handleSelect}
    >
      <span className="provider-template-modal__icon-box">
        <SIcon name={option.template.source === 'custom' ? 'Database' : 'Blocks'} size="w-3.5 h-3.5" />
      </span>
      <span className="provider-template-modal__row-main">
        <span className="provider-template-modal__row-title">{option.label}</span>
        <span className="provider-template-modal__row-sub">{option.subtitle}</span>
      </span>
      <span className="provider-template-modal__meta">
        <span className="provider-template-modal__pill">{option.sourceLabel}</span>
        <span className="provider-template-modal__pill provider-template-modal__pill--muted">{option.categoryLabel}</span>
        {option.template.source === 'custom' ? (
          <span className="provider-template-modal__actions">
            <button
              type="button"
              className="provider-template-modal__icon-button"
              data-testid="provider-template-edit-custom"
              title={tt('编辑自定义模板', 'Edit custom template')}
              onClick={handleEdit}
            >
              <SIcon name="Pencil" size="w-3.5 h-3.5" />
            </button>
            <button
              type="button"
              className="provider-template-modal__icon-button provider-template-modal__icon-button--danger"
              data-testid="provider-template-delete-custom"
              title={tt('删除自定义模板', 'Delete custom template')}
              onClick={handleDelete}
            >
              <SIcon name="Trash2" size="w-3.5 h-3.5" />
            </button>
          </span>
        ) : null}
      </span>
    </button>
  )
})
