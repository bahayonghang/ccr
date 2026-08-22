# 技术设计：Pinia store 与 composable 迁移

> 父任务：`08-22-react-migration`。状态三分类见父任务 `design.md` §4，判定表由 `08-22-arch-quality-perf` 的 `state-disposition.md` 提供。本文件写形态约定与语义差异的逐类处理。

## 1. Zustand store 形态约定

```ts
// features/... 或 shell/stores/<name>.ts
interface UiState {
  favorites: string[];
  addFavorite: (id: string) => void;
}

export const useUiStore = create<UiState>()((set) => ({
  favorites: [],
  addFavorite: (id) => set((s) => ({ favorites: [...s.favorites, id] })),
}));
```

约定：

- 模块级单例，无 Provider。跨组件单例语义与 Pinia 等价（R1）。
- **消费必须用选择器**：`useUiStore((s) => s.favorites)`，禁止 `useUiStore()` 整 store 订阅。该约束由 `08-22-arch-quality-perf` §6 的 lint 规则强制。
- action 与 state 同置于 store 内（Zustand 的常规形态），不拆 action 文件。
- 公开 API 命名沿用原名（R7）：`useUiStore` 的属性名与 Pinia `useUiStore` 的一致，减少调用点改动面。
- 需要持久化的 store（`shellPreferences`）用 `persist` 中间件，存储键与现有键一致。

`computed` 的 13 处转为选择器内计算，或用 `useShallow` 避免引用变化触发重渲染。选择器返回对象时必须用 `useShallow`，否则每次渲染返回新引用导致无限重渲染——这是 Zustand 最常见的接线错误，写入本任务的自检项。

## 2. TanStack Query 形态约定

```ts
export const usageKeys = {
  all: ["usage"] as const,
  summary: (platform?: string) =>
    [...usageKeys.all, "summary", platform] as const,
};

export function useUsageSummary(platform?: string) {
  return useQuery({
    queryKey: usageKeys.summary(platform),
    queryFn: () => getUsageSummaryV2(platform),
  });
}
```

约定：

- queryKey 用工厂对象，每个域一个（`usageKeys`、`configsKeys`、`commandsKeys`、`claudeObserverKeys`、`homeUsageKeys`）。禁止内联数组字面量，否则失效范围无法精确表达。
- `queryFn` 只调 `src/api` 下的现有 wrapper，不新增 wrapper（Out of Scope：`src/api` 保持原样）。
- `staleTime` 逐 query 设定。默认值在 `queryClient` 上设一个保守值（`08-22-react-foundation` 已建 `queryClient.ts`），逐 query 按数据变更频率覆盖。
- 写操作用 `useMutation`，成功后 `invalidateQueries` 到对应 key 前缀。

## 3. Tauri Event 与 Query 的桥接

父任务 `design.md` §4：后端 `emit` 的事件在监听回调中调用 `invalidateQueries` 或 `setQueryData`，store 不再直接持有服务端数据。

桥接层形态：

`listen()` 的返回类型是 `Promise<UnlistenFn>`（`@tauri-apps/api/event`）。effect 的 cleanup 可能在该 Promise resolve 之前执行——StrictMode 的挂载 → 卸载 → 再挂载，以及快速路由切换都会触发这一时序。此时迟到 resolve 的 unlisten 不会进入已执行完的 cleanup，该监听器永久泄漏。因此桥接层必须带取消协议，不能只把 unlisten 推进数组：

```ts
// shell/eventBridge.ts
export function useTauriEventBridge() {
  const qc = useQueryClient();
  useEffect(() => {
    let disposed = false;
    const unlistens: UnlistenFn[] = [];

    // 取消协议：cleanup 已跑过时，迟到的 unlisten 立即调用，不入数组
    const track = (p: Promise<UnlistenFn>) =>
      p.then((un) => {
        if (disposed) un();
        else unlistens.push(un);
      });

    track(listen("app-log", onAppLog));
    // 每个事件一条 track(listen(...))

    return () => {
      disposed = true;
      unlistens.forEach((un) => un());
      unlistens.length = 0;
    };
  }, [qc]);
}
```

约定：

