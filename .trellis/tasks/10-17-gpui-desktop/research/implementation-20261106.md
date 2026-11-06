# 2026-11-06 后续连续实施

## 起点与执行边界

本轮读取并执行 `next-session-implementation-prompt-20261106.md`。实际起点为 `dev@90131cd6ae5952d6f461d77e0c33105bbb9e8fc9`，`git status --short` 为空。复用父任务；01、09及父任务继续 `in_progress`。旧 CI/原生收据不验证本轮改动。

已读取三张 handoff 截图、指定版本 design-atlas、当前 DESIGN 与任务文件。既有 airy-glass 合同保留；优先信息架构、真实数据与业务入口。没有新建 task/worktree。WSL/doctor、真实 SSH/认证/配额/WebDAV、真实用户数据与尺寸/DPI矩阵不在本轮运行范围。

## 实施与验证顺序

1. 原生侧栏、字体与 Claude 工作台 → 检查13入口可达、真实读来源、Profiles保留及1 MiB渲染；当前二进制隔离fixture少量原生截图/焦点。
2. 应用级 Recent终态记录、分来源用量导入 → 窄生命周期/脱敏/来源拒绝回归。
3. MCP批量、配置来源层、监控、工具桥接/手动更新 → 逐域对照实际Tauri owner，完成原生消费者及必要写入/冲突测试。
4. 平台能力矩阵及其余可实施差异 → 保持逐项状态；外部阻塞仅阻止依赖它的项。
5. 独立check、集中GPUI三门禁、共享层/最终完整CI、便携包 → 中文本地原子提交；不push、不amend、不归档未验收任务。

## 初始源码差距

| 能力 | 当前 owning source | 本轮目标 | 当前状态 |
| --- | --- | --- | --- |
| 侧栏/平台默认页 | `shell/mod.rs`直接进入配置列表；按钮默认居中 | 图标左对齐、分组和设置区；默认工作台 | implementing |
| 实际字体 | `theme.rs`未赋值字体family | 显式原生sans及实际解析检查 | implementing |
| Claude上下文 | Profiles/Settings/Files工具栏 | 工作台内Profiles/auth/MCP/resources/usage等可发现入口及真实摘要 | implementing |
| Recent | `runtime/jobs.rs`持有终态，未写GUI Recent | retained终态写入，离页、失败/取消、幂等和脱敏 | implementing |
| 分来源导入 | `runtime/insights/import.rs`固定默认CLI选项 | 显式来源与窗口、保留任务、非支持home拒绝 | implementing |
| MCP批量 | `runtime/mcp/transfer.rs`仅单项copy | 预设多目标及整批同步、源/目标版本、逐项结果 | pending |
| 配置来源 | Tauri `settings_raw.rs`/`grok.rs`有layers | 有效来源/只读层/项目scope清单 | pending |
| 全局监控 | Tauri `system.rs`消费持久日志及event log/runtime metrics | 正确捕获main DB、脱敏/上限、真实native指标 | pending |
| 工具/更新 | `runtime/install.rs`仅llmusage；Tauri shell/system有桥接/update | 封闭进程边界、确认、真实作用域及状态 | pending |

## 证据约定

本文件的 `implementing` / `pending` 不是验收通过。每批完成后补充实际命令、退出码、源码/fixture/headless/native覆盖；完整平台矩阵不由旧planned计数推断。

## 继续实施检查点

