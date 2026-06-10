# brainstorm: ccr-ui 签到 WAF Cookie 流程优化

## Goal

优化 `ccr-ui` 签到管理页在 AnyRouter / WAF 保护服务上的自动恢复流程，避免“已打开 WAF 登录窗口但仍然重试失败并显示未知错误”的体验。目标是让 CCR 能可靠判断 WAF cookie 是否真正获取完整，按 provider 级别缓存和复用，并在无法自动恢复时给出明确、可操作的失败原因。

## What I Already Know

- 截图症状：3 个 AnyRouter 账号签到时都返回 HTML WAF 挑战页；CCR 进入“正在自动处理 WAF”，打开 AnyRouter 页面，但最终仍显示 `API error: 检测到 WAF 挑战页面（响应为 HTML）` 和 `自动获取 WAF Cookie 失败：未知错误`。
- AnyRouter 内置 provider 已标记 `requires_waf_bypass: true`，路径为 `https://anyrouter.top/api/user/sign_in`。
- 后端 `CheckinService::refresh_waf_cookies` 当前是占位实现，只删除缓存并返回“WAF 绕过功能尚未实现”。
- 现有 Tauri `open_waf_login` 只依赖注入脚本轮询 `document.cookie`，拿到任意非空 cookie 就投递和缓存，没有校验 provider 需要的 cookie 名称。
- 前端 WAF recovery 已能按 provider 分组失败账号，打开一次 WAF 登录窗口，然后对失败账号重试一次。
- 网络调研表明 AnyRouter 同类脚本通常采用 provider 级策略：声明 `bypass_method: "waf_cookies"` 和 required cookie names，例如 `acw_tc`、`cdn_sec_tc`、`acw_sc__v2`，并在 API 请求前用真实浏览器获取这些 cookie。
- WAF 文档表明 JS challenge 通过浏览器执行后会下发后续请求必须携带的验证 cookie；`document.cookie` 无法覆盖所有 cookie 形态，Tauri 2 的 WebView cookie-store API 可以读取包括 HttpOnly/Secure 在内的 cookie，但 Windows 同步上下文有 deadlock 注意事项。

## Assumptions

- MVP 不承诺绕过任意滑块 CAPTCHA 或更强的人机验证，只处理“标准浏览器完成 WAF JS 验证后获取 cookie 并复用”的场景。
- AnyRouter 当前最关键的 WAF cookie 至少包括 `acw_tc`、`cdn_sec_tc`、`acw_sc__v2`；实际实现应把这些作为 provider 策略而不是硬编码到通用流程。
- 优先修复 Tauri desktop 运行时；普通 web dev 模式可以通过 mock/smoke tests 验证 UI 状态，不要求真的打开 provider 页面。

## Requirements

- Provider 策略层必须表达 WAF 恢复所需信息：
  - 是否需要 WAF bypass。
  - login / console URL。
  - required WAF cookie names。
  - 可选验证路径，例如 `/api/user/self`。
- `open_waf_login` 不应以“任意 `document.cookie` 非空”为成功条件；必须等 required cookies 齐全，或返回结构化缺失信息。
- Cookie 提取应优先读取 Tauri WebView runtime cookie store，覆盖 `document.cookie` 看不到的 HttpOnly/Secure cookie；`document.cookie` 可作为兼容补充。
- Cookie 缓存写入必须发生在 required cookies 齐全之后，避免缓存无效 cookie。
- 签到 retry 前应使用合并后的账号 cookie + WAF cookie 做轻量验证，验证仍是 HTML/WAF 时不再盲目重试。
- 同一批次中同一 provider 只进行一次 WAF 恢复，多个账号共享恢复结果。
- UI 需要区分：
  - WAF cookie 获取中。
  - required cookies 缺失。
  - cookie 已获取但验证失败。
  - 重试成功 / 重试仍失败。
- 错误日志和 UI 不得泄露 cookie、API key、Authorization header、解密后的账号 cookie。
- 保留一次自动重试上限，避免重复触发 provider 风控。

## Acceptance Criteria

