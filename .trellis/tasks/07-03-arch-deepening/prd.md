# 架构深化优化：deep module 重构总纲

## Goal

来自 2026-07-03 架构审查（/improve-codebase-architecture，4 个并行探索 agent 全仓扫描，报告：`%TEMP%/architecture-review-20260703-130500.html`）。目标：把散落为"调用方约定"的关键不变量收进少数深模块（deep module），提升 locality（变更/bug 集中一处）与可测性（接口即测试面）。本父任务持有需求来源、任务地图、跨子任务验收与最终集成审查，自身无直接实现工作。

## Requirements

### 任务地图（8 个子任务）

| 子任务 | 优先级 | 强度 | 核心问题 |
|---|---|---|---|
| 07-03-arch-guarded-write | P1 | Strong ⭐首推 | 持久化不变量（lock/backup/原子写/权限）散落 8+ 调用点，5 种写法漂移 |
| 07-03-arch-secret-newtype | P1 | Strong | 3 套掩码算法分叉 + WebDAV 密码明文无掩码 |
| 07-03-arch-usage-projection | P1 | Strong | ccr-usage 与 llmusage_adapter 重复实现同一 SQLite 投影（source.rs 逐字重复） |
| 07-03-arch-typed-ipc | P2 | Worth exploring | 244/312 Tauri 命令返回裸 Value，IPC 往返零测试 |
| 07-03-arch-claude-settings | P2 | Worth exploring | 同一 settings.json 两个 Rust shape，变更逻辑在贫瘠侧 |
| 07-03-arch-ccr-facade | P2 | Worth exploring | ccr 是 facade 套 facade，dispatch 远离其命令且无直接测试 |
| 07-03-arch-sqlite-seam | P3 | Speculative | 两套 SQLite 栈 + GLOBAL_POOL 不可测 + checkin 整体 re-export ccr-db 内部 |
| 07-03-arch-ccr-error | P3 | Speculative | CcrError 上帝枚举，领域词汇下漏进 primitives crate |

### 建议执行顺序与依赖

1. **第一批（互相独立，可并行）**：guarded-write、secret-newtype、usage-projection。
2. **第二批**：claude-settings、typed-ipc（试点单 domain）。
3. **第三批（结构性大改，先做否决式调研）**：ccr-facade、sqlite-seam、ccr-error。ccr-error 若确认要做，宜在 ccr-facade / sqlite-seam 动手前完成评估，因为错误类型迁移影响它们的接口签名。
4. 依赖关系写在各子任务 prd.md 中；父子结构本身不是依赖系统。

### 全局约束（所有子任务必须遵守）

- 尊重 4 份既有 spec 契约，不得翻案：`tauri-handler-registry.md`（309/317 冻结计数走注册表流程）、`api-facade-boundary.md`（tauri.ts 冻结，新 wrapper 进 domains/*）、`public-api-boundary.md`（legacy 根 re-export 冻结至 breaking release）、`llmusage-provider-adapter.md`（ccr-usage 拥有投影、upstream 只走 CLI + 只读 SQLite）。
- 保持 CLAUDE.md 关键不变量：secret 掩码、破坏性变更前备份、文件锁、原子写。
- 每个子任务独立可验证、独立可合并；一次一个 concern。
- 内部实现注释中文、公共 API 文档英文。

## Acceptance Criteria

- [ ] 8 个子任务各自的 prd.md 验收标准全部满足并归档。
- [ ] 跨子任务集成审查：`just ci` 全绿（version-sync → fmt → lint-strict → check-workspace → test → release → audit → frontend-check → vscode-ci）。
- [ ] 无新增的重复实现：掩码算法仅 1 处、原子写入口仅 1 处、usage 投影仅 1 处（以 rg 抽查验证）。
- [ ] 子任务中推翻/缩水的候选，其理由回写到对应 spec（trellis-update-spec），避免未来审查重复提议。

## Notes

- 需求来源：4 份探索报告（core/persist/ui/periph），关键事实已沉淀进各子任务 prd.md，无需回读会话。
- 审查还发现一处文档矛盾待澄清：根 CLAUDE.md 称 llmusage 为 "git dependency pinned rev"，而 llmusage_adapter/mod.rs 的不变量是"绝不链接 upstream crate"。在 usage-projection 子任务中一并澄清并修正文档。
