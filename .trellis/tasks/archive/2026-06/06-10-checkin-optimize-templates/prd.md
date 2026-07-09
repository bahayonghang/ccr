# brainstorm: 优化 ccr-ui 签到管理（修复报错 + 体验/性能 + 与 Provider 模板打通）

## Goal

参考 `ref/repo` 下 4 个外部项目（all-api-hub / anyrouter-check-in / metapi / Newapi-checkin）的成熟做法，优化 `ccr-ui` 签到管理功能：

1. 解决签到报错问题（错误分类、自动恢复、可操作的失败提示）。
2. 优化签到管理的体验与性能（批量并发策略、状态反馈、大组件治理）。
3. 将签到站点（公益站/商业站）元数据与 Claude Code / Codex 等页面的 Provider 模板体系打通，单一事实来源，不再两套独立维护。

## What I already know

- 签到后端：`crates/ccr-checkin`；签到前端：`ccr-ui/src/views/CheckinView.vue` + `views/checkin/**`；模板体系：`ccr-ui/src/configs/providerTemplates.ts` 等。
- 内置签到站目录：`builtin_providers.rs` 硬编码 22 个站点（Rust）；Provider 模板 40+ 条目（前端 TS 构建期合成）。**两套目录站点零重叠、机制完全独立** —— 打通的价值在机制统一（一套实体/能力模型/选择器），而非数据去重。
- 今天（06-10）刚完成 WAF Cookie 自动恢复任务（provider 级 policy、required cookies、validate-before-retry），契约在 `.trellis/spec/ccr-checkin/backend/backend-guidelines.md`。
- 截图症状已确认（2026-06-10 第二张截图）：① AnyRouter 三账号两批 `检测到 WAF 挑战页面（响应为 HTML）` 失败，WAF 自动补救已能救回（「自动补救后成功」标签正常）；② `anyrouter_stumail` 为 `HTTP 401 / Cookie 过期`（分类与建议文案正确，用户自行更新 cookie）；③ **实锤记录筛选 bug**：「失败历史记录 (5)」面板混入成功记录（计数与列表数据源不一致）；④ 成功签到行的奖励/余额列为 `-`（印证奖励兜底需求）。

### 已定位的具体问题（来自 internal research，含代码位置）

**错误链路（解释“签到报错/未知错误”）**
1. ★ 前端 `getErrorMessage = error instanceof Error ? error.message : fallback`（useCheckinState.ts:44-45 等 4 处）——Tauri v2 invoke 对 `Result<_, String>` 以**普通字符串** reject，`instanceof Error` 为 false，后端错误信息被整体丢弃 → UI 显示「未知错误」。
2. Job 路径 `Ok(Err(error))` 经 `build_failed_checkin_result` 后 error_code 被硬编码为 `task_error`（commands/checkin.rs:86-91, 231-239），丢失 `error.error_code()` 分类（waf_blocked/cookie_expired/crypto_error…）。
3. `error_code()` 分类靠错误消息字符串关键词匹配（error.rs:34-65），是隐式契约且**零测试**。
4. 批量刷余额 `Promise.allSettled` 后 rejected 全部静默丢弃（useCheckinState.ts:221-228），单账号失败用户不可见。
5. 记录筛选/分页链路断裂：前端构造 status/provider_id/keyword/page → api 层丢参（api/domains/checkin.ts:156-169）→ Tauri 命令缺参（commands/checkin.rs:695-716）→ 后端 `get_paginated_advanced` 全仓库无调用方。「失败历史」面板显示的是任意状态记录，过滤翻页是假的。

**性能/体验热点**
6. 批量刷余额前端无界并发（N 账号 N 路并发 HTTPS）；后端签到 Job 有 Semaphore(5) 但余额刷新无信号量；无节流/minInterval。
7. WAF 补救重试用 500ms × 240 次轮询（checkinWafRecovery.ts:210-223），未复用既有事件推送。
8. `AccountManager::list` 对每个账号解密 cookies 仅为生成掩码（account_manager.rs:52-66）。
9. CheckinAccountsTab.vue 2062 行（~1050 行 scoped CSS）；CheckinManageView.vue + stores/checkin.ts 是无路由引用的死代码。
10. 错误展示通道混用 alert() 与 toast。

**目录打通的债务**
11. `to_checkin_provider()` 落库时丢弃 icon/category/WAF 标记/cdk_config/oauth_config，运行期到处按 **name 反查** builtin 补元数据（前端 ×2、后端 ×2），用户改名即断链。
12. 前端 `types/checkin.ts` 手工镜像 Rust `BuiltinProvider` struct，双端漂移风险。
13. Tauri 签到命令层 + Job 状态机零测试。

## Research References