- [ ] 当 AnyRouter 返回 HTML WAF 挑战页时，前端进入 provider 级 WAF 恢复，并只为 AnyRouter 打开一个恢复窗口。
- [ ] 如果 `acw_tc`、`cdn_sec_tc`、`acw_sc__v2` 不齐全，UI 显示缺失 cookie 名称和 provider 名称，不再显示“未知错误”。
- [ ] 如果 required cookies 齐全且 `/api/user/self` 验证通过，CCR 缓存 WAF cookies 并只重试此前失败的账号。
- [ ] 如果验证仍返回 HTML/WAF，CCR 标记恢复失败，不继续无限重试。
- [ ] 批量结果 summary 与日志能正确反映“自动补救后成功 / 自动补救失败 / 已签到”。
- [ ] 相关 Rust 和 frontend tests 覆盖 cookie 过滤、缺失诊断、retry merge、日志脱敏。

## Definition of Done

- Tests added/updated for frontend recovery state and backend cookie filtering.
- `just fmt-check` and `cargo test -p ccr-checkin -- --test-threads=1` pass for backend changes.
- `cd ccr-ui && bun run test` or targeted smoke tests pass for UI changes.
- `just ui-check` or narrower documented checks pass before final handoff, depending on implementation scope.
- No cookie values or auth secrets appear in logs, test snapshots, or task notes.

## Research References

- [`research/waf-cookie-recovery.md`](research/waf-cookie-recovery.md) — WAF cookie behavior, AnyRouter automation patterns, Tauri cookie-store API, and recommended implementation approach.

## Technical Approach

Recommended approach: **Provider-aware Tauri cookie recovery**.

1. Add a small provider WAF policy abstraction that can be derived from built-in provider metadata and later extended for custom providers.
2. Change `open_waf_login` to return a structured cookie recovery result:
   - provider id/name
   - found cookie names
   - missing cookie names
   - persisted / not persisted
   - diagnostic message
3. Use Tauri WebView cookie-store APIs for extraction and keep JS `document.cookie` as fallback.
4. Validate WAF cookies before account retry, then retry once per failed account.
5. Update progress modal and recovery merge logic so user-facing state matches actual recovery result.

## Feasible Approaches

### A. Provider-Aware Tauri Cookie Recovery (Recommended)

Use the existing Tauri WebView window, but strengthen extraction and validation. This is the best fit for CCR because it avoids new heavyweight dependencies and fixes the current failure path directly.

### B. External Browser Automation Helper

Adopt a Playwright/cloakbrowser-style helper like public AnyRouter scripts. This may be more capable but adds packaging, runtime, and anti-bot risk that does not fit a desktop config manager as a first step.

### C. Manual Cookie Import UX

Improve copy and manual cookie import while leaving auto-recovery shallow. This is safer but does not solve the visible “正在自动处理 WAF” promise.

## Out of Scope

- Solving arbitrary CAPTCHA, slider, or provider-side bot detection changes.
- Storing or managing account email/password credentials.
- Adding Playwright or a bundled browser dependency in the MVP.
- Reworking unrelated check-in provider templates, CDK recharge, or OAuth flows.
- Changing global theme or visual design beyond targeted progress/error states.

## Decision (ADR-lite)

**Context**: AnyRouter WAF recovery currently reports success too early because it accepts any non-empty `document.cookie` and retries without confirming required WAF cookies.

**Decision**: Implement Approach A, provider-aware Tauri cookie recovery.

**Consequences**: The implementation stays inside CCR's existing Tauri + encrypted check-in storage boundaries and avoids a bundled browser automation dependency. It must still be honest about provider-side WAF limits and return actionable missing-cookie or validation errors when automatic recovery is not possible.

## Open Questions

- None for MVP implementation.

## Technical Notes

- Relevant local files:
  - `crates/ccr-checkin/src/services/checkin_service.rs`
  - `crates/ccr-checkin/src/managers/checkin/waf_cookie_manager.rs`
  - `crates/ccr-checkin/src/managers/checkin/builtin_providers.rs`
  - `ccr-ui/src-tauri/src/commands/waf.rs`
  - `ccr-ui/src/views/checkin/composables/checkinWafRecovery.ts`
  - `ccr-ui/src/views/checkin/composables/checkinJobRuntime.ts`
  - `ccr-ui/src/components/CheckinProgressModal.vue`
- Relevant guidance loaded:
  - `ccr-ui/AGENTS.md`
  - `crates/AGENTS.md`
  - `.trellis/spec/ccr-checkin/backend/backend-guidelines.md`
- External source links are recorded in `research/waf-cookie-recovery.md`.
