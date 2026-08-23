# 技术设计：ccr-ui 前端框架迁移 Vue 3 → React

> 父任务：`08-22-react-migration`。本文件记录跨全部 18 个子任务的技术决策。子任务的局部设计写在各自的 `design.md`。

## 1. 选型决策表

全部版本为 2026-08-22 查询的最新版。

| 领域       | 选定                      | 版本             | 替代的 Vue 侧                 |
| ---------- | ------------------------- | ---------------- | ----------------------------- |
| UI 运行时  | React                     | 19.2.8           | vue 3.5.26                    |
| 构建       | Vite                      | 8.2.2            | vite 7.3.5                    |
| 路由       | React Router              | 8.3.0            | vue-router 4.6.4              |
| 状态       | Zustand                   | 5.0.15           | pinia 2.3.1                   |
| 服务端数据 | TanStack Query            | 5.101.4          | 无（原由 store 承载）         |
| 表单       | react-hook-form           | 7.86.0           | 无（原 353 处 `v-model`）     |
| 校验       | zod + @hookform/resolvers | 4.4.3 / 5.9.1    | 无                            |
| 样式       | Tailwind CSS              | 4.3.3            | tailwindcss 3.4.19            |
| 动画       | motion                    | 13.1.1           | Vue Transition + 580 行 CSS   |
| 编辑器桥接 | @uiw/react-codemirror     | 4.25.11          | CodeSourceEditor.vue 235 行   |
| 图标       | @iconify/react            | 6.0.2            | @iconify/vue 5.0.0            |
| 虚拟滚动   | @tanstack/react-virtual   | 3.14.10          | @tanstack/vue-virtual 3.13.18 |
| 图表       | react-apexcharts          | 2.1.1            | vue3-apexcharts 1.10.0        |
| i18n       | react-i18next + i18next   | 17.0.12 / 26.4.0 | vue-i18n 9.14.5               |
| 测试       | @testing-library/react    | 16.3.2           | @vue/test-utils               |

保留不变：`@codemirror/*`（9 包）、`apexcharts`、`dompurify`、`fuse.js`、`ansi_up`、`@tauri-apps/api`、`@iconify-json/solar`。

## 2. 目录结构

**决策：全量按域聚合 `features/`。**

```
src/
├── features/                 # 按域聚合，替代 views/ + components/ 的平铺
│   ├── claude/
│   ├── codex/
│   ├── grok/
│   ├── gemini/
│   ├── opencode/
│   ├── checkin/
│   ├── usage/
│   ├── sync/
│   ├── mcp/
│   ├── commands/
│   ├── profiles/
│   ├── configs/
│   └── platform/             # platform-unify 的 7 个功能面统一层
├── shell/                    # App 外壳、layout、路由、窗口与主题引导
├── ui/                       # 原语层（原 components/ui/ 16 个）
├── api/                      # 不变，57 文件
├── types/                    # 不变，231 文件（含 204 个 ts-rs 产物）
├── utils/                    # 纯逻辑保留，需接线的移入 shell/
├── config/                   # descriptor 与能力开关
├── configs/                  # per-surface 平台 config
├── i18n/                     # 不变，词条数据复用
└── styles/                   # token 与全局样式
```

依赖方向（由 ESLint 导入规则强制，见 `08-22-arch-quality-perf`）：

```
features/*  →  features/platform  →  ui  →  styles
     ↓                ↓
    api  →  types
     ↑
  config / configs
```

禁止：`ui/` 导入 `features/` 或 `api/`；`features/<a>/` 导入 `features/<b>/`（跨域复用必须经 `features/platform/` 或 `ui/`）。

## 3. 路由设计

**现状更正**：路由不是扁平结构。`src/router/index.ts`（594 行）有 2 个顶层条目：

- `/tray/codex` — 独立窗口，不套 MainLayout
- `/` — 布局父级，其 `children` 承载其余约 73 条路由

4 条动态参数路由：`commands/:client?`（可选参数）、`agents/:name`、`skills/:platform/:name`、`checkin/manage/:accountId`。

映射方案：

