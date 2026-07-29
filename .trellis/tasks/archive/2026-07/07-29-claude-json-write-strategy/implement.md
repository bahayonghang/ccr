# 实施计划

## 代码步骤

1. 删除 `ensure_onboarding_completed` 及 apply 调用,把现有 tests 改为 state_file 不变/不创建断言。
2. 删除 doctor onboarding warning 与对应测试/文案,保留其他 API-key profile runtime 检查。
3. 让 `ClaudeMcpContext` 使用 `ClaudeRuntimePaths::state_file`。
4. 提取可重放的 MCP root mutation,用 `content_version_token` + `write_guarded_versioned` 实现最多 3 次 CAS。
5. user/local state_file 使用 secret/no-backup;project `.mcp.json` 使用非 secret/no-backup CAS;移除 `NamedTempFile` writer。
6. 补未知字段保留、单次/连续冲突、并发结果与路径覆盖测试。
7. 更新 ccr-cli `.claude.json` 边界与 ccr-core CAS 规范说明。

## 验证顺序

```powershell
cargo test -p ccr-cli platforms::claude -- --test-threads=1
cargo test -p ccr-cli doctor -- --test-threads=1
cargo test -p ccr-ui --manifest-path ccr-ui/src-tauri/Cargo.toml claude_mcp -- --test-threads=1
just fmt-check
just lint-strict
just test
just frontend-check-quick
git diff --check
```

## 风险与停止点

- MCP mutation closure 必须可重复执行;名称冲突/不存在的结果在每次重读后重新判断。
- 不运行带真实 token 的 Claude 请求;规划探针已提供停止 onboarding 写入所需证据。
- 本任务不修改 Vue/CSS Profiles 脏文件,也不回写用户真实 `.claude.json`。
