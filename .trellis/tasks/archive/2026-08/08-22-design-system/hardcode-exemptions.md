# 硬编码豁免登记（src/styles/**，批次 6 收口后的全部残留）

> 08-22-design-system 批次 6 交付物。AC1/AC2 的口径：残留字面量数 == 本文件登记数。
> 每条保留原因均为「映射到既有 token 会改变渲染值（视觉基线锁定）」或「非样式决策常量」。
> 本文件数字对应 2026-08-23 收口后状态：px 残留 152 处（rg 口径），其中 token 定义 76 +
> 消费侧豁免 76；字面量 `rgb()/rgba()` 残留 69 处，全部在定义源头 + 1 处遗留阴影（见 §I）。

## A. token 定义源头（`src/styles/tokens.css`）

| 类型 | 数量 | 原因 |
| --- | --- | --- |
| px | 76 | 间距/圆角/个别几何 token 的定义值——token 体系自身的源头，映射目标是它自己 |
| hex | 91 | 颜色 token 的定义值（调色板本体）。另 `chart-colors.css` 5 处 hex 仅存在于注释 |
| 字面量 `rgb()/rgba()` | 68 | glow/阴影等由调色板推导的定义值（如 `--color-accent-primary-glow`） |

## B. 动画位移（motion 值，批次 7 逐段判定归属）

| 位置 | 内容 |
| --- | --- |
| `src/styles/animations.css:34,47,60,73,160,167,202,215,228,246,433,457,462,511,516,532,537,574,579`（19 处） | 进出场 keyframe 的 translate 距离；多数不在间距阶上（10px/16px/20px），改档即改变动画轨迹。批次 7 判定去留（motion 接管或保留 CSS），届时随段处理 |
| `src/styles/backgrounds.css:28` | 装饰背景漂移动画 transform |
| `src/styles/core.css:59` | 首屏过渡 `translate3d(0, 10px, 0)`（10px 不在阶上） |
| `src/styles/utilities/utilities.css:162,716` | hover 微抬升 `translateY(-2px/-4px)`（micro-interaction 值） |

## C. 媒体查询断点（视口常量，非样式决策）

| 位置 | 内容 |
| --- | --- |
| `src/styles/components/codex-auth-shared.css:572,587,608,616,630`（5 处） | 768/1280/1100/1101/900 |
| `src/styles/components/profiles-page.css:184,190,196,202`（4 处） | 1280/1279/1024/720 |

## D. 网格轨道与一次性布局尺寸（结构常量）

| 位置 | 内容 |
| --- | --- |
| `src/styles/components/profiles-page.css:45` | 页面最大宽 `1680px` |
| `src/styles/components/profiles-page.css:61` | 卡片列 `minmax(420px, 1fr)` |
| `src/styles/components/profiles-page.css:73-76`（8 处） | 表格网格轨道 `12px / 120-160px / 80-120px / 60px / 110px` |
| `src/styles/components/profiles-page.css:127` | 空态最大宽 `420px` |
| `src/styles/components/profiles-page.css:186` | 侧栏轨道 `340px` |
| `src/styles/utilities/utilities.css:534` | 空态描述最大宽 `300px` |

## E. 非阶值密集内边距（Profiles 0.75rem 契约例外族）

| 位置 | 内容 |
| --- | --- |
| `src/styles/components/profiles-page.css:78`（3 处） | `padding: 2px 14px 4px`（14px 不在间距阶上；密集行视觉基线锁定） |
| `src/styles/components/profiles-page.css:137`（2 处） | `padding: 7px 14px`（同上） |

## F. 阴影 / 焦点环 / 状态辉光几何（与颜色 token 成对出现的常量）

| 位置 | 内容 |
| --- | --- |
| `src/styles/utilities/utilities.css:80,326,339` | 焦点环 `0 0 0 3px var(--glow)`（3px 环宽不在阶上） |
| `src/styles/utilities/utilities.css:392,401,406` | 状态点辉光 `0 0 8px var(--glow)`（颜色已 token 化，8px 为辉光半径） |
| `src/styles/components/home.css:35,37,53` | 页面级 elevation 局部 token 定义（`--home-elevation-*`、焦点环双层阴影） |
| `src/styles/shell-critical.css:7`、`src/styles/utilities/utilities.css:539` | 加载 spinner `border: 3px solid`（spinner 环宽常量） |

## G. a11y 惯用法（sr-only 裁剪与分隔线）

| 位置 | 内容 |
| --- | --- |
| `src/styles/base/base.css:189,190,192`、`src/styles/utilities/utilities.css:554,555,557` | sr-only 裁剪惯用法 `width/height: 1px; margin: -1px`（标准无障碍模式，逐字面量属惯例） |
| `src/styles/utilities/utilities.css:411,418,651` | 分隔线/装饰线 `height/width: 1px`（发丝线几何） |

## H. @theme 映射定义

| 位置 | 内容 |
| --- | --- |
| `src/styles/core.css:137` | `--spacing-px: 1px`——Tailwind 命名空间映射（第 2 层）指向 `--space-px` 的定义点写法，值取自第 1 层 |

## I. 遗留字面量阴影（移交消费方视图迁移批次判定）

| 位置 | 内容 |
| --- | --- |
| `src/styles/utilities/utilities.css:211` | `box-shadow: 0 2px 8px rgb(0 0 0 / 10%)`（`.nav-item-active-glow` 遗留样式）：与 `--shadow-sm`（0 2px 6px / 9%）不等值，改 token 会改变渲染；颜色为字面量纯黑，语义上应走 `--color-accent-primary-glow` 族。该类由导航外壳（08-22-shell-port）迁移时重判 |

## J. motion 局部 token（home）

| 位置 | 内容 |
| --- | --- |
| `src/styles/components/home.css:51,71` | `--home-motion-lift: -1px / 0px`（页面级动效局部 token 定义，批次 7 判定域） |

---

**消费侧小计核对**（与逐行扫描的 76 处一致，脚本记录见 `hardcode-transform-records.json`）：
B 动画 23 + C 断点 9 + D 网格/布局 13 + E 密集内边距 5 + F 阴影/环/spinner 12 + G a11y 惯用法 9
+ H @theme 映射 1 + I 遗留阴影 2 + J motion 局部 token 2 = **76**。
I 的颜色字面量单独计入 AC2 口径：消费侧字面量 `rgb()` 仅 1 处（utilities.css:211）。
