# ccr-ui 配置管理增强:系统提示词/系统配置/Profile TOML

## Goal

父任务:承载用户提出的三项 ccr-ui 配置管理能力增强的需求总集、任务地图与跨子任务验收。本任务本身不作为实现目标,实现工作全部下沉到三个子任务。

原始需求(2026-07-17):

1. 系统提示词管理 —— 管理 CLAUDE.md、AGENTS.md 等 memory/instructions 文件,在相关平台子页面下新增子页面。
2. 完善系统配置管理 —— `~/.claude/settings.json`、`~/.codex/config.toml` 等系统级配置文件的管理能力补全。
3. Profile 管理支持 TOML 直接编辑 —— profiles.toml 的 raw 编辑模式。

> 修订记录:2026-07-17 根据 Codex 规划审阅修订(TOCTOU 并发契约、执行环境语义、明文信任边界、平台范围分级、写路径假设修正、共享组件归属、自动化测试底线)。

## Task Map

| 子任务 | 目录 | 交付物 |
| --- | --- | --- |
| 系统配置管理完善 | `.trellis/tasks/07-17-platform-settings-enhancement` | settings 分层可视化 + raw 源文件编辑 + **共享编辑器组件** + **锁内 CAS 写入 API** |
| Profile TOML 直接编辑 | `.trellis/tasks/07-17-profile-toml-editing` | Profiles 页面 raw TOML 编辑模式 |
| 系统提示词管理 | `.trellis/tasks/07-17-system-prompts-management` | 各平台"系统提示词"子页面 + 后端读写命令 |

### 执行顺序(显式依赖,非建议)

1. **07-17-platform-settings-enhancement 必须先行**:它交付两个共享前置物 ——
   - 共享编辑器组件(JSON/TOML/Markdown 三模式);
   - ccr-core 层"锁内版本校验写入"API(见并发契约)。
2. **07-17-profile-toml-editing** 与 **07-17-system-prompts-management** 是其显式后继:两者的后端命令可与 1 并行开发,但**前端编辑面与 raw 保存路径必须消费 1 的交付物,不得自造**。若 1 未完成,2/3 不满足 `task.py start` 的前置条件(在各自 implement.md 里体现为首项 checklist)。

## Cross-child Contracts(对三个子任务都生效)

### C1 并发写入契约(锁内 CAS,替代"mtime 比较")

- 所有 raw 保存**禁止**"锁外比较 mtime 后调用 `write_guarded`"的模式 —— `crates/ccr-core/src/core/guarded_write.rs` 只保证单次写互斥,读-改-写事务性由调用方负责(见该文件头注释),锁外比较存在 TOCTOU。
- 统一方案:在 ccr-core 新增 versioned 写入 API(命名与签名由 platform-settings 的 design.md 冻结,形如 `write_guarded_versioned(path, bytes, expected_token, opts)`),在**同一把路径锁内**顺序完成:读当前内容 → 计算并比对版本令牌 → 备份 → 原子写;令牌不匹配返回冲突错误,不落盘。
- 版本令牌 = **内容哈希**(工作区已有 blake3 依赖;mtime 在 Windows/网络盘粒度不可靠,不得单独作为令牌)。get 命令返回原文 + 令牌;save 命令携带读取时的令牌。
- 注意:LockManager 文件锁不可重入,调用方不得先持有同名路径锁再调 `write_guarded`(死锁);因此该 API 必须落在 ccr-core 层内部实现,而非 Tauri 层拼装。

### C2 执行环境语义(raw 仅 Local)

- Claude 表单读写走 active `ExecutionEnvironment`(Local/WSL/SSH,见 `ccr-ui/src-tauri/src/commands/claude.rs` `active_environment`)。本期所有 raw 编辑(settings/config/profiles/memory 文件)**仅在 Local 环境启用**:active env 非 Local 时,raw 入口禁用并展示原因文案("远程执行环境暂不支持源文件直接编辑"),避免表单与 raw 双轨操作不同文件。
- 远程环境的 raw 编辑与远程写入等价保障,记录为后续任务,不在本期。

### C3 明文信任边界(统一三个子任务,替代含混的"不得绕过 masking")

masking 约束的精确含义:**结构化列表/详情视图与日志继续脱敏;raw 编辑是显式授权的明文信任边界**(与既有 export include_secrets 同级)。具体要求:

