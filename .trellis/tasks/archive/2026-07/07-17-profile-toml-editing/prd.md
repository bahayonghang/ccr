# Profile 管理:profiles.toml 直接编辑

> 父任务:`.trellis/tasks/07-17-ccr-ui-config-mgmt-enhancement`。跨子任务契约(C1 锁内 CAS、C2 raw 仅 Local、C3 明文信任边界、C4 共享编辑器、C5 工程规范、C6 测试底线)以父任务 prd.md 为准。
>
> **显式后继**:本任务的前端编辑面依赖 platform-settings-enhancement 交付的共享编辑器组件;raw 保存路径依赖其交付的 ccr-core versioned 写入 API。后端命令可并行开发,但两个前置物就绪前本任务不满足 `task.py start` 条件(implement.md 首项 checklist 体现)。

## Goal

为 Claude / Codex Profiles 管理页面增加 raw TOML 编辑模式:高级用户可绕过表单,直接编辑 `~/.ccr/platforms/<platform>/profiles.toml` 全文,带语法/语义校验、锁内 CAS 冲突保护与备份。

## 现状(2026-07-17 摸底)

- Profiles 存储:`~/.ccr/platforms/claude/profiles.toml`(Codex 同构),由 ccr 核心库 `ClaudePlatform` 的 `load_profiles` / `save_profile` / `delete_profile` / `apply_profile` 结构化读写(`crates/ccr-config/src/platforms/base.rs` 一带)。
- Tauri 层:`commands/claude_profiles.rs`、`codex_profiles.rs` 提供 CRUD;`profile_lifecycle.rs` 的 `profiles_export_payload_from_path` 已能读原始 TOML 文本(export),无 raw 写回。
- 前端:`ClaudeCodeProfilesView.vue`、`CodexProfilesView.vue` 卡片 + 表单式,带 Export,无 raw 编辑入口。
- profiles.toml 含 auth_token 等明文敏感字段;raw 编辑让全文经过 IPC 与前端,与 export(include_secrets)同级信任边界 —— 按父任务 C3 统一处理。
- Profiles 读写不经 `ExecutionEnvironment`,直接走本机 `PlatformPaths`;但为与其他 raw 编辑一致,遵循父任务 C2:active env 非 Local 时禁用 raw 入口(避免用户在远程语境下误解编辑目标)。

## Requirements

### R1 后端:raw TOML 读写命令

- 每平台两条命令(建议 `claude_get_profiles_raw` / `claude_save_profiles_raw`,Codex 同构;共享 helper 下沉 `profile_lifecycle.rs`):
  - get:返回 profiles.toml 原始文本 + **内容哈希令牌**(父任务 C1;禁止用 mtime 作令牌)+ 绝对路径;路径复用 `PlatformPaths::new(Platform::X).profiles_file`。
  - save:入参原始文本 + 令牌 + `force: bool`。校验顺序:
    1. TOML 语法(toml crate,错误带行列号);
    2. 语义:反序列化为与 `load_profiles` 相同的 profiles 结构,保证保存后 CRUD/apply 不坏;
    3. 激活保护:当前激活 profile(`get_current_profile`)在新内容中被删除或改名时,`force == false` 返回专用警告错误,前端二次确认后携带 `force: true` 重试;
    4. 落盘:经 ccr-core versioned 写入 API 锁内 CAS(令牌不匹配返回冲突错误,与校验错误可区分);`WriteOptions.secret = true` 保持敏感文件权限(C3)。
- 备份策略沿用 profiles 既有 Dir 备份约定(`{prefix}.{timestamp}.{ext}.bak`),不改变备份目录与命名。

### R2 前端:Profiles 页面 raw 编辑模式

- `ClaudeCodeProfilesView` / `CodexProfilesView` 工具栏新增 "Edit TOML" 入口;active env 非 Local 时禁用并展示原因(C2)。
- 编辑界面(全屏面板或独立视图):
  - 共享编辑器组件 TOML 模式(C4,不自造);顶部显示文件绝对路径 + "包含明文密钥"警示条。
  - 打开前 requestConfirm 明文警示(C3);前端不持久化内容,离开即释放;不提供一键复制全文捷径。
  - 保存:后端校验错误按行列号内联展示;激活 profile 被删/改名警告走二次确认 + force 重发;令牌冲突提示"文件已被外部修改"并提供重新加载,不提供静默覆盖。
  - 保存成功后返回列表并强制刷新 profiles(卡片、Quick Switch、Distribution Insights 全部重算)。
- 未保存修改离开需确认。

### R3 规范

- API 包装放 `src/api/domains/` 对应 profiles domain 文件;i18n 双语;requestConfirm 契约;错误消息不回显敏感内容(C3/C6)。

## Acceptance Criteria

- [ ] Local 环境下,Claude 与 Codex Profiles 页均可进入 raw TOML 编辑,内容与磁盘一致(直读原文,非结构化 DTO 回填 —— 代码审查确认)。
- [ ] 保存合法修改后:磁盘更新、生成备份、列表页刷新可见;`ccr` CLI 侧可正常读取(手工 `ccr current` 验证作为补充)。
- [x] 自动化测试(Rust 单测,C6):非法 TOML 拒写(错误含行号)、不符合 profiles 结构拒写、删除/改名激活 profile 时 force 协议(false 拒 / true 过)、令牌冲突拒写、备份生成、secret 权限保持、错误消息不含文件内容片段。
- [x] get 后外部修改文件再 save,收到冲突错误(自动化覆盖)。
- [x] 打开编辑器前有明文警示确认;非 Local 环境入口禁用。
- [x] `bun run type-check`、`bun run lint`、`bun run test:i18n`、`bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts` 通过;profiles 既有测试(`crates/ccr/tests/commands/claude_profile.rs` 等)经 `just test` 无回归。

## Out of Scope

- 表单模式与 TOML 模式的双向实时同步(保存后整页刷新即可)。
- profiles.toml 的 diff/merge UI、历史版本浏览。
- 其它平台(OpenCode providers 等)的 raw 编辑。
- WSL/SSH 环境语境下的 raw 编辑。

## Notes

- 复杂任务:`task.py start` 前必须补 `design.md`(命令签名、force 协议与错误 DTO 枚举、与 versioned API 的对接点)与 `implement.md`(首项 checklist:确认共享编辑器组件与 versioned API 已可消费)。
- 完成后触发 rust-security-reviewer 复核(命中:credential 字段 + 配置写路径)。
