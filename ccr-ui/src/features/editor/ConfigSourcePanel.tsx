import { useCallback, useEffect, useRef, useState } from 'react'
import { useBlocker } from 'react-router'
import type {
  ConfigLayer,
  ConfigLayersResult,
  RawFileGetResult,
  RawFileSaveResult,
} from '@/api/domains/configRawTypes'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { SIcon } from '@/ui'
import { CodeSourceEditor, type EditorErrorMarker } from './CodeSourceEditor'
import { useEditorT } from './locale'
import './styles/config-source-panel.css'

export interface ConfigSourcePanelProps {
  language: 'json' | 'toml'
  getRaw: () => Promise<RawFileGetResult>
  saveRaw: (content: string, token: string) => Promise<RawFileSaveResult>
  listLayers: () => Promise<ConfigLayersResult>
  backupNotice?: string
  policyNotice?: string
  policyLayerIds?: string[]
  onSaved?: () => void
  onClose?: () => void
  onDirtyChange?: (dirty: boolean) => void
}

const isSaved = (
  result: RawFileSaveResult,
): result is Extract<RawFileSaveResult, { status: 'saved' }> => result.status === 'saved'

const isInvalid = (
  result: RawFileSaveResult,
): result is Extract<RawFileSaveResult, { status: 'invalid' }> => result.status === 'invalid'

