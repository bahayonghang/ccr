# 同步页面板边框与信息结构

## Goal

同步页只保留一层目录：页头说明策略并执行全量操作，三个分组承载五份固定资产，右栏只放 WebDAV 账号。面板用实色发丝边和不透明表面与壳层分开。同步行为、允许清单和其它路由不变。

用户价值：扫描「同步什么、本地/远端是否存在、下一步按哪个按钮」时，不再先读四遍同一份清单。

## Direction

方向是现有行情终端，出处 `ccr-ui/DESIGN.md` 与 `ccr-ui/AGENTS.md`。

- **Visual thesis**：暖黑表面，发丝边和明度台阶分开面板，琥珀只留给当前可执行的主同步命令。
- **Content plan**：页头给出连接和全量动作，分组列出资产路径与单项动作，右栏维护账号。无营销式首屏。
- **Interaction thesis**：无新增动效。边框和层级是休息态问题。按钮继续使用 `buttonClass` 的按下反馈。

## Background

2026-10-12 暗色截图的红框盖住页头、CCR / Claude / Codex 范围条、配置资产控制台和 CCR Platforms 卡。WebDAV 栏在红框右侧，同一视图。

允许清单是固定的五份资产、三个分组（`ccr-ui/src-tauri/src/commands/sync.rs:162`）：

| id | 分组 | 名称 | 形态 |
| --- | --- | --- | --- |
| `ccr-platforms` | ccr | CCR Platforms | 目录，敏感 |
| `claude-settings` | claude | settings.json | 文件，敏感 |
| `claude-memory` | claude | CLAUDE.md | 文件 |
| `codex-config` | codex | config.toml | 文件，敏感 |
| `codex-agents` | codex | AGENTS.md | 文件 |

分组标题和排除说明来自 `sync.assetGroups.*.title/description`。范围条的三项（`sync.assets.scope*`）和资产卡上的 `asset.description` 与这些句子重复。`asset.description` 只在 `SyncAssetCard.tsx:92` 渲染，而且是 Tauri 写死的英文。

当前左栏从上到下是：页头「配置资产同步」、范围条（`SyncView.tsx:26`）、控制台眉题「固定同步清单」加标题「配置资产控制台」（`SyncView.tsx:282`）、分组眉题加标题（`SyncView.tsx:190`）、再一张同名资产卡。连接状态在页头和 WebDAV 栏各出现一次（`SyncInfoSidebar.tsx:81`）。

壳层背景是 `--color-bg-base`（`ccr-ui/src/shell/shell.css:7`）。暗色阶梯为 base `#100f0c`、elevated `#171410`、surface `#1f1b14`、overlay `#2a251b`。发丝边是实色 subtle `#332c21`、default `#453b2a`（`tokens.css:196`）。本页仍用透明描边：范围条和分组 36%（`sync-view.css:47`、`sync-view.css:171`），资产卡 32% 且无填充（`sync-view.css:204`），关于同步 36%（`sync-view.css:265`），操作输出 36%（`sync-output.css:1`），WebDAV 栏 `border-border-default/25`（`SyncInfoSidebar.tsx:78`）。`.sync-page` 把整页涂成 elevated（`sync-view.css:1`）。圆角字面量 `1rem` 超过 `--radius-xl` 的 12px（`tokens.css:440`）。页头按钮被 `.sync-hero-button` 收成胶囊（`sync-view.css:11`）。门控卡用强调色填充（`sync-view.css:88`）。`sync-asset-card--missing` 没有样式（`SyncAssetCard.tsx:85`）。`.sync-kind-chip` 没有中性边（`sync-view.css:227`）。

`DESIGN.md` 要求：一个面板一个标题，不叠眉题。休息态面板是不透明 surface 加 1px 发丝边，圆角不超过 12px，无阴影。状态色只出现在发丝、圆点和状态文字上。宽字距只给短的拉丁等宽标签。

## Decisions

1. 范围条和「配置资产控制台」整块删除。分组描述已经写出要同步的文件名和排除项，页头副标题已经写出允许清单策略。
2. WebDAV 留在右栏。服务器、用户名、远端路径、测试和断开不在资产清单里。不把账号收成页头下的第二条横幅，也不改成单栏。
3. 资产卡不再渲染 `asset.description`。分组描述承担说明。Tauri 里的英文字符串保留给接口，不改 `sync.rs`。
4. 名称与分组标题相同的资产不重复标题。按现有文案，只有 `ccr-platforms` 命中。多资产分组的行标题是文件名。
5. `canonicalName` 与资产名称相同时不显示「规范名」标签。现有四份文件都相同。
6. 未配置或不可达时，主栏横幅是唯一的琥珀主按钮。右栏不再放第二颗琥珀色添加按钮。已配置时，编辑、测试、断开都用次级按钮。每行的 Sync 和页头的「全部同步一次」保持主按钮。
7. 不新增颜色或圆角 token。

## Requirements

### R1. 页头是唯一的页面标题

`PageHeader` 的 h1 保持「配置资产同步」和现有副标题。状态区保留一枚连接芯片，并放上资产数量（`sync.assets.total`）和「敏感值已遮罩」。去掉 `Assets` 徽章。刷新、全部同步、强制全部同步留在页头，圆角回到 `--radius-lg`。

### R2. 删除重复目录

