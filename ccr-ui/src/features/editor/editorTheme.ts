import { EditorView } from '@codemirror/view'

/** CodeMirror 主题。字号/间距 px 属 AC11 豁免（见任务 hardcode-exemptions.md）。 */
export const codeSourceEditorTheme = EditorView.theme({
  '&': {
    minHeight: '28rem',
    color: 'var(--text-primary)',
    backgroundColor: 'var(--bg-primary)',
    fontSize: '13px',
  },
  '.cm-content': {
    minHeight: '28rem',
    padding: '14px 0',
    color: 'var(--text-primary)',
    caretColor: 'var(--accent-primary)',
    fontFamily: 'var(--font-mono)',
  },
  '.cm-line': {
    color: 'var(--text-primary)',
  },
  '.cm-gutters': {
    color: 'var(--text-muted)',
    backgroundColor: 'var(--bg-secondary)',
    borderRight: '1px solid var(--border-subtle)',
  },
  '.cm-activeLine, .cm-activeLineGutter': {
    backgroundColor: 'var(--bg-hover)',
  },
  '&.cm-focused': { outline: 'none' },
})
