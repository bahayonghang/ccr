import { fileURLToPath } from 'node:url'
import { availableParallelism } from 'node:os'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vitest/config'

export const resolveSmokeMaxWorkers = (
  env: NodeJS.ProcessEnv = process.env,
  available = availableParallelism(),
): number => {
  const parallelism = Math.max(1, Math.floor(available))
  const requested = env.CCR_TEST_WORKERS

  if (requested && /^\d+$/.test(requested)) {
    const parsed = Number(requested)
    if (parsed > 0) {
      return Math.min(parsed, parallelism)
    }
  }

  const isCi = Boolean(env.CI && env.CI !== '0' && env.CI.toLowerCase() !== 'false')
  return Math.min(isCi ? 4 : 2, parallelism)
}

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  test: {
    name: 'smoke',
    environment: 'jsdom',
    include: ['tests/**/*.smoke.test.{ts,tsx}'],
    setupFiles: ['./tests/setup/localStorage.ts', './tests/setup/react-cleanup.ts'],
    restoreMocks: true,
    clearMocks: true,
    fileParallelism: true,
    maxWorkers: resolveSmokeMaxWorkers(),
    testTimeout: 15_000,
    // 覆盖率门（08-22-arch-quality-perf 批次 5）：阈值从 justfile CLI 参数移入此处，
    // 使 `bun run test:smoke --coverage` 直接生效。lines ≥70% 为 2026-08-23 复核后保留值
    // （React 基座实测 lines 72.86%，迁移前基线 75.4%，接近 70% 故保留，未显著偏离）。
    // design.md §4 禁止新增 functions/branches/statements 阈值，故仅设 lines。
    coverage: {
      // 只统计测试实际加载的源码；文案目录 / 资源 / 生成类型不进分母。
      exclude: [
        '**/*.css',
        '**/*.d.ts',
        '**/package.json',
        'src/i18n/locales/**',
        'src/assets/**',
        'src/types/generated/**',
      ],
      thresholds: {
        lines: 70
      }
    }
  }
})
