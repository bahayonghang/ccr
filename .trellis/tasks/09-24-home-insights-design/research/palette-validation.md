# Insights 颜色校验记录

校验器：dataviz 技能 `scripts/validate_palette.js`（OKLab ΔE×100，CVD 用 Machado 2009 severity 1.0）。表面色取卡片面 `--color-bg-surface`，4 种组合（`theme-token-contracts.md` 的 light/dark × neutral/clay）：亮 neutral `#faf7ec`、亮 clay `#fefaf2`、暗 neutral `#1f1b14`、暗 clay `#2a221e`。门槛：CVD ΔE 目标 ≥8、下限 ≥6（6-8 须有辅助编码）；正常视觉 ΔE ≥15；图形对 3:1。

## 1. 结论

| 项 | 取值 | 结果 |
| --- | --- | --- |
| 4 个新来源身份色 | 见 §2，亮暗同值、neutral/clay 同值 | 明度带、色度下限、对比度 3:1 在 4 种表面全部 PASS；两两比较未过正常视觉 15（最差 11.4） |
| 9 个来源整体 | 现有 5 色 + 新 4 色 | 现有品牌色本身未过门槛（§3），任何取值都不能使 9 色两两可分 |
| 周趋势合并规则 | 前 N=4 独立着色，其余并入"其他" | §4 |
| "其他"色 | `--color-chart-other` 亮 `#968b76` / 暗 `#a1937c`，叠加 45° 斜线纹理 | §5 |
| 热力图梯度 | `--color-chart-heat-0..4`，暖中性单色相 | heat-1..4 ordinal 校验 4 种表面 PASS；heat-0 对 4 种表面 ≥1.15:1（§6） |
| clay 复核（2026-09-24） | deepseek_harness 主色 `#635dab` → `#1e8dfe`；暗色 heat-0 `#29251d` → `#332f27`；zcode 暗色 `-text` `#42a0c2` → `#44a2c4` | §8 |

因为 9 色无法两两满足门槛，周趋势图必须带辅助编码：段间 `--space-0-5` 表面色间隙、图例（序列数 ≤4 时图例外加直接标注）、每段悬停提示、表格视图。这些是 frontend 实现的硬要求，写入 DESIGN.md。

## 2. 新来源身份色

搜索方法：在 OKLCH 网格（L 0.48-0.665，C 0.09-0.14，色相步长 5°）中选 4 色，最大化"与参考色集及彼此之间"的最小归一化距离 `min(正常ΔE/15, CVD ΔE/8)`。参考色集：claude、codex、opencode、antigravity、grok、gemini（= info `#7d97b6`）、success/warning/danger 亮暗两值、amber 强调色亮暗两值。约束：亮暗两表面对比度都 ≥3:1，所以一个值同时用于两主题（与现有平台色一致）。结果最小归一化距离 0.757。本节搜索只用了 neutral 两表面；clay 复核后 deepseek_harness 改值，见 §8。

| 来源 | token | 值 | OKLCH | 亮面对比 | 暗面对比 |
| --- | --- | --- | --- | --- | --- |
| kimi_code | `--color-platform-kimi-code` | `#b65790` | L0.59 C0.14 H345（玫红） | 4.12 | 3.88 |
| pi | `--color-platform-pi` | `#9773d0` | L0.63 C0.14 H300（紫） | 3.45 | 4.64 |
| zcode | `--color-platform-zcode` | `#1d83a4` | L0.57 C0.10 H225（青） | 4.05 | 3.95 |
| deepseek_harness（已替换） | `--color-platform-deepseek-harness` | ~~`#635dab`~~ → `#1e8dfe` | L0.52 C0.12 H285（靛）→ L0.645 C0.192 H254（天蓝） | 5.31 → 3.11 | 3.01 → 5.14 |

4 种表面的对比度见 §8.4。

未采用的方案：固定色族（青/玫红/靛/橄榄）最小距离 0.492，橄榄与 warning 亮值正常视觉 ΔE 7.4；固定色族（青/玫红/靛/紫）0.612。

