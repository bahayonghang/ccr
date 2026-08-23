# 技术设计：应用外壳、路由与 Tauri 接线

> 父任务：`08-22-react-migration`。路由映射方案见父任务 `design.md` §3，缓存路由替代见 §5。本文件写路由对象结构、外壳组合与接线细节。

## 1. 路由对象结构

现状：`src/router/index.ts` 594 行，2 个顶层条目。

```
/tray/codex          独立窗口，不套 MainLayout
/                    布局父级，children 承载其余约 73 条
```

React Router 8 的同构映射：

```tsx
createBrowserRouter([
  {
    path: "/tray/codex",
    element: <TrayCodexView />,
    handle: { hideSidebar: true },
  },
  {
    path: "/",
    element: <MainLayout />, // 内含 <Outlet />
    children: [/* ~73 条 */],
  },
]);
```

`handle` 承载 8 个原 `meta` 字段，保留原名：`cache`、`cacheKey`、`hideGlobalBackground`、`stream`、`depth`、`group`、`hideSidebar`、`deferLocaleHydration`。

类型声明：原为 `RouteMeta` 的模块augmentation。React Router 的 `handle` 是 `unknown`，因此改为一个显式 `RouteHandle` 接口 + 一个读取函数 `useRouteHandle(): RouteHandle`，在读取处做一次类型收窄。丢失编译期强制（`handle` 写错字段不报错），补一个 smoke 测试遍历路由表断言每条的 `handle` 字段名在允许集合内。

`/tray/codex` 不套布局：它是独立窗口的入口。tray 窗口有自己的 HTML 入口还是共用同一入口靠路径区分，需在实施时确认（影响 `vite.config.ts` 的多入口配置与 Tailwind v4 的源文件检测范围）。

## 2. 程序化生成的路由

mcp / agents / plugins 路由由 `genericPlatformDescriptorList`（`src/config/platformDescriptors.ts` 50 行）生成。

生成逻辑保留，输入改为 `08-22-platform-unify` 扩展后的 descriptor（父任务 `design.md` §8：descriptor 声明「该平台有哪些面」，驱动路由生成与导航）。

依赖顺序：本任务在 `08-22-platform-unify` 之前（阶段 3 对阶段 4）。因此本任务先按现有 descriptor 生成，接口按新 descriptor 的形状预留。descriptor 扩展后由 `08-22-platform-unify` 替换输入，路由路径不变（其 R9、AC8）。

## 3. 导航守卫的替代

| 现状                                                                          | 替代                                                                                                                                                |
| ----------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `router.beforeEach`（`:532`，perf 埋点 + locale 预热）                        | 拆两处：perf 埋点进 `MainLayout` 的 `useEffect`（依赖 `location`）；locale 预热由 `deferLocaleHydration` 的 handle 字段驱动，放在路由的 `loader` 内 |
| `router.afterEach`（`:543`，`recordRouteTiming`）                             | `MainLayout` 内监听 `location` 变化的 `useEffect`，在 paint 后记录                                                                                  |
| `usePageTransition.ts` 的 `beforeEach`（比较 `depth` / `group` 决定过渡方向） | 同一比较逻辑移到布局组件，读 `handle.depth` / `handle.group`。需保留前一次路由的 handle——用一个 ref 存上一次值                                      |

`loader` 与 `useEffect` 的分工判据：需要阻塞渲染直到完成的进 `loader`（locale 预热属此类，否则首帧显示 key 原文）；不影响渲染的进 `useEffect`（埋点属此类）。

## 4. 缓存路由替代的落地

父任务 `design.md` §5 决策：状态外提到 store，不做组件常驻。

5 条路由的具体处理：

| 路由                | 需保留的状态                      | 落位                                   |
| ------------------- | --------------------------------- | -------------------------------------- |
| `dashboard`         | 数据                              | Query 缓存，无额外代码                 |
| `grok`              | 数据 + 选中态                     | Query + Zustand                        |
| `commands/:client?` | 数据 + 流式输出（`stream: true`） | Query + Zustand 的累积缓冲，切回时续读 |
| `configs`           | 数据 + 选中态 + 搜索词 + 表单草稿 | Query + Zustand（草稿键为配置 id）     |
| `usage`             | 数据 + 时间范围 + 平台维度        | Query + Zustand                        |

**流式输出的续读**（`commands` 的 `stream: true`）是本节唯一非平凡项。设计：累积缓冲在 Zustand 里按 client 键存一个数组，事件桥接层（`08-22-state-logic-port` §3）持续 append，视图组件订阅该数组。视图卸载不清空缓冲，返回时直接渲染已有内容。缓冲上限需设（否则长时间运行内存持续增长），上限值与截断策略在实施时定。

滚动位置：React Router 的 `<ScrollRestoration />`，不入 store。需确认它对布局内滚动容器（非 window 滚动）是否生效——本仓的内容区可能是内部滚动容器。若不生效，改为按路由 key 存滚动位置到一个 ref map，在 `useLayoutEffect` 中恢复。该确认是实施时的第一个检查项。

`MainLayout` 从 `meta.cache === true` 派生 `cacheKey` 数组的逻辑删除。`cacheKey` 字段保留在 `handle` 内，仅用于 `path-mapping.md` 的追溯。

## 5. 懒加载

22 处 `defineAsyncComponent`。

替代：React Router 的路由级 `lazy` 属性（返回 `{ Component }`），而非 `React.lazy` + `Suspense`。理由是路由级 `lazy` 与路由定义同置，无需在每处包 `Suspense`，且加载发生在导航期而非渲染期。

约定（`08-22-arch-quality-perf` 的 `code-splitting.md`）：懒加载边界与路由边界一致，不在路由内部再分割。

