import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactNode,
} from 'react'
import { useBlocker } from 'react-router'
import type { RawFileGetResult, RawProfilesSaveResult } from '@/api/domains/configRawTypes'
import { useShellT } from '@/shell/i18n'
import { useUIStore } from '@/shell/stores/ui'
import { BaseModal, SIcon } from '@/ui'
import './profiles-shared.css'

export interface EditorErrorMarker {
  line: number
  column?: number
  message: string
}

export interface ProfilesRawEditorRenderProps {
  value: string
  onChange: (value: string) => void
  errorMarker: EditorErrorMarker | null
  onSave: () => void
}

export interface ProfilesRawEditorPanelProps {
  getRaw: () => Promise<RawFileGetResult>
  saveRaw: (content: string, token: string, force?: boolean) => Promise<RawProfilesSaveResult>
  onSaved: () => void
  onClose: () => void
  onDirtyChange?: (dirty: boolean) => void
  /**
   * 原始 TOML 编辑器。CodeMirror 桥接属 08-22-views-sync-tools；
   * 未传入时用 textarea 占位，语义仍为受控内容 + Ctrl/Cmd+S 保存。
   */
  renderEditor?: (props: ProfilesRawEditorRenderProps) => ReactNode
}

const isSaved = (
  result: RawProfilesSaveResult,
): result is Extract<RawProfilesSaveResult, { status: 'saved' }> => result.status === 'saved'

const isInvalid = (
  result: RawProfilesSaveResult,
): result is Extract<RawProfilesSaveResult, { status: 'invalid' }> => result.status === 'invalid'

function FallbackTomlEditor({ value, onChange, errorMarker, onSave }: ProfilesRawEditorRenderProps) {
  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if ((event.metaKey || event.ctrlKey) && event.key === 's') {
      event.preventDefault()
      onSave()
    }
  }
  return (
    <>
      <textarea
        className="profiles-raw-editor__fallback"
        data-testid="profiles-raw-editor"
        value={value}
        spellCheck={false}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={onKeyDown}
      />
      {errorMarker ? (
        <p className="profiles-raw-editor__error" role="alert">
          {errorMarker.message}
        </p>
      ) : null}
    </>
  )
}