## 3. 9 色两两低于门槛的配对

deepseek_harness 取 `#1e8dfe` 后的结果。原 `#635dab` 另有 kimi_code – deepseek_harness（14.9 / 6.3）、opencode – deepseek_harness（14.5 / 14.6）两对低于门槛，新值下这两对已过门槛。

| 配对 | 正常 ΔE | CVD ΔE | 说明 |
| --- | --- | --- | --- |
| antigravity – grok | 5.4 | 3.2 | 现有品牌色，本任务未改 |
| codex – antigravity | 11.1 | 9.3 | 现有品牌色 |
| codex – grok | 13.5 | 8.9 | 现有品牌色 |
| claude – codex | 17.5 | 5.6 | 现有品牌色 |
| kimi_code – pi | 11.4 | 9.8 | 新色 |
| pi – grok | 11.6 | 11.3 | |
| pi – deepseek_harness | 14.0 | 7.3 | 新色（原 `#635dab`：11.6 / 9.6） |
| zcode – deepseek_harness | 13.7 | 13.6 | 新色（原 `#635dab`：12.1 / 6.6） |
| claude – kimi_code | 14.7 | 14.6 | |
| opencode – zcode | 15.0 | 12.1 | |
| kimi_code – zcode | 21.0 | 6.2 | 新色 |
| pi – zcode | 16.1 | 7.1 | 新色 |

现有品牌色的校验结果（claude、codex、opencode、antigravity、grok，`--pairs all`）：色度下限 FAIL（codex 0.078、opencode 0.033、antigravity 0.046、grok 0.060）；暗色明度带 FAIL（claude 0.672、codex 0.696、antigravity 0.745、grok 0.714）；亮面对比 < 3:1 的有 claude 2.91、codex 2.45、antigravity 2.10、grok 2.41。修改现有品牌色会影响全应用，不在本任务范围内，在设计画布确认时提交用户决定。

## 4. 周趋势合并规则

- N = 4。取 53 周窗口内总量前 4 的来源独立着色（并列按 `SourceKind::ALL` 顺序），其余来源并为一段"其他"。依据：dataviz 规定 ≤4 个序列时图例外加直接标注；每周柱在 1280px 宽度下约 18px，5 段仍可辨认。
- 前 4 在整张图内固定，不按周重算。颜色跟随来源，不跟随名次。
- 堆叠顺序固定（自下而上）：claude、zcode、grok、deepseek_harness、antigravity、kimi_code、opencode、codex、pi，"其他"置顶。图例按同一顺序。
- 该顺序由穷举得出：对 C(9,4)=126 种前 4 组合，统计相邻段全部满足（正常 ≥15 且 CVD ≥8）的组合数。该顺序 49/126，`SourceKind::ALL` 顺序 22/126（deepseek_harness 为 `#635dab` 时）；deepseek_harness 改为 `#1e8dfe` 后重算：该顺序 65/126，`SourceKind::ALL` 顺序 26/126，堆叠顺序不变。9 色完整序列在该顺序下相邻对校验：CVD 最差 9.7、正常最差 16.0，4 种表面均 PASS（§7 B）。

## 5. "其他"段

候选（与 9 个来源的最差配对；最差配对按 §2 的归一化距离 `min(正常ΔE/15, CVD ΔE/8)` 取最小值）：

| 主题 | 候选 | 值 | 表面对比 | 最差配对 | 正常 ΔE | CVD ΔE |
| --- | --- | --- | --- | --- | --- | --- |
| 亮 | text-ghost | `#968b76` | 3.13 | claude | 11.4 | 4.1 |
| 亮 | border-strong | `#ab9f83` | 2.44 | codex | 6.9 | 1.2 |
| 亮 | text-muted | `#6b6150` | 5.67 | opencode | 1.5 | 0.4 |
| 暗 | text-muted | `#a1937c` | 5.70 | codex | 7.7 | 2.5 |
| 暗 | text-ghost | `#6f6552` | 2.99 | opencode | 1.8 | 0.9 |
| 暗 | text-secondary | `#c9bda8` | 9.25 | antigravity | 9.6 | 8.1 |

