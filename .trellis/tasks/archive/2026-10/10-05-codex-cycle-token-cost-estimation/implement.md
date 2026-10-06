# 实施计划：Codex Auth 周期统计

## 启动条件

- [x] 用户2026-10-06明确批准本版 PRD / design / implement。任务创建授权不替代实施批准。
- [x] 重读当前 git status / task state / nested AGENTS；保留任务外变更与 sub2api 的现有 AGENTS.md 改动。
- [x] 刷新 gpt-6.1-sol 官方价格和协议版本；核实后未发生规划语义变化。
- [x] implement.jsonl / check.jsonl各9项有效spec/research清单，规划预检查无阻塞。
- [x] 运行 `task.py start`，进入Phase 2；native注入与子侧补读已执行，产品改动由trellis-implement负责，随后独立检查。

## 顺序与里程碑

### M1：计量质量、归属和费用（AC1–AC5、AC9、AC11）

- [x] 在 design D7 指定 Codex usage / DTO 模块内保留缓存、推理、身份、time_basis、scan coverage；不改原始JSONL和激活历史。
- [x] 替换全文件 completed 优先为可证的按事件选择；实现单调 epoch、重复副本和fork前缀的确定去重。无法证明的记录标partial，不能猜测修复。
- [x] 归属纯函数复用现有激活区间结果，显示inferred；模型provider可见性修复不作计费正证据。
- [x] 更新可重建 usage cache version；旧格式缺项保持unknown，固定as_of排除未来记录。
- [x] ModelRateCatalog只补新canonical价格及关联long-context规则；不改其他型号或llmusage历史行。
- [x] 逐请求计价，明确Standard假设、model/version、覆盖率、UNPRICED；核对新模型cache-write与reasoning。
- [x] 合成fixtures覆盖D2/D3和estimate-examples；catalog已有消费者回归不变。

回退点：未通过解析/费用合同前不接入容量估算。失败回到M1；不得调整额度或测试阈值消除失败。

### M2：有效quota采样与有限历史（AC6、AC8、AC11）

- [x] OpenAiQuotaCore保存真实网络获取时刻与cache来源，检查所有调用者，明确旧fetched_at语义修正与JSON兼容。
- [x] CodexQuotaService绑定稳定account身份；保留main桶和原始窗口长度，不猜primary/secondary。
- [x] 新observation manager复用CodexPaths、锁和原子writer；35天/4096条/8MiB上限，无raw响应或凭证。
- [x] 只沿现有查询路径保存新网络样本，不新增周期性联网、不重复cache样本、不重放auth动作。
- [x] 测试cache、rename、缺字段、损坏/超限、并发写、权限/替换失败；成功quota与history失败分别展示。

回退点：不完整history保持旧文件和当前quota结果；没有独立有效采样时展示insufficient_samples。

### M3：纯经验估算（AC7–AC9）

- [x] 实现D5分段key、对齐时间范围、同scan水位、inferred范围状态；known scope mismatch直接抑制。
- [x] 3个独立区间、每段至少5pp、最多20段、rawpercent、median/min/max与max/min>2规则采用预定阈值；不根据真实账号结果调阈值。
- [x] 实现Token和USD独立可用性、freshness=5分钟、reset/bucket/plan/model/tier变化失效、joint min。
- [x] 每个N/A状态均有fixture；未知/缺失值不变为0，不声明全账户精确覆盖。
- [x] 回放estimate-examples断言$5.15/$10.30与10M/$5容量示例，并记录该证据为synthetic。

回退点：任何分母、时间或覆盖错误使受影响指标N/A；保留原始Token和quota，不修改原始百分比。

### M4：TUI后台接入与双语展示（AC10）

- [x] 状态层仅接收领域DTO；原有选择、pagination、u/r/Ctrl+L和quota错误保持行为。
- [x] 本地扫描、费用计算、观测存储走可注入后台loader；用账号+generation防止迟到结果覆盖。
- [x] 添加费用/分类/容量及local_estimate/coverage/N/A，次数标为用量记录数；显示真实获取时间。
- [x] TestBackend覆盖EN/ZH：80×24、100×22、100×30、120×22、140×40、180×50。慢loader、partial、UNPRICED、stale、globalfallback、error都走真实composition路径。
- [x] 原生终端使用合成隔离目录检查代表性尺寸；独立记录native结果，不能把TestBackend当作native PASS。

### M5：独立检查、合同写回与交付（AC12）

- [x] trellis-check检查R1–R8到每项AC的合同、真实diff、旧registry读写和共享consumer兼容。
- [x] 将新合同写回D7指定spec；不改未涉及的共享API或各平台模板。
- [x] 按下面门槛验证，修复本任务引入的失败。旧失败与外部阻塞单独留证，不改变正式门槛。
- [x] 完成最终产品diff/敏感字段/范围检查和native状态记录。提交、归档或发布须另行获得相应授权。

## 验证命令和证据边界

本轮领域、价格、TUI、workspace、Tauri/binding、前端、覆盖率、VS Code 和合成原生检查已执行并通过。用户批准的两份 JS 锁文件、3 条目恢复已完成并通过独立检查。第六轮完整 `just ci` 退出 0，16 项步骤全部通过，用时 12:30.169；AC12 完成。第五轮依赖审计失败及修正记录保留在 `checks/verification.md`。根 AGENTS 和 justfile 要求默认并行与 `--skip export_bindings`，执行时保持原值。所有 fixture 用临时目录和 crate 内 TestCodexEnv/注入后端，不读取个人 auth、sessions、usage cache 或 llmusage 数据库。

先运行预检查：

```powershell
just version-check
just fmt-check
```

相关子系统反馈：

```powershell
cargo test -p ccr-types --all-features -- --skip export_bindings
cargo test -p ccr-codex --all-features -- --skip export_bindings
cargo test -p ccr-tui --all-features -- --skip export_bindings
```

若catalog公共规则影响ccr-store/ccr-db，运行对应现有回归；完整workspace门槛仍需运行。若修改导出/bindings，走仓库独立binding-generation门槛，不通过测试附带导出。Tauri代码未列为默认修改面，但共享DTO/库变化必须在完整CI中验证既有Tauri编译/合同。

```powershell
just lint-strict
just test
just ci
```

`just ci`为本次跨模块最终门槛；局部PASS不能代替。成功后停止扩大检查，除非有新改动或新证据。Windows权限/原子替换与合成native终端分别留结果；Linux/macOS、真实OpenAI账户、hosted CI没有运行时用NOT_RUN/UNVERIFIED，不填PASS。

## 交付资料

更新AC检查状态、spec合同和任务内验证记录，列出改变的行为、正式命令及未验证边界。价格来源日期与synthetic示例随任务保留。无明确后续授权时不提交、不归档、不push/PR/merge/release。