- raw 读取必须直读磁盘原文,**禁止**从已脱敏的结构化 DTO 回填拼装(否则会把 `****` 占位符写回磁盘毁文件)。
- 打开 raw 编辑前 requestConfirm 明文警示(内容可能包含 API key/token)。
- 全链路(Rust log/tracing、前端 console、telemetry)禁止输出文件内容;错误信息只含行列号与原因,不回显原文片段中的敏感行。
- 保存时保持文件既有权限语义:目标文件若属 secret 类(如 profiles.toml),写入走 `WriteOptions.secret = true`(Unix 0o600)。
- 前端不持久化明文内容(不进 Pinia 持久层/localStorage),离开编辑视图即释放;不额外提供"一键复制全文"类捷径。

### C4 共享编辑器组件(归属明确)

- 由 **platform-settings-enhancement** 交付,位于 `ccr-ui/src/components/`,支持 JSON/TOML/Markdown 三种模式、错误行内展示(行列号定位)、未保存标记、明暗主题、reduced-motion 兼容。
- 技术选型(CodeMirror 6 按需引入 vs 零依赖 textarea + 行号叠层)是该任务 design.md 的决策项;当前 ccr-ui 无任何代码编辑器依赖,选型需评估包体积与维护成本,PRD 不预设结论。
- 其余两个子任务只消费,不得引入第二套编辑器或复制实现。

### C5 工程规范

- **API 门面边界**:新增前端 invoke 包装一律放 `ccr-ui/src/api/domains/<domain>.ts`;`src/api/tauri.ts` 是冻结门面(见 `.trellis/spec/ccr-ui/frontend/api-facade-boundary.md`)。
- **Tauri 命令规范**:`#[tauri::command]` + `spawn_blocking` 做文件 I/O,返回 `Result<T, String>`,注册进 `handler_registry`;参照 `tauri-command-scaffold` skill。
- **i18n**:新增文案 `zh-CN` / `en-US` 双语齐全。
- **确认交互**:破坏性动作走 requestConfirm(见 `.trellis/spec/ccr-ui/frontend/confirm-interaction-contracts.md`)。
- **设计语言**:Anthropic-like 编辑式表面 + 明暗双主题。

### C6 自动化测试底线(每个子任务的验收都必须包含)

- Rust 单测:版本令牌冲突拒写、锁内 CAS 正确性、备份生成与轮换、非法 payload(语法/语义)拒写、文件不存在分支、secret 权限保持。
- 前端 smoke:API 门面边界测试(`tests/api-facade-boundary.smoke.test.ts` 通过)、新路由可解析、`bun run test:i18n`。
- 敏感路径专项:明文不入日志(单测断言错误消息不含文件内容片段)。
- 手工验证与代码审查仅作为补充,不得替代上述自动化项。

## Acceptance Criteria(父任务收口)

- [ ] 三个子任务全部完成并归档,各自验收标准达成。
- [ ] 跨子任务集成检查:共享编辑器组件仅一份实现,三处消费;versioned 写入 API 仅一份实现,所有 raw 保存路径均经由它。
- [ ] C1–C3 契约抽查:无锁外 mtime 比较、非 Local 环境 raw 入口确认禁用、无脱敏占位符回写路径(建议由 rust-security-reviewer 子代理复核)。
- [ ] `just fmt-check`、`just frontend-check-quick`、`cd ccr-ui/src-tauri && cargo check` 通过;最终以 `just ci` 收口。
- [ ] 路由、各平台子页导航与 i18n 无回归。

## Notes

- 现状摸底与外部调研结论见各子任务 prd.md 的 Research 段。
- 已知既有缺陷(本父任务范围内处理归属):`LocalEnvironment::write_config` 为裸 `tokio::fs::write`(`ccr-ui/src-tauri/src/platform/local.rs`),无备份/锁/原子替换 —— 修复纳入 platform-settings-enhancement(其 R4)。
- 已知既有不一致(仅提醒,不在本父任务修):`resolve_config_path` 将 opencode 映射到 `~/.opencode`,而 `commands/opencode.rs` 使用 `~/.config/opencode/`;system-prompts 任务的路径解析须复用各平台命令模块,不走 ExecutionEnvironment 的 opencode 映射。
- 本任务创建于 dev 分支;子任务实现建议各自走 `feature/*` 分支或在 dev 上小步提交,遵循 Conventional Commits。
