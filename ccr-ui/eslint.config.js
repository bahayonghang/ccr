import js from '@eslint/js'
import tseslint from 'typescript-eslint'
import globals from 'globals'
import react from 'eslint-plugin-react'
import boundaries from 'eslint-plugin-boundaries'
import reactHooks from 'eslint-plugin-react-hooks'
// 分层元素与依赖策略（08-22-arch-quality-perf 批次 2）。
// 具名导出供 scripts/check-arch-boundaries.mjs 的夹具自检复用，保证规则单源。
export const boundaryElements = [
  { type: 'ui-primitive', pattern: ['src/ui', 'src/components/ui'] },
  { type: 'shell', pattern: 'src/shell' },
  { type: 'feature', pattern: 'src/features/(*)', capture: ['domain'] },
  { type: 'legacy-feature', pattern: ['src/views', 'src/components'] },
  { type: 'store', pattern: 'src/stores' },
  { type: 'composable', pattern: 'src/composables' },
  { type: 'api', pattern: 'src/api' },
  { type: 'utils', pattern: 'src/utils' },
  { type: 'types', pattern: 'src/types' },
  { type: 'shared', pattern: 'src' },
]

export const boundaryPolicies = [
            // ── 放行清单（default disallow 下逐层枚举允许的依赖方向）──────────
            // UI 原语：只依赖 types / utils / 共享层，不得导入 features、api、store（design §2 硬约束）
            {
              from: { element: { type: 'ui-primitive' } },
              allow: {
                to: { element: { types: { anyOf: ['types', 'utils', 'shared'] } } },
              },
            },
            // 外壳与共享粘合层（main / router / i18n / config）：可依赖一切内部层
            {
              from: { element: { types: { anyOf: ['shell', 'shared'] } } },
              allow: {
                to: {
                  element: {
                    types: {
                      anyOf: [
                        'shell', 'ui-primitive', 'feature', 'legacy-feature', 'store',
                        'composable', 'api', 'types', 'utils', 'shared',
                      ],
                    },
                  },
                },
              },
            },
            // feature：同域 feature 或 platform 共享域 + store/api/types/utils/原语/共享层
            {
              from: { element: { type: 'feature' } },
              allow: {
                to: {
                  element: [
                    { types: { anyOf: ['store', 'api', 'types', 'utils', 'ui-primitive', 'shared'] } },
                    { type: 'feature', captured: { domain: '{{from.domain}}' } },
                    { type: 'feature', captured: { domain: 'platform' } },
                  ],
                },
              },
            },
            // 未迁移视图/组件（legacy-feature）：迁移期间保持现状互通，收敛由统一层完成
            {
              from: { element: { type: 'legacy-feature' } },
              allow: {
                to: {
                  element: {
                    types: {
                      anyOf: [
                        'ui-primitive', 'shell', 'feature', 'legacy-feature', 'store',
                        'composable', 'api', 'types', 'utils', 'shared',
                      ],
                    },
                  },
                },
              },
            },
            // store：向下依赖 api/types/utils 与 store 同层
            {
              from: { element: { type: 'store' } },
              allow: {
                to: { element: { types: { anyOf: ['api', 'types', 'utils', 'shared', 'store'] } } },
              },
            },
            // composable：向下依赖 api/types/utils/store 与 composable 同层
            {
              from: { element: { type: 'composable' } },
              allow: {
                to: { element: { types: { anyOf: ['api', 'types', 'utils', 'shared', 'store', 'composable'] } } },
              },
            },
            // api：只依赖 types/utils/api/shared（runtime、generated）
            {
              from: { element: { type: 'api' } },
              allow: {
                to: { element: { types: { anyOf: ['api', 'types', 'utils', 'shared'] } } },
              },
            },
            // utils / types：最底层
            {
              from: { element: { types: { anyOf: ['utils', 'types'] } } },
              allow: {
                to: { element: { types: { anyOf: ['types', 'utils', 'shared'] } } },
              },
            },
]

