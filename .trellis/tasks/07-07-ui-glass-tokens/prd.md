# 玻璃材质令牌体系与对比度修复

## Goal

在 `tokens.css` / `home.css` 层面一次性解决两件事:(1) 亮色主题泛白——表面分层、边框、阴影、文字对比全面重标定;(2) 把现在"名不副实"的玻璃令牌升级为**性能受控的真实液态玻璃材质**,并定义全站玻璃使用预算。这是其余 4 个子任务的地基。

## Requirements

### R1 亮色 clay 对比度重标定(修"泛白")

- base / elevated / surface / overlay 四层背景之间拉开可感知的亮度差(目标:相邻层 ΔL ≥ 3,base 与 surface ΔL ≥ 6,以 OKLCH L 通道衡量),保持暖米色相不变。
- 边框透明度上调:subtle ≥ 12%、default ≥ 18%、strong ≥ 28%(暖褐色基),保证卡片边界在暖底上可见。
- 阴影 alpha 上调并保持暖色调 tint(不得用纯黑):sm ≥ 8%、md ≥ 12%,elevation 三档肉眼可分。
- `--color-text-secondary`/`muted` 对底色对比 ≥ 4.5:1(AA);primary 对底 ≥ 7:1(AAA 目标)。
- paper / graphite flavor 做同规则的等比修订;Catppuccin 四个 flavor 保持现有语义映射,只在必要处微调 glass 透明度档。
- **深色主题同步重标定(截图证实)**:深色下面板与底色亮度差同样过小、边框几乎不可见——深色相邻层 ΔL ≥ 2.5,`--color-border-default`(亮字基)≥ 16%,`strong` ≥ 26%;暗色卡片边界与 elevation 在截图中肉眼可辨。

### R2 液态玻璃材质升级(Web 近似,非 Apple 官方)

- 新玻璃配方:blur 8~16px + saturate 140%~180% + 分层边框(1px 外框 + inset 高光)+ 顶部 radial 高光,亮暗两套参数。
- 玻璃分级收敛为三档语义:`--material-glass-floating`(模态/命令面板/浮层,最强)、`--material-glass-chrome`(侧栏/顶栏)、`--material-glass-inline`(页面内极少数悬浮条)。
- 普通内容卡片**退出玻璃体系**:改为不透明分层表面(elevated/surface + 边框 + 阴影),映射现有 `--surface-card-*` 语义但 `blur: none`、opacity ≥ 98%。
- 玻璃预算写进令牌注释与 spec:同屏 backdrop-filter 元素 ≤3;禁止嵌套玻璃;禁止在滚动内容区内部使用玻璃。
- 玻璃元素配套 `contain: paint`(或 strict)与合成层提示的 utility class(utilities.css 提供 `.glass-floating/.glass-chrome/.glass-inline`)。

### R3 降级路径

- `prefers-reduced-transparency: reduce` → 全部玻璃回退为对应档的不透明表面(现有 media query 扩展覆盖新令牌)。
- `prefers-reduced-motion: reduce` 行为保持不变。
- 保留现有 `--surface-*` 语义名(shell/workspace/card/modal/status),旧引用不破坏——本任务只改这些语义指向的值与新增三档 material 令牌。

### R5 字体三轨分离(截图复核新增)

- 现状:`--font-sans/--font-brand/--font-mono` 全部指向 MapleBright,大号 CJK 标题呈终端等宽观感;仅 mocha 有 brand/mono 分离覆盖(tokens.css:1117-1123)。
- 目标:把 mocha 的分离模式推广为全局默认——`--font-brand` = 'SF Pro Display', 'Segoe UI Variable Display', 'PingFang SC', 'Microsoft YaHei UI' 等比例显示字体;`--font-mono` = 'Cascadia Code', 'Consolas' 等真等宽(数值/代码/表格/统计条专用);`--font-sans` 正文保留 MapleBright(阅读密度尚可,且是既有品牌选择)。
- mocha 现有覆盖块随之简化(与全局默认合并或保留更锐利的差异)。
- 同步窄化 apple-glass-surface-contract.smoke.test.ts 的字体栈受控例外(契约要求例外精确到具体覆盖块,不得整文件跳过)。

### R6 契约与测试

- 遵守 theme-token-contracts.md:三层 data-theme/flavor/accent 正交不变;mocha 覆盖块继续使用 `html:root[data-resolved-flavor='mocha']` 高优先级选择器。
- 更新/扩展 smoke tests:apple-glass-surface-contract 增加"三档 material 令牌存在 + 同屏预算注释存在 + reduced-transparency 覆盖存在"的断言;theme-bootstrap/app-settings 不回归。

## Out of Scope

- 各页面对新令牌的落地(子任务 2-5)。
- 组件级样式(Button/Card 等)的重写,只要它们引用的语义令牌值变化后不劣化即可。

## Acceptance Criteria

- [x] 亮色 clay:用取色器验证 base/elevated/surface 三层 OKLCH L 差满足 R1;截图对比可肉眼分层。
- [x] secondary/muted 文字对比经工具验证 ≥ 4.5:1(亮/暗两套)。
- [x] 新三档 material 令牌 + utility class 就位;`--glass-blur-*` 旧档位仍存在但重指向新配方或标注 deprecated。
- [x] reduced-transparency 模拟下(DevTools rendering 面板)玻璃全部回退不透明。
- [x] 大标题(如首页"运行概览")以比例字体渲染、数值区(统计 tile/表格)保持等宽;字体栈 smoke 例外已按契约窄化。
- [x] 深色 clay/graphite 下卡片边界与 elevation 肉眼可辨(对比基线截图)。
- [x] `bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts` 通过。
- [x] `bun run type-check && bun run lint` 通过。
- [x] 亮/暗 × clay/paper/graphite/mocha 的 Dashboard 截图各一张,记录 document dataset 值(按 theme-token-contracts 的验证规范)。
