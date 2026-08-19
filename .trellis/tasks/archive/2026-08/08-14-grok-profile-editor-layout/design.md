# 技术设计：Grok Profile 编辑器滚动与排版

## 1. 边界

只改 `GrokProfileEditorModal` 的外壳、分段导航与 i18n。视图仍拥有表单状态与 `buildGrokPatch`；模态仍只做展示、校验汇总与关闭/保存事件。

不改：

- `grokProfileEditor.ts` / patch 语义
- `GrokProfilesView` 的打开/保存编排（除非要把 footer 事件从 `#footer` 挪进默认槽，调用点不变）
- Claude/Codex 编辑器，除非共享 `profile-editor-shell.css` 出现真实回归
- `BaseModal` 默认行为

## 2. 高度链

当前失败链：

1. `BaseModal` 面板：`overflow-hidden`，未开 `scrollable`，无限高。
2. `content-class` 把 `pe-shell` 打在面板上；`.pe-scroll` 在 body 槽里，不是限高 flex 子项。
3. 打开时 `document.body { overflow: hidden }`。
4. 整卡被视口裁切，内部与页面都滚不动。

目标结构对齐 Claude/Codex，**不开** `scrollable`（避免 body 滚动 + `pe-scroll` 双滚动）：

```
BaseModal
  content-class="pe-modal …"（不再把 pe-shell 打在面板上）
  #header → pe-modal__head（eyebrow / 标题 / enabled pill / 关闭）
  default slot:
    .pe-shell.max-h-[calc(90vh-9rem)].overflow-hidden
      .pe-summary（校验失败 + 跳转）
      .pe-nav（段导航）
      .pe-scroll（唯一滚动根）
        #identity / #connection? / #runtime / #status
      .pe-footer（hint + Cancel + Save）
```

`.pe-scroll` 已有 `flex: 1; min-height: 0; overflow-y: auto`。限高必须落在它的直接祖先 `pe-shell` 上。

官方短表单：壳有 max-height，内容不足时 `pe-scroll` 不溢出，不出现无意义滚动条。

## 3. 分段与导航

| id | 可见条件 | 内容 |
| --- | --- | --- |
| `identity` | 始终 | kind 切换、name、provider（第三方）、description |
| `connection` | `profileKind === 'third_party'` | base URL、凭据状态、credential action |
| `runtime` | 始终 | model、reasoning；第三方另含 api backend / context window / backend search |
| `status` | 始终 | tags、enabled |

导航项由 `profileKind` 计算。官方不渲染 connection。kind 切换后重绑 IntersectionObserver。

scroll-spy 复用 Claude 模式：`root = pe-scroll`，`rootMargin: '-140px 0px -70% 0px'`。点击导航 `scrollTo` 对应 section。

校验错误带 section：

| 条件 | section |
| --- | --- |
| name 空 | identity |
| base URL / credential 动作或值 | connection |
| model / context window | runtime |

保存失败时展示 `pe-summary`，提供跳到首个错误分段的按钮。

## 4. 样式

- 继续只消费 `profile-editor-shell.css` 的 `pe-*` 与 `--cp-*`（Teleport 回退到 `--color-*`）。
- 平台色仅保留 kind 选中态的 `--color-platform-grok`，不扩大装饰。
- status 段补标题/说明，避免 Tags/Enabled 无分段收尾。
- 双列 `md:grid-cols-2` 保留；窄宽单列。
- 不新增平行 token，不改 `tokens.css`。

## 5. 兼容

- 字段 id（`#grok-profile-name` 等）保持，现有 smoke 选择器继续有效。
- BaseModal 测试 mock 只渲染 default + footer。页脚改走 default 后，现有测试仍能看到按钮；header 槽在 jsdom 中可不出现。
- write-only、kind 锁定、官方隐藏连接控件，行为不变。

## 6. 取舍

| 方案 | 结论 |
| --- | --- |
| 只开 `BaseModal.scrollable` | 能滚，但无段导航，且与 Claude/Codex 壳分叉。已否决。 |
| 对齐 Claude/Codex `pe-shell` | 采用。同一套限高/滚动/导航/粘性页脚。 |
| 改 `BaseModal` 默认 scrollable | 超出范围，会波及其他短模态。 |
| 抽共享 scroll-spy composable | 第三次复制才抽。本次内联对齐 Claude 即可。 |

## 7. 回滚

改动集中在一个 Vue 组件 + zh/en 文案 + 一条 smoke。回退该提交即恢复旧弹窗。共享 CSS 默认不动。