/** Profiles 原始 TOML 全屏编辑面板。写入仍走调用方注入的 getRaw/saveRaw。 */
export function ProfilesRawEditorPanel({
  getRaw,
  saveRaw,
  onSaved,
  onClose,
  onDirtyChange,
  renderEditor,
}: ProfilesRawEditorPanelProps) {
  const t = useShellT()
  const requestConfirm = useUIStore((state) => state.requestConfirm)
  const showError = useUIStore((state) => state.showError)
  const showSuccess = useUIStore((state) => state.showSuccess)

  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [content, setContent] = useState('')
  const [baseline, setBaseline] = useState('')
  const [token, setToken] = useState('')
  const [filePath, setFilePath] = useState('')
  const [conflict, setConflict] = useState(false)
  const [unsupportedEnvironment, setUnsupportedEnvironment] = useState(false)
  const [errorMarker, setErrorMarker] = useState<EditorErrorMarker | null>(null)

  const dirty = content !== baseline
  const blocker = useBlocker(dirty)

  const tRef = useRef(t)
  tRef.current = t
  const showErrorRef = useRef(showError)
  showErrorRef.current = showError

  const confirmDiscard = useCallback(async () => {
    if (!dirty) return true
    return requestConfirm({
      title: t('profilesRaw.discardTitle'),
      message: t('profilesRaw.discardMessage'),
      confirmText: t('profilesRaw.discard'),
      cancelText: t('common.cancel'),
      type: 'warning',
      surface: 'solid',
    })
  }, [dirty, requestConfirm, t])

  const load = useCallback(async () => {
    setLoading(true)
    setConflict(false)
    setErrorMarker(null)
    try {
      const result = await getRaw()
      if (result.status === 'unsupported_environment') {
        setUnsupportedEnvironment(true)
        return
      }
      setUnsupportedEnvironment(false)
      setContent(result.content)
      setBaseline(result.content)
      setToken(result.token)
      setFilePath(result.path)
    } catch (error) {
      showErrorRef.current(`${tRef.current('profilesRaw.loadFailed')}: ${String(error)}`)
    } finally {
      setLoading(false)
    }
  }, [getRaw])

  useEffect(() => {
    void load()
  }, [load])

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

  const applyTerminalResult = useCallback(
    (result: RawProfilesSaveResult) => {
      if (isSaved(result)) {
        setToken(result.token)
        setBaseline(content)
        showSuccess(t('profilesRaw.saveSuccess', { count: result.profiles_count }))
        onSaved()
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
      if (result.status === 'unsupported_environment') {
        setUnsupportedEnvironment(true)
      }
    },
    [content, onSaved, showSuccess, t],
  )

  const confirmActivationRetry = useCallback(
    async (current: string) => {
      setSaving(false)
      return requestConfirm({
        title: t('profilesRaw.activationTitle'),
        message: t('profilesRaw.activationMessage', { name: current }),
        confirmText: t('profilesRaw.activationConfirm'),
        cancelText: t('common.cancel'),
        type: 'danger',
        surface: 'solid',
      })
    },
    [requestConfirm, t],
  )

  const resolveSaveResult = useCallback(
    async (result: RawProfilesSaveResult): Promise<RawProfilesSaveResult | null> => {
      if (result.status !== 'activation_conflict') return result
      const confirmed = await confirmActivationRetry(result.current)
      if (!confirmed) return null
      setSaving(true)
      return saveRaw(content, token, true)
    },
    [confirmActivationRetry, content, saveRaw, token],
  )

  const save = useCallback(async () => {
    if (!dirty || saving) return
    setSaving(true)
    setConflict(false)
    setErrorMarker(null)
    try {
      const first = await saveRaw(content, token, false)
      const next = await resolveSaveResult(first)
      if (next) applyTerminalResult(next)
    } catch (error) {
      showError(`${t('profilesRaw.saveFailed')}: ${String(error)}`)
    } finally {
      setSaving(false)
    }
  }, [applyTerminalResult, content, dirty, resolveSaveResult, saveRaw, saving, showError, t, token])

  const reload = async () => {
    if (!(await confirmDiscard())) return
    await load()
  }

  const close = async () => {
    if (await confirmDiscard()) onClose()
  }

  const Editor = renderEditor ?? FallbackTomlEditor

  return (
    <BaseModal
      modelValue
      persistent
      showClose={false}
      size="full"
      scrollable
      surface="solid"
      contentClass="profiles-raw-editor-modal !rounded-md"
      header={({ titleId }) => (
        <div className="profiles-raw-editor__header">
          <div className="profiles-raw-editor__heading">
            <SIcon name="FileCode2" size="w-5 h-5" />
            <div>
              <h2 id={titleId}>{t('profilesRaw.title')}</h2>
              <code>{filePath || t('settingsRaw.pathPending')}</code>
            </div>
          </div>
          <div className="profiles-raw-editor__actions">
            {dirty ? <span className="profiles-raw-editor__dirty">{t('profilesRaw.unsaved')}</span> : null}
            <button
              type="button"
              className="profiles-raw-editor__button"
              disabled={loading || saving}
              onClick={() => {
                void reload()
              }}
            >
              <SIcon name="RefreshCw" size="w-4 h-4" />
              {t('profilesRaw.reload')}
            </button>
            <button
              type="button"
              className="profiles-raw-editor__button profiles-raw-editor__button--primary"
              disabled={loading || saving || !dirty}
              onClick={() => {
                void save()
              }}
            >
              <SIcon name="Save" size="w-4 h-4" />
              {saving ? t('profilesRaw.saving') : t('profilesRaw.save')}
            </button>
            <button
              type="button"
              className="profiles-raw-editor__icon-button"
              aria-label={t('profilesRaw.close')}
              title={t('profilesRaw.close')}
              onClick={() => {
                void close()
              }}
            >
              <SIcon name="X" size="w-5 h-5" />
            </button>
          </div>
        </div>
      )}
    >
      <div className="profiles-raw-editor__notice" role="note">
        <SIcon name="ShieldAlert" size="w-4 h-4" />
        <span>{t('profilesRaw.plaintextNotice')}</span>
      </div>

      {conflict ? (
        <div className="profiles-raw-editor__message" role="alert">
          <div>
            <strong>{t('profilesRaw.conflictTitle')}</strong>
            <p>{t('profilesRaw.conflictMessage')}</p>
          </div>
          <button
            type="button"
            onClick={() => {
              void reload()
            }}
          >
            {t('profilesRaw.reload')}
          </button>
        </div>
      ) : null}

      {unsupportedEnvironment ? (
        <div className="profiles-raw-editor__message" role="status">
          {t('settingsRaw.unsupportedEnvironment')}
        </div>
      ) : null}

      {loading ? (
        <div className="profiles-raw-editor__loading">{t('profilesRaw.loading')}</div>
      ) : null}
      {!loading && !unsupportedEnvironment ? (
        <Editor
          value={content}
          onChange={setContent}
          errorMarker={errorMarker}
          onSave={() => {
            void save()
          }}
        />
      ) : null}
    </BaseModal>
  )
}