| Vue Router                                                       | React Router 8                                            |
| ---------------------------------------------------------------- | --------------------------------------------------------- |
| 2 个顶层 + 1 层 children                                         | `createBrowserRouter` 对象式路由，同构嵌套 + `<Outlet />` |
| `RouteMeta` 8 个字段                                             | 路由对象的 `handle`，保留同名字段与类型声明               |
| `genericPlatformDescriptorList` 生成 mcp/agents/plugins 路由     | 同样的生成逻辑，输入改为新 descriptor（见第 8 节）        |
| `router.beforeEach`（perf 埋点 + locale 预热）                   | `loader` 与顶层布局的副作用组合                           |
| `router.afterEach`（`recordRouteTiming`）                        | 路由变更监听                                              |
| `usePageTransition.ts` 的 `beforeEach`（`depth` / `group` 比较） | 同一逻辑移到布局组件，读 `handle.depth` / `handle.group`  |
| `commands/:client?`                                              | React Router 的可选参数语法，需在实现时验证匹配行为       |

`meta` 8 个字段保留原名：`cache`、`cacheKey`、`hideGlobalBackground`、`stream`、`depth`、`group`、`hideSidebar`、`deferLocaleHydration`。`cacheKey` 的语义随第 5 节改变，字段保留以便映射表追溯。

## 4. 状态分层

R6.6 的三分类落地。每一类有唯一承载位置，不混用。

| 类别       | 承载                         | 判据                                          | 现状来源                                                                               |
| ---------- | ---------------------------- | --------------------------------------------- | -------------------------------------------------------------------------------------- |
| 服务端数据 | TanStack Query               | 数据来自 IPC 命令或 Tauri Event，有新鲜度概念 | `usage`、`configs`、`commands`、`claudeObserver`、`homeUsageOverview` 中的数据缓存部分 |
| 跨页面共享 | Zustand                      | 多个路由读写同一份状态，且非服务端数据        | `ui`（toast / 收藏 / 历史）、`shellPreferences`、`commandsView`                        |
| 组件本地   | `useState` / react-hook-form | 单个组件或单个表单内的瞬态                    | 现散在组件内的 `ref`                                                                   |

10 个 Pinia setup store 的处理：

| store                         | 行数 | 处理                                                       |
| ----------------------------- | ---- | ---------------------------------------------------------- |
| `usage.ts`                    | —    | 数据部分 → Query，视图偏好 → Zustand                       |
| `configs.ts`                  | —    | 数据部分 → Query，选中态 → Zustand                         |
| `commands.ts`                 | —    | → Query                                                    |
| `claudeObserver.ts`           | —    | 事件流数据 → Query（配合 Event 订阅失效），UI 态 → Zustand |
| `homeUsageOverview.ts`        | —    | → Query                                                    |
| `ui.ts`                       | —    | → Zustand                                                  |
| `shellPreferences.ts`         | —    | → Zustand                                                  |
| `commandsView.ts`             | —    | → Zustand                                                  |
| `usageDashboardPayload.ts`    | 171  | 纯变换，移入 `utils/`，不进状态层                          |
| `usageImportNormalization.ts` | 83   | 同上                                                       |

映射依据：10 个 store 全为 setup 写法，含 21 处 `ref`、13 处 `computed`、0 处 `watch`、0 处 `reactive`。无 `watch` 意味着 store 内无订阅式副作用，迁移不需要处理时序。13 处 `computed` 转为 Zustand selector 或 Query 的 `select`。

Tauri Event 与 Query 的衔接：后端 `emit` 的事件（`app-log`、`token-stats` 等）在监听回调中调用 `queryClient.invalidateQueries` 或 `setQueryData`，不再由 store 直接持有数据。订阅的建立与解绑保持在组件生命周期内（R4 of `08-22-state-logic-port`）。

## 5. 缓存路由替代

**决策：状态外提到 store，不做组件常驻。**

5 条路由带 `meta.cache: true`（现状更正：先前记为 4 条，实为 5 条）：

