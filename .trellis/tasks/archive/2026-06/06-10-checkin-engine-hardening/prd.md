# 签到引擎强化（指纹 / 运行时检测 / 宽容判定 / 4 态契约后端）

> 父任务: [06-10-checkin-optimize-templates](../06-10-checkin-optimize-templates/prd.md) · 工作包 3 + 工作包 5 后端侧 · 依赖: providers-catalog-json（静态标记降级依赖 catalog 字段语义）

## Goal

从源头减少签到被拦截与误判：请求指纹贴近真实浏览器、WAF/CF 改运行时检测、响应判定宽容化并统一出口、签到结果升级为 4 态契约（后端侧）。

## Requirements

1. **请求指纹**（`crates/ccr-checkin/src/services/checkin_service.rs`）：
   - reqwest 启用 HTTP/2（确认 `AppState.http_client` 构建处与 ccr-checkin 自建 client 路径都生效）。
   - 签到/余额/用户信息请求补齐浏览器头：现代 Chrome UA、`Accept: application/json, text/plain, */*`、`Accept-Language`、`Referer`/`Origin`（= provider base_url）、`Sec-Fetch-Dest: empty`/`Sec-Fetch-Mode: cors`/`Sec-Fetch-Site: same-origin`；签到 POST 加 `Content-Type: application/json` + `X-Requested-With: XMLHttpRequest`（参考 anyrouter-check-in checkin.py:434-445）。
2. **运行时 WAF/CF 检测**：整合现有 `is_waf_challenge`/`is_cf_challenge`（:280-298）+ Newapi-checkin 四签名（403+"Just a moment" / 403+DOCTYPE+cloudflare / 503+cloudflare+challenge / 非 JSON+challenge-platform），对**所有**站点的每个响应生效（不再仅 requires\_\* 标记站走检测分支）；catalog 静态标记降级为「预期提示」（UI 提前展示该站需要绕过，但行为以运行时检测为准）。
3. **宽容响应判定统一出口**（消灭「其实已签到却报失败」）：
   - 成功 = `success==true || status=="success" || ret==1 || code==0 || code==200`；message 取 `message || msg || data`（现有 :967-970 扩展）。
   - 已签到关键词归一：`已签到/已经签到/重复签到/签到过/already checked/already signed/already` → `AlreadyCheckedIn`，在 `do_checkin` 统一出口处理，替换现 `[ALREADY_CHECKED_IN]` 字符串前缀 hack（:988-991）。
4. **4 态结果契约（后端）**：结果状态 `Success / AlreadyCheckedIn / Failed / Skipped` + `skip_reason` 枚举（`provider_unsupported`（balance_only/不支持签到）、`provider_disabled`、`account_disabled`…）。落点：`CheckinExecutionResult`、`CheckinRecord.status`（DB 字符串兼容，确认无需 migration 或补 migration）、`CheckinJobLogEntry/Delta/Snapshot` 透传、summary 统计含 skipped。`已签到` 不计入失败统计（现已如此，保持）。
5. **奖励兜底**：签到响应解析不出 reward 时，用签到前后两次 user_info 余额差推断（`(after_quota+after_used)-(before_quota+before_used)`，参考 anyrouter/metapi 模式；预查请求已存在，仅复用数据）。**用户截图佐证**（2026-06-10）：AnyRouter 签到成功的记录行「奖励/余额」两列均为 `-`，说明当前 reward/余额快照在成功路径上经常缺失，本需求直接对应该体验缺口。
6. **测试**：判定矩阵单测（4 种成功风格 × 已签到变体 × WAF/CF 四签名 × 非 JSON）；skip_reason 路径测试；指纹头存在性测试（mock server 或 request builder 断言）。

## Acceptance Criteria

- [ ] 签到请求含完整浏览器头与 HTTP/2（测试断言 request builder / 本地 mock 验证）。
- [ ] 无静态标记的站点返回 CF 挑战页时，error_code 正确为 `cf_blocked`（运行时检测生效），且 UI 可见分类（链路依赖子任务 1 已修复）。
- [ ] `ret==1` / `status=="success"` / `code==200` 风格响应均判成功；6+ 已签到消息变体均归一为 AlreadyCheckedIn；单测矩阵全绿。
- [ ] balance_only 站点签到结果为 Skipped(provider_unsupported) 而非 Failed；Job summary 区分 skipped。
- [ ] reward 缺失时记录余额差推断值（标注推断来源）。
- [ ] `cargo test -p ccr-checkin -- --test-threads=1` + `just lint-strict` 绿；WAF 契约（backend-guidelines）行为不回退。

## Out of Scope

- 前端 4 态展示与 i18n（子任务 checkin-ux-concurrency）。
- 无头浏览器绕过实现（`refresh_waf_cookies` 占位维持，恢复仍走 Tauri WebView 流）。
- Turnstile/CAPTCHA 处理。

## Technical Notes

- 现有判定/检测代码位置：[`../06-10-checkin-optimize-templates/research/internal-checkin-architecture.md`](../06-10-checkin-optimize-templates/research/internal-checkin-architecture.md)（§单账号签到执行流）。
- 判定与检测参考：[`research/newapi-checkin.md`](../06-10-checkin-optimize-templates/research/newapi-checkin.md)（四签名 + 宽容判定 + 统一出口教训）、[`research/anyrouter-check-in.md`](../06-10-checkin-optimize-templates/research/anyrouter-check-in.md)（指纹清单 + 余额差奖励）、[`research/metapi.md`](../06-10-checkin-optimize-templates/research/metapi.md)（不支持→skipped 降级、消息分类器清单）、[`research/all-api-hub.md`](../06-10-checkin-optimize-templates/research/all-api-hub.md)（4 态契约 + 不信任本地已签缓存）。
- 注意 error_code 分类依赖消息关键词（子任务 1 已补测试）——修改错误消息文案时同步更新分类测试。
