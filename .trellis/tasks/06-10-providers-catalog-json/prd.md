# providers-catalog.json 共享站点目录（方案 A）

> 父任务: [06-10-checkin-optimize-templates](../06-10-checkin-optimize-templates/prd.md) · 工作包 2 · 决策见父 PRD「Decision D1」

## Goal

建立仓库内单一站点目录数据文件 `providers-catalog.json`，同时驱动 Rust 签到内置站与前端 Provider 模板，消除两套硬编码目录与 name-join 断链，让「公益站/商业站」既可签到也可一键填入 Claude/Codex/OpenCode 配置。

## Requirements

1. **Schema 设计**（schemaVersion: 1）：站点条目 = 通用元数据 + 两个可选能力块：
   - 通用：`id`（保留 `builtin-` 前缀语义）、`name`、`description`、`domain`、`websiteUrl?`、`icon`、双轴分类（业务轴 `bizCategory`: community/commercial/official/aggregator/local；机制轴 `checkinCategory`: standard/waf_required/cf_required/special/balance_only/cdk，仅签到站需要）、`aliases?`、`tags?`。
   - `checkin?` 块：`checkinPath?`（null = 查询即签到）、`balancePath`、`userInfoPath`、`authHeader`、`authPrefix`、`supportsCheckin`、`requiresWafBypass`、`requiresCfClearance`、`checkinBugged`、`wafCookieNames?`（WAF policy 数据化）、`cdk?{...}`、`oauth?{...}` —— 字段集合与 `BuiltinProvider`（builtin_providers.rs:10-78）等价。
   - `platforms?` 块：与 `ProviderTemplate.platforms`（types/providerTemplates.ts:12-51）一致的 claude/codex/opencode override + `baseUrls?`、`modelCatalog?`。
2. **文件位置与双端可达**：放在双端都能稳定引用的路径（建议 `crates/ccr-checkin/data/providers-catalog.json`，前端经 Vite alias/相对 import 引用；或仓库根 `assets/`）。实现时确认 Tauri 打包与 `npm run dev` 纯 Web 模式都能读到（构建期 import，非运行时 fetch）。
3. **Rust 端接入**：`builtin_providers.rs` 改为 `include_str!` + serde 解析（`LazyLock`），公开 API `get_builtin_providers()/get_builtin_provider_by_id()` 不变；**golden test**：22 站数量、关键字段与现硬编码逐一等价（迁移期把现 vec 转成 JSON 的脚本/单测对照）。解析失败必须是编译期/启动期 panic-free 的显式错误（带 schemaVersion 校验）。
4. **前端接入**：
   - `configs/providerTemplates.ts` 的 `BUILT_IN_PROVIDER_TEMPLATES` 合并来源加入 catalog 中带 `platforms` 块的站点（经适配函数投影为 `ProviderTemplate`），现有 claudePresets/codexOverrides/opencode presets 维持（它们的合并入 catalog 可渐进，本期不强制全量搬迁，但**签到站目录必须全部来自 catalog**）。
   - `types/checkin.ts` 的 `BuiltinProvider` 手工镜像接口标注来源并补一致性测试（基于同一 JSON 断言字段），或直接由 catalog 类型推导。
   - 签到页内置站列表继续走 `list_builtin_providers` 命令（数据已同源）。
5. **为适用的签到公益站补 `platforms` 数据**：标准 NewAPI 站的 API base 即 `domain`，为其生成 claude/codex override（用户在模板选择器可一键选公益站）。`balance_only`/特殊站按实际能力标注。
6. **消除 name-join**：
   - `CheckinProvider` 落库增加 `builtin_id?` 字段（ccr-db migration，向后兼容：旧行为 NULL 时回退现有 name 匹配一次性迁移）。
   - `to_checkin_provider()` 写入 `builtin_id`；4 处 name 反查改为 id 查：前端 WAF 标记（CheckinProvidersTab.vue:436-448）、前端 CDK 表单（CheckinAccountsTab.vue:672-685）、后端 CDK 充值（commands/checkin.rs:840-847）、WAF policy（waf_cookie_manager.rs:79-103，required cookies 从 catalog 读取替代 anyrouter 硬编码，保持 backend-guidelines 契约语义）。
7. **防漂移**：Rust serde roundtrip 单测 + 前端类型校验测试（zod 或 ts 断言）+ CI（`just lint-strict` 或 frontend-check 阶段）解析同一 JSON；schemaVersion 不符显式报错。

## Acceptance Criteria

- [ ] `builtin_providers.rs` 不再含站点数据字面量；`cargo test -p ccr-checkin` golden test 证明 22 站与改造前等价。
- [ ] 模板选择器出现签到公益站条目（带 claude/codex override），选择后生成的 patch 不含密钥字段（沿用 provider-template-contracts 白名单测试）。
- [ ] 签到 provider 改名后 WAF 标记/CDK 表单/CDK 充值/WAF policy 仍正确（builtin_id 生效；新增测试覆盖改名场景）。
- [ ] ccr-db migration 通过 `cargo test -p ccr-db -- --test-threads=1`；旧数据（无 builtin_id）升级路径有测试。
- [ ] 双端解析同一 JSON 的校验测试进 CI；故意改坏 schemaVersion 时双端都显式报错。
- [ ] `npm run dev` 纯 Web 模式模板功能不回退。

## Out of Scope

- 远程目录拉取/热更新（schema 预留 schemaVersion 即可）。
- claudePresets 等存量模板数据全量搬入 catalog（渐进，另行任务）。
- 自定义模板 localStorage → SQLite 迁移。

## Technical Notes

- 字段对比表与集成点盘点：[`../06-10-checkin-optimize-templates/research/internal-checkin-architecture.md`](../06-10-checkin-optimize-templates/research/internal-checkin-architecture.md)（§两套站点目录字段对比 / §打通集成点）。
- 模式参考：[`research/metapi.md`](../06-10-checkin-optimize-templates/research/metapi.md)（纯数据文件单源 + include_str!）、[`research/all-api-hub.md`](../06-10-checkin-optimize-templates/research/all-api-hub.md)（schemaVersion 严格校验 + 类型/实例分离）。
- 契约：`.trellis/spec/ccr-ui/frontend/provider-template-contracts.md`（非敏感约束、平台可见性、mapper 白名单）+ `.trellis/spec/ccr-checkin/backend/backend-guidelines.md`（WAF cookie 名单留后端 policy —— catalog 属后端数据源，合规）。
- OAuth client_id 等公开发布元数据可入 JSON（本就硬编码在源码中，非 secret）。