export function ConfigSourcePanel({
  language,
  getRaw,
  saveRaw,
  listLayers,
  backupNotice,
  policyNotice,
  policyLayerIds = [],
  onSaved,
  onClose,
  onDirtyChange,
}: ConfigSourcePanelProps) {
  const t = useEditorT()
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [content, setContent] = useState('')
  const [baseline, setBaseline] = useState('')
  const [token, setToken] = useState('')
  const [filePath, setFilePath] = useState('')
  const [layers, setLayers] = useState<ConfigLayer[]>([])
  const [conflict, setConflict] = useState(false)
  const [unsupportedEnvironment, setUnsupportedEnvironment] = useState(false)
  const [errorMarker, setErrorMarker] = useState<EditorErrorMarker | null>(null)
  const [ready, setReady] = useState(false)

  const dirty = content !== baseline
  const blocker = useBlocker(dirty)
  const hasPolicyLayer = layers.some((layer) => layer.exists === true && policyLayerIds.includes(layer.id))

  const tRef = useRef(t)
  tRef.current = t

  const confirmDiscard = useCallback(async () => {
    if (!dirty) return true
    return surfaceNotify.confirm({
      title: t('settingsRaw.discardTitle'),
      message: t('settingsRaw.discardMessage'),
      confirmText: t('settingsRaw.discard'),
      cancelText: t('common.cancel'),
      type: 'warning',
      surface: 'solid',
    })
  }, [dirty, t])

  const load = useCallback(async () => {
    setLoading(true)
    setConflict(false)
    setErrorMarker(null)
    try {
      const [raw, layerResult] = await Promise.all([getRaw(), listLayers()])
      if (raw.status === 'unsupported_environment') {
        setUnsupportedEnvironment(true)
        return
      }
      setUnsupportedEnvironment(false)
      setContent(raw.content)
      setBaseline(raw.content)
      setToken(raw.token)
      setFilePath(raw.path)
      setLayers('layers' in layerResult ? layerResult.layers : [])
    } catch (error) {
      surfaceNotify.error(`${tRef.current('settingsRaw.loadFailed')}: ${String(error)}`)
    } finally {
      setLoading(false)
    }
  }, [getRaw, listLayers])

  useEffect(() => {
    let cancelled = false
    void surfaceNotify
      .confirm({
        title: tRef.current('settingsRaw.warningTitle'),
        message: tRef.current('settingsRaw.warningMessage'),
        confirmText: tRef.current('settingsRaw.continue'),
        cancelText: tRef.current('common.cancel'),
        type: 'warning',
        surface: 'solid',
      })
      .then((confirmed) => {
        if (cancelled) return
        if (!confirmed) {
          onClose?.()
          return
        }
        setReady(true)
      })
    return () => {
      cancelled = true
    }
  }, [onClose])

  useEffect(() => {
    if (!ready) return
    void load()
  }, [load, ready])

  useEffect(() => {
    onDirtyChange?.(dirty)
  }, [dirty, onDirtyChange])

  useEffect(() => {
    return () => onDirtyChange?.(false)
  }, [onDirtyChange])

  useEffect(() => {
    if (blocker.state !== 'blocked') return
    let cancelled = false
    void confirmDiscard().then((ok) => {
      if (cancelled) return
      if (ok) blocker.proceed()
      else blocker.reset()
    })
    return () => {
      cancelled = true
    }
  }, [blocker, confirmDiscard])

  const save = useCallback(async () => {
    if (!dirty || saving) return
    setSaving(true)
    setConflict(false)
    setErrorMarker(null)
    try {
      const result = await saveRaw(content, token)
      if (isSaved(result)) {
        setToken(result.token)
        setBaseline(content)
        surfaceNotify.success(t('settingsRaw.saveSuccess'))
        onSaved?.()
        return
      }
      if (result.status === 'conflict') {
        setConflict(true)
        return
      }
      if (isInvalid(result)) {
        setErrorMarker({
          line: result.line ?? 1,
          column: result.column,
          message: result.message,
        })
        return
      }
      setUnsupportedEnvironment(true)
    } catch (error) {
      surfaceNotify.error(`${t('settingsRaw.saveFailed')}: ${String(error)}`)
    } finally {
      setSaving(false)
    }
  }, [content, dirty, onSaved, saveRaw, saving, t, token])

  const reload = useCallback(async () => {
    if (!(await confirmDiscard())) return
    await load()
  }, [confirmDiscard, load])

  const handleReload = useCallback(() => {
    void reload()
  }, [reload])
  const handleSave = useCallback(() => {
    void save()
  }, [save])

  return (
    <section className="config-source-panel">
      <header className="config-source-panel__header">
        <div className="config-source-panel__path">
          <span>{t('settingsRaw.filePath')}</span>
          <code>{filePath || t('settingsRaw.pathPending')}</code>
        </div>
        <div className="config-source-panel__actions">
          {dirty ? <span className="config-source-panel__dirty">{t('settingsRaw.unsaved')}</span> : null}
          <button type="button" className="config-source-panel__button" disabled={loading || saving} onClick={handleReload}>
            <SIcon name="RefreshCw" size="w-4 h-4" />
            {t('settingsRaw.reload')}
          </button>
          <button
            type="button"
            className="config-source-panel__button config-source-panel__button--primary"
            disabled={loading || saving || !dirty}
            onClick={handleSave}
          >
            <SIcon name="Save" size="w-4 h-4" />
            {saving ? t('settingsRaw.saving') : t('settingsRaw.save')}
          </button>
        </div>
      </header>

      <div className="config-source-panel__notice" role="note">
        <SIcon name="ShieldAlert" size="w-4 h-4" />
        <span>{t('settingsRaw.plaintextNotice')}</span>
      </div>

      {backupNotice ? (
        <div className="config-source-panel__notice config-source-panel__notice--warning" data-testid="config-source-backup-notice" role="note">
          <SIcon name="ShieldAlert" size="w-4 h-4" />
          <span>{backupNotice}</span>
        </div>
      ) : null}

      {conflict ? (
        <div className="config-source-panel__message config-source-panel__message--warning" role="alert">
          <div>
            <strong>{t('settingsRaw.conflictTitle')}</strong>
            <p>{t('settingsRaw.conflictMessage')}</p>
          </div>
          <button type="button" className="config-source-panel__text-button" onClick={handleReload}>
            {t('settingsRaw.reload')}
          </button>
        </div>
      ) : null}

      {unsupportedEnvironment ? (
        <div className="config-source-panel__message config-source-panel__message--warning" role="status">
          {t('settingsRaw.unsupportedEnvironment')}
        </div>
      ) : null}

      {loading ? <div className="config-source-panel__loading">{t('settingsRaw.loading')}</div> : null}
      {!loading && !unsupportedEnvironment ? (
        <CodeSourceEditor
          value={content}
          language={language}
          errorMarker={errorMarker}
          onChange={setContent}
          onSave={handleSave}
        />
      ) : null}

      <section className="config-source-panel__layers" aria-labelledby="config-layers-title">
        <div className="config-source-panel__layers-heading">
          <div>
            <h3 id="config-layers-title">{t('settingsRaw.layersTitle')}</h3>
            <p>{t('settingsRaw.layersDescription')}</p>
          </div>
          <span>{layers.length}</span>
        </div>
        {hasPolicyLayer && policyNotice ? (
          <div className="config-source-panel__message config-source-panel__message--warning" data-testid="config-source-policy-notice" role="status">
            {policyNotice}
          </div>
        ) : null}
        <div className="config-source-panel__layer-list">
          {layers.map((layer) => (
            <div key={`${layer.id}-${layer.path ?? layer.label}`} className="config-source-panel__layer">
              <SIcon name={layer.exists ? 'FileCheck2' : 'FileQuestion'} size="w-4 h-4" />
              <div>
                <strong>{layer.label}</strong>
                {layer.path ? <code>{layer.path}</code> : <span>{t('settingsRaw.projectContextRequired')}</span>}
              </div>
              <span className="config-source-panel__layer-state">
                {layer.editable ? t('settingsRaw.editable') : t('settingsRaw.readOnly')}
              </span>
            </div>
          ))}
        </div>
      </section>
    </section>
  )
}
