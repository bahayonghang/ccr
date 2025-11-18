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
      '**/coverage/**',
      '**/node_modules/**',
      '**/src-tauri/target/**',
      '**/src-tauri/gen/**',
      '**/.vite/**',
      '**/tests/**',
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
      'vue/no-v-html': 'warn',
      'vue/require-default-prop': 'off',
      'vue/require-explicit-emits': 'off',
      'vue/one-component-per-file': 'off',

      // TypeScript rules - 更宽松的设置适合开发阶段
      '@typescript-eslint/no-explicit-any': 'off', // API响应、动态数据等使用any是合理的
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

      // General rules
      'no-console': 'off',
      'no-debugger': 'warn',
      'prefer-const': 'warn',
      'no-var': 'error',
      'no-undef': 'off', // TypeScript handles this
    },
  },
]
