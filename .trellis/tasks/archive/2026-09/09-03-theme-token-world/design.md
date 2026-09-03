# Design — 主题 Token 体系与配色世界替换

> 方向契约：行情终端（`ccr-ui/.impeccable/surfaces/ui-src-features-usage-dashboard-dashboardview-tsx.md`）。本设计把契约配色翻译成 `tokens.css` 的具体值，所有值必须过 `theme-contrast-contract.smoke.test.ts` 阈值门禁——**只调值，不动阈值**。

## 边界

- 本任务只动**配色与 token 值**：不动页面结构、不动组件、不动字体轨、不动玻璃/阴影体系（终端世界的结构改造归 overview/settings 子任务）。
- 层级纪律：Layer-1 值全在 `tokens.css`；`core.css` 的 `@theme inline` 只引用 var()，本任务**不需要改 core.css**（名字不变，只改值）。
- `data-accent` 值域保持 `'clay'` 不变（migration map 里 `amber → clay` 已存在，重新引入 'amber' 值会与存量迁移冲突）。只改 'clay' 这个 key 解析到的颜色值。命名与色值的语义错位在 spec 注释中说明。

## Token 值映射（old → new）

### 1. 中性深色暖化（`[data-theme='dark']`，tokens.css:161-282）

背景坡道（保持明度单调递增公理）：
| token | old（冷灰） | new（暖黑终端） |
|---|---|---|
| `--color-bg-base` | `#131316` | `#100f0c` |
| `--color-bg-elevated` | `#1a1b1f` | `#171410` |
| `--color-bg-surface` | `#22242a` | `#1f1b14` |
| `--color-bg-overlay` | `#2c2f37` | `#2a251b` |

文字（对 surface `#1f1b14` 的估算对比：primary ≈13:1 ✓≥12，secondary ≈8.5:1 ✓≥7，muted ≈5.7:1 ✓≥4.5）：
| token | old | new |
|---|---|---|
| `--color-text-primary` | `#f2f3f5` | `#e9e1d1` |
| `--color-text-secondary` | `#c9ccd3` | `#c9bda8` |
| `--color-text-muted` | `#9ba1ab` | `#a1937c` |
| `--color-text-ghost` | `#6d727c` | `#6f6552` |
| `--color-text-disabled` | `#4f545d` | `#564e3e` |
| `--color-text-inverted` | `#17181c` | `#1d1408` |

边框暖化（保持三级递进）：subtle `#37393d`→`#332c21`，default `#48494e`→`#453b2a`，strong `#616368`→`#5d5138`。
功能色 tint 暖化：success-tint `#2d3435`→`#26301f`，warning-tint `#383432`→`#362c1a`，danger-tint `#383033`→`#331f16`，info-tint `#30353d`→`#232b31`。
scrim 保持黑透明不变。

### 2. 强调色：clay key → 终端琥珀（`[data-accent='clay']` × 2 + `:root`/`dark` 默认块同步）

| 位置 | old | new |
|---|---|---|
| light `--color-accent-primary` | `#cf6239` | `#8f650e`（深琥珀，对浅 surface ≈5.5:1） |
| light hover / active | `#d9714a`/`#b8542f` | `#a2740f` / `#7c580b` |
| light contrast | `#fff8f2` | `#fffaf0` |
| dark `--color-accent-primary` | `#e8835b` | `#f0a32b`（契约琥珀，对 dark surface ≈7.9:1） |
| dark hover / active | `#f0926c`/`#d4744a` | `#f5b14a` / `#d98f1d` |
| dark contrast | `#1d1207` | `#1d1204` |

`--color-accent-primary-rgb`、`-glow`（保持 10%/16% 配方）、`--color-border-accent`（18%/24%）按新 rgb 同步。**四处定义点必须一致**：`:root`、`[data-theme='dark']`、`[data-accent='clay']`、`[data-theme='dark'][data-accent='clay']`（accent 块靠后胜出，但默认值块保持同步以免困惑）。

辅助强调色 `--color-accent-secondary`（暖沙）保留角色，值随新世界微调：light `#a0854f`→ 保持观感；dark `#d0ae86`→`#c9a35f`。若对比门禁失败则以测试为准微调。

### 3. 功能色对齐终端语义（dark 块）

| token | old | new |
|---|---|---|
| `--color-success` (dark) | `#7cab82` | `#5fa05a`（契约绿） |
| `--color-danger` (dark) | `#db8a73` | `#cc5b45`（契约砖红提亮保 ≥3.5） |
| `--color-warning` (dark) | `#d6a76d` | `#d9c05a`（偏黄，与琥珀强调色拉开 hue 距离） |
| `--color-info` (dark) | `#98afc9` | `#7d94b0` |
| 对应 `-contrast` | 现有深墨 | `#17181c`→ 保持深墨，按新值过 ≥3.5 门禁微调 |

light 功能色仅微调观感（success `#5b8a62` 保持、warning `#bc8540`→`#a07c1e` 与琥珀拉开、danger `#c76953` 保持、info `#7d97b6` 保持），同样以门禁为准。

### 4. 平台色确权

