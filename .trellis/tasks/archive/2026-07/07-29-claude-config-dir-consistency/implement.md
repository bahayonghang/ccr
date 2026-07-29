# 实施计划

## 代码步骤

1. 在 `ccr-config` 新增 `ClaudeRuntimePaths` 与注入式解析入口,导出到稳定模块;补完整优先级和 Windows 路径测试。
2. 迁移 `SettingsManager::with_default`/Claude `get_platform_paths`,删除重复的 home/覆盖变量分支。
3. 迁移 `ClaudeAuthService`,让 settings、credentials、state、backup 均来自共享结构;保留 `from_parts` 测试构造或改为 `from_runtime_paths`。
4. 迁移 `ClaudePlatform` 和 doctor 对 state/settings 的路径读取。
5. 迁移 Tauri `LocalEnvironment`、`settings_raw.rs` 与 `ClaudeMcpContext` 的用户级 Claude 路径;远程 SSH/WSL 路径语义保持不变。
6. 搜索 `rg '\.claude' crates/ccr-cli/src ccr-ui/src-tauri/src`,逐项标注本任务已迁移或明确 out-of-scope 的生产调用点。
7. 更新 ccr-config 路径所有权规范和 ccr-cli 消费契约。

## 验证顺序

```powershell
cargo test -p ccr-config claude_runtime_paths -- --test-threads=1
cargo test -p ccr-cli claude_config_dir -- --test-threads=1
cargo test -p ccr-ui --manifest-path ccr-ui/src-tauri/Cargo.toml platform::local -- --test-threads=1
cargo test -p ccr-ui --manifest-path ccr-ui/src-tauri/Cargo.toml claude_mcp -- --test-threads=1
just fmt-check
just lint-strict
just test
just frontend-check-quick
git diff --check
```

以实际 Tauri package 名称为准调整 `-p`;不得跳过对应测试。

## 风险与停止点

- 不读写用户真实 home;所有 env 测试通过 `TestHome`/进程 env 锁串行执行。
- 不搬迁文件、不创建兼容副本;发现现有数据迁移需求时退回 planning,不得隐式扩展。
- 本任务不触碰 Profiles Vue/CSS 脏文件。
