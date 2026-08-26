import { readFile, readdir } from 'node:fs/promises'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

// 父任务 AC6 / 视图门：src/**/*.{ts,tsx} 内硬编码 Npx 与 rgba()/rgb() 必须为 0，
// 仅登记豁免可保留。token 形态 rgb(var(--*-rgb) / α) 不计。
// 权威清单：本文件 EXEMPTIONS；说明见
// .trellis/tasks/08-22-react-migration/hardcode-exemptions.md。

type Kind = 'px' | 'rgb'

type Exemption = {
  file: string
  kind: Kind
  snippet: string
  reason: string
}

type Hit = {
  file: string
  kind: Kind
  text: string
  index: number
}

const SOURCE_FILE_RE = /\.(ts|tsx)$/
const PX_RE = /\d+(?:\.\d+)?px\b/g
const RGB_RE = /rgba?\(\s*(?!var\(--)/g

const CODEMIRROR = 'CodeMirror theme（EditorView.theme 运行时 stylesheet）'
const APEX = 'ApexCharts canvas（画布字号/回退色，不走 CSS rem）'
const STARTUP = 'startup fatal HTML（JS 未就绪，不能依赖 token）'
const DRAG = '6px drag（BaseModal 原拖拽阈值，常量 DRAG_THRESHOLD = 6）'
const THEME_WRITER = 'themeBootstrap rgb() writer（自定义 accent 第 1 层变量生成）'
const VIEWPORT = 'viewport breakpoint（matchMedia 用 CSS px，不随根字号缩放）'

export const EXEMPTIONS: Exemption[] = [
  { file: 'src/features/editor/editorTheme.ts', kind: 'px', snippet: "fontSize: '13px'", reason: CODEMIRROR },
  { file: 'src/features/editor/editorTheme.ts', kind: 'px', snippet: "padding: '14px 0'", reason: CODEMIRROR },
  { file: 'src/features/editor/editorTheme.ts', kind: 'px', snippet: "borderRight: '1px solid var(--border-subtle)'", reason: CODEMIRROR },

  { file: 'src/views/usage/usageChartOptions.ts', kind: 'px', snippet: "PIE_DATA_LABEL_STYLE = Object.freeze({ fontSize: '11px'", reason: APEX },
  { file: 'src/views/usage/usageChartOptions.ts', kind: 'px', snippet: "style: { colors: theme.textMuted, fontSize: '11px' }", reason: APEX },
  {
    file: 'src/views/usage/usageChartOptions.ts',
    kind: 'px',
    snippet: "fontSize: '11px',\n            color: theme.textMuted,\n            offsetY: -2",
    reason: APEX,
  },
  { file: 'src/views/usage/usageChartOptions.ts', kind: 'px', snippet: "fontSize: '15px'", reason: APEX },
  { file: 'src/views/usage/usageChartOptions.ts', kind: 'px', snippet: "fontSize: '10px'", reason: APEX },
  { file: 'src/views/usage/usageChartOptions.ts', kind: 'rgb', snippet: "'rgb(29 29 31 / 8%)'", reason: APEX },
  { file: 'src/views/usage/usageChartOptions.ts', kind: 'rgb', snippet: "'rgb(29 29 31 / 12%)'", reason: APEX },

  { file: 'src/utils/startupRecovery.ts', kind: 'px', snippet: "'padding:24px'", reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'px', snippet: "'width:min(560px,100%)'", reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'px', snippet: "'border-radius:20px'", reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'px', snippet: "'border:1px solid rgba(148,163,184,0.24)'", reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'px', snippet: "'box-shadow:0 24px", reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'px', snippet: '80px rgba(15,23,42,0.45)', reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'px', snippet: "'padding:28px'", reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'px', snippet: 'margin:0 0 12px', reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'px', snippet: 'font-size:24px', reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'px', snippet: 'font-size:14px', reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'rgb', snippet: 'rgba(29,78,216,0.18)', reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'rgb', snippet: 'rgba(148,163,184,0.24)', reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'rgb', snippet: 'rgba(15,23,42,0.92)', reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'rgb', snippet: 'rgba(15,23,42,0.45)', reason: STARTUP },
  { file: 'src/utils/startupRecovery.ts', kind: 'rgb', snippet: 'rgba(226,232,240,0.82)', reason: STARTUP },

  { file: 'src/ui/base-modal.tsx', kind: 'px', snippet: '遮罩点击关闭的 6px 拖拽阈值', reason: DRAG },
  { file: 'src/ui/base-modal.tsx', kind: 'px', snippet: '受 6px 拖拽阈值约束', reason: DRAG },
  { file: 'src/ui/base-modal.tsx', kind: 'px', snippet: '≤6px 视为点击', reason: DRAG },

  {
    file: 'src/utils/themeBootstrap.ts',
    kind: 'rgb',
    snippet: '--color-accent-primary-glow: rgb(${toTriplet(primary)} / ${alpha.glow});',
    reason: THEME_WRITER,
  },
  {
    file: 'src/utils/themeBootstrap.ts',
    kind: 'rgb',
    snippet: '--color-border-accent: rgb(${toTriplet(primary)} / ${alpha.border});',
    reason: THEME_WRITER,
  },

  {
    file: 'src/shell/hooks/useMainLayoutShell.ts',
    kind: 'px',
    snippet: "window.matchMedia('(max-width: 1023px)')",
    reason: VIEWPORT,
  },
]

const normalizePath = (value: string) => value.replace(/\\/g, '/')

const collectSourceFiles = async (root: string): Promise<string[]> => {
  const entries = await readdir(root, { withFileTypes: true })
  const files = await Promise.all(
    entries.map(async (entry) => {
      const fullPath = path.join(root, entry.name)
      if (entry.isDirectory()) return collectSourceFiles(fullPath)
      return entry.isFile() && SOURCE_FILE_RE.test(entry.name) ? [fullPath] : []
    }),
  )
  return files.flat()
}

const collectHits = (file: string, source: string): Hit[] => {
  const hits: Hit[] = []
  for (const match of source.matchAll(PX_RE)) {
    hits.push({ file, kind: 'px', text: match[0], index: match.index ?? 0 })
  }
  for (const match of source.matchAll(RGB_RE)) {
    hits.push({ file, kind: 'rgb', text: match[0], index: match.index ?? 0 })
  }
  return hits
}

describe('AC6 hardcoded px / rgb gate', () => {
  it('remaining Npx and rgb()/rgba() equal the exemption list', async () => {
    const sourceRoot = path.resolve('src')
    const files = await collectSourceFiles(sourceRoot)
    expect(files.length).toBeGreaterThan(0)

    const sources = new Map<string, string>()
    const hits: Hit[] = []
    for (const absolute of files) {
      const file = normalizePath(path.relative(process.cwd(), absolute))
      const source = (await readFile(absolute, 'utf8')).replace(/\r\n/g, '\n')
      sources.set(file, source)
      hits.push(...collectHits(file, source))
    }

    const claimed = new Set<number>()
    const problems: string[] = []

    for (const exemption of EXEMPTIONS) {
      const source = sources.get(exemption.file)
      if (source === undefined) {
        problems.push(`missing file ${exemption.file} (${exemption.reason})`)
        continue
      }
      const pos = source.indexOf(exemption.snippet)
      if (pos < 0) {
        problems.push(`snippet not found in ${exemption.file}: ${JSON.stringify(exemption.snippet)}`)
        continue
      }
      const end = pos + exemption.snippet.length
      const matched = hits
        .map((hit, index) => ({ hit, index }))
        .filter(
          ({ hit, index }) =>
            !claimed.has(index) &&
            hit.file === exemption.file &&
            hit.kind === exemption.kind &&
            hit.index >= pos &&
            hit.index < end,
        )
      if (matched.length !== 1) {
        problems.push(
          `${exemption.file} kind=${exemption.kind} snippet=${JSON.stringify(exemption.snippet)} matched ${matched.length} (want 1)`,
        )
        continue
      }
      claimed.add(matched[0].index)
    }

    const leftover = hits.filter((_, index) => !claimed.has(index))
    if (leftover.length > 0) {
      problems.push(
        `unregistered hits: ${leftover.map((hit) => `${hit.file} ${hit.kind}:${hit.text}@${hit.index}`).join('; ')}`,
      )
    }

    expect(problems).toEqual([])
    expect(hits).toHaveLength(EXEMPTIONS.length)
  })
})