export default [
  {
    name: 'app/files-to-lint',
    files: ['**/*.{ts,mts,tsx,js,jsx}'],
  },
  {
    name: 'app/files-to-ignore',
    ignores: [
      '**/dist/**',
      '**/dist-ssr/**',
      '**/storybook-static/**',
      '**/coverage/**',
      '**/.tmp/**',
      '**/node_modules/**',
      '**/ref/**',
      '**/src-tauri/target/**',
      '**/src-tauri/gen/**',
      '**/.vite/**',
      '**/docs/**',
      // 未迁移的 .vue 文件在 React 基座阶段退出检查管线，由各视图子任务改写后重新纳入
      '**/*.vue',
      // ts-rs 生成的 TypeScript 绑定（漂移守卫走 just tauri-bindings-check，不走 lint）
      // 架构边界违规夹具（tests/fixtures/arch-violations）只做定向自检（check:arch-boundaries），不进常规 lint
      'tests/fixtures/**',
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    name: 'app/typescript-files',
    files: ['**/*.{ts,tsx,mts}'],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    // React 插件注册：具体规则集（含 hooks 规则）归 08-22-arch-quality-perf 落地，本阶段不启用新规则
    name: 'app/react-plugins',
    files: ['**/*.{tsx,jsx}'],
    plugins: {
      react,
      'react-hooks': reactHooks,
    },
    languageOptions: {
      parserOptions: {
        ecmaFeatures: { jsx: true },
      },
    },
    settings: {
      react: { version: 'detect' },
    },
  },
  {
    name: 'app/custom-rules',
    rules: {
      // TypeScript rules - 严格类型检查
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': [
        'warn',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrors: 'none', // 忽略catch块中的未使用错误
          destructuredArrayIgnorePattern: '^_',
        },
      ],
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/no-require-imports': 'off',

      // General rules - 安全检查
      // console 全量收口到 utils/logger.ts，源码中禁止直用
      'no-console': 'error',
      'no-debugger': process.env.NODE_ENV === 'production' ? 'error' : 'warn',
      'prefer-const': 'warn',
      'no-var': 'error',
      // 门面消费侧边界（08-22-arch-quality-perf 批次 2）：src/api/tauri.ts 为只读冻结门面，
      // 消费方一律从 '@/api' 或 '@/api/domains/<domain>' 导入；定义侧由 api-facade-boundary smoke 冻结。
      // 白名单（既有导入点，逐文件登记）：src/api/** 内部相对导入（规则天然不命中）、
      // tests/api-facade-coverage.smoke.test.ts（门面覆盖断言本体，见 app/tests 白名单块）。
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['**/api/tauri', '**/api/tauri.*'],
              message:
                '冻结门面：禁止直接导入 src/api/tauri.ts，请从 "@/api" 或 "@/api/domains/<domain>" 导入',
            },
          ],
        },
      ],
      'no-undef': 'off', // TypeScript handles this
    },
  },

  {
    // 门面覆盖断言测试是唯一允许直接导入 tauri.ts 的消费点（白名单逐文件登记，layering-contracts.md）
    name: 'app/facade-coverage-test-whitelist',
    files: ['tests/api-facade-coverage.smoke.test.ts'],
    rules: {
      'no-restricted-imports': 'off',
    },
  },

  {
    // 分层与依赖方向强制（08-22-arch-quality-perf 批次 2，契约见 .trellis/spec/ccr-ui/frontend/layering-contracts.md）
    name: 'app/arch-boundaries',
    files: ['src/**/*.{ts,tsx,mts}'],
    plugins: { boundaries },
    settings: {
      'boundaries/elements': boundaryElements,
      // ts-rs 生成绑定不走边界分析
      // 模块解析：extensionless 相对导入与 @ 别名走 TS resolver（boundaries 依赖 eslint-module-utils 解析目标文件）
      'import/resolver': {
        typescript: {
          alwaysTryTypes: true,
          project: './tsconfig.json',
        },
      },
      'boundaries/ignore-paths': ['src/types/generated/**'],
    },
    rules: {
      'boundaries/dependencies': [
        'error',
        {
          default: 'disallow',
          message:
            '违反分层依赖方向：视图 → 域逻辑 → API → 类型；原语不得导入域逻辑与 store；feature 域间禁止直连。契约：layering-contracts.md',
          policies: boundaryPolicies,
        },
      ],
    },
  },
  {
    // 测试代码纳入 lint 质量门；放宽 console 与部分仅影响生产代码的规则
    name: 'app/tests',
    files: ['tests/**/*.{ts,mts,cjs,js}'],
    languageOptions: {
      globals: {
        ...globals.node,
        ...globals.browser,
      },
    },
    rules: {
      'no-console': 'off',
      '@typescript-eslint/no-require-imports': 'off',
    },
  },
]