- [`research/internal-checkin-architecture.md`](research/internal-checkin-architecture.md) — 签到链路全景、错误五层转换路径、性能热点、两套目录逐字段对比、打通集成点、测试缺口。
- [`research/anyrouter-check-in.md`](research/anyrouter-check-in.md) — 请求指纹（HTTP/2 + 完整浏览器头）是抗 WAF 第一杠杆；必需 cookie 不齐全即失败；奖励用余额差推断；内置 provider = 可被用户增量覆盖的默认值。
- [`research/newapi-checkin.md`](research/newapi-checkin.md) — 标准 NewAPI 站只需 base_url 一个输入；WAF/CF 用运行时检测（4 条签名）替代静态标记；宽容响应判定（4 种成功风格 + 已签到关键词归一）须放在统一出口。
- [`research/all-api-hub.md`](research/all-api-hub.md) — 站点「软件类型注册表 + 实例目录」分离；bundled JSON + 远程更新 + schemaVersion 严格校验；签到结果 4 态契约（success/already_checked/failed/skipped + 跳过原因枚举）；minInterval 节流 + per-origin 串行队列；每日/重试双调度。
- [`research/metapi.md`](research/metapi.md) — 「预设(数据) + 实例(DB)」两层；能力按平台协议推导而非逐站硬编码；跨语言单一事实源 = 纯数据文件（Rust include_str! + 前端同源消费）；签到工程细节清单（不支持→skipped 降级、凭证过期自动重试一次、奖励余额差兜底、站内串行站间并行）。

## Feasible Approaches（目录打通）

**Approach A: 共享 JSON 站点目录，双端各自消费（Recommended）**

- 新增一份仓库内站点目录数据文件（如 `providers-catalog.json`，含 schemaVersion）：每站点 = 通用元数据（id/name/域名/双轴分类/官网/图标）+ 能力块 `checkin`（paths/auth/WAF/CDK/OAuth，仅签到站有）+ `platforms`（claude/codex/opencode override，仅 API 模板站有）。一个站可同时具备两种能力。
- Rust 端 `include_str!` + serde 编译期内嵌，替换 builtin_providers.rs 硬编码 vec（公开 API 不变）；前端构建期 import 同一份 JSON 替换 providerTemplates.ts 的硬编码部分，模板选择器与签到内置站列表都从它投影。
- Pros: 真单源；无代码生成步骤；Web dev 模式（无 Tauri）仍可用；与 metapi/all-api-hub 的成熟模式一致；未来可平滑升级为「远程拉取 + bundled 兜底」。
- Cons: 需要设计 schema 与双轴分类；Rust struct 与 TS 类型仍需各写一份（用 serde 测试 + JSON Schema 校验防漂移）。

**Approach B: Rust 为唯一 owner，前端模板全部改走 Tauri command**

- 模板目录整体下沉 Rust，新增 `list_provider_catalog` 命令；前端 useProviderTemplates 改为运行时 invoke。
- Pros: 单一 owner 最彻底；自定义条目可统一进 SQLite（随 WebDAV 同步）。
- Cons: `npm run dev`（纯 Web 预览）丢失模板数据；模板消费方（6+ 处 View）全要改造数据时序；改动面最大。

**Approach C: 前端模板 registry 吸收签到元数据，build script 生成 Rust 常量**

- ProviderTemplate 增加 `platforms.checkin` override；用类 `version-sync` 脚本从 TS/JSON 生成 builtin_providers.rs。
- Pros: 前端改动最小。
- Cons: 引入代码生成步骤与 CI 校验；后端核心数据依赖前端目录文件，方向别扭。

## Decision (ADR-lite)

### D1: 目录打通采用 Approach A —— 共享 JSON 站点目录

**Context**: 22 个签到站硬编码在 Rust、40+ Provider 模板硬编码在前端 TS，机制独立、字段语义部分重叠（name/域名/分类/官网），且签到侧落库丢元数据后靠 name 反查（改名断链）、前端手工镜像 Rust struct 有漂移风险。需要单一事实来源同时服务签到页与 Claude/Codex/OpenCode 模板选择器。

**Decision**: 用户选定 Approach A（2026-06-10）。仓库内新增一份带 schemaVersion 的 `providers-catalog.json`：站点条目 = 通用元数据（id/name/domain/icon/双轴分类）+ 可选 `checkin` 能力块（paths/auth/WAF/CDK/OAuth）+ 可选 `platforms` 模板能力块（claude/codex/opencode override）。Rust 端 `include_str!` + serde 编译期内嵌替换 builtin_providers.rs 硬编码；前端构建期 import 同一份 JSON，模板选择器与签到内置站列表都从它投影。双端各持类型定义，以 serde 单测 + JSON 校验（CI）防漂移。

**Consequences**: 无代码生成步骤、`npm run dev` 纯 Web 预览仍可用；新增/修订站点只改 JSON；未来可平滑升级为「远程拉取 + bundled 兜底」（all-api-hub sponsor-catalog 模式）。代价是需设计双轴分类 schema，且 Rust/TS 两份类型定义需要校验机制保持一致。

## Requirements

按 6 个工作包收敛（详细需求在各子任务 PRD）：

