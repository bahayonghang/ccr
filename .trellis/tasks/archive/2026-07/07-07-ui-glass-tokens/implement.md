# 执行计划:玻璃材质令牌体系与对比度修复

## Checklist

1. [x] 基线截图:亮/暗 × clay/paper/graphite/mocha 的 Dashboard、Claude Profiles、Usage 三页(`npm run dev` web 预览 + Playwright,按 theme-token-contracts 预写 localStorage 三键并断言 dataset)。
   - 验证:截图存入 `.trellis/tasks/07-07-ui-glass-tokens/research/baseline/`。✓ 25 张基线截图已就位。
2. [x] tokens.css:亮色 clay 背景四层、边框三档、阴影、文字色重标定(按 design.md §2,用对比度工具核对 4.5:1 / 7:1)。
   - 验证:`bun run type-check` 不适用,跑 `bunx vitest run --config vitest.smoke.config.ts tests/theme-bootstrap.smoke.test.ts`;取色器核对 ΔL。✓ contrast-check.mjs 实测:base/elevated/surface OKLCH L = 91.31/95.11/98.60,ΔL 3.80/3.49/7.29,全部达标(见 research/contrast-notes.md)。
3. [x] tokens.css:paper/graphite 等比修订;暗色 clay 边框微调。✓ contrast-notes.md 全矩阵(light/dark × clay/paper/graphite)对比度全部 PASS。
4. [x] tokens.css:新增三档 `--material-glass-*` 令牌(亮/暗/mocha 覆盖);`--surface-modal/shell/status` 重映射;`--surface-card/workspace` 改不透明。
   - 验证:检查 mocha 覆盖块 specificity(`html:root[data-resolved-flavor='mocha']`)。✓ smoke test 断言语义重映射 + mocha 覆盖块存在,`--surface-card-blur: none` / `--surface-workspace-blur: none` 已落地(tokens.css:531-534)。
5. [x] utilities.css:`.glass-floating/.glass-chrome/.glass-inline` + reduced-transparency 回退 + 预算注释。✓ 已落地并有 smoke test 覆盖,浏览器实测 backdrop-filter 渲染正常。
6. [x] home.css:`--home-surface-card` 改不透明档、`--home-border-card`/hairline 跟随新边框强度。✓ diff 确认 card 92%→98%、border 14%→19%。
6b. [x] 字体三轨分离:--font-brand/--font-mono 按 design.md §5 全 flavor 生效,精简 mocha 覆盖块,窄化字体栈 smoke 例外。
   - 验证:首页大标题为比例字体、统计 tile 数值为真等宽(截图);中英混排无异常;apple-glass-surface-contract 通过。✓ 浏览器 getComputedStyle 实测 `--font-brand`/`--font-mono` 值正确;截图确认标题字体已切换。
7. [x] 更新 smoke test:apple-glass-surface-contract 增加 material 令牌与降级断言。
   - 验证:`bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts`。✓ 39/39 passed。
8. [x] 复查 31 个引用旧 glass 令牌的文件是否有视觉劣化(抽查 MainLayout、BaseModal、Card、Button、UsageDashboard)。✓ 旧 `--glass-*`/`--liquid-glass-*` 令牌已标注 deprecated 并保持轻量档位(不越玻璃预算);仅 ConfigCard.vue 直接引用 `--liquid-glass-*`,截图/live 渲染未见劣化。BaseModal 在页面加载链路中正常编译渲染。
9. [x] 对比截图(同第 1 步矩阵),与基线并排放入 research/,记录结论。✓ 25 张 after 截图就位,light-clay/dark-clay/light-graphite/dark-mocha 抽查确认分层与边界改善明显。
10. [x] `bun run lint` + `bun run type-check` + review gate:请求人工确认亮色对比效果后进入 3.x 收尾。✓ lint / type-check 均干净通过;**review gate 待用户在本轮对话中确认**(见任务汇报)。

## Rollback

任一步视觉劣化:revert 对应 commit;第 4 步语义重映射是唯一影响面大的步骤,单独成 commit。
