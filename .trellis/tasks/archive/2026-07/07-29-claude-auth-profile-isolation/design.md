# 集成设计:Claude profile 与官方账号隔离

## 边界

父任务不直接拥有产品代码。它负责需求源、子任务依赖、跨子任务契约与最终集成验收;实际代码分别由五个子任务交付。执行期间父任务保持 planning,只激活当前拥有下一项交付的子任务。

当前工作树另有 `07-29-profiles-claude-page` 的前端改动。本任务族默认不修改其已变更文件;只有最后的诊断展示确需触碰 `ClaudeAuthView.vue` 等独立认证页面时才纳入,并在提交前按路径证明没有夹带 Profiles 重构改动。

## 目标架构

1. `ccr-types` 维护显式 CCR 托管 Claude env 键注册表。Profile apply/off、auth switch 与 clear 都只变更该集合,用户自有凭据源只诊断不删除。
2. `ccr-config::ClaudeRuntimePaths` 成为 Claude 运行时路径的唯一解析器,供 CLI、AuthService 与本地 Tauri UI 复用。
3. `ccr-core::guarded_write` 继续作为物理持久化策略层;`ccr-cli::SettingsManager` 在其上提供 Claude settings 的 CAS-RMW。OAuth 凭据、快照和 registry 全部走 secret guarded write。
4. profile/auth 路径停止为 onboarding 或身份同步写 `.claude.json`;Tauri MCP user/local 的既有合法写入保留并改为 CAS。身份展示优先采用与当前凭据精确匹配的 ccr 快照元数据。
5. `ccr-types` 定义可序列化的认证来源观察模型;`ClaudeAuthService` 单点检测,doctor、CLI、TUI、Tauri/UI 只消费同一诊断结果。

## 核心数据流

### 第三方 profile -> 官方账号

`auth switch` 读取当前 profile并使用 `effective_auth_mode` -> 通过 `SettingsManager` CAS-RMW 删除显式托管键 -> guarded write 替换已保存目标账号凭据 -> 重新读取诊断 -> 返回仍存在的用户自有压制源警告。任何 CAS 冲突或凭据写失败都不得报告成功。

### 官方账号 -> 第三方 profile

`profile use` 先持久化纠正错误 auth_mode -> 通过 CAS-RMW 只替换显式托管 env 区域 -> 保留 OAuth 凭据与 `.claude.json` -> 更新 profile current 指针。profiles.toml 自愈失败时 settings.json 保持原样。

### 官方账号 A -> B

切换前校验当前 `.credentials.json` 是否精确匹配任一已保存快照;不匹配则拒绝并要求先 save。切换后当前凭据与目标快照匹配,身份展示从该快照的 `oauth_account` 读取,不回写 `.claude.json`,避免 B 凭据配 A 元数据。

## 兼容与安全

- 账号快照保持现有 JSON v1 结构;本任务不引入快照加密或新生产依赖。权限、锁、fsync 和无明文旁路先达到既有持久化契约,加密另立任务评估。
- Windows/Linux 继续使用 `.credentials.json`;macOS Keychain 不在文件搬运能力内,save/switch 明确报不支持。
- 诊断 wire 字段只做追加并提供 serde 默认;不得返回 token 值、helper 输出或凭据指纹。
- `CLAUDE_CONFIG_DIR`、开发覆盖变量及 Windows 路径展开由共享解析器统一,避免组件各自读 env。
- profile/auth 对 `.claude.json` 的竞争通过停止写入消除;MCP 保留写面以 CAS 检测冲突,但仍明确不承诺与 Claude Code 外部进程的跨进程事务。

## 实施顺序

按 `#1 authmode -> #2 config-dir -> #3 credentials -> #4 claude-json -> #5 doctor/spec` 顺序串行交付。#4 的停写结论已在规划阶段确定,因此 #3 的 C10 选择零写面的快照元数据方案,不存在对 #4 代码的硬依赖。

## 回滚

每个子任务独立提交、检查和归档。若某个子任务验收失败,只回滚该子任务尚未提交的局部改动;不得回退已通过的前置契约。最终集成失败时保留前置修复,将失败的下游子任务退回 planning 修订。
