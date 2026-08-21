# 规划审阅处置（2026-08-21）

对照仓库核对 Claude Code 审阅后的结论。未联网复核官方 logout 行为。

| ID | 审阅 | 核对 | 处置 |
| --- | --- | --- | --- |
| TPR-01 | 阻断 | 成立。`needs_auth_off` 在 macOS Claude 与 Codex keyring/auto 恒 true；R4 的不 spawn 只约束 file。 | AC3 限定 file。native：可重复 spawn，成功则 `changed=true`。R4 拆开。 |
| TPR-02 | 应修 | 成立。Keychain / keyring 下 current 不可观察。 | AC1 按 file / native 分前置条件。 |
| TPR-03 | 应修 | 成立。`login_prep_codex_dirs()` 在 `CODEX_HOME` 重定向时可返回两目录。 | `needs_auth_off` 遍历同一列表。 |
| TPR-04 | 应修 | 成立。顺序在 `ccr-config` `TuiTabId` / `DEFAULT_TAB_ORDER`；`Usage` 已有 deprecated 过滤先例。 | 新增 `GrokAuth`；`OpencodeAuth` 按 Usage 保留并在 `load()` 过滤。默认顺序替换该槽，不重排其余五页。 |
| TPR-05 | 应修 | 成立。`check_secret_writes.py` 无条件 `read_text`。 | 从 `SENSITIVE_MODULES` 去掉两条 OpenCode 路径。不把 `auth_off.rs` 加入（与 `profile_off.rs` 一致）。 |
| TPR-06 | 应修 | 成立。侧边栏 `docs/.vitepress/config.mjs:62,238`；`version.md` 中英各一处。 | 清单改为「所有指向 `commands/opencode` 的链接」，含 VitePress 两侧。 |
| TPR-07 | 应修 | 成立。`show_version()` 在 `dispatch.rs:801,806-809`。 | 列入删除清单；help 测试覆盖 `ccr version` 输出。AC8 含 version。 |
| TPR-08 | 应修 | 成立。Grok 无 CCR 快照可恢复。 | D9：Claude/Codex `warning`，Grok `danger`。 |
| TPR-09 | 应修 | 成立。现有 `can_off` / `handleOff` 是 profile off。 | DTO `can_auth_off`；handler `handleAuthOff`。 |
| TPR-10 | 应修 | 成立。spec 禁止 read/write/backup/validate。 | spec/文档修订覆盖存在性读取、备份、删除。 |
| TPR-11 | 应修 | 成立。 | 增 AC12 备份回滚、AC13 Claude/Grok 第三方 profile 路由仍有效。 |
| TPR-12 | 应修 | 成立。profile-off 成功后保留快照。 | D10：file 路径成功 `commit` 后删除 `auth-off` 快照目录。失败走 Drop 回滚。 |
| TPR-13 | 提示 | 成立。符号是 `ensure_local_env`。 | 更正。 |
| TPR-14 | 提示 | 成立。 | implement 加入 `just tauri-command-inventory` 与 `-check`。 |
| TPR-15 | 提示 | 成立。 | 清单加入 `dispatch_routing.rs`。 |
| TPR-16 | 提示 | 成立。脚本认 `- R1：` 与 `AC1（R1, R2）：`。 | 改 PRD 标识格式。 |
| TPR-17 | 提示 | 成立。与 profile off 包装同形态。 | AC7 保持 argv；不新增入口。 |