| 路由                | cacheKey             | 需保留的状态                      | 处理                                                   |
| ------------------- | -------------------- | --------------------------------- | ------------------------------------------------------ |
| `dashboard`         | `DashboardView`      | 数据                              | 由 Query 缓存承担，无需额外处理                        |
| `grok`              | `GrokView`           | 数据 + 选中态                     | 数据走 Query，选中态入 Zustand                         |
| `commands/:client?` | `CommandsView`       | 数据 + 流式输出（`stream: true`） | 数据走 Query；流式输出的累积缓冲入 Zustand，切回时续读 |
| `configs`           | `ConfigsView`        | 数据 + 选中态 + 搜索词            | 数据走 Query，选中态与搜索词入 Zustand                 |
| `usage`             | `UsageDashboardView` | 数据 + 时间范围 + 平台维度        | 数据走 Query，筛选条件入 Zustand                       |

滚动位置：React Router 的滚动恢复机制处理，不入 store。

未提交表单：5 条缓存路由中只有 `configs` 存在表单，其草稿状态入 Zustand，键为配置 id。

该方案下 `cacheKey` 字段不再驱动 `<keep-alive :include>`，保留字段仅用于迁移映射表追溯。`MainLayout` 派生 `cacheKey` 数组的逻辑删除。

## 6. 样式承载

**决策：Tailwind 工具类为主，残余进 CSS Modules。**

| 现状                                                                                | 目标                                                                                                                                 |
| ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| 24,434 行组件内 `<style>`，覆盖 139/185 个组件                                      | 尽量压为工具类；复杂选择器、关键帧动画留 `.module.css`                                                                               |
| 1,639 处 px 字面量 + 932 处 `rgba()`                                                | 收回 token（`08-22-design-system` R2）                                                                                               |
| 4,097 处 `var(--)` 引用                                                             | 保留，token 名不变                                                                                                                   |
| 448 个 CSS 变量（`tokens.css` 26.7 KB）                                             | 分两层：可切换语义变量留普通 CSS 变量，Tailwind namespace 映射进 `@theme inline`，常量 token 进 `@theme`（`08-22-design-system` §1） |
| 648 处 `@apply`（集中在 25 个文件）                                                 | v4 下在组件级样式文件中需 `@reference`，逐文件处理                                                                                   |
| `corePlugins.preflight: false` + `src/styles/base.css` 自带 reset                   | v4 下等价保留（R8.6）                                                                                                                |
| `fontWeight` 8 档位压缩到 400/500                                                   | v4 `@theme` 下保留该语义                                                                                                             |
| 三层 CSS 加载（`shell-critical` / `deferred-decorations` / `deferred-interactive`） | 等价保留（R8.6）                                                                                                                     |
| `src/styles/{base,components,themes,utilities}/` 4 个空目录                         | 填充或删除，不留空目录                                                                                                               |

约束：单组件的局部样式行数不超过其 JSX 行数（R3.5），由 `08-22-arch-quality-perf` 落为检查规则。

## 7. 表单与校验

353 处 `v-model` 的统一转换目标为 react-hook-form 的非受控注册。密度最高的落点：`CodexSettingsView`（33 处）、`ClaudeCodeSettingsView`（32 处）、`OpenCodeSettingsView` 与 `HooksView`（各 15 处）。

非受控注册使输入不触发父组件重渲染，直接服务 R8.1。

zod schema 的两处用途：

1. 表单校验，经 `@hookform/resolvers` 接入。
2. IPC 返回值的运行时校验。现状为 ts-rs 只提供编译期类型，无运行时校验。该用途为可选增益，先在新增 wrapper 上试点，不回填 57 个既有 wrapper。

zod 与 motion 均增加 bundle 体积。`08-22-arch-quality-perf` 的 bundle 预算需为两者预留额度并记录（见第 12 节）。

## 8. 平台 config 契约

**决策：descriptor 层与 per-surface config 层并存。**

```
路由与能力（descriptor 层）
  src/config/platformDescriptors.ts     现 50 行 → 扩展
  src/config/platformCapabilities.ts    现 74 行 → 不动

    claude: { rootPath: '/claude',
              surfaces: ['settings','profiles','auth','mcp',
                         'agents','plugins','commands'] }
    grok:   { rootPath: '/grok',
              surfaces: ['settings','profiles','auth'] }

功能面细节（per-surface config 层）
  src/configs/settings.ts    src/configs/profiles.ts
  src/configs/auth.ts        src/configs/commands.ts
  src/configs/slashCommands.ts   现 192 行 → 不动，作为参照实现
```

