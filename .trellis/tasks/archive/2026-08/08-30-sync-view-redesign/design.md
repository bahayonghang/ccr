# 技术设计：Sync 页布局与交互重构

## 设计方向

- 模式：**Operate**（操作台）。扫描性、状态真实性、操作主次优于表达。
- 视觉世界：沿用既有 "Editorial Control Room"（`ccr-ui/DESIGN.md`），不新增主题/令牌；仅使用现有 surface / accent / semantic tokens。实施前加载 impeccable `reference/craft-floor.md`。
- 结构论点：页面按「前置条件 → 同步对象 → 操作反馈 → 辅助信息」的优先级线性组织；主栏是任务区，侧栏是状态与反馈区。

## 布局拓扑（桌面）

```
PageHeader: 标题 + 副标题 | [连接状态 chip] [Refresh(ghost)] [Sync all(primary pill)] [Force retry all(warning, 条件)]
（移除 Back to Home —— 与全局侧边栏导航重复）

[Gating 区 —— 仅未配置/不可达时出现，置于主栏顶部]
  未配置: 引导卡（图标 + 说明 + "配置 WebDAV 账户" CTA，打开 SyncAccountDialog）
  不可达: 警告横幅（说明 + "重新测试" 按钮）

主栏 (sync-console-main):
  scope strip: 三个等分单元格，label 在上 value 在下，单元格间分隔线 —— 修复标签/值重叠
  console intro: eyebrow + 标题 + 描述；meta 两项改为有间距的 chip 行 —— 修复粘连
  asset groups: 组头（eyebrow + 标题 + 描述 + count chip）+ 资产卡片列表

侧栏 (sync-console-side)，按序：
  1. WebDAV Configuration 卡（状态 chip、server/username/remote path、Edit/Test/Disconnect）
  2. Operation Output 面板（新输出出现时滚动入视野）
  3. "关于同步" 折叠区（<details>/accordion，默认收起）：安全说明 + Features + Supported Services
     —— 原 safety 卡、Features 卡、Supported Services 卡合并入此
```

## 组件边界与改动

| 文件 | 改动 |
|---|---|
| `useSyncPage.ts` | 新增派生 `connectionState: 'unconfigured' \| 'unreachable' \| 'connected' \| 'unknown'`（由 `syncStatus.configured` / `remote_accessible` 推导）；新增 `accountDialogOpen` / `openAccountDialog()`，把 `SyncAccountDialog` 状态从 `SyncInfoSidebar` 上提，供 gating 引导卡与侧栏共用；`gated` 派生（`connectionState !== 'connected'` 时禁用同步操作） |
| `SyncView.tsx` | 头部动作区移除 Back to Home，连接状态 chip 进 header status；新增 gating 区；侧栏顺序调整；移除 safety 卡（并入折叠区） |
| `SyncAssetCard.tsx` | 操作主次：Sync 为 accent 小主按钮，Push/Pull 为 ghost 方向按钮，Force 为 warning（仅 offer 时）；修复 busy 旋转归属——仅 `busyAssetId === asset.id` 时该卡 Sync 按钮旋转，全局操作只禁用不旋转 |
| `SyncInfoSidebar.tsx` | 删除 Features / Supported Services 独立卡，改为「关于同步」折叠区并吸收 safety 列表；账户对话框改由 props 接收 open 状态（状态上提） |
| `styles/sync-view.css` | 修复 scope strip 重叠与 meta 粘连；新增 gating 卡/横幅、操作按钮层级、折叠区样式；全部使用现有 tokens |
| locale 文件（sync 命名空间，en + zh） | 新增 gating 文案、关于同步折叠区标题、按钮 aria 文案等 key |

## 交互契约

- **门控**：`connectionState === 'unconfigured'` 时主栏顶部显示引导卡，所有 Push/Pull/Sync/Sync all 按钮 `disabled` 且带原因提示（title/aria-describedby）；`'unreachable'` 时显示警告横幅并同样禁用同步按钮（配置操作可用）；`'unknown'`（未测试）不门控，仅在侧栏显示 untested chip。
- **忙碌归属**：per-asset 忙碌 → 仅该卡 Sync 按钮 spin + busy label；全局忙碌 → 仅头部 Sync all 显示进行态，各卡按钮禁用但不旋转。
- **Force retry**：保持现有 offer 机制（错误信息匹配 already exists/overwrite/force 时出现 warning 按钮），样式层级低于主操作。
- **反馈可见性**：操作完成后 Operation Output 面板更新并滚动入视野（`scrollIntoView`，尊重 reduced-motion 用 `behavior: 'auto'`）。
- **断开确认**：沿用现有 BaseModal 确认流，符合 confirm-interaction-contracts（不用原生对话框）。

## 状态清单

loading / load error / empty assets / 未配置（gating 引导）/ 不可达（警告横幅）/ 未测试 / per-asset 忙碌 / 全局忙碌 / force-retry offer / passphrase modal（不变）/ 折叠区展开收起。

## 约束与兼容

- 不改动后端同步逻辑、IPC payload、加密与 passphrase 生命周期（sync-security-contracts）。
- 遵守 layering-contracts（features/sync 内部改动，不跨层）；react-rerender-discipline（memo/useCallback 现状保持）。
- i18n：所有新文案走 locale 文件（en + zh），完成后跑 `bun run test:i18n`。
- 明暗双主题 AA 对比；motion 100–300ms，reduced-motion 兼容。
- 风险：SyncAccountDialog 状态上提涉及父子组件接口变化——保留 `SyncInfoSidebar` 对未配置/已配置两种渲染路径，仅把 open 状态改为受控。

## 验证

1. `cd ccr-ui && bun run type-check && bun run lint`
2. `cd ccr-ui && bun run test`（含 i18n 与 smoke）
3. `bun run dev:web -- --host 127.0.0.1 --strictPort` + Playwright 截图（desktop；Tauri-only invoke 失败视为环境限制）：覆盖已配置/未配置两种渲染路径、明暗主题各一轮。
4. 缺陷修复一轮后，跑 impeccable 检测器一次：`node C:\Users\lyh\.skillsmanage\skills\impeccable\scripts/detect.mjs --json <changed targets>`。
