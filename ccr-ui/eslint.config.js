// For more info, see https://github.com/storybookjs/eslint-plugin-storybook#configuration-flat-config-format
import storybook from 'eslint-plugin-storybook'

import js from '@eslint/js'
import pluginVue from 'eslint-plugin-vue'
import * as parserVue from 'vue-eslint-parser'
import tseslint from 'typescript-eslint'
import globals from 'globals'

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
  ...storybook.configs['flat/recommended'],
]