职责划分：

- descriptor 声明「该平台有哪些面」，驱动路由生成与导航。
- config 模块声明「该面在该平台怎么表现」，驱动对应的 base 组件。

变更成本：

- 新增平台 → 改 descriptor 一行 + 各 config 模块加一个导出。
- 改某面共性行为 → 只改该面的 base 组件。
- 改某平台某面的差异 → 只改该 config 模块的一个导出。

base 组件内禁止平台名称条件分支（R9.3）。差异必须经 config 字段或 props 表达。

参照实现已在仓库内运行：`BaseSlashCommands.vue`（507 行）+ `configs/slashCommands.ts`（192 行）+ 18–27 行薄壳视图，三平台视图层合计 274 行。

## 9. 动画

**决策：motion 13.1.1。**

| 现状                                               | 处理                                                                             |
| -------------------------------------------------- | -------------------------------------------------------------------------------- |
| `animations.css` 580 行                            | 逐段判定：进出场类交给 motion，装饰类与关键帧保留 CSS                            |
| 12 处 Vue `Transition`                             | `AnimatePresence`，卸载动画由其接管                                              |
| `prefers-reduced-motion` 分散在多个组件的 `@media` | 与 `useAnimationVisibility.ts` 的逻辑合并为一处，用 motion 的 reduced motion API |
| `src/styles/animations/` 空目录                    | 填充或删除                                                                       |

约束：不允许 CSS 动画与 motion 对同一元素的同一属性并存。判定结果需逐段落盘。

## 10. 编辑器桥接

**决策：@uiw/react-codemirror 4.25.11。**

现状：`CodeSourceEditor.vue` 235 行（10 处 `EditorView` / `EditorState` / `Compartment` 引用）+ `ConfigSourcePanel.vue` 463 行。

前置核对（`08-22-dep-upgrade`）：`@uiw/react-codemirror` 自带的 `@codemirror/*` 依赖版本范围与现有 9 个直接依赖的兼容性。若产生重复实例（CodeMirror 6 对 `@codemirror/state` 的多实例敏感），需通过 `overrides` 收敛到单一版本。

`raw-config-editor-contracts.md` 的断言（语法高亮、JSON/Markdown 模式、lint 提示、搜索、快捷键）需能通过 `@uiw` 的 API 表达。无法表达的项逐条记录，达到阈值时评估换自建 hook（见第 12 节）。

## 11. 分支与评审策略

**决策：子分支 → 迁移分支 → dev。**

```
dev  ──────────────────────────────────────────────────►  (始终可发版)
  │                                                    ▲
  └─► feature/react-migration ────────────────────────┘  (最终一次 merge)
         ▲   ▲   ▲   ▲
         │   │   │   └─ feature/react-migration/regression-release
         │   │   └───── feature/react-migration/views-codex
         │   └───────── feature/react-migration/design-system
         └───────────── feature/react-migration/react-foundation
```

- 18 个子任务各开子分支，PR 目标为 `feature/react-migration`，逐个评审。
- `feature/react-migration` 的 PR 目标为 `dev`，在 `08-22-regression-release` 完成后合入。
- 迁移期 `dev` 的紧急修复定期 rebase 到 `feature/react-migration`。
- `08-22-workspace-cargo-upgrade`（子任务 2b）与前端迁移无技术依赖，其 PR 可直接目标 `dev`，不必经迁移分支。

C1 的 75 天不可发版窗口由此收敛为「迁移分支不可发版」，`dev` 不受影响。

## 12. 与推荐不同的三项选择：代价与缓解

用户在了解代价后选定以下三项，与本文档作者的推荐不同。缓解措施如下。

### 12.1 目录结构：全量按域聚合

代价：185 个文件同时更换框架与位置，AC11 的 185 界面逐屏比对与回归定位难度上升。`git log --follow` 的追溯能力下降。

缓解：

