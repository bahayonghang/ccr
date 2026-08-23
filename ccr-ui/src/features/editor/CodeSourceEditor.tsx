import { useCallback, useEffect, useMemo, useRef } from 'react'
import CodeMirror, { type ReactCodeMirrorRef } from '@uiw/react-codemirror'
import { setDiagnostics, type Diagnostic } from '@codemirror/lint'
import { EditorView } from '@codemirror/view'
import { SIcon } from '@/ui'
import { buildEditorExtensions, type EditorLanguage } from './editorExtensions'
import { useEditorT } from './locale'
import './styles/code-source-editor.css'

export interface EditorErrorMarker {
  line: number
  column?: number
  message: string
}

export interface CodeSourceEditorProps {
  value: string
  language: EditorLanguage
  readOnly?: boolean
  errorMarker?: EditorErrorMarker | null
  onChange: (value: string) => void
  onSave: () => void
}

function diagnosticsOf(view: EditorView, marker: EditorErrorMarker | null): Diagnostic[] {
  if (!marker) return []
  const lineNumber = Math.min(Math.max(marker.line, 1), view.state.doc.lines)
  const line = view.state.doc.line(lineNumber)
  const offset = Math.min(Math.max((marker.column ?? 1) - 1, 0), line.length)
  const from = line.from + offset
  return [{
    from,
    to: Math.min(from + 1, line.to),
    severity: 'error',
    message: marker.message,
  }]
}

export function CodeSourceEditor({
  value,
  language,
  readOnly = false,
  errorMarker = null,
  onChange,
  onSave,
}: CodeSourceEditorProps) {
  const t = useEditorT()
  const editorRef = useRef<ReactCodeMirrorRef>(null)
  const onSaveRef = useRef(onSave)
  onSaveRef.current = onSave

  const extensions = useMemo(
    () =>
      buildEditorExtensions({
        language,
        readOnly,
        onSave: () => {
          onSaveRef.current()
        },
      }),
    [language, readOnly],
  )

  const handleChange = useCallback(
    (next: string) => {
      onChange(next)
    },
    [onChange],
  )

  useEffect(() => {
    const view = editorRef.current?.view
    if (!view) return
    const next = diagnosticsOf(view, errorMarker)
    view.dispatch(setDiagnostics(view.state, next))
    if (!next[0]) return
    view.dispatch({ effects: EditorView.scrollIntoView(next[0].from, { y: 'center' }) })
  }, [errorMarker, value])

  return (
    <div className="code-source-editor">
      <div className="code-source-editor__host" aria-label={t('settingsRaw.editorLabel')}>
        <CodeMirror
          ref={editorRef}
          value={value}
          theme="none"
          basicSetup={false}
          indentWithTab={false}
          editable={!readOnly}
          readOnly={readOnly}
          extensions={extensions}
          onChange={handleChange}
        />
      </div>
      {errorMarker ? (
        <p className="code-source-editor__error" role="alert">
          <SIcon name="CircleAlert" size="w-4 h-4" />
          {errorMarker.message}
        </p>
      ) : null}
    </div>
  )
}