采用亮 `#968b76`、暗 `#a1937c`（对比度过 3:1，且与 opencode 可分）。中性灰与低色度品牌色的色相距离不足，所以"其他"段必须叠加 45° 斜线纹理（表面色条纹），并在图例中标"其他"。两主题取值来自不同的现有 token，所以新设 `--color-chart-other`，不复用 text token。

## 6. 热力图梯度

形式：53 周 × 7 天格子，按日总量分 5 档（0 档 = 无活动）。色相取正文墨色轨道（OKLCH H≈82°，C 0.025），为暖中性灰，不使用 amber（C≈0.13）和状态色，符合 Amber Scarcity Rule 与 Status-Only Color Rule。

| token | 亮 | 亮 neutral | 亮 clay | 暗 | 暗 neutral | 暗 clay |
| --- | --- | --- | --- | --- | --- | --- |
| `--color-chart-heat-0` | `#ebe5db` | 1.17 | 1.20 | `#332f27`（原 `#29251d`：1.12 / 1.02） | 1.29 | 1.17 |
| `--color-chart-heat-1` | `#b4ab9b` | 2.12 | 2.18 | `#5b5344` | 2.26 | 2.05 |
| `--color-chart-heat-2` | `#958d7d` | 3.07 | 3.16 | `#7a7263` | 3.60 | 3.28 |
| `--color-chart-heat-3` | `#776f60` | 4.63 | 4.77 | `#9b9383` | 5.63 | 5.12 |
| `--color-chart-heat-4` | `#5b5344` | 7.08 | 7.29 | `#beb5a4` | 8.44 | 7.68 |

- heat-0 只是格子底色，表示无活动，不承担数值，对比度可低于 2:1，但必须与卡片面可区分：门槛 ≥1.15:1（取亮 neutral 原值 1.17 附近），4 种表面都满足。heat-0..4 五级整体单调、相邻 ΔL ≥0.06、单色相（§7 D；该组的 Light-end 行检查的是 heat-0，按本条不适用 2:1）。
- heat-1..4 按 ordinal 门槛校验（单调、相邻 ΔL ≥0.06、第 1 级 ≥2:1、单色相）4 种表面 PASS（§7 C）。
- 每个主题一个取值同时用于 neutral 与 clay，clay 块不覆盖。
- heat-1 低于图形 3:1：替代通道是热力图的表格视图（每周一行、周一至周日各一列的完整请求数，DESIGN.md Activity heatmap 条目）。每格另有 `title`/悬停提示给出日期与数值，图例标"少→多"。按 dataviz 规则，悬停提示不能单独作为替代通道。

## 7. 校验器原始输出

2026-09-24 clay 复核后重跑（deepseek_harness `#1e8dfe`、暗色 heat-0 `#332f27`），每组覆盖 4 种表面。命令：`node --no-warnings validate_palette.js "<hex,...>" --mode <light|dark> --surface <hex> [--pairs all | --ordinal]`。

