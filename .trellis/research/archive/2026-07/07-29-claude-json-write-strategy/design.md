# 设计:.claude.json 写入最小化与 CAS

## 结论

停止 ccr-cli 为 API-key profile 写 `hasCompletedOnboarding`;保留 Tauri Claude MCP 的 user/local 配置写入,因为它是独立的现有产品能力。所有保留写入必须基于最新内容重放 mutation 并用 CAS 提交。

证据见 `research/claude-2.1.220-onboarding-probe.md`:当前 2.1.220 在全新 config dir、无该字段时已进入认证阶段,且 Claude Code 自己创建 state_file。ccr 没有理由维护这一私有状态键。

## onboarding 路线

- 删除 `ClaudePlatform::ensure_onboarding_completed`、apply 调用及对应 imports/tests。
- API-key profile apply 只更新 settings.json 与 profile current 状态,不创建/修改 state_file。
- 删除 doctor 对缺失 `.claude.json` / `hasCompletedOnboarding != true` 的警告;缺少这个私有键不再被视为故障。
- 保留“profile apply 不改 oauthAccount/unknown state”回归,改为断言整个 state_file 字节不变。

交互式首次 onboarding 的 UI/登录流程由 Claude Code 管理。ccr 不通过预置私有布尔值跳过它。

## MCP 合法写入

`ClaudeMcpContext` 从前置子任务的 `ClaudeRuntimePaths` 获取 user state_file。项目级 `.mcp.json` 与 `.claude/settings*.json` 继续由 project root 派生。

把现有 `read_root_for_scope -> mutate -> write_root_for_scope` 重构为纯 mutation CAS:

1. 读取当前 bytes(缺失视为空 object)并计算 content version token。
2. 解析完整 JSON object。
3. 对最新 object 执行 add/update/delete 的确定性 mutation。
4. 序列化并调用 `write_guarded_versioned`。
5. Conflict 时重新读、重新定位 project key、重放 mutation,最多 3 次。
6. 三次均冲突则返回明确并发错误;UI 不得显示成功。

state_file 使用 `WriteOptions { secret:true, backup:None }`,避免复制可能含 `primaryApiKey`/其他敏感状态。项目 `.mcp.json` 同样使用 CAS,但保持非 secret/no-backup,避免改变可共享项目文件权限。删除 `NamedTempFile` 私有 writer。

## 并发承诺

CAS 只保证“从本次读取到提交检查期间若文件变化,本次不会覆盖该变化”。Claude Code 可能在 rename 之后立即写入,ccr 无法让外部进程参与同一事务。因此:

- 不使用“文件锁保证第三方不丢字段”表述。
- 每次 mutation 都保留当次读取的未知字段。
- 冲突显式失败优于静默覆盖。
- 外部进程后续自主覆盖属于残余风险,写入规范需明示。

## 与其他子任务的接口

- config-dir 子任务提供正确 state_file。
- credentials 子任务 C10 不写 state_file,因此不依赖本 CAS 来同步 oauthAccount。
- doctor 子任务只读 `primaryApiKey`/`customApiKeyResponses`,不加入写路径。

## 测试

- profile apply 前后 state_file bytes 完全一致;缺失时不创建。
- doctor 不再因 `hasCompletedOnboarding` 缺失告警。
- MCP user/local add/update/delete 保留 oauthAccount、primaryApiKey、projects 与未知顶层字段。
- 注入一次冲突后重读重放成功;连续冲突 3 次后文件保持外部版本且返回 conflict。
- 两个并发 MCP mutation 要么合并保留,要么一方显式失败,绝不静默丢字段。

## 回滚

若未来 Claude Code 官方文档重新要求外部工具设置 onboarding 状态,先回到 planning 并提供版本化证据;任何恢复写入都必须走本任务 CAS helper,不得恢复整读整写旁路。
