# 技术设计:玻璃材质令牌体系与对比度修复

## 1. 边界

只改:`ccr-ui/src/styles/tokens.css`、`ccr-ui/src/styles/home.css`、`ccr-ui/src/styles/utilities.css`(新增 utility)、相关 smoke tests。
不改:任何 .vue 组件、theme bootstrap 逻辑、flavor/accent 枚举。

## 2. 新色值基线(亮色 clay)

保持暖米色相(hue ≈ 70°~75° OKLCH),拉开明度:

```css
:root {
  --color-bg-base: #efe6d8;      /* L≈92,压暗做"桌面" */
  --color-bg-elevated: #f7f0e5;  /* L≈95 */
  --color-bg-surface: #fdf8ef;   /* L≈98,卡片最亮 */
  --color-bg-overlay: #e6dac9;   /* L≈89,sunk 区域 */

  --color-border-subtle: rgb(70 53 41 / 12%);
  --color-border-default: rgb(70 53 41 / 19%);
  --color-border-strong: rgb(70 53 41 / 30%);

  --shadow-sm: 0 2px 6px rgb(73 54 40 / 9%);
  --shadow-md: 0 10px 24px rgb(73 54 40 / 13%);
  --shadow-lg: 0 18px 38px rgb(73 54 40 / 16%);
}
```

要点:**卡片比底亮**(surface 最亮、base 压暗),这与现状方向一致但差距拉大;泛白的观感来自"全部一样亮",不是"太亮"本身。文字色 `--color-text-secondary` 从 #5f4d3f 适度加深(实作时用对比度工具定值,目标 ≥4.5:1 vs surface)。暗色主题对比已达标,只微调 border/subtle +2%。

## 3. 三档 material 令牌

```css
:root {
  /* floating: 模态 / 命令面板 / 浮动操作条(同屏最多 1) */
  --material-glass-floating-bg: rgb(var(--color-bg-elevated-rgb) / 72%);
  --material-glass-floating-blur: blur(16px) saturate(170%);
  --material-glass-floating-border: rgb(var(--color-border-strong-rgb) / 26%);
  --material-glass-floating-highlight: inset 0 1px 0 rgb(255 251 245 / 55%);
  --material-glass-floating-shadow: 0 24px 64px rgb(73 54 40 / 20%);

  /* chrome: 侧栏 / 顶栏(常驻,同屏 ≤2) */
  --material-glass-chrome-bg: rgb(var(--color-bg-elevated-rgb) / 82%);
  --material-glass-chrome-blur: blur(10px) saturate(150%);
  --material-glass-chrome-border: rgb(var(--color-border-default-rgb) / 22%);
  --material-glass-chrome-highlight: inset 0 1px 0 rgb(255 251 245 / 45%);
  --material-glass-chrome-shadow: 0 10px 30px rgb(73 54 40 / 10%);

  /* inline: 页面内悬浮条(sticky 工具条等,少用) */
  --material-glass-inline-bg: rgb(var(--color-bg-elevated-rgb) / 88%);
  --material-glass-inline-blur: blur(8px) saturate(140%);
  --material-glass-inline-border: var(--color-border-default);
  --material-glass-inline-highlight: inset 0 1px 0 rgb(255 251 245 / 38%);
  --material-glass-inline-shadow: var(--shadow-sm);
}
```

暗色 / Catppuccin:同名令牌换值(bg 取暗基 60~78% 透明、highlight 降到 6~10%);mocha 在 `html:root[data-resolved-flavor='mocha']` 块内覆盖(遵守既有 specificity 契约)。

语义重映射(向后兼容,零组件改动):
- `--surface-modal-*` → floating 档
- `--surface-shell-*` / topbar 用途 → chrome 档
- `--surface-status-*` → inline 档
- `--surface-card-*` / `--surface-workspace-*` → **不透明**:bg ≥ 98% opacity、blur none、border default、shadow sm/md。亮色下卡片不再半透明,直接消灭"白雾感"。

## 4. Utility classes(utilities.css)

```css
.glass-floating { background: var(--material-glass-floating-bg); backdrop-filter: var(--material-glass-floating-blur); -webkit-backdrop-filter: var(--material-glass-floating-blur); border: 1px solid var(--material-glass-floating-border); box-shadow: var(--material-glass-floating-highlight), var(--material-glass-floating-shadow); contain: paint; }
/* .glass-chrome / .glass-inline 同构 */
@media (prefers-reduced-transparency: reduce) {
  .glass-floating, .glass-chrome, .glass-inline { background: var(--color-bg-elevated); backdrop-filter: none; -webkit-backdrop-filter: none; }
}
```

注释中写明玻璃预算(≤3 同屏、禁嵌套、禁滚动内容区),作为后续 review 依据。

## 5. 字体三轨分离(截图复核新增)

```css
:root {
  --font-brand:
    'SF Pro Display', 'Segoe UI Variable Display', 'PingFang SC',
    'Microsoft YaHei UI', 'Microsoft YaHei', sans-serif;   /* 标题:比例字体 */
  --font-sans:
    'MapleBright', 'SF Pro Text', 'PingFang SC',
    'Microsoft YaHei UI', 'Microsoft YaHei', sans-serif;   /* 正文:保留品牌字体 */
  --font-mono:
    'Cascadia Code', 'Cascadia Mono', 'SFMono-Regular', ui-monospace,
    'Consolas', 'MapleBright', monospace;                  /* 数值/代码:真等宽 */
}
```

- 即"把 mocha 的既有做法上移到 :root",mocha 覆盖块随之删除或只保留差异项。
- 风险:`--font-mono` 变为真等宽后,统计 tile/表格中的中文退到 fallback(MapleBright)——混排观感需在截图步骤专项检查;home.css 的 `--home-mono-feature`(tnum 等)保持有效。
- apple-glass-surface-contract 的字体栈守卫:把原先针对 mocha 块的受控例外改写为对新 :root 声明的精确断言。

## 6. 权衡

- 不引入 SVG 折射滤镜(Safari/Firefox 不支持 backdrop-filter+SVG 组合,且 GPU 成本高;WebView2 是 Chromium 但收益/风险比不划算)。高光用 gradient 近似折射即可。
- 不新增第四层 data-material 轴——三档 material 是令牌,不是用户可配置维度,避免破坏 theme/flavor/accent 三层契约。
- 旧 `--glass-*`/`--liquid-glass-*` 令牌保留并重指向新值(31 个文件在引用),子任务 2-5 逐步迁移到 material 语义后再评估删除。

## 7. 回滚

单 commit 纯 CSS 变量变更,`git revert` 即回滚;字体三轨单独成 commit;smoke tests 锁住新旧两套语义共存的状态。