```text
### A. 新增 4 色（kimi_code, pi, zcode, deepseek_harness），--pairs all

Palette (light, surface #faf7ec, categorical): 4 slots
  [PASS] Lightness band         all 4 inside L 0.43–0.77
  [PASS] Chroma floor           all 4 >= 0.1
  [WARN] CVD separation         worst all-pairs #1d83a4↔#b65790 ΔE 6.2 (protan) · tritan 9.2
  [FAIL] Normal-vision floor    worst all-pairs #9773d0↔#b65790 ΔE 11.4 (normal) — below 15, hard to tell apart even with full color vision
  [PASS] Contrast vs surface    all 4 >= 3:1

  → FAILED — fix the marked checks  (CVD in the 6–8 floor band is legal ONLY with secondary encoding: direct labels, gaps, or texture)
  scope: categorical palettes only. For a lone status/text color check WCAG text contrast; for a sequential ramp, lightness monotonicity.

exit=1

Palette (light, surface #fefaf2, categorical): 4 slots
  [PASS] Lightness band         all 4 inside L 0.43–0.77
  [PASS] Chroma floor           all 4 >= 0.1
  [WARN] CVD separation         worst all-pairs #1d83a4↔#b65790 ΔE 6.2 (protan) · tritan 9.2
  [FAIL] Normal-vision floor    worst all-pairs #9773d0↔#b65790 ΔE 11.4 (normal) — below 15, hard to tell apart even with full color vision
  [PASS] Contrast vs surface    all 4 >= 3:1

  → FAILED — fix the marked checks  (CVD in the 6–8 floor band is legal ONLY with secondary encoding: direct labels, gaps, or texture)
  scope: categorical palettes only. For a lone status/text color check WCAG text contrast; for a sequential ramp, lightness monotonicity.

exit=1

Palette (dark, surface #1f1b14, categorical): 4 slots
  [PASS] Lightness band         all 4 inside L 0.48–0.67
  [PASS] Chroma floor           all 4 >= 0.1
  [WARN] CVD separation         worst all-pairs #1d83a4↔#b65790 ΔE 6.2 (protan) · tritan 9.2
  [FAIL] Normal-vision floor    worst all-pairs #9773d0↔#b65790 ΔE 11.4 (normal) — below 15, hard to tell apart even with full color vision
  [PASS] Contrast vs surface    all 4 >= 3:1

  → FAILED — fix the marked checks  (CVD in the 6–8 floor band is legal ONLY with secondary encoding: direct labels, gaps, or texture)
  scope: categorical palettes only. For a lone status/text color check WCAG text contrast; for a sequential ramp, lightness monotonicity.

exit=1

Palette (dark, surface #2a221e, categorical): 4 slots
  [PASS] Lightness band         all 4 inside L 0.48–0.67
  [PASS] Chroma floor           all 4 >= 0.1
  [WARN] CVD separation         worst all-pairs #1d83a4↔#b65790 ΔE 6.2 (protan) · tritan 9.2
  [FAIL] Normal-vision floor    worst all-pairs #9773d0↔#b65790 ΔE 11.4 (normal) — below 15, hard to tell apart even with full color vision
  [PASS] Contrast vs surface    all 4 >= 3:1

  → FAILED — fix the marked checks  (CVD in the 6–8 floor band is legal ONLY with secondary encoding: direct labels, gaps, or texture)
  scope: categorical palettes only. For a lone status/text color check WCAG text contrast; for a sequential ramp, lightness monotonicity.

exit=1
### B. 9 个来源按堆叠顺序（claude, zcode, grok, deepseek_harness, antigravity, kimi_code, opencode, codex, pi），adjacent

Palette (light, surface #faf7ec, categorical): 9 slots
  [PASS] Lightness band         all 9 inside L 0.43–0.77
  [FAIL] Chroma floor           below floor (reads gray): [["#a79bc4",0.06],["#98afc9",0.046],["#735f52",0.033],["#7cab82",0.078]]
  [PASS] CVD separation         worst adjacent #735f52↔#b65790 ΔE 9.7 (protan) · tritan 10.1
  [PASS] Normal-vision floor    worst adjacent #735f52↔#b65790 ΔE 16.0 (normal)
  [WARN] Contrast vs surface    below 3:1 — relief required (visible labels or table view): [["#d97757",2.91],["#a79bc4",2.41],["#98afc9",2.1],["#7cab82",2.45]]

  → FAILED — fix the marked checks  (CVD in the 6–8 floor band is legal ONLY with secondary encoding: direct labels, gaps, or texture)
  scope: categorical palettes only. For a lone status/text color check WCAG text contrast; for a sequential ramp, lightness monotonicity.

exit=1

Palette (light, surface #fefaf2, categorical): 9 slots
  [PASS] Lightness band         all 9 inside L 0.43–0.77
  [FAIL] Chroma floor           below floor (reads gray): [["#a79bc4",0.06],["#98afc9",0.046],["#735f52",0.033],["#7cab82",0.078]]
  [PASS] CVD separation         worst adjacent #735f52↔#b65790 ΔE 9.7 (protan) · tritan 10.1
  [PASS] Normal-vision floor    worst adjacent #735f52↔#b65790 ΔE 16.0 (normal)
  [WARN] Contrast vs surface    below 3:1 — relief required (visible labels or table view): [["#d97757",3],["#a79bc4",2.48],["#98afc9",2.17],["#7cab82",2.52]]

  → FAILED — fix the marked checks  (CVD in the 6–8 floor band is legal ONLY with secondary encoding: direct labels, gaps, or texture)
  scope: categorical palettes only. For a lone status/text color check WCAG text contrast; for a sequential ramp, lightness monotonicity.

exit=1

Palette (dark, surface #1f1b14, categorical): 9 slots
  [FAIL] Lightness band         outside band: [["#d97757",0.672],["#a79bc4",0.714],["#98afc9",0.745],["#7cab82",0.696]]
  [FAIL] Chroma floor           below floor (reads gray): [["#a79bc4",0.06],["#98afc9",0.046],["#735f52",0.033],["#7cab82",0.078]]
  [PASS] CVD separation         worst adjacent #735f52↔#b65790 ΔE 9.7 (protan) · tritan 10.1
  [PASS] Normal-vision floor    worst adjacent #735f52↔#b65790 ΔE 16.0 (normal)
  [WARN] Contrast vs surface    below 3:1 — relief required (visible labels or table view): [["#735f52",2.85]]

  → FAILED — fix the marked checks  (CVD in the 6–8 floor band is legal ONLY with secondary encoding: direct labels, gaps, or texture)
  scope: categorical palettes only. For a lone status/text color check WCAG text contrast; for a sequential ramp, lightness monotonicity.

exit=1

Palette (dark, surface #2a221e, categorical): 9 slots
  [FAIL] Lightness band         outside band: [["#d97757",0.672],["#a79bc4",0.714],["#98afc9",0.745],["#7cab82",0.696]]
  [FAIL] Chroma floor           below floor (reads gray): [["#a79bc4",0.06],["#98afc9",0.046],["#735f52",0.033],["#7cab82",0.078]]
  [PASS] CVD separation         worst adjacent #735f52↔#b65790 ΔE 9.7 (protan) · tritan 10.1
  [PASS] Normal-vision floor    worst adjacent #735f52↔#b65790 ΔE 16.0 (normal)
  [WARN] Contrast vs surface    below 3:1 — relief required (visible labels or table view): [["#735f52",2.59]]

  → FAILED — fix the marked checks  (CVD in the 6–8 floor band is legal ONLY with secondary encoding: direct labels, gaps, or texture)
  scope: categorical palettes only. For a lone status/text color check WCAG text contrast; for a sequential ramp, lightness monotonicity.

exit=1
### C. 热力图梯度 heat-1..4，--ordinal

Palette (light, surface #faf7ec, ordinal ramp): 4 slots
  [PASS] Lightness monotone     steps read light→dark
  [PASS] Adjacent ΔL            all gaps >= 0.06
  [PASS] Light-end contrast     #b4ab9b at 2.12:1 vs surface
  [PASS] Single hue             hue spread 2°

  → ALL CHECKS PASS  (ordinal: one hue, monotone L, visible step gaps, light end clears surface)
exit=0

Palette (light, surface #fefaf2, ordinal ramp): 4 slots
  [PASS] Lightness monotone     steps read light→dark
  [PASS] Adjacent ΔL            all gaps >= 0.06
  [PASS] Light-end contrast     #b4ab9b at 2.18:1 vs surface
  [PASS] Single hue             hue spread 2°

  → ALL CHECKS PASS  (ordinal: one hue, monotone L, visible step gaps, light end clears surface)
exit=0

Palette (dark, surface #1f1b14, ordinal ramp): 4 slots
  [PASS] Lightness monotone     steps read light→dark
  [PASS] Adjacent ΔL            all gaps >= 0.06
  [PASS] Light-end contrast     #5b5344 at 2.26:1 vs surface
  [PASS] Single hue             hue spread 1°

  → ALL CHECKS PASS  (ordinal: one hue, monotone L, visible step gaps, light end clears surface)
exit=0

Palette (dark, surface #2a221e, ordinal ramp): 4 slots
  [PASS] Lightness monotone     steps read light→dark
  [PASS] Adjacent ΔL            all gaps >= 0.06
  [PASS] Light-end contrast     #5b5344 at 2.05:1 vs surface
  [PASS] Single hue             hue spread 1°

  → ALL CHECKS PASS  (ordinal: one hue, monotone L, visible step gaps, light end clears surface)
exit=0
### D. 热力图梯度 heat-0..4（含无活动格），--ordinal。Light-end 行检查的是 heat-0；heat-0 不承担数值，按 §6 改用 ≥1.15:1 门槛，该行 FAIL 为预期

Palette (light, surface #faf7ec, ordinal ramp): 5 slots
  [PASS] Lightness monotone     steps read light→dark
  [PASS] Adjacent ΔL            all gaps >= 0.06
  [FAIL] Light-end contrast     #ebe5db at 1.17:1 vs surface — below 2:1 floor
  [PASS] Single hue             hue spread 4°

  → FAILED — fix the marked checks  (ordinal: one hue, monotone L, visible step gaps, light end clears surface)
exit=1

Palette (light, surface #fefaf2, ordinal ramp): 5 slots
  [PASS] Lightness monotone     steps read light→dark
  [PASS] Adjacent ΔL            all gaps >= 0.06
  [FAIL] Light-end contrast     #ebe5db at 1.20:1 vs surface — below 2:1 floor
  [PASS] Single hue             hue spread 4°

  → FAILED — fix the marked checks  (ordinal: one hue, monotone L, visible step gaps, light end clears surface)
exit=1

Palette (dark, surface #1f1b14, ordinal ramp): 5 slots
  [PASS] Lightness monotone     steps read light→dark
  [PASS] Adjacent ΔL            all gaps >= 0.06
  [FAIL] Light-end contrast     #332f27 at 1.29:1 vs surface — below 2:1 floor
  [PASS] Single hue             hue spread 1°

  → FAILED — fix the marked checks  (ordinal: one hue, monotone L, visible step gaps, light end clears surface)
exit=1

Palette (dark, surface #2a221e, ordinal ramp): 5 slots
  [PASS] Lightness monotone     steps read light→dark
  [PASS] Adjacent ΔL            all gaps >= 0.06
  [FAIL] Light-end contrast     #332f27 at 1.17:1 vs surface — below 2:1 floor
  [PASS] Single hue             hue spread 1°

  → FAILED — fix the marked checks  (ordinal: one hue, monotone L, visible step gaps, light end clears surface)
exit=1
```

