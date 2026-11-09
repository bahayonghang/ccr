# 对比度与 OKLCH 分层计算记录（步骤 2-7 实现时落盘）

- 计算脚本: `contrast-check.mjs`（同目录，`bun contrast-check.mjs` 可复现）
- 公式: WCAG 2.x relative luminance + 对比度比值；OKLCH L 为标准 OKLab L×100
- 日期: 2026-07-07

## 关键裁决

design.md §2 给出的亮色 clay 具体 hex（base `#efe6d8` / elevated `#f7f0e5` / surface `#fdf8ef`）
实测 OKLCH ΔL = 2.94 / 2.29、span 5.23，不满足 PRD R1 的 相邻层 ΔL ≥ 3、base↔surface ≥ 6。
保持暖米色相与"卡片最亮、桌面压暗"的设计意图不变，微调为：

| 层 | design.md | 实际落地 | OKLCH L |
| --- | --- | --- | --- |
| base | #efe6d8 | **#ebe1d0** | 91.31 |
| elevated | #f7f0e5 | **#f5eee1** | 95.11 |
| surface | #fdf8ef | **#fefaf2** | 98.60 |
| overlay | #e6dac9 | **#e2d6c3** | 88.07 |

ΔL：base→elevated 3.80，elevated→surface 3.49，base↔surface 7.29。全部达标。

文字色：secondary 保持 `#5f4d3f`（design.md 说"适度加深，目标 ≥4.5:1 vs surface"——
在新背景梯度下实测 7.70:1，已远超目标，不再加深以避免不必要的色相漂移）；
muted 从 `#7f6a5b` 加深为 `#715d4c`（原值对 base 只有 ~4.1:1）。

## 全矩阵结果（vs surface / elevated / base）

### light clay（新）
- primary #31241c: 14.41 / 13.00 / 11.58（目标 ≥7）PASS
- secondary #5f4d3f: 7.70 / 6.95 / 6.19（目标 ≥4.5）PASS
- muted #715d4c: 5.98 / 5.40 / 4.81 PASS

### light paper（新: #e7e7e7/#f2f2f2/#fdfdfd/#dbdbdb, L=92.80/96.12/99.40, ΔL=3.32/3.29, span 6.61）
- primary #1a1a1c: 17.08 / 15.52 / 14.05 PASS
- secondary #3f3f46: 10.27 / 9.33 / 8.45 PASS
- muted #626268（由 #6b6b70 加深）: 5.96 / 5.41 / 4.90 PASS

### light graphite（新: #e4e4e9/#f0f0f4/#fbfbfd/#d8d8de, L=92.03/95.63/98.86, ΔL=3.60/3.24, span 6.83）
- primary #1f2024: 15.75 / 14.32 / 12.84 PASS
- secondary #43464c: 9.16 / 8.33 / 7.47 PASS
- muted #5f636c（由 #6a6e76 加深）: 5.83 / 5.30 / 4.75 PASS

### dark clay（背景层保持，ΔL=4.21/3.04 ≥2.5 已达标；只上调边框）
- primary #f3eadf: 13.11 PASS；secondary #dacbbc: 9.84 PASS；muted #b9a695: 6.65 PASS

### dark paper（保持: ΔL=3.83/2.92 ≥2.5）
- primary 12.80 / secondary 9.68 / muted 5.57 全 PASS

### dark graphite（保持: ΔL=3.91/3.39 ≥2.5）
- primary 12.87 / secondary 9.11 / muted 5.44 全 PASS

## 边框与阴影档位（对照 PRD 下限）

- 亮色三 flavor：subtle 12%（≥12 ✓）、default 19%（≥18 ✓）、strong 30%（≥28 ✓），各自色相基（clay 暖褐 70 53 41 / paper 20 20 22 / graphite 31 32 36）
- 暗色三 flavor：subtle 10%（+2）、default 16%（≥16 ✓）、strong 26%（≥26 ✓）
- 阴影（亮色暖褐 tint 73 54 40）：xs 6% / sm 9%（≥8 ✓）/ md 13%（≥12 ✓）/ lg 16% / xl 19% / 2xl 22%
- Catppuccin 语义边框映射未动（surface2@58% 等，远高于下限）；只微调 glass 透明度档：
  frappe/macchiato 46/58/70 → 54/66/80，mocha 44/56/68 → 52/64/78，latte 保持 72/84/92
