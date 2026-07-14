# 配置资产同步页改版实现计划

> Archive status: implemented, retained as historical execution evidence. Archived on 2026-07-14.
> Current behavior is defined by `src/views/SyncView.vue` and `src-tauri/src/commands/sync.rs`.

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将 WebDAV 同步页从文件夹同步改为固定 manifest 配置资产同步控制台，支持 CCR platforms、Claude settings/CLAUDE.md、Codex config/AGENTS.md 的单项与全量同步，并移除 Gemini/Antigravity 同步入口。

**架构：** 后端新增 manifest asset 命令，前端通过 typed API 读取资产状态并渲染 Editorial Control Desk。保留现有 folder CRUD 命令兼容旧调用，但新页面不再使用 folder selection/custom folder UI。

**技术栈：** Vue 3 + TypeScript + Vue I18n + Tauri Rust commands + existing `ccr-sync::SyncService` WebDAV client。

---

## 文件职责

- 修改：`src-tauri/src/commands/sync.rs` — 定义 sync asset manifest、状态查询、单项/全量 push/pull/sync 命令、路径大小写归一与备份行为。
- 修改：`src-tauri/src/commands/mod.rs` — 注册新增 Tauri invoke 命令。
- 修改：`src/types/syncSelection.ts` — 增加前端 sync asset DTO 与 operation 状态类型。
- 修改：`src/api/domains/sync.ts` — 增加 manifest asset API wrappers。
- 重写：`src/views/SyncView.vue` — 使用资产控制台布局替换 folder selection/custom folder/batch panels。
- 修改：`src/components/sync/SyncInfoSidebar.vue` — 文案从“文件夹同步”改成“配置资产同步”说明，保留 WebDAV 账号管理。
- 修改：`src/i18n/locales/zh-CN.ts`、`src/i18n/locales/en-US.ts` — 新增/替换同步页文案，去掉 sync 页面 Gemini/Antigravity 文案。

## 任务 1：后端 manifest asset 命令

**文件：**
- 修改：`src-tauri/src/commands/sync.rs`
- 修改：`src-tauri/src/commands/mod.rs`

- [ ] **步骤 1：添加 manifest DTO 与固定资产定义**

在 `sync.rs` 的响应类型附近添加：

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SyncAssetKind { Directory, File }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncAssetInfo {
    pub id: String,
    pub group: String,
    pub name: String,
    pub description: String,
    pub kind: SyncAssetKind,
    pub sensitive: bool,
    pub local_path: String,
    pub resolved_local_path: String,
    pub remote_path: String,
    pub local_exists: bool,
    pub remote_exists: Option<bool>,
    pub canonical_name: Option<String>,
}
```

并增加固定 manifest 常量/函数，包含 `ccr-platforms`、`claude-settings`、`claude-memory`、`codex-config`、`codex-agents`。

- [ ] **步骤 2：实现路径解析与 Windows 大小写兜底**

添加 helpers：

```rust
fn resolve_asset_local_path(asset: &SyncAssetDefinition) -> Result<(PathBuf, String), String>
fn find_case_insensitive_child(parent: &Path, canonical_name: &str) -> Option<PathBuf>
fn backup_existing_path(path: &Path) -> Result<Option<PathBuf>, String>
```

规则：目录直接 expand；文件先 expand canonical path；若不存在且目标是 canonical Markdown 文件，则在父目录大小写不敏感查找同名文件。返回实际本地路径与 canonical display name。

- [ ] **步骤 3：实现状态查询**

添加 `#[tauri::command] pub async fn list_sync_assets() -> Result<Vec<SyncAssetInfo>, String>`：加载 WebDAV 配置；未配置时 remote_exists 为 None；已配置时逐项创建 `SyncService` 检查 remote_exists；本地状态用 `tokio::fs::try_exists`。

- [ ] **步骤 4：实现单项 push/pull/sync 与全量 sync once**

添加命令：

```rust
pub async fn sync_push_asset(id: String, force: Option<bool>) -> Result<SyncOperationResult, String>
pub async fn sync_pull_asset(id: String, force: Option<bool>) -> Result<SyncOperationResult, String>
pub async fn sync_asset(id: String, force: Option<bool>) -> Result<SyncOperationResult, String>
pub async fn sync_all_assets(force: Option<bool>) -> Result<SyncOperationResult, String>
```