## 8. clay 复核（2026-09-24）

### 8.1 复核发现

前面各节只用了 neutral 两表面。按 `theme-token-contracts.md`（4 种表面组合）补测 clay 后，有 3 处问题：

| token | 原值 | 问题 |
| --- | --- | --- |
| `--color-platform-deepseek-harness` | `#635dab` | 对暗 clay `#2a221e` 2.74:1，低于 3:1；对暗 neutral 3.007:1，余量 0.007 |
| `--color-chart-heat-0`（暗） | `#29251d` | 对暗 clay `#2a221e` 1.02:1，无活动格与面板无法区分（OKLCH L 0.266 对 0.260） |
| `--color-platform-zcode-text`（暗） | `#42a0c2` | 对 `-surface` `#1f3234` 4.5004:1，余量 0.0004 |

### 8.2 热力图 heat-0（暗）

- 取值 `#332f27`（OKLCH L0.307 C0.015 H85），比两种暗色卡片面都亮：对暗 neutral 1.29:1，对暗 clay 1.17:1，均 ≥1.15:1。
- 对 heat-1 `#5b5344` 的 ΔL 为 0.139（≥0.06），五级单调、单色相（§7 D）。
- heat-1..4 不变，暗 clay 上 heat-1 为 2.05:1（≥2:1，§7 C）。
- 一个取值同时满足两种 flavor，所以不在 `[data-theme='dark'][data-flavor='clay']` 块内覆盖。亮色梯度在亮 clay `#fefaf2` 上原值即通过，不改。

