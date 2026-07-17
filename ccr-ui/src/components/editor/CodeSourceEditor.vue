<template>
  <div class="code-source-editor">
    <div
      v-if="loading"
      class="code-source-editor__loading"
    >
      {{ t('settingsRaw.editorLoading') }}
    </div>
    <div
      ref="host"
      class="code-source-editor__host"
      :aria-label="t('settingsRaw.editorLabel')"
    />
    <p
      v-if="errorMarker"
      class="code-source-editor__error"
      role="alert"
    >
      <SIcon
        name="CircleAlert"
        size="w-4 h-4"
      />
      {{ errorMarker.message }}
    </p>
  </div>
</template>

<script setup lang="ts">
import type { Extension } from '@codemirror/state'
import type { Diagnostic } from '@codemirror/lint'
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'

export interface EditorErrorMarker {
  line: number
  column?: number
  message: string
}

const props = withDefaults(defineProps<{
  modelValue: string
  language: 'json' | 'toml' | 'markdown'
  readonly?: boolean
  errorMarker?: EditorErrorMarker | null
}>(), {
  readonly: false,
  errorMarker: null,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  save: []
}>()

const { t } = useI18n()
const host = ref<HTMLElement | null>(null)
const loading = ref(true)
let editorView: import('@codemirror/view').EditorView | null = null
let setDiagnosticsEffect: typeof import('@codemirror/lint').setDiagnostics | null = null
let scrollIntoViewEffect: typeof import('@codemirror/view').EditorView.scrollIntoView | null = null

function diagnostics(): Diagnostic[] {
  if (!editorView || !props.errorMarker) return []
  const lineNumber = Math.min(
    Math.max(props.errorMarker.line, 1),
    editorView.state.doc.lines,
  )
  const line = editorView.state.doc.line(lineNumber)
  const offset = Math.min(Math.max((props.errorMarker.column ?? 1) - 1, 0), line.length)
  const from = line.from + offset
  return [{
    from,
    to: Math.min(from + 1, line.to),
    severity: 'error',
    message: props.errorMarker.message,
  }]
}

function syncDiagnostics() {
  if (!editorView || !setDiagnosticsEffect) return
  const next = diagnostics()
  editorView.dispatch(setDiagnosticsEffect(editorView.state, next))
  if (next[0] && scrollIntoViewEffect) {
    editorView.dispatch({ effects: scrollIntoViewEffect(next[0].from, { y: 'center' }) })
  }
}

onMounted(async () => {
  if (!host.value) return

  const [
    stateModule,
    viewModule,
    commandsModule,
    languageModule,
    lintModule,
    searchModule,
    jsonModule,
    markdownModule,
    tomlModule,
  ] = await Promise.all([
    import('@codemirror/state'),
    import('@codemirror/view'),
    import('@codemirror/commands'),
    import('@codemirror/language'),
    import('@codemirror/lint'),
    import('@codemirror/search'),
    import('@codemirror/lang-json'),
    import('@codemirror/lang-markdown'),
    import('@codemirror/legacy-modes/mode/toml'),
  ])

  const languageExtension: Extension = props.language === 'json'
    ? jsonModule.json()
    : props.language === 'markdown'
      ? markdownModule.markdown()
      : languageModule.StreamLanguage.define(tomlModule.toml)

  const extensions: Extension[] = [
    viewModule.lineNumbers(),
    viewModule.highlightActiveLineGutter(),
    viewModule.highlightSpecialChars(),
    viewModule.drawSelection(),
    viewModule.EditorView.lineWrapping,
    languageModule.syntaxHighlighting(languageModule.defaultHighlightStyle),
    commandsModule.history(),
    lintModule.lintGutter(),
    languageExtension,
    stateModule.EditorState.readOnly.of(props.readonly),
    viewModule.keymap.of([
      ...commandsModule.defaultKeymap,
      ...commandsModule.historyKeymap,
      ...searchModule.searchKeymap,
      commandsModule.indentWithTab,
      {
        key: 'Mod-s',
        run: () => {
          emit('save')
          return true
        },
      },
    ]),
    viewModule.EditorView.updateListener.of((update) => {
      if (update.docChanged) emit('update:modelValue', update.state.doc.toString())
    }),
    viewModule.EditorView.theme({
      '&': {
        minHeight: '28rem',
        color: 'var(--text-primary)',
        backgroundColor: 'var(--bg-primary)',
        fontSize: '13px',
      },
      '.cm-content': {
        minHeight: '28rem',
        padding: '14px 0',
        caretColor: 'var(--accent-primary)',
        fontFamily: 'var(--font-mono)',
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
    }),
  ]

  editorView = new viewModule.EditorView({
    state: stateModule.EditorState.create({ doc: props.modelValue, extensions }),
    parent: host.value,
  })
  setDiagnosticsEffect = lintModule.setDiagnostics
  scrollIntoViewEffect = viewModule.EditorView.scrollIntoView
  loading.value = false
  syncDiagnostics()
})

watch(() => props.modelValue, (value) => {
  if (!editorView || value === editorView.state.doc.toString()) return
  editorView.dispatch({
    changes: { from: 0, to: editorView.state.doc.length, insert: value },
  })
})

watch(() => props.errorMarker, syncDiagnostics, { deep: true })

onBeforeUnmount(() => {
  editorView?.destroy()
  editorView = null
})
</script>

<style scoped>
.code-source-editor {
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: 6px;
  background: var(--bg-primary);
}

.code-source-editor__loading {
  display: grid;
  min-height: 28rem;
  place-items: center;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.code-source-editor__host:empty {
  display: none;
}

.code-source-editor__error {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  margin: 0;
  padding: 0.65rem 0.85rem;
  border-top: 1px solid color-mix(in srgb, var(--color-danger) 35%, transparent);
  color: var(--color-danger);
  background: color-mix(in srgb, var(--color-danger) 8%, var(--bg-secondary));
  font-size: 0.8125rem;
}
</style>
