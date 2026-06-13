# WS6 批次④ modal 收口完成总结

## 完成状态

✅ 表单弹窗收口到 BaseModal（3 个）+ 死代码清理（2 个）+ 防回归锁定
◐ 2 个 bespoke 弹窗评估为「不宜强行收口」，给出建议

## PRD 8 个目标的处置

| 目标                         | 处置              | 说明                                                                                          |
| ---------------------------- | ----------------- | --------------------------------------------------------------------------------------------- |
| AddConfigModal               | ✅ 收口 BaseModal | size=4xl scrollable surface=solid；修坏类名/text-white/glass-surface；web 实测通过            |
| EditConfigModal              | ✅ 收口 BaseModal | 渐变标题→实色；彩虹分区色→text-text-muted；删 glass 别名；同构 AddConfig                      |
| CommandFormModal             | ✅ 收口 BaseModal | size=md；按钮留在 form 内保原生校验；web 实测通过                                             |
| UnifiedMcpFormModal          | 🗑️ 删除           | 死代码（父视图 WS2 已删，零引用）                                                             |
| UnifiedMcpDeleteConfirmModal | 🗑️ 删除           | 死代码（同上）                                                                                |
| GlobalConfirmDialog          | ✓ 已完成          | 早已委托 ConfirmModal（基于 BaseModal）                                                       |
| UpdateModal                  | ⚠️ 不宜收口       | 多阶段 + 顶部贴边装饰渐变线 + 自带 Transition，与 BaseModal 头/体/脚槽位模型冲突              |
| ProviderStatsModal           | ⚠️ 不宜收口       | 5xl 宽 + 头部含排序/刷新控件 + 固定高图表，BaseModal 适配差；已在批次③扁平化（surface-modal） |

## BaseModal 加性增强（不破坏既有 23 消费者）

- `size` 扩展 `2xl/3xl/4xl/5xl`。
- 新增 `scrollable` prop：开启「头/脚 `shrink-0` 固定 + 主体 `flex-1 overflow-y-auto min-h-0`」长表单滚动布局；
  未开启时所有 class 输出逐字节不变（`props.scrollable ? x : ''`），既有消费者零影响。

## 死代码补漏（WS2 遗漏）

`UnifiedMcpView.vue`（WS2 commit 1dcf84e7 删除）的 4 个专属子组件遗留为孤儿，已删：
UnifiedMcpFormModal / UnifiedMcpServerGrid / UnifiedMcpCommandBar / UnifiedMcpDeleteConfirmModal（共 1485 行）。
`unifiedMcp` API 域与类型仍由 McpManagerView 使用，保留。

## 防回归（WS7.2）

`apple-glass-surface-contract` 的 `styleLockedPaths` 新增 3 个收口弹窗，
禁调色板工具类 / raw rgb / glass 别名 / text-white。

## 验证

```
type-check 0 · lint 0 · smoke 78/351 全绿
web 预览实测：AddConfigModal、CommandFormModal 打开/Esc 关闭、扁平表面、表单/页脚布局正常
```

## 后续建议（未做）

- UpdateModal / ProviderStatsModal：建议作为「批次③ 旧语言孤岛就地扁平化」处理
  （去 backdrop blur、顶部渐变线收敛、rounded-3xl→2xl），而非强行 BaseModal 收口。
- z-index Tailwind 工具类（54 处）：数值已与 token 等价，局部堆叠语义保留数字更清晰，低收益，建议不做。
- 动效时长 token 化（80 处）：多为关键帧/错峰延迟不宜动；仅精确匹配的过渡时长可机会性收敛。
