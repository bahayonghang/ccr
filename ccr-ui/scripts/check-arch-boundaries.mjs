#!/usr/bin/env node
/* eslint-disable no-console */
// 架构边界规则自检（08-22-arch-quality-perf 批次 2，AC2）
// 对 tests/fixtures/arch-violations 下的违规夹具做定向 lint（--no-ignore 绕过常规排除），
// 断言每个夹具被预期规则报错。规则单源：复用 eslint.config.js 具名导出的
// boundaryElements / boundaryPolicies，仅追加夹具目录的元素映射。
// 定义面冻结用例不在本脚本内：tauri.ts 新增 invoke() 由
// tests/api-facade-boundary.smoke.test.ts 的冻结用例拦截（见 layering-contracts.md）。
import fs from 'node:fs'
import path from 'node:path'
import { execFileSync } from 'node:child_process'

const root = process.cwd()
const tmpConfig = path.join(root, '.eslint.arch-selfcheck.mjs')

fs.writeFileSync(
  tmpConfig,
  `import boundaries from 'eslint-plugin-boundaries'
import { boundaryElements, boundaryPolicies } from './eslint.config.js'

export default [
  {
    name: 'arch-selfcheck/fixtures',
    files: ['tests/fixtures/arch-violations/**/*.ts'],
    plugins: { boundaries },
    settings: {
      // mode:'file' 元素描述符依赖向后兼容，抑制弃用告警（stderr 噪音）
      'boundaries/legacy-warnings': false,
      'boundaries/elements': [
        ...boundaryElements,
        // 夹具目录映射到与 src 同名的元素类型，使违规路径可被分类
        { type: 'ui-primitive', pattern: 'tests/fixtures/arch-violations/ui' },
        { type: 'feature', pattern: 'tests/fixtures/arch-violations/features/(*)', capture: ['domain'] },
        { type: 'store', pattern: 'tests/fixtures/arch-violations/store' },
        // 夹具根 reverse-dep.ts 单文件映射为 utils 元素（mode:'file'），触发 utils → store 反向依赖
        { type: 'utils', pattern: 'tests/fixtures/arch-violations/reverse-dep.ts', mode: 'file' },
      ],
      // 模块解析：extensionless 相对导入与目录索引依赖 TS resolver（与 eslint.config.js 一致）
      'import/resolver': {
        typescript: {
          alwaysTryTypes: true,
          project: './tsconfig.json',
        },
      },
    },
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['**/api/tauri', '**/api/tauri.*'],
              message: '冻结门面：禁止直接导入 src/api/tauri.ts',
            },
          ],
        },
      ],
      'boundaries/dependencies': ['error', { default: 'disallow', policies: boundaryPolicies }],
    },
  },
]
`,
)

try {
  const expectations = [
    {
      file: 'ui/FixturePrimitive.ts',
      rule: 'boundaries/dependencies',
      desc: '跨层导入：UI 原语导入 feature 域',
    },
    {
      file: 'features/claude/CrossDomainImport.ts',
      rule: 'boundaries/dependencies',
      desc: 'feature 跨域直连：claude → codex',
    },
    {
      file: 'reverse-dep.ts',
      rule: 'boundaries/dependencies',
      desc: '反向依赖：底层模块导入 store',
    },
    {
      file: 'facade-bypass.ts',
      rule: 'no-restricted-imports',
      desc: '门面消费侧绕过：直接导入 tauri.ts',
    },
  ]

  let failed = false
  for (const exp of expectations) {
    const target = path.join(root, 'tests/fixtures/arch-violations', exp.file)
    let messages = []
    try {
      const json = execFileSync(
        'bunx',
        ['eslint', '--no-ignore', '--config', tmpConfig, '--format', 'json', target],
        { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
      )
      messages = JSON.parse(json)[0]?.messages ?? []
    } catch (err) {
      try {
        messages = JSON.parse(err.stdout)[0]?.messages ?? []
      } catch {
        messages = []
      }
    }
    if (messages.some((m) => m.ruleId === exp.rule)) {
      console.log(`[check:arch-boundaries] PASS ${exp.desc}（${exp.rule} 报错）`)
    } else {
      failed = true
      console.error(
        `[check:arch-boundaries] FAIL ${exp.desc}：预期 ${exp.rule} 未报错，实际 ${JSON.stringify(messages.map((m) => m.ruleId))}`,
      )
    }
  }

  if (failed) process.exit(1)
  console.log('[check:arch-boundaries] 全部边界违规夹具均按预期报错')
} finally {
  fs.rmSync(tmpConfig, { force: true })
}
