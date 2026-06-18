import js from '@eslint/js'
import pluginVue from 'eslint-plugin-vue'
import * as parserVue from 'vue-eslint-parser'
import tseslint from 'typescript-eslint'
import globals from 'globals'
import vueI18n from '@intlify/eslint-plugin-vue-i18n'

export default [
  {
    name: 'app/files-to-lint',
    files: ['**/*.{ts,mts,tsx,vue,js,jsx}'],
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
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...pluginVue.configs['flat/recommended'],
  {
    name: 'app/vue-parser',
    files: ['**/*.vue'],
    languageOptions: {
      parser: parserVue,
      parserOptions: {
        ecmaVersion: 'latest',
        extraFileExtensions: ['.vue'],
        parser: tseslint.parser,
        sourceType: 'module',
      },
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
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
    name: 'app/custom-rules',
    rules: {
      // Vue rules
      'vue/multi-word-component-names': 'off',
      // v-html 仅允许在有 DOMPurify/escapeHtml 防护的渲染点使用，逐行 eslint-disable 豁免
      'vue/no-v-html': 'error',
      'vue/require-default-prop': 'off',
      'vue/require-explicit-emits': 'error',
      'vue/one-component-per-file': 'off',

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
      'no-undef': 'off', // TypeScript handles this
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
  {
    // i18n 防回归（WS7.3）：模板硬编码文案锁死为 warn，记录债务但不阻断 CI；新增项应走 t()/tf()
    name: 'app/i18n-no-raw-text',
    files: ['**/*.vue'],
    plugins: { '@intlify/vue-i18n': vueI18n },
    settings: {
      'vue-i18n': {
        // 不配置 localeDir：locale 为 .ts 模块，插件无法静态解析以生成「建议 key」（会崩溃）；
        // no-raw-text 仅需检测能力，不需要 key 建议。
        messageSyntaxVersion: '^9.0.0',
      },
    },
    rules: {
      '@intlify/vue-i18n/no-raw-text': [
        'warn',
        {
          // 忽略纯符号/数字/标点（如 ':' '*' '%' '·' '|'）——这些不是需要翻译的文案，
          // 仅锁死真正的硬编码词句。
          ignorePattern: '^[\\s\\d\\-–—:：*%·•|/\\\\()\\[\\]{}.,，。、；;!！?？#&+=<>"\'“”‘’~@]+$|^(?:\\$|v|ms|HTTP|STDIO|Esc|↑↓|↵|…|\\.?mcp\\.json|mcpServers|api_user|session|TOML|JSON|URL|ID:?|px|s|low|medium|high|xhigh)$',
        },
      ],
    },
  },
]