- 桥接层集中在一处，挂在应用外壳（`08-22-shell-port` 接线）。全局事件名清单集中可见。**局部事件不进桥接层但必须登记**：`08-22-views-checkin` 的 WAF 一次性等待是组件级 `listen()`（其 `design.md` §4），这类事件需登记到统一的前端事件 inventory，否则 `08-22-test-contract-rebuild` AC6 的「全部 Tauri Event 名」断言只覆盖桥接层，漏掉局部事件。inventory 的字段：事件名、所有者（`eventBridge` 或具体组件）、生命周期（常驻 / 一次性）、对应的 Rust `emit` 位置。本任务提供全局部分，协同点 M。
- **`setQueryData` 与 `invalidateQueries` 的选择**：事件 payload 含完整新数据用 `setQueryData`（避免多余 IPC 往返）；payload 只是变更通知用 `invalidateQueries`。逐事件判定并记录。
- 高频事件（`app-log`、`token-stats`）不能逐条 `setQueryData`——每条触发一次重渲染。这两个事件走 ref 累积 + 定时批量提交，间隔在实施时按 `08-22-arch-quality-perf` 的场景 3 基线数据定。该形态写入 `react-rerender-discipline.md` 的对应条目。
- **StrictMode 下 effect 双调用**：`listen()` 的建立必须幂等或在 cleanup 中完整解绑，且遵守上文的取消协议。组件级 `listen()`（如 WAF 等待）同样适用该协议。

## 4. 10 个 store 的拆分

父任务 `design.md` §4 已给出处理，本任务补充拆分后的落位与接口：

| store                                  | Query 侧                      | Zustand 侧                              | 落位                 |
| -------------------------------------- | ----------------------------- | --------------------------------------- | -------------------- |
| `usage.ts`                             | 用量数据                      | 视图偏好（时间范围、平台维度）          | `features/usage/`    |
| `configs.ts`                           | 配置列表                      | 选中态、搜索词、表单草稿（键为配置 id） | `features/configs/`  |
| `commands.ts`                          | 命令数据                      | —                                       | `features/commands/` |
| `claudeObserver.ts`                    | 事件流数据（配合 Event 失效） | UI 态                                   | `features/claude/`   |
| `homeUsageOverview.ts`                 | 全部                          | —                                       | `features/usage/`    |
| `ui.ts`                                | —                             | toast / 收藏 / 历史                     | `shell/stores/`      |
| `shellPreferences.ts`                  | —                             | 全部（带 `persist`）                    | `shell/stores/`      |
| `commandsView.ts`                      | —                             | 全部                                    | `features/commands/` |
| `usageDashboardPayload.ts`（171 行）   | —                             | —                                       | `utils/`，纯变换     |
| `usageImportNormalization.ts`（83 行） | —                             | —                                       | `utils/`，纯变换     |

后两者的纯变换判定依据需记录（PRD Notes）：判定标准为该模块是否持有跨调用存活的状态。只做输入到输出映射的移入 `utils/`。

现状数据支撑迁移可行性：10 个 store 共 21 处 `ref`、13 处 `computed`、**0 处 `watch`、0 处 `reactive`**。无 `watch` 意味着 store 内无订阅式副作用，迁移不需要处理时序。

## 5. 35 个 composable 的三类归类

| 类               | 判据                                          | 处理                                                              |
| ---------------- | --------------------------------------------- | ----------------------------------------------------------------- |
| 纯逻辑           | 不导入 `vue`，无 `ref` / `computed` / `watch` | 原样复用或改函数签名                                              |
| 响应式状态       | 有 `ref` / `computed`，无生命周期钩子         | `ref` → `useState`，`computed` → `useMemo`，`watch` → `useEffect` |
| 生命周期与副作用 | 有 `onMounted` / `onUnmounted` / `listen()`   | `useEffect` + cleanup，订阅解绑时机逐个复核                       |

归类方法：`rg -l "from 'vue'" src/composables` 分出纯逻辑；剩余的按 `rg -c 'onMounted|onUnmounted|listen\('` 分出第三类。

归类清单落盘为 `composable-classification.md`，35 行无空缺（AC3）。

## 6. 语义差异的逐类处理

### 6.1 `ref` 深层响应式 → 不可变更新

风险：现有代码可能依赖就地修改（`arr.push(x)`、`obj.field = v` 后 Vue 自动触发更新）。React 的 `useState` 与 Zustand 的 `set` 都要求新引用。

排查方法：在迁移每个 store / composable 时，`rg` 其内部对状态对象的 `.push(`、`.splice(`、`.sort(`、`[i] =`、`.field =` 形态。逐点改为不可变写法。

改写清单落盘为 `mutation-rewrite.md`（AC6），列：文件、行、原写法、新写法。

### 6.2 `computed` → `useMemo` / 选择器

风险：依赖集合遗漏导致陈旧值。

处理：`computed` 的依赖是自动追踪的，`useMemo` 是手写的。逐个 `computed` 列出其读到的全部响应式来源，作为依赖数组。`react-hooks/exhaustive-deps`（error 级别，`08-22-arch-quality-perf` R3）拦截遗漏。

