# Design — Profile 管理:profiles.toml 直接编辑

> 对应 prd.md;依赖 platform-settings-enhancement 交付的 versioned API(D1)与共享编辑器(D5)。协议与该任务 design.md D2 保持同构,本文只写差异。

## D1 后端命令

共享 helper 下沉 `commands/profile_lifecycle.rs`,平台包装放 `claude_profiles.rs` / `codex_profiles.rs`:

| 命令 | 说明 |
| --- | --- |
| `claude_get_profiles_raw` / `codex_get_profiles_raw` | 读 `PlatformPaths::new(Platform::X)?.profiles_file` 原文;返回 `{ status:"ok", content, token, path, exists }`;文件不存在 → `exists:false, content:"", token:""` |
| `claude_save_profiles_raw` / `codex_save_profiles_raw` | 入参 `content, token, force: bool`;校验链见 D2;返回 status 判别 |

save 返回(在 settings_raw 协议基础上追加一种):

```jsonc
{ "status": "saved", "token": "<新令牌>", "profiles_count": 21 }
{ "status": "conflict" }
{ "status": "invalid", "kind": "syntax" | "semantic", "message": "...", "line": n, "column": n }
{ "status": "activation_conflict", "current": "anyrouter2" }   // force=false 且激活 profile 被删/改名
{ "status": "unsupported_environment", "envType": "wsl" }
```

## D2 校验链(save,顺序执行)

1. **语法**:`toml::from_str::<toml::Value>` — 失败 → invalid/syntax(span → 行列)。
2. **语义**:在 ccr-config `platforms/base.rs` 抽 `pub fn parse_profiles_from_str(content: &str) -> Result<IndexMap<String, ProfileConfig>>`(把 `load_profiles_from_toml` 的 CcsConfig → 简化格式双轨解析逻辑提出来,原函数改为读文件后调用它)— 失败 → invalid/semantic。空 profiles 集合视为 semantic 错误(防误清空;CLI 至少需要一个 profile)。
3. **激活保护**:`platform.get_current_profile()` 为 `Some(name)` 且解析结果不含 `name` → force=false 时返回 activation_conflict;force=true 放行(用户已二次确认;后续激活状态由 CLI 侧自身兜底)。
4. **落盘**:`write_guarded_versioned`,原文 verbatim;`WriteOptions { backup: Dir { dir: paths.backups_dir, prefix: "profiles" }, secret: true, ..Default::default() }`。同时把 `save_profiles_to_toml` 与 `update_current_config` 的既有结构化写路径改为 `secret:true`,确保两条写路径权限语义一致,满足父任务 C3(Unix 目标权限 0o600)。
5. 环境门禁:同 settings_raw 的 `ensure_local_env`(C2;profiles 虽本就只读本机,统一语义防误解)。

注:raw 保存**不持有** `profile_lock_resource` RMW 命名锁——CAS 令牌已保证"结构化写与 raw 写并发"时后到者收到 conflict,无需嵌套两把锁(且嵌套会引入锁序问题)。此点写入代码注释。

## D3 前端

- 入口:两个 Profiles 视图工具栏(Back/Command/Reload/Export/Add Profile 一排)加 **Edit TOML** 按钮;环境非 Local 禁用 + tooltip。
- 编辑面:全屏覆盖面板(路由不变,组件内状态,与既有页面风格一致):
  - 打开前 requestConfirm 明文密钥警示(C3,与 export include_secrets 同语气)。
  - `CodeSourceEditor` TOML 模式;顶部固定条:文件路径 + 明文警示徽标。
  - 保存流:syntax/semantic invalid → errorMarker 定位;activation_conflict → requestConfirm(danger 语义,文案含被删的激活名)确认后带 `force:true` 重发;conflict → [重新加载]/[取消]。
  - 成功 → 关闭面板 + 全量刷新(profiles 列表、Quick Switch、Distribution Insights、当前 profile 卡)。
  - 未保存关闭 → requestConfirm;卸载即弃内容,无任何持久化;不提供复制全文按钮。
- API 包装:`src/api/domains/claude.ts` / `codex.ts` 追加(与 settings raw 类型共用判别联合基型,放 `domains` 内共享 types 或各自文件再导出)。
- i18n:`profilesRaw.*` 命名空间双语。

## D4 测试设计(C6)

Rust(profile_lifecycle 单测,tempdir + 假 profiles.toml):

- 非法 TOML 拒写且返回行号;不符合 profiles 结构拒写;空 profiles 拒写。
- force 协议:删除激活 profile → force=false 得 activation_conflict / force=true 落盘。
- 令牌冲突:get 后改文件再 save → conflict,磁盘未动。
- 备份生成于 backups_dir,前缀 `profiles`;错误 message 不含 fixture 中的探针 token 字符串。

前端:api-facade-boundary smoke 通过;`test:i18n`。

回归:`just test` 全量(重点 `crates/ccr/tests/commands/claude_profile.rs`、`codex_profile.rs`;`parse_profiles_from_str` 重构不得改变 `load_profiles_from_toml` 行为,由既有测试守护)。

## 决策记录

| # | 决策 | 理由 |
| --- | --- | --- |
| 1 | 语义校验复用核心库解析(抽 `parse_profiles_from_str`) | 保证"raw 保存后 CLI 一定能读",单一事实源 |
| 2 | 空 profiles 集合拒写 | 防手滑清空;删除全部 profiles 属结构化流程的显式操作 |
| 3 | activation_conflict 独立 status + force 二段式 | 破坏性动作显式确认(父任务 C5) |
| 4 | raw 保存不持 RMW 命名锁,仅 CAS | 两把锁嵌套引入锁序风险;CAS 已覆盖并发正确性 |
| 5 | raw 与结构化 profiles 写入统一 `secret=true` | profiles 含 credential;满足父任务 C3,并避免两路写出权限翻转 |
| 6 | 全屏面板而非新路由 | 编辑是列表页的模态延伸,保留返回语境 |
