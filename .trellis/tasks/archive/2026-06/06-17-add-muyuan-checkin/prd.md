# 将 muyuan.do 签到站添加到提供商列表

## Goal

将 muyuan.do（君の公益）签到站添加到 ccr-ui 的提供商列表中，使其支持添加账号、执行签到、查询余额等标准操作。

## Background

用户提供了 muyuan.do 签到页面截图（https://muyuan.do/console/personal），显示该站支持：
- 每日签到（"Daily Check-in", 已签到 15 天）
- 余额显示（$2892.10 Current balance）
- 月度统计（本月消费 $2600，总收入 $3120）
- 日历签到记录（每日签到状态 + 奖励金额）

## Requirements

### FR-1: 添加 Catalog 条目
在 `crates/ccr-checkin/data/providers-catalog.json` 的 `providers` 数组中添加新条目：
- **id**: `builtin-muyuan`
- **name**: 君の公益 或 Muyuan
- **domain**: `muyuan.do`
- **websiteUrl**: `https://muyuan.do`
- **icon**: 🎌 或 🌸
- **bizCategory**: `community`（公益站）
- **checkinCategory**: 待确定（standard / waf_required / cf_required）

### FR-2: 签到能力配置
必须包含 `checkin` 块，基于标准模式推断：
- **baseUrl**: `https://muyuan.do`
- **checkinPath**: 需验证（推断 `/api/user/sign_in` 或 `/api/user/checkin`）
- **balancePath**: `/api/user/self`（标准）
- **userInfoPath**: `/api/user/self`（标准）
- **authHeader**: `Authorization`
- **authPrefix**: `Bearer`
- **supportsCheckin**: `true`（截图证实）
- **requiresWafBypass**: 待测试
- **requiresCfClearance**: 待测试
- **checkinBugged**: `false`（初始假设）

### FR-3: API 端点验证
- 通过浏览器开发者工具或 Web 技能访问 https://muyuan.do 确认实际 API 端点
- 确认签到接口路径（查看 Network 面板中的 POST 请求）
- 确认认证方式（Authorization header 格式）

### FR-4: 双端一致性
- Rust 后端通过 `include_str!` 自动加载（无需修改代码）
- 前端通过 `PROVIDERS_CATALOG` 自动解析（无需修改代码）
- 仅修改 `providers-catalog.json` 文件

## Acceptance Criteria

### AC-1: API 端点调研完成
- [ ] 通过 /web-access 技能访问 https://muyuan.do/console/personal
- [ ] 确认了签到按钮的实际请求路径
- [ ] 确认了余额查询的实际请求路径
- [ ] 确认了认证方式（Bearer token / Cookie）

### AC-2: Catalog 配置正确
- [ ] `providers-catalog.json` 中添加了 `builtin-muyuan` 条目
- [ ] 所有必填字段存在且非空
- [ ] JSON 语法正确（无逗号、引号错误）
- [ ] 符合 schema version 1 规范

### AC-3: 编译验证通过
- [ ] `cargo check -p ccr-checkin` 通过
- [ ] `just tauri-check` 通过
- [ ] `just frontend-typecheck` 通过

### AC-4: 前端显示验证
- [ ] 启动 `npm run tauri dev`
- [ ] 在签到页面的"提供商管理"Tab 中能看到"君の公益 (muyuan.do)"
- [ ] 提供商信息显示正确（名称、图标、域名）

## Technical Constraints

### 文件修改限制
- **仅修改**: `crates/ccr-checkin/data/providers-catalog.json`
- **不修改**: TypeScript 类型定义、Rust 结构体、前端组件

### Schema 契约
- 当前 schema version: 1
- 必填字段由 `providersCatalog.ts` 的 `assertCatalogEntry` 校验
- 必须与 Rust 侧 `BuiltinProvider` 结构体字段对齐

### ID 命名规范
- 内置站点 ID 格式: `builtin-<slug>`
- 本次使用: `builtin-muyuan`

## Out of Scope

- ❌ 添加 platforms 块（Claude/Codex/OpenCode 配置）—— 本次仅添加签到功能
- ❌ OAuth 配置 —— 除非调研发现该站支持 GitHub/LinuxDo OAuth
- ❌ CDK 配置 —— 除非调研发现该站支持 CDK 充值

## Risks & Mitigation

### R-1: API 端点可能不是标准格式
**缓解**: 优先使用 /web-access 技能实际探测，如果无法访问则使用保守配置（标记为需用户反馈）

### R-2: 可能需要 WAF 绕过
**缓解**: 初始标记 `requiresWafBypass: false`，如果用户报告 403 错误再调整

### R-3: 签到接口可能有特殊响应格式
**缓解**: 标记 `checkinBugged: false`，如果后端解析失败再标记为 true 并添加特殊处理

## Notes

- 本任务为**轻量级任务**，PRD-only 即可，无需 design.md 和 implement.md
- 参考现有示例: `builtin-anyrouter`, `builtin-agentrouter`, `builtin-coderouter`
- 如果 /web-access 技能无法访问，可以先使用标准模式添加配置，后续根据用户反馈调整
