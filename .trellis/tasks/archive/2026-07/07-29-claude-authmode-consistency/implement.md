# 实施计划

## 代码步骤

1. 在 `crates/ccr-types/src/claude_settings.rs` 建立 `CCR_MANAGED_KEYS`,实现显式清理、检测、枚举,并让 `clear_managed_vars`/`apply_managed_env` 使用它。
2. 扩展 ccr-types 测试,覆盖注册表不变量、`ANTHROPIC_API_KEY`/`ANTHROPIC_CUSTOM_HEADERS` 保留及非 Anthropic 托管键清理。
3. 在 `ClaudeAuthService` 的清理与 runtime summary 调用 `effective_auth_mode` + `has_managed_overrides` + `clear_ccr_managed_vars`。
4. 在 `ClaudePlatform::apply_profile` 先验证并持久化纠正后的 profile,成功后才进入 settings RMW;补失败不改 runtime 的回归。
5. 把 `profile_off` 与 lifecycle `clear` 迁移到显式集合;clear 预览、文案、计数和执行共用 `managed_env_entries`。
6. 审计 `rg 'clear_anthropic_vars|clear_managed_vars|has_anthropic_overrides'` 的每个生产调用点,记录保留 legacy 语义的理由。
7. 同步更新 ccr-types env 所有权规范和 ccr-cli auth_mode/Claude settings 契约。

## 验证顺序

```powershell
cargo test -p ccr-types claude_settings -- --test-threads=1
cargo test -p ccr-cli claude_auth -- --test-threads=1
cargo test -p ccr-cli platforms::claude -- --test-threads=1
cargo test -p ccr-cli lifecycle::clear -- --test-threads=1
just fmt-check
just lint-strict
just test
git diff --check
```

若测试过滤名与实际模块不匹配,改用对应 package 全量测试并在提交记录中写明实际命令。

## 风险与停止点

- profiles.toml 自愈写入失败用注入/隔离文件系统 fixture 验证;不得依赖用户真实配置。
- 若新增注册表导致 `ConfigSection::to_managed_env_pairs` 漏映射,先补不变量测试再继续调用点迁移。
- 本子任务不改 doctor 输出模型、不改凭据文件、不触碰当前 Profiles 前端脏文件。