1. **P0 报错修复链路**：getErrorMessage 处理 Tauri 字符串 rejection；Job 路径保留 error_code；分类逻辑补测试；记录筛选/分页接通 `get_paginated_advanced`；刷余额失败可见。
2. **目录打通（方案 A）**：`providers-catalog.json` 单源（schemaVersion + 双轴分类 + checkin/platforms 能力块）；Rust `include_str!`；前端同源消费；落库保留 builtin_id 消除 name-join；双端防漂移校验。
3. **抗 WAF 请求指纹**：reqwest HTTP/2 + 完整浏览器头；WAF/CF 四签名运行时检测对所有站点生效，静态标记降级为提示。
4. **并发/节流治理**：余额刷新并发上限 + 30s minInterval + 同站串行异站并行；WAF 补救重试事件化（删轮询）；list 路径去无谓解密。
5. **4 态结果契约 + 展示统一**：success/already_checked/failed/skipped + 跳过原因枚举（i18n）贯穿后端→事件→前端；宽容判定与已签到归一在统一出口；alert 全部替换为 toast。
6. **组件治理**：CheckinAccountsTab 拆分（FormModal/ActionsMenu/卡片）+ CSS 公共层；删除 CheckinManageView + stores/checkin.ts 死代码。

横切约束：保持 secret 脱敏、加密存储、原子写入、日志不出 cookie/token；WAF 恢复契约（backend-guidelines.md）不回退。

## Acceptance Criteria

- [ ] 后端错误信息原样到达 UI；「未知错误」仅在 message 真缺失时出现（子任务 1）。
- [ ] 记录页失败历史过滤/翻页真实生效（子任务 1）。
- [ ] `builtin_providers.rs` 无站点数据字面量，22 站 golden test 等价；模板选择器出现签到公益站；provider 改名不再断元数据（子任务 2）。
- [ ] 签到请求带 HTTP/2 + 完整浏览器头；未标记站点的 CF 挑战被运行时正确分类；已签到变体归一不再误报失败（子任务 3）。
- [ ] 批量刷余额并发受限 + 节流；WAF 重试无轮询；结果面板 4 态展示；签到代码 `alert(` 清零（子任务 4）。
- [ ] CheckinAccountsTab ≤600 行；死代码删除无残留引用（子任务 5）。
- [ ] 全子任务: `just lint-strict` + `cargo test -p ccr-checkin -- --test-threads=1` + `just frontend-check-quick` 绿。

## Implementation Plan（子任务，按依赖顺序）

| # | 任务目录 | 范围 | 优先级 |
|---|---|---|---|
| 1 | [`06-10-checkin-error-chain`](../06-10-checkin-error-chain/prd.md) | 工作包 1（纯 bugfix，可立即开工） | P0 |
| 2 | [`06-10-providers-catalog-json`](../06-10-providers-catalog-json/prd.md) | 工作包 2（目录单源 + name-join 消除） | P1 |
| 3 | [`06-10-checkin-engine-hardening`](../06-10-checkin-engine-hardening/prd.md) | 工作包 3 + 5 后端侧（依赖 #2 的 catalog 字段语义） | P1 |
| 4 | [`06-10-checkin-ux-concurrency`](../06-10-checkin-ux-concurrency/prd.md) | 工作包 4 + 5 前端侧（依赖 #3 的 4 态契约） | P2 |
| 5 | [`06-10-checkin-component-split`](../06-10-checkin-component-split/prd.md) | 工作包 6（纯重构收尾） | P2 |

## Definition of Done (team quality bar)

- Tests added/updated（Rust: `cargo test -p ccr-checkin -- --test-threads=1`；前端: `bun run test` / smoke；error_code 分类与 getErrorMessage 行为补测试）
- `just fmt-check` / `just lint-strict` / `just frontend-check-quick` 绿
- 行为变化更新 `.trellis/spec/ccr-checkin/` 与 provider-template contracts spec

## Out of Scope (explicit)

- 远程目录拉取/热更新（schema 预留 schemaVersion，本期仅 bundled）。
- 定时自动签到调度（cron/interval）。
- 邮箱密码登录与 api_user 自动拦截（anyrouter-check-in 模式，远期可选）。
- CDK `try_cdk_topup` 实现补全（现为 no-op，维持现状）。
- 绕过任意 CAPTCHA/Turnstile；无头浏览器绕过实现（`refresh_waf_cookies` 占位维持）。
- 自定义模板 localStorage → SQLite 迁移；claudePresets 存量模板数据全量搬入 catalog。
- CheckinView.vue / CheckinProvidersTab / ProviderTemplateSelector 的拆分。

## Open Questions

- ~~打通方案 A / B / C~~ → 已决策 A（见 Decision D1）。
- ~~MVP 范围~~ → 6 个工作包全选，拆 5 个子任务（2026-06-10 用户确认）。
- ~~截图报错具体形态~~ → 已确认（见 What I already know 末条）：WAF 批量失败（补救已生效）+ stumail Cookie 过期 401 + 失败历史面板混入成功记录 + 成功行奖励/余额缺失。全部已被子任务 1/3/4 覆盖。

## Technical Notes

- 任务目录: `.trellis/tasks/06-10-checkin-optimize-templates`
- 关键文件清单见 research/internal-checkin-architecture.md（含 file:line）。
- 前序任务: `06-10-ccr-ui-checkin-waf-cookie-flow`（已归档）、`06-08-ccr-ui-provider-templates`（已归档）。
- `ref/repo` 为只读外部镜像，仅作资料参考。
