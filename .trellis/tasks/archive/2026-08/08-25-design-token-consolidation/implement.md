# 执行计划：令牌层收敛与名称治理

## 阶段 A：名称增量审计（必须先做完，结论决定后续范围）

- [x] A1. 记录基线名称集：`rg -o -- '--[a-z0-9-]+(?=:)' ccr-ui/src/styles/ | sort -u > research/token-names-before.txt`，行数应为 448 或记录实际值与差异原因。
- [x] A2. tint 审计：`rg -n 'stage-chip|chip-neutral' ccr-ui/src` 列出全部消费点，判断是否确需五色 tint。结论写入 `research/token-name-delta.md`。
- [x] A3. 字号审计：对四项字号逐项判断能否复用既有档（`design.md` §7 表）。默认结论为复用；若判定不可复用，记录具体视觉问题。
- [x] A4. accent tint 处置：在 `design.md` §5 的选项 A / B 中二选一并记录理由。
- [x] A5. 产出 `research/token-name-delta.md`，每个「确需新增」的名称有五项结论：分类、`core.css` 归属层、四作用域定义结论、自定义强调色影响结论、对应测试断言位置。

阶段 A 结束时，新增名称清单已固定。若清单为空（只有 `--color-platform-opencode` 及其 `-rgb`），后续治理动作相应缩减。

## 阶段 B：取值收敛（不新增名称）

- [x] B1. 重新定位 `tokens.css` 四个作用域的实际行号（`design.md` §1 的行号是改前快照）。
- [x] B2. 边框（clay 暗）：按 `design.md` §2 表写入三个令牌与三个 `-rgb` 伴随令牌。
- [x] B3. 边框（其余三作用域）：按 alpha 合成规则逐通道计算，写入实色与 `-rgb`。计算过程记到 `research/border-derivation.md`。
- [x] B4. 圆角：按 `design.md` §3 修改 `--radius-sm` / `--radius-xl` / `--radius-3xl` 三个取值。其余四个不动。
- [x] B5. 调用点核对：`rg -n 'color-border-[a-z]+-rgb' ccr-ui/src`，逐个确认改为实色后取色仍合理。
- [x] B6. 确认 `--surface-card-border: var(--color-border-subtle)` 的引用关系未变（`apple-glass-surface-contract.smoke.test.ts` 断言的是引用关系）。

## 阶段 C：新增名称与治理（范围由阶段 A 决定）

- [x] C1. `--color-platform-opencode` 与其 `-rgb`：按既有 `--color-platform-*` 的作用域与层归属写入。
- [x] C2. 阶段 A 判定确需新增的其他名称，逐个写入并在对应作用域配对。
- [x] C3. 更新 `research/token-names-after.txt` 并与 before 对比，确认增量等于登记清单。
- [x] C4. 更新 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md` 的名称冻结段落：改为「448 + 本次登记增量」并列出增量清单，注明治理任务为本任务。

## 阶段 D：桥接与测试

- [x] D1. 桥接核对：`theme.css` 中每个 `var()` 目标仍有定义；必要时补行。opencode 短名按 `design.md` §6 判断是否需要桥接。
- [x] D2. 新增测试断言（`design.md` §8 表）：边框实色化、圆角四档收敛、新增名称四作用域可解析。
- [x] D3. `theme-switch.smoke.test.tsx` 的锚点值若涉及被改取值的令牌，同步锚点值，保持断言结构不变。
- [x] D4. 确认 `theme-contrast-contract.smoke.test.ts` 的阈值常量未被修改。

## 阶段 E：验证

- [x] E1. `just frontend-check-quick`。
- [x] E2. 主题契约测试全套（见下）。
- [x] E3. 视觉回归走查（见下），结论写入 `research/token-regression.md`。

## 验证命令

```bash
just frontend-check-quick
```

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-contrast-contract.smoke.test.ts tests/theme-switch.smoke.test.tsx tests/token-single-point.smoke.test.tsx tests/theme-domain-extension.smoke.test.tsx
```

用于 AC8：五个文件全绿，且 `theme-contrast-contract.smoke.test.ts` 的阈值常量与改动前一致。

```bash
rg -n 'color-border-(subtle|default|strong)(-rgb)?:' ccr-ui/src/styles/tokens.css
```

用于 AC2：非 `-rgb` 的命中必须是十六进制实色，无 `rgb(... / ...%)`；`-rgb` 命中的三元组与对应实色一致。

```bash
rg -n -- '--radius-[a-z0-9]+:' ccr-ui/src/styles/tokens.css
```

用于 AC3：取值集合恰为 `{0, 6px, 8px, 12px, 9999px}`。

```bash
rg -n -- '--color-bg-chrome|--radius-(chip|control|card|pill)' ccr-ui/src/styles/
```

用于 AC4：应无命中（这些名称本次不引入）。

```bash
diff research/token-names-before.txt research/token-names-after.txt
```

用于 AC6：差异行必须与 `research/token-name-delta.md` 的登记清单逐条对应。

## 回归走查

`cd ccr-ui && npm run dev`，逐组合走查：

| 组合 | 检查点 |
|---|---|
| light × neutral | 边框在浅底上可见但不刺眼；卡片圆角同屏一致 |
| light × clay | 同上；暖色底下边框不偏色 |
| dark × neutral | 边框在深底上有明确边界（旧 alpha 版本的主要缺陷） |
| dark × clay | 与设计稿 `1c` 表面阶梯一致；shell 层与卡片层可辨 |

页面：Dashboard、Profiles、MCP、Commands、Sync、Check-ins、Usage、Settings。
另需在设置页把强调色设为自定义值，确认 accent 相关表现符合阶段 A4 选定的方案。

判定：原 4px chip 变 6px、原 16px 容器变 12px 属预期收敛，记录但不判缺陷。

## 回滚

```bash
git checkout -- ccr-ui/src/styles/tokens.css ccr-ui/src/styles/theme.css ccr-ui/tests/
```

`research/` 与 spec 改动是记录，可单独保留。

## 提交

`refactor(ui): ♻️ 令牌层收敛为实色边框与四档圆角`

change list 必须包含测试文件（父任务 XC5）。
提交前执行父任务 XC4 的三条检查，确认 `ccr-ui/src-tauri/Cargo.toml` 不在工作区改动之外、不在暂存区、不在提交中。
</content>
