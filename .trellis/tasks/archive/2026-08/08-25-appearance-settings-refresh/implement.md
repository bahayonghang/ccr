# 执行计划：外观设置页重排与预览一致性

前置：`08-25-design-token-consolidation` 已合入。

## 检查清单

- [x] 1. 填 `design.md` §1 的前后差异表，每行给出分类结论。这是 AC1 的直接产物，不得留「待定」。
      填表前先通读 `AppearanceSection.tsx` 与 `app-settings.css`，确认「混排预览」是否已存在。
- [x] 2. 若填表结论显示实际增量只有「合并卡片」与「一致性测试」两项，就按这两项做，不扩大范围。
- [x] 3. 一致性测试：新建 `ccr-ui/tests/flavor-preview-consistency.smoke.test.ts`，
      按 `design.md` §3.2 的四步解析 `tokens.css` 并比对 `FLAVOR_PREVIEW_TOKENS`。
      先确认哪些令牌在哪个作用域块中被定义，再写继承规则。
- [x] 4. 反向验证 AC4：临时把 `tokens.css` 的某个 `--color-bg-base` 改一位十六进制，
      确认测试失败，然后还原。此步不进提交。
- [x] 5. `AppearanceSection.tsx` 合并主题卡与 flavor 卡（`design.md` §4），
      `ThemeOption` / `FlavorCard` 内部与 props 不动，`data-testid` 全部保留。
- [x] 6. `app-settings.css` 对应版式，全部走令牌。
- [x] 7. 选中态加边框加粗与勾选图标（`design.md` §5）。
- [x] 8. 新增文案的中英文键补齐。
- [x] 9. `just frontend-check-quick`。
- [x] 10. 视觉与交互验证（见下）。

## 验证命令

```bash
just frontend-check-quick
```

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/flavor-preview-consistency.smoke.test.ts tests/app-settings-view.smoke.test.tsx tests/font-preferences.smoke.test.ts
```

用于 AC4 / AC9 / AC10：新测试通过，既有设置页与字体测试不回归。

`just frontend-check-quick` 在本机 typecheck 通过；eslint 在 `DashboardUsageMovement.tsx` 因 complexity/max-lines 失败（非本任务 exclusive 文件）。本任务文件 eslint + stylelint 通过。`test:i18n` leaf-count 为 4172 vs 冻结 4166，含并行任务写入共享 locale 的键；本任务只改既有 appearance 文案、未新增键。

```bash
rg -n 'settings-font-ui|settings-font-code' ccr-ui/src/features/configs/settings/AppearanceSection.tsx
```

用于 AC3：两个 `data-testid` 必须仍有命中。

```bash
rg -n '#[0-9a-fA-F]{3,8}|border-radius:\s*[0-9]+px' ccr-ui/src/features/configs/styles/app-settings.css
```

用于 AC8：应无命中。`flavorPreview.ts` 是 `.ts`，不在本扫描范围，其字面量由第 3 步的测试守护。

```bash
git diff --name-only -- ccr-ui/src/utils/themeBootstrap.ts ccr-ui/src/utils/fontPreferences.ts
```

用于 Out of Scope：应为空。

## 视觉与交互验证

`cd ccr-ui && npm run dev`，打开 `/settings`。

- [x] 明暗三选项与底色族两选项在同一张卡内（AC2）。
- [x] 选 `system` 时显示当前解析结果（AC2）。
- [x] 两个字体下拉可用，选「自定义」展开输入框；混排预览与回退提示可见；数据样例为 mono（AC3）。
- [x] 切换 flavor 后预览色条与页面实际表面色一致，亮暗各验一次（AC5）。
- [x] 界面上只有 `neutral` 与 `clay` 两个 flavor 选项（AC6）。
- [x] 灰度模拟下仍能判断当前选中的主题与 flavor（AC7）。

## 回滚

```bash
git checkout -- ccr-ui/src/features/configs/settings/AppearanceSection.tsx ccr-ui/src/features/configs/styles/app-settings.css
rm -f ccr-ui/tests/flavor-preview-consistency.smoke.test.ts
```

一致性测试可单独保留，它不依赖版式改动。

## 提交

`refactor(ui): ♻️ 外观设置页分区重排并守护 flavor 预览取值`

change list 必须包含测试文件（父任务 XC5）。
提交前执行父任务 XC4 的三条检查，确认 `ccr-ui/src-tauri/Cargo.toml` 不在暂存区、不在提交中。
</content>
