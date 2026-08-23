import { useCallback, useEffect, useMemo } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { BaseModal } from '@/ui'
import { tt } from '../locale'
import {
  customTemplateSchema,
  emptyCustomTemplateForm,
  PLATFORM_ITEMS,
  type CustomTemplateForm,
} from '../lib/templateForm'

interface CustomTemplateEditorProps {
  open: boolean
  editing: boolean
  initial: CustomTemplateForm
  error: string
  onClose: () => void
  onSave: (values: CustomTemplateForm) => void
}

export function CustomTemplateEditor({
  open,
  editing,
  initial,
  error,
  onClose,
  onSave,
}: CustomTemplateEditorProps) {
  const form = useForm<CustomTemplateForm>({
    resolver: zodResolver(customTemplateSchema),
    defaultValues: emptyCustomTemplateForm(),
  })
  const { register, handleSubmit, reset, watch } = form

  useEffect(() => {
    if (open) reset(initial)
  }, [initial, open, reset])

  const platformClaude = watch('platformClaude')
  const platformCodex = watch('platformCodex')
  const platformOpencode = watch('platformOpencode')
  const selected = useMemo(
    () =>
      PLATFORM_ITEMS.filter((item) => {
        if (item.id === 'claude') return platformClaude
        if (item.id === 'codex') return platformCodex
        return platformOpencode
      }),
    [platformClaude, platformCodex, platformOpencode],
  )

  const onValid = useCallback(
    (values: CustomTemplateForm) => {
      onSave(values)
    },
    [onSave],
  )
  const onSubmit = useMemo(() => handleSubmit(onValid), [handleSubmit, onValid])
  const handleOpenChange = useCallback(
    (next: boolean) => {
      if (!next) onClose()
    },
    [onClose],
  )

  const fieldClass = 'provider-template-editor__field'
  const inputClass = 'provider-template-editor__input'
  const textareaClass = 'provider-template-editor__textarea'

  return (
    <BaseModal
      modelValue={open}
      title={editing ? tt('编辑模板', 'Edit template') : tt('自定义模板', 'Custom template')}
      description={tt('保存不含密钥的 provider 元数据，方便后续复用。', 'Store non-secret provider metadata for later reuse.')}
      size="full"
      surface="solid"
      contentClass="provider-template-editor-modal"
      onUpdateModelValue={handleOpenChange}
      onClose={onClose}
      footer={
        <>
          <button type="button" className="provider-template-modal__secondary" onClick={onClose}>
            {tt('取消', 'Cancel')}
          </button>
          <button
            type="button"
            className="provider-template-modal__primary"
            data-testid="provider-template-save-custom"
            onClick={onSubmit}
          >
            {tt('保存模板', 'Save template')}
          </button>
        </>
      }
    >
      <div className="provider-template-editor">
        {error ? <div className="provider-template-editor__error">{error}</div> : null}
        <div className="provider-template-editor__grid">
          <label className={fieldClass}>
            <span>{tt('名称', 'Name')}</span>
            <input className={inputClass} data-testid="provider-template-custom-name" placeholder="OpenRouter" {...register('name')} />
          </label>
          <label className={fieldClass}>
            <span>ID</span>
            <input className={inputClass} placeholder="openrouter" {...register('id')} />
          </label>
          <label className={fieldClass}>
            <span>{tt('分类', 'Category')}</span>
            <select className={inputClass} {...register('category')}>
              <option value="official">{tt('官方', 'Official')}</option>
              <option value="cn_official">{tt('国内官方', 'CN official')}</option>
              <option value="aggregator">{tt('聚合商', 'Aggregator')}</option>
              <option value="third_party">{tt('第三方', 'Third party')}</option>
              <option value="local">{tt('本地', 'Local')}</option>
            </select>
          </label>
          <label className={fieldClass}>
            <span>{tt('网站 URL', 'Website URL')}</span>
            <input className={inputClass} placeholder="https://..." {...register('websiteUrl')} />
          </label>
          <label className={fieldClass}>
            <span>{tt('API key 文档 URL', 'API key docs URL')}</span>
            <input className={inputClass} placeholder="https://..." {...register('apiKeyUrl')} />
          </label>
          <fieldset className={`${fieldClass} provider-template-editor__field--platforms`}>
            <legend>{tt('平台', 'Platforms')}</legend>
            <label className="provider-template-editor__check">
              <input type="checkbox" data-testid="provider-template-platform-claude" {...register('platformClaude')} />
              <span>Claude Code</span>
            </label>
            <label className="provider-template-editor__check">
              <input type="checkbox" data-testid="provider-template-platform-codex" {...register('platformCodex')} />
              <span>Codex</span>
            </label>
            <label className="provider-template-editor__check">
              <input type="checkbox" data-testid="provider-template-platform-opencode" {...register('platformOpencode')} />
              <span>OpenCode</span>
            </label>
          </fieldset>
        </div>
        <div className="provider-template-editor__stack">
          <label className={fieldClass}>
            <span>{tt('基础 URL', 'Base URLs')}</span>
            <textarea className={textareaClass} rows={4} placeholder={tt('每行一个 URL', 'One URL per line')} {...register('baseUrlsInput')} />
          </label>
          <label className={fieldClass}>
            <span>{tt('模型目录', 'Model catalog')}</span>
            <textarea className={textareaClass} rows={4} placeholder={tt('每行一个模型', 'One model per line')} {...register('modelCatalogInput')} />
          </label>
          <label className={fieldClass}>
            <span>{tt('别名', 'Aliases')}</span>
            <textarea className={textareaClass} rows={3} placeholder={tt('搜索别名，每行一个', 'Search aliases, one per line')} {...register('aliasesInput')} />
          </label>
          <label className={fieldClass}>
            <span>{tt('标签', 'Tags')}</span>
            <textarea className={textareaClass} rows={3} placeholder={tt('标签，每行一个', 'Tags, one per line')} {...register('tagsInput')} />
          </label>
        </div>
        {selected.length > 0 ? (
          <div className="provider-template-editor__override-list">
            {selected.map((item) => (
              <label key={item.id} className={fieldClass}>
                <span>{`${item.label} override JSON`}</span>
                <textarea
                  className={`${textareaClass} provider-template-editor__textarea--json`}
                  rows={7}
                  spellCheck={false}
                  data-testid={`provider-template-platform-override-${item.id}`}
                  placeholder="{}"
                  {...register(item.id === 'claude' ? 'overrideClaude' : item.id === 'codex' ? 'overrideCodex' : 'overrideOpencode')}
                />
              </label>
            ))}
          </div>
        ) : null}
      </div>
    </BaseModal>
  )
}
