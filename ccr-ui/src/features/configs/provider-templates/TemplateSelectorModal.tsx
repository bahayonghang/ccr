import { useCallback, useMemo, type KeyboardEvent } from 'react'
import type { ProviderTemplate, ProviderTemplateOption } from '@/types/providerTemplates'
import { BaseModal, SIcon } from '@/ui'
import { isZhLocale, tt } from '../locale'
import { ManualTemplateRow } from './ManualTemplateRow'
import { TemplateOptionRow } from './TemplateOptionRow'

interface TemplateSelectorModalProps {
  open: boolean
  platformLabel: string
  searchInputId: string
  query: string
  results: ProviderTemplateOption[]
  activeIndex: number
  optionId: (index: number) => string
  onClose: () => void
  onQueryChange: (value: string) => void
  onHover: (index: number) => void
  onSelectManual: () => void
  onSelectOption: (option: ProviderTemplateOption) => void
  onEdit: (template: ProviderTemplate) => void
  onDelete: (id: string) => void
  onNewCustom: () => void
  onSaveCurrent?: () => void
  onKeyDown: (event: KeyboardEvent<HTMLInputElement>) => void
}

export function TemplateSelectorModal({
  open,
  platformLabel,
  searchInputId,
  query,
  results,
  activeIndex,
  optionId,
  onClose,
  onQueryChange,
  onHover,
  onSelectManual,
  onSelectOption,
  onEdit,
  onDelete,
  onNewCustom,
  onSaveCurrent,
  onKeyDown,
}: TemplateSelectorModalProps) {
  const handleOpenChange = useCallback(
    (next: boolean) => {
      if (!next) onClose()
    },
    [onClose],
  )
  const handleQuery = useCallback(
    (event: { target: EventTarget | null }) => {
      onQueryChange((event.target as HTMLInputElement).value)
    },
    [onQueryChange],
  )
  const hoverManual = useCallback(() => {
    onHover(0)
  }, [onHover])
  const renderHeader = useCallback(
    (scope: { titleId: string }) => (
      <div className="provider-template-modal__header">
        <div className="provider-template-modal__heading">
          <span className="provider-template-modal__eyebrow">{platformLabel}</span>
          <h2 id={scope.titleId} className="provider-template-modal__title">
            {tt('Provider templates', 'Provider templates')}
          </h2>
        </div>
        <span className="provider-template-modal__count">
          {isZhLocale() ? `${results.length} 条匹配` : `${results.length} matches`}
        </span>
      </div>
    ),
    [platformLabel, results.length],
  )

  const footer = useMemo(
    () => (
      <div className="provider-template-modal__footer">
        <div className="provider-template-modal__keys">
          <span>
            <kbd>{tt('回车', 'Enter')}</kbd> {tt('应用', 'apply')}
          </span>
          <span>
            <kbd>↑↓</kbd> {tt('选择', 'select')}
          </span>
          <span>
            <kbd>Esc</kbd> {tt('关闭', 'close')}
          </span>
        </div>
        <div className="provider-template-modal__footer-actions">
          <button type="button" className="provider-template-modal__secondary" data-testid="provider-template-new-custom" onClick={onNewCustom}>
            {tt('新建模板', 'New template')}
          </button>
          {onSaveCurrent ? (
            <button type="button" className="provider-template-modal__primary" data-testid="provider-template-save-current" onClick={onSaveCurrent}>
              {tt('保存当前内容', 'Save current')}
            </button>
          ) : null}
        </div>
      </div>
    ),
    [onNewCustom, onSaveCurrent],
  )

  return (
    <BaseModal
      modelValue={open}
      title={tt('Provider templates', 'Provider templates')}
      description={isZhLocale() ? `${platformLabel} 提供商模板选择器` : `${platformLabel} provider template selector`}
      size="full"
      surface="solid"
      contentClass="provider-template-modal"
      showClose={false}
      header={renderHeader}
      footer={footer}
      onUpdateModelValue={handleOpenChange}
      onClose={onClose}
    >
      <div className="provider-template-modal__body">
        <label className="sr-only" htmlFor={searchInputId}>
          {tt('搜索 provider templates', 'Search provider templates')}
        </label>
        <div className="provider-template-modal__search">
          <SIcon name="Search" size="w-4 h-4" className="provider-template-modal__search-icon" />
          <input
            id={searchInputId}
            className="provider-template-modal__search-input"
            data-testid="provider-template-search"
            placeholder={tt('按 provider、host、model、tag 搜索...', 'Search by provider, host, model, tag...')}
            aria-activedescendant={optionId(activeIndex)}
            aria-controls="provider-template-listbox"
            key={open ? 'search-open' : 'search-closed'}
            defaultValue={query}
            onChange={handleQuery}
            onKeyDown={onKeyDown}
          />
        </div>
        <div id="provider-template-listbox" className="provider-template-modal__list" role="listbox" aria-label={tt('Provider template 结果', 'Provider template results')}>
          <ManualTemplateRow active={activeIndex === 0} optionId={optionId(0)} onHover={hoverManual} onSelect={onSelectManual} />
          {results.length > 0
            ? results.map((option, index) => (
                <TemplateOptionRow
                  key={option.id}
                  option={option}
                  index={index + 1}
                  active={activeIndex === index + 1}
                  optionId={optionId(index + 1)}
                  onHover={onHover}
                  onSelect={onSelectOption}
                  onEdit={onEdit}
                  onDelete={onDelete}
                />
              ))
            : (
              <div className="provider-template-modal__empty" data-testid="provider-template-empty">
                {tt('当前关键词下没有匹配模板。', 'No template matched the current keywords.')}
              </div>
            )}
        </div>
      </div>
    </BaseModal>
  )
}
