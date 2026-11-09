# Research：Platform 枚举扩展影响图（ARCH-002）

> 结论：`Platform::Grok` 不是两文件增量。共享枚举被多个域穷举 match，新增变体会在以下所有位置强制编译期决策。每个 match 臂必须显式写 Grok 分支（禁止 `_ =>` 兜底掩盖决策），并按下表登记语义。以 `cargo check --workspace` 的编译错误为完备性最终清单，发现新位置须回填本表。

## Capability 边界总表

| 域 | 位置 | Grok 决策 | 说明 |
|---|---|---|---|
| 平台工厂 | `crates/ccr-cli/src/platforms/mod.rs:47` `create_platform` | ✅ 支持 | 返回 `GrokPlatform`（core 任务交付物） |
| 枚举元数据 | `crates/ccr-config/src/models/platform.rs`（display/short/icon/all/implemented/FromStr） | ✅ 支持 | core 任务 R1 |
| auth/profile 命令门控 | `crates/ccr-config/src/models/platform.rs:108` `auth_profile_supported` + `crates/ccr/tests/platforms/auth_profile_surface.rs` | ✅ 支持（cli-surface 任务改） | core 任务不动此函数 |
| Doctor 设置校验 | `crates/ccr-cli/src/services/doctor_service.rs:1050` | ⏭️ skip | 仿 Qwen：`DoctorCheck::skip(id, "Grok settings validation is skipped.")`，本期不做 grok config 医生检查 |
| Doctor 运行时校验 | `crates/ccr-cli/src/services/doctor_service.rs:1427` | ⏭️ skip | 同上 |
| Doctor profile 展示分组 | `crates/ccr-cli/src/services/doctor_service.rs:1619` | ✅ 通用臂 | 并入 `Gemini | Droid | Qwen` 通用 profile 展示分支 |
| Skills / MCP preset home 映射 | `crates/ccr-skills/src/managers/mcp_preset_manager.rs:110` | ✅ 映射 `.grok` | 仅路径映射，无副作用 |
| Skills / MCP preset 安装 | `crates/ccr-skills/src/managers/mcp_preset_manager.rs:156,167` | 🚫 明确拒绝 | 返回明确的"Grok 暂不支持 MCP preset 安装"错误；本期范围明确排除 skills 注入，禁止为编译通过而误启用 |
| Sessions 解析 | `crates/ccr-store/src/sessions/parser.rs:41,510,521` | 🚫 明确不支持 | parse 返回空/None、chat-file 判定 false、projects_dir 返回不支持错误或 None（按现有签名选最小值）；不伪造 `~/.grok` 会话目录解析 |
| Sessions resume 命令 | `crates/ccr-store/src/sessions/models.rs:121` | 🚫 占位 | 本期 Grok 会话不进入 store，此臂不可达；写保守占位（如 `grok --resume {id}` 注明未验证）或 unreachable 说明，禁止对外暴露 |
| Profile current 展示名 | `crates/ccr-cli/src/commands/profile/current.rs:453` | ✅ `"Grok"` | 纯展示 |
| Root 平台枚举集成测试 | `crates/ccr/tests/platforms/general.rs` | ✅ 更新固定集合 | 数量、成员、display/short/icon 必须覆盖 Grok，避免共享枚举扩展后固定断言漂移 |
| TUI tab 构建 | `crates/ccr-tui/src/tui/app.rs`（非穷举，白名单过滤） | ✅ 支持（tui-tab 任务） | core 合入后 TUI 编译不受影响（filter 是 `matches!` 白名单，不穷举） |

## 执行纪律

1. core 任务实现时以 `cargo check --workspace` 驱动：每个因新变体报错的 match 都对照本表落决策；表中没有的新位置 → 先补表（含理由）再写代码。
2. 所有 skip/拒绝分支的用户可见文案与 Qwen 先例风格一致（英文 doctor 文案跟随现有英文、CLI 错误跟随现有中文习惯）。
3. 明确排除域（sessions/usage/skills 注入）若未来启用，是独立任务，不得在本树顺手实现。
