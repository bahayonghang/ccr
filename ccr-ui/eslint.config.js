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
      '**/.impeccable/**',
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
    // React 插件注册（08-22-arch-quality-perf 批次 4）：react 提供 JSX 解析与组件规则；
    // react-hooks 提供 Hooks 规则（见 app/react-hooks-rules 块，error 级）。
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
    // React Hooks 规则（08-22-arch-quality-perf 批次 4，R3）：对所有被 lint 的 TS/TSX 生效，error 级。
    // rules-of-hooks：禁止条件 / 循环 / 嵌套中调用 hooks；exhaustive-deps：effect 依赖数组必须声明齐全。
    // 注意：Vue 组合式函数（.ts，函数名 useXxx 且内部调用 useXxx）可能被误判，误报逐文件登记豁免（见批次 4 豁免块）。
    name: 'app/react-hooks-rules',
    files: ['**/*.{ts,tsx,mts}'],
    plugins: { 'react-hooks': reactHooks },
    rules: {
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'error',
    },
  },
  {
    // 重渲染纪律（design.md §6 可 lint 四项，08-22-arch-quality-perf 批次 4，契约见
    // .trellis/spec/ccr-ui/frontend/react-rerender-discipline.md）：
    // 3a. Zustand 订阅必须传 selector，禁止整 store 订阅。
    //     当前 src/ 命中该模式的均为 legacy Pinia（Vue）store 调用（useXxxStore()），非 Zustand，
    //     逐文件登记豁免并归 state-logic-port / 视图子任务（见批次 4 豁免块）。
    name: 'app/rerender-store-subscription',
    files: ['src/**/*.{ts,tsx,mts}'],
    rules: {
      'no-restricted-syntax': [
        'error',
        {
          selector: 'CallExpression[callee.name=/^use[A-Z]\\w*Store$/][arguments.length=0]',
          message:
            'Zustand 订阅必须传 selector，禁止整 store 订阅（react-rerender-discipline.md）',
        },
      ],
    },
  },
  {
    // 3b. 列表 key 不用数组索引（react/no-array-index-key），对全部 JSX 生效。
    name: 'app/rerender-jsx-keys',
    files: ['**/*.{tsx,jsx}'],
    rules: {
      'react/no-array-index-key': 'error',
    },
  },
  {
    // 3c / 3d. 视图层重渲染纪律，作用域 src/features/** 与 src/views/**（未来视图代码从第一天起遵守）。
    //   3c. react/jsx-no-bind：memo 列表项不传内联函数。src/ui/（原语）与 src/shell/（外壳）在非列表场景
    //       可合法接收内联处理器，不在本约束内（作用域决策见 react-rerender-discipline.md）。
    //   3d. 表单输入受控禁令：<input value+onChange>（无 defaultValue）必须改用 react-hook-form
    //       非受控注册（useForm/register）。src/ui/ 原语与 src/shell/ 可实现受控原语，不在本约束内。
    name: 'app/rerender-views',
    files: ['src/features/**/*.{tsx,jsx}', 'src/views/**/*.{tsx,jsx}'],
    rules: {
      'react/jsx-no-bind': 'error',
      'no-restricted-syntax': [
        'error',
        {
          selector:
            "JSXOpeningElement[name.name='input']:has(JSXAttribute[name.name='value']):has(JSXAttribute[name.name='onChange']):not(:has(JSXAttribute[name.name='defaultValue']))",
          message:
            '表单输入必须用 react-hook-form 非受控注册（useForm/register），禁止受控 value+onChange（react-rerender-discipline.md）',
        },
      ],
    },
  },
  {
    // 统一层 base：禁止平台名条件分支（08-22-platform-unify AC4 / R3）。
    // 本块覆盖 app/rerender-views 的 no-restricted-syntax，故必须同时保留受控 input 禁令。
    name: 'app/platform-unify-no-platform-branch',
    files: [
      'src/features/platform/**/Base*.tsx',
      'src/features/platform/**/*-model.ts',
      'src/features/platform/settings/**/*.{ts,tsx}',
      'src/features/platform/SurfacePage.tsx',
      'src/features/platform/NamedItemCard.tsx',
      'src/features/platform/TabButton.tsx',
      'src/features/platform/translate.ts',
    ],
    rules: {
      'no-restricted-syntax': [
        'error',
        {
          selector:
            "JSXOpeningElement[name.name='input']:has(JSXAttribute[name.name='value']):has(JSXAttribute[name.name='onChange']):not(:has(JSXAttribute[name.name='defaultValue']))",
          message:
            '表单输入必须用 react-hook-form 非受控注册（useForm/register），禁止受控 value+onChange（react-rerender-discipline.md）',
        },
        {
          selector:
            'BinaryExpression[operator=/===|==|!==|!=/][right.value=/^(claude|codex|grok|opencode|gemini|antigravity|claude-code)$/]',
          message:
            'base 组件禁止平台名称条件分支，差异走 config/props（platform-surface-contracts.md）',
        },
        {
          selector:
            'BinaryExpression[operator=/===|==|!==|!=/][left.value=/^(claude|codex|grok|opencode|gemini|antigravity|claude-code)$/]',
          message:
            'base 组件禁止平台名称条件分支，差异走 config/props（platform-surface-contracts.md）',
        },
      ],
    },
  },
  // src/features/platform/profiles/shared.ts：再导出 components/profiles 共享层。
  // features/platform 不得跨域导入 legacy-feature；此文件是协同点 F 指定的再导出点
  // （profiles-shared-interfaces.md §1）。归属 08-22-platform-unify。
  { files: ['src/features/platform/profiles/shared.ts'], rules: { 'boundaries/dependencies': 'off' } },

  // ── 批次 4 逐文件登记豁免（08-22-arch-quality-perf，完整清单与处置见 implement.md 批次 4 证据块）────────
  // 原则（prd R12、AC11）：无全局豁免、源文件不加 eslint-disable，只在配置中按「文件 × 规则」关闭，并注明处置。
  // 类型安全 no-unsafe-* 系列（R4）：当前 src/ 全部 69 处已就地修复（类型收窄 / 显式标注 / 根因修复 vue-i18n
  //   I18nComposer 别名），零剩余违规，无需豁免。
  // React Hooks 规则（R3）与重渲染 3a（R8）：命中项均为 legacy Pinia（Vue）store 调用或 Vue 组合式函数，
  //   非 Zustand / 非 React，逐文件豁免并归 state-logic-port / 视图子任务（重写为 React 后移除）。
  // ── hooks 规则误报（R3）：Vue 组合式函数顶层调用被 rules-of-hooks 误判（非 React hooks，重写为 React 后不再存在）──
  // src/composables/useBackendHealth.ts：模块顶层 usePolledData()（Vue composable 轮询器），rules-of-hooks 误判，归属 state-logic-port
  { files: ['src/composables/useBackendHealth.ts'], rules: { 'react-hooks/rules-of-hooks': 'off' } },
  // ── 重渲染 3a（裸 store 订阅禁令）：legacy Pinia（Vue）store 调用，非 Zustand，重写为 Zustand 时补 selector ──
  // src/composables/useCodexOAuthFlow.ts：useUIStore()（Pinia），归属 state-logic-port
  { files: ['src/composables/useCodexOAuthFlow.ts'], rules: { 'no-restricted-syntax': 'off' } },
  // src/composables/useCodexProviders.ts：useUIStore()（Pinia），归属 state-logic-port
  { files: ['src/composables/useCodexProviders.ts'], rules: { 'no-restricted-syntax': 'off' } },
  // src/composables/useMainLayoutShell.ts：useShellPreferencesStore()（Pinia），归属 state-logic-port
  { files: ['src/composables/useMainLayoutShell.ts'], rules: { 'no-restricted-syntax': 'off' } },
  // src/composables/usePlatformMcp.ts：useUIStore()（Pinia），归属 state-logic-port
  { files: ['src/composables/usePlatformMcp.ts'], rules: { 'no-restricted-syntax': 'off' } },
  // src/composables/usePlatformPlugins.ts：useUIStore()（Pinia），归属 state-logic-port
  { files: ['src/composables/usePlatformPlugins.ts'], rules: { 'no-restricted-syntax': 'off' } },
  // src/composables/useUnifiedMcp.ts：useUIStore()（Pinia），归属 state-logic-port
  { files: ['src/composables/useUnifiedMcp.ts'], rules: { 'no-restricted-syntax': 'off' } },
  // src/views/checkin/composables/useCheckinState.ts：useUIStore()（Pinia），归属 views-checkin
  { files: ['src/views/checkin/composables/useCheckinState.ts'], rules: { 'no-restricted-syntax': 'off' } },
  // src/views/usage/useUsageDashboardState.ts：useUsageStore()（Pinia），归属 views-usage
  { files: ['src/views/usage/useUsageDashboardState.ts'], rules: { 'no-restricted-syntax': 'off' } },

  {
    // 类型安全 no-unsafe-* 系列（08-22-arch-quality-perf 批次 4，R4，error 级）。
    // 需类型感知（projectService）lint。tsconfig include 覆盖 src/** 与 tests/**，
    // 作用域与之对齐；src/types/generated/** 与 **/*.vue 已在全局 ignore。
    // 耗时：type-aware 单遍约 +3.5s（详见 implement.md 批次 4 证据块），低于 2× 预算（design.md §1），
    // 故保留在常规 lint 内，无需拆分 lint:typecheck。
    name: 'app/type-safe-rules',
    files: ['src/**/*.{ts,tsx,mts}', 'tests/**/*.{ts,tsx,mts}'],
    plugins: { '@typescript-eslint': tseslint.plugin },
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: { projectService: true },
    },
    rules: {
      '@typescript-eslint/no-unsafe-assignment': 'error',
      '@typescript-eslint/no-unsafe-member-access': 'error',
      '@typescript-eslint/no-unsafe-call': 'error',
      '@typescript-eslint/no-unsafe-return': 'error',
      '@typescript-eslint/no-unsafe-argument': 'error',
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
      // tests/api/api-facade-coverage.smoke.test.ts（门面覆盖断言本体，见 app/tests 白名单块）。
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
    // 规模与复杂度阈值（08-22-arch-quality-perf 批次 3，暂定值，取值与反馈轮依据见 thresholds.md）。
    // 只作用于 src/**/*.{ts,tsx,mts}：tests/ 与 scripts/ 不在测量集合内（distribution.md 活文件集 217 个）。
    // src/types/generated/** 与 **/*.vue 已在全局 ignore。超限文件的逐文件豁免见下方的逐文件覆盖块。
    name: 'app/threshold-rules',
    files: ['src/**/*.{ts,tsx,mts}'],
    rules: {
      // 行数：阶段 4 冻结 P90=334 向上取整到 100 的倍数 → 400（见 archived thresholds.md §6）
      'max-lines': ['error', { max: 400, skipBlankLines: false, skipComments: false }],
      // 圈复杂度：P90=16，反馈轮 6.0% 在 [3%,15%] 带内，保留
      complexity: ['error', { max: 16 }],
      // 嵌套深度：P90=3，反馈轮 0.9% < 3% → 下调一档至 2
      'max-depth': ['error', { max: 2 }],
      // 参数个数：P90=4，反馈轮 2.8% < 3% → 下调一档至 3
      'max-params': ['error', { max: 3 }],
    },
  },

  // 超限文件逐文件登记豁免（08-22-arch-quality-perf 批次 3，完整清单见 .trellis/tasks/08-22-arch-quality-perf/thresholds.md）。
  // 原则（prd R12、AC11）：无全局豁免、源文件不加 eslint-disable，只在配置中按「文件 × 规则」关闭，并注明处置。
  // 处置分三类：纯数据表/生成物/冻结门面 → 注册豁免（登记在册，阶段 4 复核）；
  //               归迁移批次（state-logic-port / 视图子任务 / shell-port / i18n-port）→ 在对应批次中拆分或改写，届时移除此块。
  // ── 注册豁免：纯数据表 / 生成物 / 冻结门面（拆分无收益或不可拆分，登记在册）─────────────────
  // src/api/generated/commandCapabilities.ts：max-lines 6057 > 500，ts-rs 生成命令能力数据表，零逻辑（distribution.md 行数 Top1）
  { files: ['src/api/generated/commandCapabilities.ts'], rules: { 'max-lines': 'off' } },
  // src/i18n/locales/en-US.ts：max-lines 5456 > 500，翻译数据表，零逻辑
  { files: ['src/i18n/locales/en-US.ts'], rules: { 'max-lines': 'off' } },
  // src/i18n/locales/zh-CN.ts：max-lines 5300 > 500，翻译数据表，零逻辑
  { files: ['src/i18n/locales/zh-CN.ts'], rules: { 'max-lines': 'off' } },
  // src/i18n/bootMessages.ts：max-lines 1203 > 500，启动文案数据表，零逻辑
  { files: ['src/i18n/bootMessages.ts'], rules: { 'max-lines': 'off' } },
  // src/types/checkin.ts：max-lines 667 > 500，类型数据表，零逻辑
  { files: ['src/types/checkin.ts'], rules: { 'max-lines': 'off' } },
  // src/types/codex.ts：max-lines 503 > 500，类型数据表，零逻辑
  { files: ['src/types/codex.ts'], rules: { 'max-lines': 'off' } },
  // src/api/tauri.ts：max-lines 736 > 500，冻结门面（constraint C5，只读），定义侧由 api-facade-boundary smoke 冻结，不可拆分
  { files: ['src/api/tauri.ts'], rules: { 'max-lines': 'off' } },
  // src/api/domains/codex.ts：max-lines 952 > 500，域门面，60 个 export 均为对 generated/invoke 的薄类型封装（typed wrapper facade），拆分无收益；API 层在迁移期原样保留（state-logic-port Out of Scope）
  { files: ['src/api/domains/codex.ts'], rules: { 'max-lines': 'off' } },
  // src/api/domains/claude.ts：max-lines 624 > 500，域门面，与 codex.ts 同型（typed wrapper facade），API 层迁移期原样保留
  { files: ['src/api/domains/claude.ts'], rules: { 'max-lines': 'off' } },
  // src/api/generated/codex.ts：max-params 4 > 3，ts-rs 生成绑定，命令签名与后端一致，不可改
  { files: ['src/api/generated/codex.ts'], rules: { 'max-params': 'off' } },
  // src/api/generated/systemPrompts.ts：max-params 4 > 3，ts-rs 生成绑定，不可改
  { files: ['src/api/generated/systemPrompts.ts'], rules: { 'max-params': 'off' } },
  // src/api/generated/uiState.ts：max-params 4 > 3，ts-rs 生成绑定，不可改
  { files: ['src/api/generated/uiState.ts'], rules: { 'max-params': 'off' } },
  // src/api/generated/usageV2.ts：max-params 7 > 3，ts-rs 生成绑定，不可改
  { files: ['src/api/generated/usageV2.ts'], rules: { 'max-params': 'off' } },
  // src/api/domains/environment.ts：max-params 5 > 3，域门面薄封装，参数镜像后端命令签名，API 层迁移期原样保留
  { files: ['src/api/domains/environment.ts'], rules: { 'max-params': 'off' } },
  // src/api/domains/sync.ts：max-params 6 > 3，域门面薄封装，参数镜像后端命令签名，API 层迁移期原样保留
  { files: ['src/api/domains/sync.ts'], rules: { 'max-params': 'off' } },
  // src/api/domains/systemPrompts.ts：max-params 4 > 3，域门面薄封装，参数镜像后端命令签名，API 层迁移期原样保留
  { files: ['src/api/domains/systemPrompts.ts'], rules: { 'max-params': 'off' } },
  // src/api/domains/unifiedMcp.ts：max-params 4 > 3，域门面薄封装，参数镜像后端命令签名，API 层迁移期原样保留
  { files: ['src/api/domains/unifiedMcp.ts'], rules: { 'max-params': 'off' } },
  // ── 归 08-22-state-logic-port（store / composable 重写）──────────────────────────────
  // src/stores/usage.ts：max-lines 991、complexity 27、max-depth 3，store 归属 state-logic-port（重写为 Zustand/TanStack Query 时拆分）
  { files: ['src/stores/usage.ts'], rules: { 'max-lines': 'off', complexity: 'off', 'max-depth': 'off' } },
  // src/composables/useCodexDashboard.ts：max-lines 657、complexity 27，composable 归属 state-logic-port
  { files: ['src/composables/useCodexDashboard.ts'], rules: { 'max-lines': 'off', complexity: 'off' } },
  // src/composables/useGrokDashboard.ts：max-lines 580、complexity 18、max-depth 3，composable 归属 state-logic-port
  { files: ['src/composables/useGrokDashboard.ts'], rules: { 'max-lines': 'off', complexity: 'off', 'max-depth': 'off' } },
  // src/composables/useUnifiedMcp.ts：max-lines 534、complexity 17，composable 归属 state-logic-port
  { files: ['src/composables/useUnifiedMcp.ts'], rules: { 'max-lines': 'off', complexity: 'off' } },
  // src/composables/useMonitoringFeed.ts：complexity 19，composable 归属 state-logic-port
  { files: ['src/composables/useMonitoringFeed.ts'], rules: { 'max-lines': 'off', complexity: 'off' } },
  // src/composables/useStream.ts：max-depth 4，composable 归属 state-logic-port
  { files: ['src/composables/useStream.ts'], rules: { 'max-depth': 'off' } },
  // src/composables/useAgents.ts：max-depth 3，composable 归属 state-logic-port
  { files: ['src/composables/useAgents.ts'], rules: { 'max-depth': 'off' } },
  // src/composables/usePolledData.ts：max-depth 3，composable 归属 state-logic-port
  { files: ['src/composables/usePolledData.ts'], rules: { 'max-depth': 'off' } },
  // src/composables/useProfilesFilter.ts：max-depth 3，composable 归属 state-logic-port
  { files: ['src/composables/useProfilesFilter.ts'], rules: { 'max-depth': 'off' } },
  // src/composables/useProfilesInsights.ts：max-depth 3，composable 归属 state-logic-port
  { files: ['src/composables/useProfilesInsights.ts'], rules: { 'max-depth': 'off' } },
  // src/stores/homeUsageOverview.ts：complexity 23，store 归属 state-logic-port
  { files: ['src/stores/homeUsageOverview.ts'], rules: { complexity: 'off' } },
  // src/utils/usageDashboardPayload.ts：max-params 4，纯变换迁入 utils（08-22-state-logic-port 批次 4，路径更新）
  { files: ['src/utils/usageDashboardPayload.ts'], rules: { 'max-params': 'off' } },
  // ── 归 08-22-views-usage（Usage / Dashboard / platform-usage 视图子任务）───────────────
  // src/views/dashboard/dashboardPresentation.ts：max-lines 663、complexity 27、max-params 5，dashboard 展示层归属 views-usage
  { files: ['src/views/dashboard/dashboardPresentation.ts'], rules: { 'max-lines': 'off', complexity: 'off', 'max-params': 'off' } },
  // src/views/usage/usageOpsCockpit.ts：max-lines 516、complexity 51、max-params 4，usage 展示层归属 views-usage
  { files: ['src/views/usage/usageOpsCockpit.ts'], rules: { 'max-lines': 'off', complexity: 'off', 'max-params': 'off' } },
  // src/views/platform-usage/platformUsagePresentation.ts：complexity 18、max-params 5，platform-usage 展示层归属 views-usage
  { files: ['src/views/platform-usage/platformUsagePresentation.ts'], rules: { complexity: 'off', 'max-params': 'off' } },
  // src/views/usage/usageChartOptions.ts：max-params 4，usage 图表配置归属 views-usage
  { files: ['src/views/usage/usageChartOptions.ts'], rules: { 'max-lines': 'off', 'max-params': 'off' } },
  // src/views/usage/usageOverviewInsights.ts：max-params 4，usage 展示层归属 views-usage
  { files: ['src/views/usage/usageOverviewInsights.ts'], rules: { 'max-params': 'off' } },
  // src/views/usage/usageSummaryCards.ts：max-params 5，usage 展示层归属 views-usage
  { files: ['src/views/usage/usageSummaryCards.ts'], rules: { 'max-params': 'off' } },
  // ── 归 08-22-views-checkin（CheckIn 视图子任务）───────────────────────────────────
  // src/views/checkin/composables/useCheckinState.ts：max-lines 569，checkin 视图内 composable 归属 views-checkin
  { files: ['src/views/checkin/composables/useCheckinState.ts'], rules: { 'max-lines': 'off' } },
  // src/views/checkin/composables/balanceRefreshQueue.ts：max-depth 3，checkin 视图内 composable 归属 views-checkin
  { files: ['src/views/checkin/composables/balanceRefreshQueue.ts'], rules: { 'max-depth': 'off' } },
  // src/views/checkin/composables/checkinJobRuntime.ts：max-params 4、max-depth 3，checkin 视图内 composable 归属 views-checkin
  { files: ['src/views/checkin/composables/checkinJobRuntime.ts'], rules: { 'max-params': 'off', 'max-depth': 'off' } },
  // src/views/checkin/composables/checkinWafRecovery.ts：max-params 4、max-depth 4，checkin 视图内 composable 归属 views-checkin
  { files: ['src/views/checkin/composables/checkinWafRecovery.ts'], rules: { 'max-lines': 'off', 'max-params': 'off', 'max-depth': 'off' } },
  // ── 归 08-22-views-profiles-config（Profiles / Provider 模板 / 配置视图子任务）──────────────
  // src/utils/claudeProfiles.ts：max-lines 521，profile 工具归属 views-profiles-config（其关联资产清单含 claudeProfiles.ts）
  { files: ['src/utils/claudeProfiles.ts'], rules: { 'max-lines': 'off' } },
  // src/utils/providerTemplates.ts：max-lines 513、complexity 26，provider 模板工具归属 views-profiles-config
  { files: ['src/utils/providerTemplates.ts'], rules: { 'max-lines': 'off', complexity: 'off' } },
  // src/utils/claudeProfileEditor.ts：complexity 25，profile 编辑器工具归属 views-profiles-config
  { files: ['src/utils/claudeProfileEditor.ts'], rules: { complexity: 'off' } },
  // src/configs/providersCatalog.ts：complexity 20，provider 站点目录消费链归属 views-profiles-config
  { files: ['src/configs/providersCatalog.ts'], rules: { complexity: 'off' } },
  // ── 归 08-22-views-secondary-platforms（Grok 视图子任务）──────────────────────────────
  // src/utils/grokProfileEditor.ts：complexity 20，Grok profile 编辑器工具，消费方为 grokProfileEditorAdapter → views-secondary-platforms
  { files: ['src/utils/grokProfileEditor.ts'], rules: { complexity: 'off' } },
  // ── 归 08-22-shell-port（应用外壳 / 路由 / 通用工具）──────────────────────────────────
  // src/router/index.ts：max-lines 594，75 条路由表在 shell-port 迁移路由时改写（其 Scope 明确含 src/router/index.ts）
  { files: ['src/router/index.ts'], rules: { 'max-lines': 'off' } },
  // src/utils/logger.ts：max-depth 3，通用日志收口（no-console 依赖），归属 shell-port
  { files: ['src/utils/logger.ts'], rules: { 'max-depth': 'off' } },
  // src/utils/errorHandler.ts：max-depth 3，通用错误处理工具，归属 shell-port
  { files: ['src/utils/errorHandler.ts'], rules: { 'max-depth': 'off' } },
  // src/utils/logRedact.ts：max-params 4、max-depth 3，凭据脱敏工具，通用收口（logger 依赖），归属 shell-port
  { files: ['src/utils/logRedact.ts'], rules: { 'max-params': 'off', 'max-depth': 'off' } },
  // ── 归 08-22-i18n-port（i18n 运行时迁移）───────────────────────────────────────────
  // src/i18n/formatMessage.ts：max-params 4，i18n 占位符插值工具，归属 i18n-port
  { files: ['src/i18n/formatMessage.ts'], rules: { 'max-params': 'off' } },
  // ── 阶段 4 冻结 max-lines=400 后新超限（08-22-platform-unify 协同点 N）────────
  // src/utils/themeBootstrap.ts：max-lines 471 > 400，主题启动，归属 shell-port / design-system
  { files: ['src/utils/themeBootstrap.ts'], rules: { 'max-lines': 'off' } },
  // src/composables/usePlatformMcp.ts：max-lines 455 > 400，legacy composable，归属视图子任务改写
  { files: ['src/composables/usePlatformMcp.ts'], rules: { 'max-lines': 'off' } },
  // src/utils/perfTelemetry.ts：max-lines 414 > 400，性能探针，归属 arch-quality-perf
  { files: ['src/utils/perfTelemetry.ts'], rules: { 'max-lines': 'off' } },
  // src/configs/providerPresets/claude.ts：max-lines 404 > 400，provider 预设数据表，归属 views-profiles-config
  { files: ['src/configs/providerPresets/claude.ts'], rules: { 'max-lines': 'off' } },

  {
    // 门面覆盖断言测试是唯一允许直接导入 tauri.ts 的消费点（白名单逐文件登记，layering-contracts.md）
    name: 'app/facade-coverage-test-whitelist',
    files: ['tests/api/api-facade-coverage.smoke.test.ts'],
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
