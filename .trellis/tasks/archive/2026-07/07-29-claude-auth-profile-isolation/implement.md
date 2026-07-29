# 父任务执行计划

## 启动规则

- 父任务没有直接代码交付,不作为 Phase 2 实现目标。
- 获得最新规划批准后,一次只 start 一个子任务;前置子任务完成检查、规范更新、提交与归档后再 start 下一个。
- 全程本地执行,不 push、不建 PR。

## 子任务序列

1. `07-29-claude-authmode-consistency`:显式托管键、effective auth_mode 与 profiles.toml 自愈。
2. `07-29-claude-config-dir-consistency`:共享 `ClaudeRuntimePaths` 与三端路径统一。
3. `07-29-claude-credentials-hardening`:未保存登录守卫、快照身份匹配、secret guarded write、settings CAS-RMW、macOS 明确边界。
4. `07-29-claude-json-write-strategy`:删除 `.claude.json` 写入及过时 onboarding 诊断,保留只读边界。
5. `07-29-claude-auth-doctor-spec`:统一认证来源诊断模型、三端警告与速查表规范。

## 每个子任务的完成门槛

- 先运行子任务 implement.md 中的窄测试,再运行其完整检查清单。
- 调用 `trellis-check`,修复所有阻断项;更新子任务触碰的 spec。
- 检查 staged/unstaged/untracked 路径,排除 Profiles 重构与其他用户改动。
- 按仓库中文 Conventional Commit + emoji 形成一个或多个真实语义提交。
- 仅归档已完成的当前子任务,并把工作提交记录到 journal;不把 archive 提交写入 journal 的 work commit 列表。

## 父任务最终验收

1. 运行各子任务回归与跨任务场景:第三方->官方、官方->第三方、账号 A->B->A、用户自有 env 保留、`CLAUDE_CONFIG_DIR` 自定义目录。
2. 运行 `git diff --check`、`just lint-strict`、`just test`、`just frontend-check-quick`。
3. 运行 `ccr doctor --json` 的隔离 fixture/命令测试,确认 confirmed/potential/unobservable 输出不泄露 secret。
4. 核对 `.trellis/spec` 索引和契约与最终代码一致。
5. 完成父任务集成记录与归档;不 push。
