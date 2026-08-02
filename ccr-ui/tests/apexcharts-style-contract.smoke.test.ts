// @vitest-environment node

import { readFile } from 'node:fs/promises'
import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'

import { parse, type Root, type Rule } from 'postcss'
import { describe, expect, it } from 'vitest'

const APEXCHARTS_CORE_PATH = fileURLToPath(
  new URL('../src/utils/apexChartsCore.ts', import.meta.url)
)
const STYLESHEET_IMPORT = 'apexcharts/dist/apexcharts.css'
const WRAPPER_IMPORT = 'vue3-apexcharts/core'
const APEXCHARTS_STYLESHEET_PATH = createRequire(import.meta.url).resolve(STYLESHEET_IMPORT)
const REQUIRED_REGISTRATIONS = [
  'apexcharts/area',
  'apexcharts/line',
  'apexcharts/bar',
  'apexcharts/donut',
  'apexcharts/heatmap',
  'apexcharts/features/legend',
] as const

const expectRuleDeclarations = (
  root: Root,
  selector: string,
  expected: Readonly<Record<string, string>>
): void => {
  const matchingRules: Rule[] = []
  root.walkRules((rule) => {
    if (rule.selector === selector) matchingRules.push(rule)
  })

  expect(
    matchingRules,
    `ApexCharts stylesheet contract drifted: expected exactly one ${selector} rule`
  ).toHaveLength(1)

  const declarations = new Map<string, string>()
  matchingRules[0]?.walkDecls((declaration) => {
    declarations.set(declaration.prop, declaration.value)
  })

  for (const [property, value] of Object.entries(expected)) {
    expect(
      declarations.get(property),
      `ApexCharts stylesheet contract drifted: ${selector} must keep ${property}: ${value}`
    ).toBe(value)
  }
}

describe('ApexCharts stylesheet contract', () => {
  it('loads the complete stylesheet once alongside every modular registration', async () => {
    const source = await readFile(APEXCHARTS_CORE_PATH, 'utf8')
    const wrapperImports = Array.from(
      source.matchAll(/^\s*import\s+(\w+)\s+from\s+['"]([^'"]+)['"]\s*;?\s*$/gm),
      (match) => ({ binding: match[1], modulePath: match[2] })
    )
    const sideEffectImports = Array.from(
      source.matchAll(/^\s*import\s+['"]([^'"]+)['"]\s*;?\s*$/gm),
      (match) => match[1]
    )

    expect(
      wrapperImports.filter(
        ({ binding, modulePath }) => binding === 'VueApexCharts' && modulePath === WRAPPER_IMPORT
      ),
      'apexChartsCore.ts must preserve the modular vue3-apexcharts/core wrapper import'
    ).toHaveLength(1)

    expect(
      sideEffectImports.filter((modulePath) => modulePath === STYLESHEET_IMPORT),
      'apexChartsCore.ts must statically import the complete ApexCharts stylesheet exactly once'
    ).toHaveLength(1)

    for (const modulePath of REQUIRED_REGISTRATIONS) {
      expect(
        sideEffectImports.filter((candidate) => candidate === modulePath),
        `apexChartsCore.ts must preserve the ${modulePath} modular registration exactly once`
      ).toHaveLength(1)
    }
  })

  it('keeps the installed tooltip layout and marker sizing rules intact', async () => {
    const stylesheet = await readFile(APEXCHARTS_STYLESHEET_PATH, 'utf8')
    const root = parse(stylesheet, { from: APEXCHARTS_STYLESHEET_PATH })

    expectRuleDeclarations(root, '.apexcharts-tooltip', {
      position: 'absolute',
    })
    expectRuleDeclarations(root, '.apexcharts-tooltip-series-group', {
      display: 'none',
    })
    expectRuleDeclarations(root, '.apexcharts-tooltip-marker', {
      width: '12px',
      height: '12px',
    })
    expectRuleDeclarations(root, '.apexcharts-tooltip-marker svg', {
      width: '100%',
      height: '100%',
    })
  })
})
