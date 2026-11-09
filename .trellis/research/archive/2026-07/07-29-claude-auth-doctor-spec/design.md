# 设计:统一认证来源诊断

## 单一诊断模型

在 `ccr-types` 增加可序列化、无 secret 的共享类型:

- `ClaudeAuthSourceKind`:cloud provider、auth token、API key、apiKeyHelper、OAuth token、subscription OAuth、primaryApiKey issue。
- `ClaudeAuthSourceLocation`:process env、settings env、settings root、state file、credentials file。
- `ClaudeAuthConfidence`:confirmed / potential / unobservable。
- `ClaudeAuthEvidence`:official_contract / issue_report。
- `ClaudeAuthOwnership`:ccr_managed / user_owned / external_runtime。
- `ClaudeAuthSourceObservation`:kind、location、confidence、evidence、ownership、suppresses_subscription;不含值。
- `ClaudeAuthDiagnosis`:按优先级排序的 observations、`presumed_effective_source`、`custom_api_key_responses_present`、固定 unobservable 列表。

字段通过 serde default/skip 保持已有 runtime summary JSON 向后兼容。Tauri DTO 与 ts-rs 生成类型追加同构字段;CLI/TUI/UI 不再自行推断优先级。

## 检测服务

`ClaudeAuthService::diagnose_auth_sources()` 是唯一检测入口,依赖前置子任务提供的:

- `ClaudeRuntimePaths` 读取 settings/credentials/state。
- `CCR_MANAGED_KEYS` 区分 ccr_managed 与 user_owned。
- 有效 auth_mode 与账号快照匹配。

检测值只用于非空/布尔存在性判断。不得执行 `apiKeyHelper`,不得序列化 env/token 原文,不得把凭据哈希放入结果。

排序按 research 矩阵。`ANTHROPIC_API_KEY` 在无法证明批准时为 potential;`apiKeyHelper` 因未执行为 potential;`primaryApiKey` 永远为 issue_report/potential。`customApiKeyResponses` 仅设置解释 flag,不创建 observation、不增加压制源计数。

`presumed_effective_source` 取最高优先级 observation,并保留其 confidence;若同一级存在多个互斥 cloud provider 或位置冲突,返回该级 observations 并把结论降为 potential,不武断挑一个。

## runtime summary 与 doctor

`ClaudeRuntimeSummary` 追加 diagnosis。现有 `mode/login_state` 继续服务兼容 UI,但 API-key/profile override 判定使用有效 auth_mode + 显式托管键;新的 diagnosis 解释更高优先级用户来源。

doctor 增加独立 `platform.claude.auth_sources` check:

- 没有高于 subscription 的来源且 subscription usable -> ok。
- confirmed/potential 压制源 -> warn,detail 只列 kind/location/confidence/evidence。
- 读取目标文件失败/JSON 损坏 -> fail,不降级成“未发现来源”。
- detail 固定列出 unobservable 能力边界。

现有 settings validation 不再用“任意 ANTHROPIC_* 必须形成 ccr API-key pair”解释用户自有 `ANTHROPIC_API_KEY`;doctor 来源检查与 profile settings 一致性检查分开。

## 切换动作反馈

`switch_account`/profile off 成功修改后重新运行 diagnosis,返回结构化 action outcome:

- `cleared_managed_sources`:实际清理的 ccr 托管键名/数量(不含值)。
- `remaining_suppressors`:仍存在的 user_owned/external observations。
- `warnings`:由 presentation 层按 observation 生成,CLI/TUI/Tauri/UI 使用同一含义。

CLI 输出明细,TUI/UI 显示紧凑警告与详情;不得把“potential”翻译为“确定正在使用”。Tauri action response 从单一 message 扩展为追加 warnings 字段,保持 success/message 兼容。

## 规范

新增 `.trellis/spec/ccr-cli/backend/claude-auth-runtime.md`,记录:

- 六级官方优先级与 gateway/云厂商例外。
- primaryApiKey issue 附注及核实日期。
- credentials/state/settings 的读写所有权链接。
- confirmed/potential/unobservable 契约。
- doctor 与 action 输出不得泄露 secret。

在 ccr-cli backend index 挂载;ccr-types guidelines 只记录共享 DTO 与托管键边界,atomic writer 只链接持久化规则,避免重复权威文本。

## 测试

- 每种官方可观测来源的单项与组合优先级表驱动测试。
- API_KEY/apiKeyHelper potential、custom responses 非来源、primary issue evidence。
- 清单外 `ANTHROPIC_*` user_owned 告警且不删除。
- unobservable 固定列表与 JSON/TS wire 兼容。
- doctor ok/warn/fail;CLI/TUI/UI 文案不泄露注入假 token。
- auth switch 三端反馈包含剩余压制源且区分 confidence。

## 回滚

共享诊断字段为追加型。若某来源规则不确定,降低 confidence 或移除 presumed 结论,不得删掉原始 observation 或改成自动清理。