首屏加载的模块集合不扩大（R3、AC3）。验证方式：`bun run build` 后对比 entry chunk 的模块清单与迁移前基线。

## 6. `Teleport` 与 `Transition`

| 现状               | 替代                                                                                  |
| ------------------ | ------------------------------------------------------------------------------------- |
| 8 处 `Teleport`    | `createPortal`。落在本任务范围内的部分（Toast 容器、全局确认弹层、Titlebar 相关）改写 |
| 12 处 `Transition` | `AnimatePresence`（motion 13.1.1），卸载动画由其接管                                  |

`AnimatePresence` 的接入需与 `08-22-design-system` 的 `animation-disposition.md` 一致：该文件已逐段判定哪些动画交给 motion。本任务只改写落在外壳范围内的部分。

`AnimatedBackground.vue` 与 `StageBackground.vue` 属装饰性持续动画，按 `animation-disposition.md` 的判定应保留 CSS 驱动，但需遵循 reduced motion 降级（PRD Notes）。

## 7. `defineExpose` → `useImperativeHandle`

全仓 8 处。落在本任务范围内的部分（R9）逐个判定：

| 判定                                 | 条件                                          |
| ------------------------------------ | --------------------------------------------- |
| `useImperativeHandle` + `forwardRef` | 父组件调用子组件的方法（如 `dialog.open()`）  |
| 改为受控属性                         | 暴露的只是状态（如 `isOpen`），可提升到父组件 |

优先「改为受控属性」。`useImperativeHandle` 引入命令式接口，与 React 的数据流方向相反，只在无替代时使用。判定逐处记录。

## 8. `MasterDetailLayout` 接口定稳

`src/components/common/MasterDetailLayout.vue` 被多个视图子任务消费，接口变更会波及七个并行子任务（PRD Notes、协同点 H）。

定稳的含义：接口在阶段 5 开始前公示，阶段 5 期间不改。公示内容为：

- props 的完整列表与类型。
- slot → children / render props 的映射（Vue slot 到 React 的转换是接口形态变化，必须一次定完）。
- 列表侧与详情侧的滚动、选中、空状态、加载态的责任划分（哪些由布局承担，哪些由消费方传入）。

同一要求适用于 `src/ui/` 的 16 个原语（原 `src/components/ui/`，`08-22-design-system` 已判定替换或保留并迁到 `src/ui/`，本任务落地消费点）。

接口文档落盘为 `shared-interfaces.md`，是阶段 4 → 5 门的准出项。

## 9. Tauri 与运行时接线

11 个 `src/utils` 文件（由 `08-22-react-foundation` 的 `utils-disposition.md` 标记为「需接线」）：

| 文件                                                               | 接线点                                                                  |
| ------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| `windowChrome.ts` / `tauriWindow.ts` / `nativeWindowAppearance.ts` | `Titlebar` 组件与应用启动时                                             |
| `themeBootstrap.ts`                                                | 应用启动最早期（在 React 挂载前执行，避免主题闪烁），三层模型解析与写入 |
| `fontPreferences.ts`                                               | 与 `themeBootstrap` 同期                                                |
| `startupRecovery.ts`                                               | 应用启动，React 挂载后                                                  |
| `perfTelemetry.ts`                                                 | 路由变更监听（第 3 节）与启动埋点                                       |
| `runtimeState.ts` / `tauriRuntime.ts`                              | 全局，判定是否在 Tauri 内运行                                           |
| `errorHandler.ts`                                                  | React `ErrorBoundary` + `window.onerror` / `unhandledrejection`         |
| `logger.ts`                                                        | 全局，含 `logRedact.ts` 脱敏                                            |

**`themeBootstrap` 必须在 React 挂载前执行**：主题写入 `<html>` 的 data 属性，若等 React 挂载后再写，首帧会用默认主题渲染然后跳变。在 `main.tsx` 的 `createRoot` 之前调用，或在 `index.html` 的内联脚本中调用。二选一取决于其是否依赖 Tauri API（依赖则不能内联，因为 `@tauri-apps/api` 需模块加载）。

`errorHandler.ts` 的 React 侧接入需要一个 `ErrorBoundary`——Vue 的 `app.config.errorHandler` 捕获渲染错误，React 需显式边界组件。边界放在 `MainLayout` 内（每条路由的错误不影响 Titlebar 与导航）与 `/tray/codex` 各一个。

日志脱敏行为不变（R8、AC8）。

## 10. 外壳组件的迁移

约 6,877 行，15 个条目（PRD Scope 表）。迁移顺序按依赖方向：`ui/` 原语消费点 → `common/` → 根级组件 → `MainLayout` → `App`。

16 个原语由 `08-22-design-system` 决定形态并迁到 `src/ui/`（原 `src/components/ui/`），本任务落地其消费点（PRD Scope）。

9 个根级组件归属其他子任务（PRD 表），本任务不动。

## 11. 未决项

- `commands/:client?` 的可选参数在 React Router 8 下的匹配行为需验证（父任务 `design.md` §15）。验证方法：构造 `/commands` 与 `/commands/claude` 两个导航，断言同一组件渲染且 `params.client` 分别为 `undefined` 与 `'claude'`。语义不等价则改为两条路由（`/commands` 与 `/commands/:client`），路径集合不变。
- `<ScrollRestoration />` 对内部滚动容器是否生效（第 4 节）。
- tray 窗口是否需要独立 HTML 入口（第 1 节）。
- 流式输出缓冲的上限值与截断策略（第 4 节）。
- `themeBootstrap` 放 `main.tsx` 还是 `index.html` 内联（第 9 节）。
- 8 处 `defineExpose` 的逐处判定（第 7 节）。