删除 `ScopeStrip`。删除控制台眉题、标题「配置资产控制台」和那段重复说明。中英文同时删除因此不再使用的 key：`sync.assets.badge`、`eyebrow`、`title`、`description`、全部 `scope*`、`sync.assetGroups.*.label`。分组的 `title` 与 `description` 保留。

### R3. 分组是资产目录

每个有资产的分组是一块面板：一个标题、一段排除说明、数量。内部按资产一行，行与行之间用 `--color-border-subtle` 分隔，不再套第二层卡片。

CCR 行不重复「CCR Platforms」，直接给出类型、敏感、加密、路径、存在状态和 Push / Pull / Sync。Claude 两行的标题是 `settings.json` 与 `CLAUDE.md`。Codex 两行是 `config.toml` 与 `AGENTS.md`。不渲染后端 `description`。规范名与标题相同时不显示。

类型标签使用中性发丝边。本地不存在的行有可见的状态区分，`sync-asset-card--missing` 生效或删除。

### R4. 右栏只承担账号和事后反馈

WebDAV 面板保留服务器、用户名、远端路径和账号动作，使用与分组相同的面板填充和发丝边。标题用 section 层级，不再用 `text-xl`。不重复连接芯片。

未配置与不可达的说明放在主栏分组之上，一块中性横幅，一颗主按钮，沿用现有门控文案。右栏在这两种状态下不再放琥珀色主按钮。

操作输出仍在右栏，有结果才出现。失败和部分成功的状态色只留在发丝、圆点或状态文字上。关于同步保持折叠，继续承载备份恢复、脱敏和支持的服务。这些句子不搬回主栏。

窄于 `64rem` 时，主栏和右栏改为单列，顺序仍是主栏在上、右栏在下。

### R5. 面板边界

分组、WebDAV、关于同步、操作输出、门控横幅使用不透明 `--color-bg-surface` 和 `1px solid var(--color-border-default)`，圆角 `--radius-xl`。删除 `.sync-page` 的 elevated 铺底，让壳层 base 透出，面板亮一阶。休息态不加阴影，不使用 `rgb(var(--color-border-*-rgb) / <100%)` 做这些面板的描边。同步页 CSS 不写新的 hex。暗色、浅色、`clay` 都走现有 token。

### R6. 行为不变

资产 id、门控禁用条件、刷新、全量同步、单项 Push / Pull / Sync、强制重试、口令弹层、操作输出滚动定位保持现状。不改 Tauri 允许清单、IPC 或 WebDAV 保存逻辑。

## Acceptance Criteria

1. 暗色中性、桌面宽度下，同步页只有一个页面标题「配置资产同步」。页面上没有「配置资产控制台」「固定同步清单」和三列范围条。（R1、R2）
2. 五份资产都在时，主栏是三块面板。CCR 面板内没有第二处「CCR Platforms」。Claude 与 Codex 的行标题分别是文件名。资产行不出现 Tauri 英文 description。（R3）
3. 页头有一枚连接芯片、数量和敏感值提示，没有 `Assets` 徽章。WebDAV 栏不再显示第二枚连接芯片。（R1、R4）
4. 未配置时，主栏有一条中性说明和一颗琥珀主按钮。右栏没有第二颗琥珀添加按钮。不可达时同样只有横幅上的一颗主重试按钮。（R4）
5. 分组、WebDAV、关于同步、操作输出的计算填充是不透明 surface，计算边框是 `#332c21` 或 `#453b2a`，计算圆角不超过 12px。页头主操作圆角为 8px。状态胶囊仍可为 pill。（R5）
6. 视口窄于 `64rem` 时主栏在上、右栏在下。零资产时不崩，只显示页头、门控或右栏，不出现空的分组壳。（R4）
7. 浅色和 `clay` 下这些表面跟随 token，同步页样式文件没有新的 hex。中英文 key 成对增删。（R2、R5）
8. `ccr-ui/tests/sync/` 现有冒烟测试通过。Push / Pull / Sync 的可点条件与改动前一致。`src-tauri` 与其它路由样式无 diff。（R6）

## Out of Scope

- 改五份资产的 id、路径、敏感标记或分组归属。
- 把 WebDAV 移进页头，或取消右栏。
- 用量页、首页和其它仍使用透明边框的路由。
- 新增 token 名。
- 导航、设置、命令状态条。
- 入场动画。
- 把用户截图提交进仓库。画面含本机路径和 WebDAV 账号。

## Technical Notes

- 结构主人：`ccr-ui/src/features/sync/SyncView.tsx`、`SyncAssetCard.tsx`、`SyncInfoSidebar.tsx`。样式主人：`styles/sync-view.css`、`styles/sync-output.css`。
- 词条：`ccr-ui/src/i18n/locales/zh-CN.ts`、`en-US.ts` 的 `sync.assets` 与 `sync.assetGroups`。
- 共享按钮：`@/ui` 的 `buttonClass`。列表项保持 `memo`，不要给 `SyncAssetCard` 传入内联对象或内联函数（`.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md`）。
- 不改 `ccr-ui/src-tauri/src/commands/sync.rs` 的 `SYNC_ASSETS`。行为边界见 `.trellis/spec/ccr-ui/frontend/sync-security-contracts.md`。
- 主题约束见 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`。
- 验证：`cd ccr-ui && bun run type-check`，`bun run lint`，`bun run test:i18n`，以及 `tests/sync/` 与 `tests/quality/worst-views.smoke.test.tsx`。视觉检查暗色桌面、窄于 `64rem`、浅色各一次。
