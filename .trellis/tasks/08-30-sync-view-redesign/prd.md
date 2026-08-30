# 重新设计 ccr-ui 同步界面（Sync 页布局与交互重构）

## Goal

重新设计 `ccr-ui` 的 Sync 页面（`ccr-ui/src/features/sync/SyncView.tsx` 及同目录组件），使页面**界限分明、功能清晰、交互逻辑符合常理**，同时保持现有 "Editorial Control Room" 设计体系（`ccr-ui/DESIGN.md`，calm / precise / editorial，明暗双主题）。

用户价值：高级用户在执行真实的配置同步（涉及本地文件与凭据）时，能快速看清「同步什么、当前状态如何、下一步做什么」，不被重叠文字、混杂分区和含糊操作误导。

## Background（代码证据）

- 页面结构（`ccr-ui/src/features/sync/SyncView.tsx`）：PageHeader（标题 + Assets 徽章 + Refresh / Sync all once / 条件 Force retry all / Back to Home 链接）→ scope strip（CCR / Claude / Codex 三项范围摘要）→ 主区（console intro + 资产分组 + 资产卡片）+ 右侧栏（WebDAV 配置卡、Features 卡、Supported Services 卡、安全说明卡、操作输出面板）。
- 资产卡片（`SyncAssetCard.tsx`）：图标 + 名称 + Sensitive / v2 加密 / 类型徽章 + 描述 + 本地/远程路径 + 状态徽章 + Push / Pull / Sync（条件 Force）按钮。
- WebDAV 侧栏（`SyncInfoSidebar.tsx:85-103`）：已配置时显示 server/username/remote path + Edit / Test / Disconnect；未配置时显示说明 + Add CTA。
- 全局操作期间 `isBusy` 对所有资产卡片为 true（`SyncView.tsx:41`），导致每张卡片 Sync 按钮都旋转（`SyncAssetCard.tsx:110` busy 即 spin）。
- 门控数据已可得：`syncStatus.configured` / `syncStatus.remote_accessible`（`useSyncPage.ts:22`、`SyncInfoSidebar.tsx:9-17`）。

## 已确认的缺陷（截图 + 代码）

1. **文字重叠**：scope strip 标签与值重叠（截图 "CCRAll platform configs"）；console intro meta 两项粘连（"5 assetsSensitive values are masked"）。
2. **界限不清**：主区与右侧栏卡片同为 elevated 背景 + 相近圆角，缺乏层级；eyebrow 标签与标题视觉冲撞；右栏混杂 5 个不同性质区块。
3. **导航冗余**：PageHeader 的 "Back to Home" 与全局侧边栏导航重复。
4. **交互逻辑问题**：Sync all 语义不解释；每资产三按钮平权无主次；WebDAV 未配置/不可达时同步操作无门控；操作输出面板埋在右栏底部；全局忙碌时所有卡片 Sync 按钮一起旋转，状态归属误导。

## Requirements

- R1: 修复所有文字重叠/粘连缺陷（scope strip、console intro meta）。
- R2: 重新划分页面区域：主栏为资产任务区；侧栏按「WebDAV 配置 → 操作反馈 → 关于同步（折叠）」排序；操作输出面板提升可见性。
- R3: 功能主次分明：每资产 Sync 为主操作、Push/Pull 为次要方向操作；移除页头 "Back to Home"。
- R4: 交互门控【用户已确认】：WebDAV 未配置时主栏顶部显示引导卡（含配置 CTA）并禁用全部同步按钮；连接不可达时显示警告横幅并禁用同步按钮；未测试状态不门控。
- R5: 静态信息收纳【用户已确认】：Features、Supported Services 与安全说明合并为默认收起的「关于同步」折叠区，信息不丢失。
- R6: 忙碌状态归属清晰：仅正在操作的资产卡片显示旋转/忙碌标签；全局操作只禁用各卡按钮，不全部旋转。
- R7: 遵守 `ccr-ui/DESIGN.md` 与 `ccr-ui/AGENTS.md`：editorial 体系、黏土色克制、无新增装饰玻璃/渐变、明暗双主题 AA、i18n 走 locale 文件（en + zh 成对）。
- R8: 保留现有全部同步能力（per-asset push/pull/sync/force、sync all、force retry all、refresh、passphrase 流程、WebDAV 配置/测试/断开），仅重排/重分层，不删除功能。

## Acceptance Criteria

- [ ] AC1（R1）：scope strip 与 console intro meta 在桌面宽度无明暗主题下无文字重叠/粘连（截图验证）。
- [ ] AC2（R2/R5）：侧栏依次为 WebDAV 配置卡、操作输出面板、「关于同步」折叠区（默认收起，展开可见安全说明/Features/Supported Services）。
- [ ] AC3（R3）：页头无 Back to Home；资产卡片 Sync 为主按钮样式，Push/Pull 为次要样式。
- [ ] AC4（R4）：WebDAV 未配置时主栏顶部出现引导卡且所有同步按钮禁用；不可达时出现警告横幅且同步按钮禁用；配置/测试/断开操作始终可用。
- [ ] AC5（R6）：单资产操作时仅该卡显示忙碌；Sync all 时仅头部按钮显示进行态。
- [ ] AC6（R7）：`bun run test:i18n` 通过；明暗主题关键文字/标签对比度满足 AA；无新增硬编码色值（用 tokens）。
- [ ] AC7（R8）：`bun run type-check && bun run lint && bun run test` 全绿；passphrase 流程与 force retry 行为不变。

## Out of Scope

- 后端同步逻辑、Tauri 命令、IPC payload、加密流程改动。
- 全局设计体系（DESIGN.md 令牌）变更；其他页面。

## Key Decisions

- D1：静态信息卡折叠收纳而非移除（用户选择，保留信息、右栏紧凑）。
- D2：未配置/不可达时门控 + 引导，而非仅警告（用户选择，避免点了才报错）。
- D3：停留在既有 Editorial Control Room 视觉世界，本任务不做视觉世界替换。

## Risks / Deferred

- SyncAccountDialog 状态需从 `SyncInfoSidebar` 上提到页面层以供 gating 引导卡复用（见 design.md 组件边界）。
- web 预览中 Tauri-only invoke 失败视为环境限制，不算缺陷。

## Notes

- 技术设计与组件边界：`design.md`；执行顺序与验证命令：`implement.md`。
- 视觉验证：`bun run dev:web -- --host 127.0.0.1 --strictPort` + Playwright（desktop 明/暗、已配置/未配置两路径）。
