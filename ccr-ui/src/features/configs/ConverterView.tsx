import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useForm } from 'react-hook-form'
import { Link } from 'react-router'
import { convertConfig } from '@/api'
import { copyText } from '@/utils/clipboard'
import type { CliType, ConverterResponse } from '@/types'
import { PageHeader, PageShell, SIcon } from '@/ui'
import { t } from './locale'
import { ConfigsSubnav } from './components/ConfigsSubnav'
import { ConverterResult } from './components/converter/ConverterResult'
import { FormatOption } from './components/converter/FormatOption'
import {
  CLI_DEFINITIONS,
  CONVERTER_EXAMPLE,
  cliLabelOf,
  emptyConverterForm,
  resultExtension,
  toConverterRequest,
  type ConverterFormValues,
} from './lib/converterModel'
import './styles/converter.css'

export function ConverterView() {
  const form = useForm<ConverterFormValues>({ defaultValues: emptyConverterForm() })
  const { register, getValues, setValue, watch } = form
  const sourceFormat = watch('sourceFormat')
  const targetFormat = watch('targetFormat')
  const [isConverting, setIsConverting] = useState(false)
  const [result, setResult] = useState<ConverterResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  const successTimer = useRef<ReturnType<typeof setTimeout> | null>(null)

  const flashSuccess = useCallback((message: string, duration = 2000) => {
    setSuccessMessage(message)
    if (successTimer.current) clearTimeout(successTimer.current)
    successTimer.current = setTimeout(() => setSuccessMessage(null), duration)
  }, [])

  useEffect(
    () => () => {
      if (successTimer.current) clearTimeout(successTimer.current)
    },
    [],
  )

  const cliTypes = useMemo(
    () => CLI_DEFINITIONS.map((type) => ({ ...type, description: t(type.descriptionKey) })),
    [],
  )

  const selectSource = useCallback(
    (value: CliType) => {
      setValue('sourceFormat', value)
    },
    [setValue],
  )
  const selectTarget = useCallback(
    (value: CliType) => {
      setValue('targetFormat', value)
    },
    [setValue],
  )

  const handleFileUpload = useCallback(
    (event: { target: EventTarget | null }) => {
      const file = (event.target as HTMLInputElement).files?.[0]
      if (!file) return
      const reader = new FileReader()
      reader.onload = () => {
        setValue('configData', String(reader.result ?? ''))
        flashSuccess(t('converter.fileLoaded', { name: file.name }), 3000)
      }
      reader.onerror = () => setError(t('converter.fileLoadFailed'))
      reader.readAsText(file)
    },
    [flashSuccess, setValue],
  )

  const handleConvert = useCallback(async () => {
    const values = getValues()
    setError(null)
    setSuccessMessage(null)
    setResult(null)
    if (!values.configData.trim()) {
      setError(t('converter.inputRequired'))
      return
    }
    if (values.sourceFormat === values.targetFormat) {
      setError(t('converter.sameFormatError'))
      return
    }
    setIsConverting(true)
    try {
      const response = await convertConfig(toConverterRequest(values))
      setResult(response)
      setSuccessMessage(t('converter.convertSuccess'))
    } catch (err) {
      setError(err instanceof Error ? err.message : t('converter.convertError'))
    } finally {
      setIsConverting(false)
    }
  }, [getValues])

  const handleCopy = useCallback(() => {
    if (!result?.content) return
    void copyText(result.content)
    flashSuccess(t('converter.copied'))
  }, [flashSuccess, result])

  const handleDownload = useCallback(() => {
    if (!result?.content) return
    const blob = new Blob([result.content], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = `${cliLabelOf(sourceFormat)}-to-${cliLabelOf(targetFormat)}.${resultExtension(result.format)}`
    document.body.appendChild(anchor)
    anchor.click()
    document.body.removeChild(anchor)
    URL.revokeObjectURL(url)
    flashSuccess(t('converter.fileDownloaded'))
  }, [flashSuccess, result, sourceFormat, targetFormat])

  const handleLoadExample = useCallback(() => {
    setValue('configData', CONVERTER_EXAMPLE)
    setValue('sourceFormat', 'claude-code')
    flashSuccess(t('converter.exampleLoaded'), 3000)
  }, [flashSuccess, setValue])

  return (
    <PageShell
      className="converter-view"
      header={
        <PageHeader
          title={t('converter.title')}
          description={t('converter.description')}
          actions={
            <Link to="/" className="converter-chip-button">
              <SIcon name="Home" size="w-4 h-4" />
              <span>{t('converter.backToHome')}</span>
            </Link>
          }
        />
      }
      subnav={<ConfigsSubnav module="converter" />}
    >
      {error ? (
        <div className="converter-alert converter-alert--error">
          <SIcon name="AlertCircle" size="w-5 h-5" className="converter-alert__icon" />
          <div>{error}</div>
        </div>
      ) : null}
      {successMessage ? (
        <div className="converter-alert converter-alert--success">
          <SIcon name="Check" size="w-5 h-5" className="converter-alert__icon" />
          <div>{successMessage}</div>
        </div>
      ) : null}
      <div className="converter-selection-grid">
        <div className="converter-card">
          <div className="converter-section-heading">
            <SIcon name="FileJson" size="w-5 h-5" className="converter-section-heading__icon" />
            <h2 className="converter-card__title">{t('converter.sourceFormat')}</h2>
          </div>
          <p className="converter-section-copy">{t('converter.selectSource')}</p>
          <div className="converter-option-list">
            {cliTypes.map((type) => (
              <FormatOption
                key={`source-${type.value}`}
                value={type.value}
                label={type.label}
                description={type.description}
                active={sourceFormat === type.value}
                onSelect={selectSource}
              />
            ))}
          </div>
        </div>
        <div className="converter-card">
          <div className="converter-section-heading">
            <SIcon name="FileCode" size="w-5 h-5" className="converter-section-heading__icon" />
            <h2 className="converter-card__title">{t('converter.targetFormat')}</h2>
          </div>
          <p className="converter-section-copy">{t('converter.selectTarget')}</p>
          <div className="converter-option-list">
            {cliTypes.map((type) => (
              <FormatOption
                key={`target-${type.value}`}
                value={type.value}
                label={type.label}
                description={type.description}
                active={targetFormat === type.value && sourceFormat !== type.value}
                disabled={sourceFormat === type.value}
                onSelect={selectTarget}
              />
            ))}
          </div>
        </div>
      </div>
      <div className="converter-card">
        <h2 className="converter-card__title converter-card__title--with-gap">{t('converter.convertOptions')}</h2>
        <p className="converter-section-copy">{t('converter.convertOptionsDesc')}</p>
        <div className="converter-toggle-list">
          <label className="converter-toggle">
            <input type="checkbox" className="converter-checkbox" {...register('convertMcp')} />
            <span>{t('converter.mcpServers')}</span>
          </label>
          <label className="converter-toggle">
            <input type="checkbox" className="converter-checkbox" {...register('convertCommands')} />
            <span>{t('converter.slashCommands')}</span>
          </label>
          <label className="converter-toggle">
            <input type="checkbox" className="converter-checkbox" {...register('convertAgents')} />
            <span>{t('converter.agentsConfig')}</span>
          </label>
        </div>
      </div>
      <div className="converter-card">
        <div className="converter-toolbar">
          <div>
            <h2 className="converter-card__title converter-card__title--compact">{t('converter.configInput')}</h2>
            <p className="converter-section-copy converter-section-copy--compact">{t('converter.configInputDesc')}</p>
          </div>
          <div className="converter-toolbar__actions">
            <button type="button" className="converter-toolbar-button" onClick={handleLoadExample}>
              {t('converter.loadExample')}
            </button>
            <label>
              <span className="converter-toolbar-button converter-toolbar-button--label">
                <SIcon name="Upload" size="w-4 h-4" />
                {t('converter.uploadFile')}
              </span>
              <input type="file" accept=".json,.toml,.yaml,.yml,.txt" className="hidden" onChange={handleFileUpload} />
            </label>
          </div>
        </div>
        <textarea className="converter-textarea" placeholder={t('converter.inputPlaceholder')} {...register('configData')} />
        <div className="converter-help-text">{t('converter.supportedFormats')}</div>
      </div>
      <div className="converter-action-row">
        <button
          type="button"
          className="converter-primary-action"
          disabled={isConverting}
          onClick={handleConvert}
        >
          <SIcon name={isConverting ? 'Loader2' : 'ArrowRight'} size="w-5 h-5" className={isConverting ? 'animate-spin' : ''} />
          {isConverting ? t('converter.converting') : t('converter.startConvert')}
        </button>
      </div>
      {result ? <ConverterResult result={result} onCopy={handleCopy} onDownload={handleDownload} /> : null}
      <div className="converter-card">
        <h2 className="converter-card__title converter-card__title--section">{t('converter.usageGuide')}</h2>
        <div className="converter-guide">
          <div>
            <h4 className="converter-guide__title">{t('converter.usageNotes.supportedPathsTitle')}</h4>
            <ul className="converter-guide__list">
              <li>{t('converter.usageNotes.claudeCodex')}</li>
              <li>{t('converter.usageNotes.otherFormats')}</li>
            </ul>
          </div>
          <div>
            <h4 className="converter-guide__title">{t('converter.usageNotes.conversionNotesTitle')}</h4>
            <ul className="converter-guide__list">
              <li>{t('converter.usageNotes.note1')}</li>
              <li>{t('converter.usageNotes.note2')}</li>
              <li>{t('converter.usageNotes.note3')}</li>
              <li>{t('converter.usageNotes.note4')}</li>
            </ul>
          </div>
          <div>
            <h4 className="converter-guide__title">{t('converter.usageNotes.importantNotesTitle')}</h4>
            <ul className="converter-guide__list">
              <li>{t('converter.usageNotes.caution1')}</li>
              <li>{t('converter.usageNotes.caution2')}</li>
              <li>{t('converter.usageNotes.caution3')}</li>
            </ul>
          </div>
        </div>
      </div>
    </PageShell>
  )
}