### 8.3 deepseek_harness 主色

约束：

- 对 4 种卡片面都 ≥3:1（搜索时留余量，取 ≥3.1:1）。换算为相对亮度 Y 在 0.159（暗 clay 下限）到 0.266（亮 neutral 上限）之间。
- 明度带 L 0.48–0.67、色度 ≥0.10（dataviz 两主题带宽的交集）。
- 不劣化：与参考色集（§2）及其他 3 个新色的最小归一化距离 ≥0.773（原值）；最差正常视觉 ΔE ≥11.6、最差 CVD ΔE（protan/deutan）≥6.3（原值）。
- §7 B 不劣化：与堆叠相邻的 grok、antigravity 正常 ΔE ≥16.0、CVD ΔE ≥9.7（保持 §7 B 最差相邻对不变）。

按色度上限分档的最优结果（OKLCH 网格。C ≤0.14 与 C ≤0.16 两档：L/C 步长 0.005，H 250°-300° 步长 2.5°；其余两档：L/C 步长 0.0025，H 230°-320° 步长 1°）：

| C 上限 | 最优值 | 最小归一化距离 | 满足全部约束 |
| --- | --- | --- | --- |
| 0.14（§2 原搜索范围） | `#5b67bb`（H275） | 0.693 | 否（低于 0.773，最近 zcode 10.4） |
| 0.16 | `#4e67c9`（H270） | 0.741 | 否 |
| 0.175 | `#4c66d7`（H270） | 0.788 | 是，但 tritan 对 zcode 4.2 |
| 0.20 | `#1e8dfe`（H254） | 0.912 | 是 |