13 处 `computed` 逐个核对（R6）。

### 6.3 `watch` 的选项

现状：10 个 store 内 0 处 `watch`。composable 内的 `watch` 数量在归类时统计。

`immediate` / `deep` / `flush` 三个选项无 `useEffect` 等价物：

| Vue 选项           | React 处理                                  |
| ------------------ | ------------------------------------------- |
| `immediate: true`  | `useEffect` 默认首次即执行，语义相同        |
| `immediate: false` | 需一个 ref 标记跳过首次执行                 |
| `deep: true`       | 依赖数组放序列化后的值，或改为监听具体字段  |
| `flush: 'post'`    | `useEffect`（默认在 paint 后）              |
| `flush: 'sync'`    | 无等价物。改为在触发处直接调用，不用 effect |
| `flush: 'pre'`     | `useLayoutEffect`                           |

逐点登记，含原选项与替代实现。

### 6.4 `nextTick`（全仓 52 处）

`nextTick` 无 React 等价物。按原始意图分类替代：

| 原始意图                                            | React 替代                         |
| --------------------------------------------------- | ---------------------------------- |
| 等 DOM 更新后测量尺寸 / 位置                        | `useLayoutEffect`，或 ref callback |
| 等 DOM 更新后 focus                                 | ref callback，或 `useLayoutEffect` |
| 等 DOM 更新后滚动                                   | `useLayoutEffect`                  |
| 等 DOM 更新后同步第三方库（CodeMirror、ApexCharts） | 该库的 ref + `useLayoutEffect`     |
| 状态更新后立即读新 DOM（同一事件内）                | `flushSync`，仅在无替代时使用      |
| 等浏览器绘制一帧（动画起点）                        | `requestAnimationFrame`            |

52 处中落在本任务范围（store 与 composable 内）的逐点登记（R3、AC4），落盘为 `next-tick-register.md`。落在视图内的由各视图子任务登记（其对应 AC）。

### 6.5 `provide` / `inject`（各 1 处）

直接映射为 React Context。1 对，改动面小。

### 6.6 Pinia 单例 → Zustand 单例

订阅粒度变化：Pinia 组件读 store 属性即建立细粒度依赖；Zustand 靠选择器。选择器写错会导致整 store 订阅，重渲染范围扩大。第 1 节的选择器约束与 lint 规则覆盖此点。

## 7. 订阅泄漏检测（AC5）

方法：写一个测试，对每个含 `listen()` 的 hook 做 100 次挂载与卸载，断言监听器数量回到基线。

计数手段：mock `@tauri-apps/api/event` 的 `listen`，记录调用次数与返回的 unlisten 被调用次数，断言两者相等。不依赖真实 Tauri 运行时。

StrictMode 下开发模式 effect 双调用会使 listen 次数翻倍——断言的是 listen 次数与 unlisten 次数相等，不是 listen 次数等于 100，因此该断言在 StrictMode 下仍成立。

**mock 必须延迟 resolve。** 同步（或立即）resolve 的 mock 会让第 3 节描述的泄漏形态无法暴露：cleanup 先于 resolve 执行时，迟到的 unlisten 不进入已执行完的 cleanup。因此测试须包含三个用例：

| 用例 | 时序                                                       | 断言                              |
| ---- | ---------------------------------------------------------- | --------------------------------- |
| 1    | `listen` 立即 resolve，正常挂载卸载 100 次                 | listen 次数 == unlisten 次数      |
| 2    | `listen` 的 Promise 在**卸载之后**才 resolve               | 该 unlisten 仍被调用（取消协议）  |
| 3    | StrictMode 下挂载 → 卸载 → 再挂载，`listen` 延迟 resolve   | listen 次数 == unlisten 次数      |

用例 2 与 3 是本节的实质内容，用例 1 单独通过不构成 AC5 满足。

## 8. 单元测试范围

每个 store 的核心状态转移（AC8）。「核心状态转移」的定义：该 store 的每个 action 至少一个用例，覆盖初始态到目标态的转换。

Query 侧的 hook 测试用 `QueryClientProvider` 包裹 + mock 的 `queryFn`，不打真实 IPC。

## 9. 未决项

- 35 个 composable 的具体归类，按第 5 节的方法测量后确定。
- 高频事件的批量提交间隔，按 `08-22-arch-quality-perf` 场景 3 的基线数据确定。
- 逐事件的 `setQueryData` / `invalidateQueries` 选择，按第 3 节的判据在实施时逐个确定。
- `shellPreferences.ts` 与 `themeBootstrap.ts`、`fontPreferences.ts` 的耦合接口，需与 `08-22-shell-port` 对齐（PRD Notes）。
