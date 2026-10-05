# 执行计划：Sync 页布局与交互重构

## 检查清单（按序）

1. **Locale 文案**：在 sync 命名空间（en + zh）新增 key：gating 引导卡（未配置标题/说明/CTA）、不可达警告横幅、关于同步折叠区标题、操作按钮原因提示。先找到现有 sync locale 文件（`src/features/sync/locale*` 或全局 i18n 资源）。
2. **`useSyncPage.ts`**：派生 `connectionState` 与 `gated`；上提 `accountDialogOpen` / `openAccountDialog()`；保持现有公开 API 其余不变。
3. **`SyncAssetCard.tsx`**：操作按钮层级（Sync 主 / Push Pull ghost / Force warning）；busy 旋转归属修复（仅 `busyAssetId === asset.id` 旋转）。
4. **`SyncInfoSidebar.tsx`**：移除 Features / Supported Services 独立卡；新增「关于同步」折叠区（含安全列表）；`SyncAccountDialog` 改受控（props 传入 open/onOpenChange）。
5. **`SyncView.tsx`**：移除 Back to Home；header status 加连接 chip；主栏顶部 gating 区；侧栏顺序 = WebDAV 卡 → Output 面板 → 折叠区；output 更新后 scrollIntoView。
6. **`styles/sync-view.css`**：修 scope strip 重叠（grid 三列、label/value 分行）、console intro meta 间距；gating 卡/横幅、按钮层级、折叠区样式；只用现有 tokens。
7. **静态检查**：`cd ccr-ui && bun run type-check && bun run lint && bun run test`。
8. **视觉验证**：`bun run dev:web -- --host 127.0.0.1 --strictPort` + Playwright 截图（desktop，明/暗，已配置/未配置两路径）；一轮批量修复 + 至多一轮确认。
9. **Impeccable 检测器**：完成后跑一次 detect.mjs（见 design.md 验证节）。
10. **实施前**：加载 impeccable `reference/craft-floor.md`（质量地板与禁令）。

## 验证命令

- `cd ccr-ui && bun run type-check`
- `cd ccr-ui && bun run lint`（no-fix；修问题用 `lint:fix` 需谨慎）
- `cd ccr-ui && bun run test`
- 视觉：`cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`，目标 `http://127.0.0.1:5173/`

## 风险与回滚点

- SyncAccountDialog 状态上提：改动集中在 sync feature 内部，回滚 = 还原该 feature 目录。
- i18n key 缺失会致 test:i18n 失败——en/zh 必须成对添加。
- 不触碰 `src/api/**`、Tauri 后端、加密流程。

## 子代理派发

Phase 2 由主会话派发 `trellis-implement`（Kimi Code: 内置 coder 子代理 + `.kimi-code/skills/trellis-implement/SKILL.md`），prompt 首行 `Active task: .trellis/tasks/08-30-sync-view-redesign`。