`sync_asset` 的默认策略是 push 当前本地资产到远端；remote exists 且未 force 时返回清晰失败。`pull` 在本地存在且 force=true 时先备份。

- [ ] **步骤 5：注册命令并添加 Rust 单元测试**

在 `commands/mod.rs` 注册新增命令。给 helper 添加测试：manifest ids 完整、remote path 归一、Windows/case-insensitive helper 对临时目录大小写变体可找到、backup path 对文件/目录都能生成并移动。

## 任务 2：前端 API 与类型

**文件：**
- 修改：`src/types/syncSelection.ts`
- 修改：`src/api/domains/sync.ts`

- [ ] **步骤 1：添加 TypeScript 类型**

新增：

```ts
export type SyncAssetKind = 'directory' | 'file'
export type SyncAssetOperation = 'push' | 'pull' | 'sync'
export interface SyncAssetInfo { ... }
export interface SyncAssetGroup { key: string; title: string; description: string; assets: SyncAssetInfo[] }
```

字段使用 camelCase，同时兼容后端 snake_case normalization。

- [ ] **步骤 2：添加 API wrappers**

新增 `listSyncAssets`、`pushSyncAsset`、`pullSyncAsset`、`syncAsset`、`syncAllAssets`，命令名对应后端。

## 任务 3：重构 SyncView 为 Editorial Control Desk

**文件：**
- 重写：`src/views/SyncView.vue`

- [ ] **步骤 1：移除旧 folder selection/custom folder 页面依赖**

删除旧 imports 与状态：`SyncSelectionPanel`、`SyncEnabledFoldersPanel`、`SyncBatchOperationsPanel`、folder CRUD、optionalItems、customFolder 等。

- [ ] **步骤 2：实现资产加载与分组**

用 `listSyncAssets` 加载资产，按 `group` 归类为 CCR/Claude/Codex；保留 `getSyncStatus` 给右侧账号卡。

- [ ] **步骤 3：实现单项和全量操作**

对每个 asset 提供 Push/Pull/Sync。操作时仅禁用对应 asset；全量操作禁用全部。失败时显示 `formatOperationResult`，遇到 already exists/force 提示时显示二次按钮或在输出中提示可用 force retry。

- [ ] **步骤 4：实现页面样式**

使用 scoped CSS 构造暖中性色 editorial surfaces：header summary、asset groups、asset rows、right sidebar/status rail、operation output。避免旧 glass/purple 视觉。

## 任务 4：i18n 与说明卡更新

**文件：**
- 修改：`src/components/sync/SyncInfoSidebar.vue`
- 修改：`src/i18n/locales/zh-CN.ts`
- 修改：`src/i18n/locales/en-US.ts`

- [ ] **步骤 1：更新 features 文案**

把“预设平台选择/独立文件夹/Gemini”改成“固定配置资产/单项同步/敏感字段遮罩/只同步 allowlist”。

- [ ] **步骤 2：新增 SyncView 所需文案**

添加 `sync.assets.*`、`sync.assetGroups.*`、`sync.assetActions.*`、`sync.assetStatus.*` 等 key，中英文都完整。

- [ ] **步骤 3：确认 sync 页面不出现 Gemini/Antigravity 文案**

运行 `rg -n "Gemini|Antigravity|gemini" src/views/SyncView.vue src/components/sync src/i18n/locales/*`，允许其他非 sync 页面保留，但 sync 页面相关文案不能出现。

## 任务 5：验证与修复

**文件：**
- 可能修改：前面任务中失败的文件。

- [ ] **步骤 1：运行前端类型检查**

运行：`bun run type-check`
预期：通过。

- [ ] **步骤 2：运行前端测试**

运行：`bun run test`
预期：通过。

- [ ] **步骤 3：运行 Tauri Rust 检查/测试**

运行：`bun run tauri:check`；若时间允许运行 `bun run tauri:test`。
预期：通过或给出明确非本次变更阻塞。

- [ ] **步骤 4：检查 git diff 与未跟踪文件**

运行：`git status --short` 和 `git diff --check`。确认没有把 `.superpowers/` 加入提交；保留用户既有未跟踪文件。