- 本地提交 `96376961cd49b7914e3457efba2fc92e9d8ae076`：Recent retained终态持久化，独立check已闭合；运行者终态快照不受registry eviction影响，参数值不落盘。
- 本地提交 `20d6f23a0a4892385219ef1d4f1327500d137a99`：共享分层配置元数据与Tauri委托，详见 `source-layers-20261106.md`。
- 本地提交 `a25df5f495d9610789e7fc76d96593fb532e1edb`：版本化恢复Candidate/Plan及归档恢复可见，domain11和最新shared refresh3通过，保留legacy恢复兼容。
- 侧栏、真实平台工作台、Claude常用资源表单、Observer可读表格、Codex滚动用量、用量导入、MCP批量、Tools/手动更新、全局监控均有对应源码与窄fixture/headless收据；详见 `platform-parity-20261106.md` 和各独立check报告。它们不共享一条“全通过”结论。
- Grok activation/drift/auth-mode概览与完整用量能力页已接入真实owner：Grok领域2/控件1、能力页控件1通过；共享用量能力2测通过。当前 `cargo clippy --locked --offline -p ccr-app --all-targets --all-features -- -D warnings` 退出0（包括新能力与直接共享依赖生产检查）。
- 普通Settings/供应商/MCP表单独立check完成，修复空header/env替换覆盖秘密、HTTP→stdio残留type和确认焦点；两个新MCP控件/旧MCP4通过。MCP默认窗口裁切已修动作顺序；OpenCode fixture原MAX_PATH备份失败通过沿既有canonical路径合同修复，生产writer/备份未绕过。
- 会话保存/恢复已完成metadata列表→可取消的单项prepare→确认→retained commit。domain11/session7通过；独立检查发现的legacy相对home、大文件、批量payload I/O问题已闭合。随后完整门禁捕获真实生产trash renderer大frame；自然拆分页/行builder后同路径1MiB红→绿，最终19子守卫通过，未增大栈。
- Tauri显式fixture基础和reviewed coregraph已独立check，补齐锁线程scope和认证summary捕获路径；最新fixture3、锁2、auth26、registry21、runtime policy11与strictclippy通过。未审计业务仍拒绝，实际WebView存储/加载缺原生收据，不称完整可比较应用。
- S0/S1/S2两端数据、显式 native/Tauri Benchmark source、共同校验/UTC默认报表时间已实现且独立check。最终原生全量分页/usage/metadata读取1/1通过234.60s，旧实际source ID和DB hash保持；两端默认7天请求0/65469/65469已核对。在HEAD998dd9f检查点S2流仍not_emitted，随后已实施持续源及消费者，见performance-s2-implementation-20261106.md。真正process-create/OS-present/action-ready协议与样本未完成，性能not_run/not_comparable；锁栈源码边界见performance-present-boundary-20261106.md。
- 用户已明确“稍后再做原生验证”。不再运行原生交互；已有字体解析日志不代表完整视觉通过。当前binary截图受游戏遮挡，不能作为原生证据。
- 扩展批次当前GPUI fmt/lint/test完整通过：lib228/0/4ignored、native_shell9/0/1、native_ui2、shutdown2/1ignored、Python16、AccessKit3及26个实际1MiB子守卫通过。旧OAuth测试的isolated preflight已按新安全边界分离，真实临时端口取消/join覆盖保留。根完整just ci曾在65a5fe1f退出0；此后共享变更仍需新完整CI，候选包未运行。后续S2源码不由这轮覆盖；详见integration-gates-20261106.md。父任务/01/09保持in_progress；不归档，不push，不amend。

## 扩展批次本地提交与消费者

- `92b3e9e`：scope/login/nonce/revision绑定的原生OAuth清理；`69a8875`：缺钥metadata更新保留密文；`1a02641`：凭据纯读不初始化key。
- `1a59445`：已验证SSH会话固定CLI检测；`57abc91`：共同Seed校验与报告clock（包括WAL/journal/闰秒/截止边界）。
- 原生System info、Codex诊断、SSH控件、普通Session表单/高级Cookie编辑、captured crypto锁与缺钥保护均完成独立check及当前整批门禁，实际native/host/SSH行为不由headless收据冒领。
- S2持续runtime源的下一小批次边界已具体审查；只允许显式Benchmark/S2、真实ACK/实际速率、bounded queue和join，不能预填60000行或用requested rate冒充交付。

## S2持续源批次

- 共享fixed S2 producer、native/Tauri retained adapter、真实日志保留/逐条持久化票据与三条typed IPC已接入；两端Monitoring提供显式启动确认、取消、真实分阶段计数与重订阅。普通日志触发mixed flush/eviction时仍归属正确；COMMIT BUSY后遗留事务的既有缺陷已通过真实临时DB红绿修复。
- 原生确认改为独立review、safe Cancel、挂载后armed及generation门；Tauri重连的same-revision错误恢复与历史over-budget终态文案也由独立检查闭合，不削弱revision或来源守卫。
- cfg(test)短源及生产60秒合同的主动取消adapter测试属于source/headless覆盖；没有把它们当成60秒原生吞吐、OS呈现或跨客户端性能样本。原生窗口仍user-deferred，父任务/01/09保持in_progress。
- 最终集中门禁、交付包与本地提交以integration-gates-20261106.md及git log为准，不复用旧HEAD门禁。
