import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import { json } from '@codemirror/lang-json'
import { markdown } from '@codemirror/lang-markdown'
import { defaultHighlightStyle, StreamLanguage, syntaxHighlighting } from '@codemirror/language'
import { toml } from '@codemirror/legacy-modes/mode/toml'
import { lintGutter } from '@codemirror/lint'
import { search, searchKeymap } from '@codemirror/search'
import { EditorState, type Extension } from '@codemirror/state'
import {
  drawSelection,
  EditorView,
  highlightActiveLineGutter,
  highlightSpecialChars,
  keymap,
  lineNumbers,
} from '@codemirror/view'
import { readPageCspNonce } from './cspNonce'
import { codeSourceEditorTheme } from './editorTheme'

export type EditorLanguage = 'json' | 'toml' | 'markdown'

export function languageExtensionOf(language: EditorLanguage): Extension {
  if (language === 'json') return json()
  if (language === 'markdown') return markdown()
  return StreamLanguage.define(toml)
}

export function buildEditorExtensions(input: {
  language: EditorLanguage
  readOnly: boolean
  onSave: () => void
}): Extension[] {
  const nonce = readPageCspNonce()
  return [
    ...(nonce ? [EditorView.cspNonce.of(nonce)] : []),
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    drawSelection(),
    EditorView.lineWrapping,
    syntaxHighlighting(defaultHighlightStyle),
    history(),
    search(),
    lintGutter(),
    languageExtensionOf(input.language),
    EditorState.readOnly.of(input.readOnly),
    keymap.of([
      ...defaultKeymap,
      ...historyKeymap,
      ...searchKeymap,
      indentWithTab,
      {
        key: 'Mod-s',
        run: () => {
          input.onSave()
          return true
        },
      },
    ]),
    codeSourceEditorTheme,
  ]
}