采用 `#1e8dfe`。原搜索范围（C ≤0.14）内不存在满足"4 种表面 ≥3:1 且不劣化"的值；靛色相（H 270-285）在满足 3:1 的明度下与 pi、zcode 距离不足。

新旧对比（对象：参考色集 + 其他 3 个新色；最差相邻对取 §7 B）：

| 指标 | `#635dab`（原） | `#1e8dfe`（新） |
| --- | --- | --- |
| 最小归一化距离 | 0.773（pi） | 0.912（zcode） |
| 最差正常视觉 ΔE | 11.6（pi） | 13.7（zcode） |
| 最差 CVD ΔE（protan/deutan） | 6.3（kimi_code） | 7.3（pi） |
| 最差 tritan ΔE（全部参考色） | 8.0（opencode） | 6.2（gemini） |
| §7 A tritan（4 个新色两两） | 9.5 | 9.2 |
| §7 B 最差相邻对（正常 / CVD / tritan） | 16.0 / 9.7 / 10.4 | 16.0 / 9.7 / 10.1 |
| 与 grok / antigravity（正常 / CVD） | 20.3 / 19.7，24.0 / 24.3 | 16.9 / 11.9，17.8 / 15.2 |
| 低于门槛的配对（§3）中含 deepseek_harness 的数量 | 4 | 2 |
| 前 4 组合相邻全过门槛（堆叠顺序 / `SourceKind::ALL`） | 49 / 22 | 65 / 26 |
| 4 种表面对比度最小值 | 2.74 | 3.11 |

