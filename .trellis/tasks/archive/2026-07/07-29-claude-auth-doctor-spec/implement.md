# 实施计划

## 代码步骤

1. 在 ccr-types 定义 auth source/diagnosis 共享类型与 serde tests。
2. 在 ClaudeAuthService 实现只读来源检测、优先级排序、置信度与 capability boundary;复用共享 paths/managed keys。
3. 将 diagnosis 加入 runtime summary,修正 effective auth_mode 与显式 managed override 判定。
4. 在 doctor 增加独立 auth source check,拆开 profile settings 一致性与用户自有凭据来源语义。
5. 扩展 auth switch/profile off action outcome,在 CLI/TUI/Tauri/UI 透出 remaining suppressors;不自动清理用户源。
6. 更新 Tauri DTO 与 ts-rs 生成 TypeScript 类型,补前端显示/烟测;避免触碰 Profiles 页面重构文件。
7. 新增 `claude-auth-runtime.md`,更新 ccr-cli/ccr-types 对应 index/guidelines 链接。
8. 用假 token 运行泄露搜索与 JSON snapshot 测试。

## 验证顺序

```powershell
cargo test -p ccr-types claude_auth -- --test-threads=1
cargo test -p ccr-cli claude_auth -- --test-threads=1
cargo test -p ccr-cli doctor -- --test-threads=1
cargo test -p ccr-tui claude_auth -- --test-threads=1
cargo test -p ccr-ui --manifest-path ccr-ui/src-tauri/Cargo.toml claude_auth -- --test-threads=1
just tauri-bindings-check
just fmt-check
just lint-strict
just test
just frontend-check-quick
git diff --check
```

若 `tauri-bindings-check` 生成预期类型,仅纳入与新增 DTO 对应的生成差异;恢复无关生成副作用。

## 风险与停止点

- 不执行 apiKeyHelper、不读取 macOS Keychain、不探测其他 shell/project cwd。
- 文案必须保留 confidence/evidence 等级,禁止把 issue 报告升级成官方事实。
- UI 只触碰 Claude Auth/运行时概览及独立生成类型;当前 Profiles 编辑器/CSS/utils 脏文件排除。
