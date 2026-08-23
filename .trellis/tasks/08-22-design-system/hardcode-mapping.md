# 硬编码值 → token 查表映射（七个视图子任务的收口依据）

> 08-22-design-system 批次 6 交付物。`.css` 侧已按本表收口（见 `implement.md` 批次 6 证据）；
> `.vue`/`.tsx` 侧的 2,591 处（1,639 px + 932 rgba/rgb + 20 hex）由各视图子任务随迁移按本表收口，
> 归属父任务视图门核对（AC12），不是本任务的准出条件。

## 使用规则

1. **只做数值精确映射**：token 值必须与原字面量在根字号 16px 下严格相等（本仓 root 未改字号，
   `0.5rem` == `8px`）。视图迁移时不得「就近取档」——10px 不是 8px 也不是 12px。
2. **值不在阶上 → 登记豁免**：写入各视图子任务自己的豁免清单（格式见 `hardcode-exemptions.md`），
   不允许静默近似。
3. **颜色一律走 triplet**：禁止新写 `rgba(R, G, B, A)` 字面量，用 `rgb(var(--color-<name>-rgb) / <A>%)`。
   该形态在主题切换、flavor/accent 切换时随第 1 层变量自动重解析。

## 间距 / 尺寸（px → `--space-*`，Tailwind 侧经 `--spacing-*` 命名空间生成 p-/m-/gap- 工具类）

| 字面量 | token | 备注 |
| --- | --- | --- |
| 1px | `var(--space-px)` | 发丝线边框宽度：`border: 1px solid X` → `border: var(--space-px) solid X`（dashed 同理） |
| 2px | `var(--space-0-5)` | 双线强调边框、outline 宽度与 offset |
| 4px | `var(--space-1)` | |
| 6px | `var(--space-1-5)` | |
| 8px | `var(--space-2)` | 状态点直径、小图标尺寸 |
| 10px | `var(--space-2-5)` | |
| 12px | `var(--space-3)` | |
| 16px | `var(--space-4)` | |
| 20px | `var(--space-5)` | |
| 24px | `var(--space-6)` | |
| 32px | `var(--space-8)` | 头像/图标中号 |
| 40px | `var(--space-10)` | |
| 48px | `var(--space-12)` | 空状态图标 |
| 64px | `var(--space-16)` | 空状态图标大号 |
| 80px | `var(--space-20)` | |

## 圆角（px → `--radius-*`）

| 字面量 | token |
| --- | --- |
| 4px | `var(--radius-sm)` |
| 6px | `var(--radius-md)` |
| 8px | `var(--radius-lg)` |
| 10px | `var(--radius-xl)` |
| 12px | `var(--radius-2xl)` |
| 16px | `var(--radius-3xl)` |
| 999px / 9999px / 50%（胶囊/圆形） | `var(--radius-full)` |

## 颜色（rgba/rgb/hex → token）

| 字面量形态 | 映射 | 例 |
| --- | --- | --- |
| `rgba(R, G, B, A)` / `rgb(R G B / A)` | 先按 R G B 查 `token-classification.md` 归属的语义 token（`--color-<name>-rgb` triplet），再写 `rgb(var(--color-<name>-rgb) / <A×100>%)` | `rgba(207, 98, 57, 0.1)` → `rgb(var(--color-accent-primary-rgb) / 10%)` |
| 不透明色 `#RRGGBB` | 语义 token 直取 | `#cf6239` → `var(--color-accent-primary)` |
| 阴影整串 `0 2px 8px rgb(0 0 0 / 10%)` | 优先取 `--shadow-xs/sm/md/lg/xl`（明暗两套已定义，随 `data-theme` 切换）；与全部档位都不等值时**不要近似**，登记豁免并注明 | |

## 字号

- 禁止 px 字号（`theme-token-contracts.md` 既有约束）。档位见 `tokens.css` `--text-*`（0.8125rem 起）。
- **例外保留**：Profiles 共享层密集元信息 `0.75rem`（低于 Label 下限 `--text-sm` 一档），系
  `theme-token-contracts.md` 已登记的契约例外，视图迁移时原样保留，不改为 token。

## 豁免类别速查（详见 `hardcode-exemptions.md`）

动画位移（批次 7 判定）、媒体查询断点、网格轨道/一次性布局尺寸、阴影与焦点环几何、
sr-only 裁剪惯用法、非阶值密集内边距（Profiles 0.75rem 族）、token 定义源头（tokens.css）。