- tritan 不是校验器的门槛项（校验器只以 protan/deutan 判定，tritan 仅报告）。新值的 tritan 指标低于原值（全部参考色 8.0 → 6.2，§7 A 9.5 → 9.2，§7 B 10.4 → 10.1）；这是本次调整的已知代价。周趋势已有的辅助编码（段间间隙、图例、悬停提示、表格视图）覆盖该风险。
- "其他"色（§5）的最差配对不变（亮 claude 11.4 / 4.1，暗 codex 7.7 / 2.5）；与新 deepseek_harness 的距离为 22.5 / 20.6（亮）、23.0 / 20.5（暗）。
- 色度 0.192 高于其他 3 个新色（0.10-0.14），色相从靛（285°）移到天蓝（254°）。DESIGN.md 中名称改为 "DeepSeek Harness azure"。

`-surface` / `-border` 按原方法重算（卡片面与主色在 sRGB 中按比例混合）；`-text` 保持色相，降低（亮）或升高（暗）OKLCH L，超出 sRGB 色域时按 0.005 步长降低色度，直到对 `-surface` 与两种 flavor 的卡片面都 ≥4.55:1。

| 主题 | `-surface` | `-border` | `-text` |
| --- | --- | --- | --- |
| 亮 | `#e0eaee`（12%） | `#b4d5f2`（32%） | `#0567c2`（L0.517 C0.163） |
| 暗 | `#1f3447`（22%） | `#1f4972`（40%） | `#4b9dfc`（L0.689 C0.162） |

`-rgb` 为 `30 141 254`。

### 8.4 最终取值与 4 种表面对比度

来源主色（图形 ≥3:1）：

| 来源 | 主色 | 亮 neutral `#faf7ec` | 亮 clay `#fefaf2` | 暗 neutral `#1f1b14` | 暗 clay `#2a221e` |
| --- | --- | --- | --- | --- | --- |
| kimi_code | `#b65790` | 4.12 | 4.24 | 3.88 | 3.53 |
| pi | `#9773d0` | 3.45 | 3.55 | 4.64 | 4.22 |
| zcode | `#1d83a4` | 4.05 | 4.17 | 3.95 | 3.59 |
| deepseek_harness | `#1e8dfe` | 3.11 | 3.20 | 5.14 | 4.68 |

`-text`（文本 ≥4.5:1；clay 下 `-surface` 不重定义，取值同 neutral）：

| 来源 | 亮 `-text` | 对 `-surface` | 对 `#faf7ec` | 对 `#fefaf2` | 暗 `-text` | 对 `-surface` | 对 `#1f1b14` | 对 `#2a221e` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| kimi_code | `#a3467f` | 4.53 | 5.24 | 5.39 | `#d775ae` | 4.51 | 5.75 | 5.23 |
| pi | `#7b56b0` | 4.54 | 5.15 | 5.31 | `#ac88e7` | 4.55 | 6.06 | 5.52 |
| zcode | `#007091` | 4.54 | 5.25 | 5.41 | `#44a2c4` | 4.61 | 5.89 | 5.36 |
| deepseek_harness | `#0567c2` | 4.61 | 5.26 | 5.41 | `#4b9dfc` | 4.59 | 6.14 | 5.59 |

图表色：

| token | 亮 | 亮 neutral | 亮 clay | 暗 | 暗 neutral | 暗 clay |
| --- | --- | --- | --- | --- | --- | --- |
| `--color-chart-other` | `#968b76` | 3.13 | 3.23 | `#a1937c` | 5.70 | 5.19 |
| `--color-chart-heat-0` | `#ebe5db` | 1.17 | 1.20 | `#332f27` | 1.29 | 1.17 |
| `--color-chart-heat-1` | `#b4ab9b` | 2.12 | 2.18 | `#5b5344` | 2.26 | 2.05 |
| `--color-chart-heat-2` | `#958d7d` | 3.07 | 3.16 | `#7a7263` | 3.60 | 3.28 |
| `--color-chart-heat-3` | `#776f60` | 4.63 | 4.77 | `#9b9383` | 5.63 | 5.12 |
| `--color-chart-heat-4` | `#5b5344` | 7.08 | 7.29 | `#beb5a4` | 8.44 | 7.68 |