- 值不变（六平台 dot/surface/border/text 四角色已在 `tokens.css:107-137,249-279` 定义齐全），**改消费端**：
  - `features/usage/styles/dashboard-usage-movement.css:123-125,200-202`：图表段与图例 `[data-platform='antigravity']` → `var(--color-platform-antigravity)`（不再用 gemini）。
  - `features/usage/styles/dashboard-platform-matrix.css:51-52`：平台卡 accent 映射同上。
  - `features/usage/dashboard/DashboardView.tsx:164`：`text-platform-gemini` → `text-platform-antigravity`（确认 core.css 有对应 utility；若无则经 `@theme inline` 既有 platform 组补齐引用）。
  - `shell/MainLayoutNav.tsx:15`：导航色样 `bg-platform-gemini` → `bg-platform-antigravity`。
- 结果：dark 下 antigravity `#98afc9` ≠ gemini `#7d97b6`，两个蓝色各归其主。

### 5. 图表 ramp（`styles/chart-colors.css:9-24`）

`--chart-color-0..4` 现值 `#e8835b/#7cab82/#d6a76d/#98afc9/#db8a73`（dark）随新功能色同步：accent-amber `#f0a32b`、success `#5fa05a`、warning `#d9c05a`、info `#7d94b0`、danger `#cc5b45`；light 对应同步。注意：此 ramp 服务 ApexCharts 视图，与首页手写堆叠柱（平台色）是两套，互不替代。

### 6. 启动 Loader（`ccr-ui/index.html:60-62`）

- dark 背景 `#000000` → `#100f0c`（新 base）；spinner `#2997ff` → `#f0a32b`（琥珀）。
- light 底色如有硬编同样对齐新 light base `#e9e4d8`。
- IIFE 的 theme/flavor/accent 迁移表与白名单**一个字节都不动**（契约：与 themeBootstrap.ts 行为字节等价）。

### 7. 浅色中性族暖化（`:root`，tokens.css:9-47）

冷灰 `#e8e9ec/#f2f3f5/#fbfcfd/#dcdee3` → 暖纸面：`#e9e4d8/#f2eee3/#faf7ec/#ddd5c2`。
文字暖化：primary `#191b20`→`#211c12`，secondary `#3f434c`→`#4a4232`，muted `#5f646e`→`#6b6150`，ghost `#878d98`→`#968b76`，disabled `#b3b8c0`→`#b5a98f`，inverted `#f7f8fa`→`#fbf7ec`。
边框：subtle `#d8d9db`→`#d9d2c0`，default `#c9cacd`→`#c9bfa8`，strong `#b1b2b5`→`#ab9f83`。
（对比估算：primary vs surface ≈15:1 ✓，muted ≈5.9:1 ✓。）
clay flavor 两组（`tokens.css:601-666`）保持现有暖纸/暖墨 identity 不动——它本来就是暖的；新 neutral 与 clay 的差异从"冷 vs 暖"变为"暖纸灰 vs 暖陶棕"的色度差异。

### 8. 玻璃/阴影

不动结构；仅确认暗色旧玻璃档（`tokens.css:504-527`）里硬编的 `rgb(34 27 24 …)`、`rgb(42 34 30 …)` 等暖棕残留与新面板色不冲突（deprecated 档保持薄值原则，允许不动）。`--shadow-inner` 高光 `rgb(235 238 245 / 3%)` 冷调 → `rgb(243 234 220 / 3%)` 暖调微调（dark 块）。

## 死代码评审结论（spec 有记载的 API，不动）

- `applyCustomAccent/clearCustomAccent`（`themeBootstrap.ts:421-453`）：`theme-token-contracts.md` batch 5 契约登记在册（归 `08-22-shell-port` R6 接线）。**保留**，在 spec 中补注"当前无调用方，接线任务未落地"。
- `data-resolved-flavor`：spec 明文 `equals data-flavor`。**保留**，同样补注 vestigial。
- 理由：两者都是契约面，删除收益小、契约变更成本高；本任务只做值替换。

## 测试锚点更新（允许改锚点值，禁止降阈值）

- `tests/theme/theme-switch.smoke.test.tsx` 的 per-theme computed-value 断言：按新 palette 更新锚点 hex。
- `tests/theme/theme-contrast-contract.smoke.test.ts`：阈值常量不动；若新值不达标，调 token 值。
- `tests/theme/theme-bootstrap.smoke.test.ts` / `theme-domain-extension.smoke.test.tsx` / `apple-glass-surface-contract.smoke.test.ts`：应保持绿（值域与 key 未变）；若断言了具体色值则同步锚点。

## 兼容与回滚

- 存储 key（`ccr-theme`/`ccr-flavor`/`ccr-accent`）与值域不变 → 老用户无迁移；回滚 = git revert 本任务提交即恢复旧配色，无状态残留。
- spec 更新：`theme-token-contracts.md` 的值表叙述、死代码注记；不改契约条款与阈值。

## 验证命令

```bash
cd ccr-ui && bun run type-check && bun run lint
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme/
cd ccr-ui && bun run test && bun run build && bun run tauri:check
# 视觉验收：bun run dev:web -- --host 127.0.0.1 --strictPort，四组合截图评审
```