- `08-22-react-foundation` 产出旧路径 → 新路径的完整映射表并落盘。AC11 的逐屏比对按该表对照，不按目录浏览。
- 文件基名保持不变（`CodexSettingsView.vue` → `CodexSettingsView.tsx`），便于按名检索。
- 每个文件的移动与改写在同一提交内完成，提交粒度为单文件或单个紧密相关的小组，便于二分定位。
- 映射表纳入 `08-22-regression-release` 的前置输入。

### 12.2 动画：motion 13.1.1

代价：新增运行时依赖，与 R8.4 bundle 预算冲突。580 行 `animations.css` 与 motion 存在职责重叠。

缓解：

- `08-22-arch-quality-perf` 的 bundle 预算为 motion 与 zod 两者显式预留额度，并记录预留前后的对比数据。
- 580 行 CSS 逐段判定去留，判定结果落盘。禁止同一元素同一属性由两套机制同时驱动。
- `prefers-reduced-motion` 的处理收敛到一处，避免 CSS `@media` 与 motion API 双轨。

### 12.3 编辑器：@uiw/react-codemirror

代价：在 9 个已 pin 的 `@codemirror/*` 包之上多一层版本耦合。契约断言需经其 API 表达。

缓解：

- `08-22-dep-upgrade` 先核对 peer 依赖范围，必要时用 `overrides` 收敛 `@codemirror/state` 到单一版本，避免多实例。
- `raw-config-editor-contracts.md` 的断言逐条验证可表达性。无法表达的项累计超过 3 条时，`08-22-views-sync-tools` 评估换自建 hook（现有封装仅 235 行，自建成本约 200–250 行）。
- 该评估点写入 `08-22-views-sync-tools` 的 `implement.md` 作为显式检查门。

## 13. 兼容性

不变的契约：

- IPC 命令的名称与签名（334 base / 342 含 Windows 专属 8 条 / 271 typed，数据源 `ccr-ui/src/api/generated/command-manifest.json`）。
- 全部 Tauri Event 名称。
- `src/api/tauri.ts` 冻结门面的边界约定（新 wrapper 只落 `src/api/domains/<domain>.ts`）。该边界的定义面由既有测试 `api-facade-boundary.smoke.test.ts` 的 `freezes legacy direct invoke calls in tauri.ts` 用例冻结（当前允许集合为 9 条命令）。
- 204 个 ts-rs 生成类型（`ts-rs` 11 → 12 的重新生成差异由 `08-22-workspace-cargo-upgrade` 逐条判定）。
- 75 条路由路径。
- 4,164 个 i18n key 与两个 locale 的词条内容。
- 视觉偏好存储键（`ccr-theme` 等），旧值可正常解析。
- `data-theme` / `data-flavor` / `data-accent` 三层主题模型语义。
- 凭据掩码、原子写入、文件锁、备份四项行为（由 Rust 侧实现，前端不绕过）。

## 14. 回滚形状

| 阶段              | 回滚方式                                                                                |
| ----------------- | --------------------------------------------------------------------------------------- |
| 单个子任务        | 该子分支不合入 `feature/react-migration`，或 revert 其 merge commit                     |
| 依赖升级（2、2b） | 各自独立提交，可单独 revert。`ts-rs` 生成产物与 Rust 侧升级同提交，一并 revert          |
| 整体迁移          | `feature/react-migration` 不合入 `dev`。`dev` 全程保持 Vue 版本可发版状态，无需回滚动作 |
| 已合入 `dev` 后   | revert 迁移分支的 merge commit。`dev` 上迁移期间的独立提交不受影响                      |

`08-22-workspace-cargo-upgrade` 直接目标 `dev`，其回滚独立于前端迁移。

## 15. 未决项

- `commands/:client?` 的可选参数在 React Router 8 下的匹配行为需在 `08-22-shell-port` 实现时验证，若语义不等价则改为两条路由。
- zod 用于 IPC 返回值运行时校验的范围（第 7 节）在 `08-22-react-foundation` 试点后确定，不预先承诺。
- `08-22-platform-unify` 的 Auth 面统一范围由差异普查决定（R9.6）。
- 规模、复杂度、覆盖率三类阈值的具体取值由 `08-22-arch-quality-perf` 按现状分布确定。
