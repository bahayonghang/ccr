# WAF Cookie Recovery Research

## Research Scope

Investigate why CCR UI check-in fails on AnyRouter with "WAF challenge HTML response" and identify a robust, implementation-ready optimization plan for WAF-protected NewAPI/OneAPI-style check-in services.

## Local Evidence

- Screenshots show the batch check-in flow reaches `3 / 3` accounts, enters "正在自动处理 WAF", opens an AnyRouter WAF login window, then still ends with `API error: 检测到 WAF 挑战页面（响应为 HTML）` and `自动获取 WAF Cookie 失败：未知错误`.
- AnyRouter is configured as a built-in provider requiring WAF bypass, with base URL `https://anyrouter.top` and check-in path `/api/user/sign_in`:
  - `crates/ccr-checkin/src/managers/checkin/builtin_providers.rs`
- Backend `CheckinService::refresh_waf_cookies` is currently a placeholder that deletes cached WAF cookies and returns an error. Backend auto-refresh therefore cannot repair the first failed request by itself:
  - `crates/ccr-checkin/src/services/checkin_service.rs`
- Tauri `open_waf_login` injects JavaScript that polls `document.cookie`, sends the first non-empty string through IPC, then parses and saves it. It does not currently wait for provider-required cookie names:
  - `ccr-ui/src-tauri/src/commands/waf.rs`
- Frontend WAF recovery groups failed accounts by provider, calls `openWafLogin`, then retries the failed accounts once:
  - `ccr-ui/src/views/checkin/composables/checkinWafRecovery.ts`

## External Sources

- Alibaba Cloud WAF docs: JavaScript Validation returns a JS challenge and, after successful validation, sets WAF cookies such as `acw_sc__v2`; the client must include the identifier in later Cookie headers. The same doc also notes JS Validation / slider checks are designed for synchronous browser navigation and need special handling for async API requests.
  - https://www.alibabacloud.com/help/en/waf/web-application-firewall-3-0/user-guide/configure-custom-rules-to-defend-against-specific-requests
- MDN Set-Cookie docs: frontend JavaScript cannot read `Set-Cookie`, and `HttpOnly` cookies are not accessible through `document.cookie` even though browsers still send them with requests.
  - https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Set-Cookie
- Tauri 2 Webview docs: `cookies_for_url` / `cookies` can return the runtime cookie store, including HTTP-only and secure cookies, but Windows has a known deadlock caveat for synchronous command/event contexts, so cookie reads should stay async / off the blocking path.
  - https://docs.rs/tauri/latest/tauri/webview/struct.Webview.html
- AnyRouter community automation script patterns:
  - A public AnyRouter multi-account script models provider-specific WAF configuration with `bypass_method: "waf_cookies"` and `waf_cookie_names` such as `acw_tc`, `cdn_sec_tc`, and `acw_sc__v2`.
  - It uses browser automation to collect WAF cookies before API sign-in and treats missing required cookie names as failure instead of retrying blindly.
  - It also notes AnyRouter check-in appears to behave as a 24-hour cadence rather than a strict midnight reset in its observed workflow.
  - https://github.com/loks666/anyrouter-autolog
  - https://github.com/millylee/anyrouter-check-in

## Inference

The current CCR behavior can fail even when the WAF login window visibly loads because "a non-empty `document.cookie` was observed" is weaker than "the provider's required WAF cookies are present and valid for API requests."

The likely failure path is:

1. API request receives an HTML WAF challenge.
2. Frontend opens the AnyRouter page in a Tauri WebView.
3. Injected script reports the first available `document.cookie` string.
4. That cookie string may not include `acw_sc__v2`, `acw_tc`, `cdn_sec_tc`, or may miss secure/HttpOnly cookies only available from the WebView cookie store.
5. CCR caches incomplete cookies and retries the API call once.
6. The retry receives the same HTML WAF page, so the UI reports WAF recovery failure.

## Recommended Implementation Direction

### Approach A: Provider-Aware Tauri Cookie Recovery (Recommended)

Implement a provider policy layer and strengthen the existing Tauri WebView recovery flow.

- Add WAF policy metadata per provider:
  - `requires_waf_bypass`
  - login URL/path
  - required cookie names
  - optional validation URL, usually `/api/user/self`
  - cache TTL override when known
- Update `open_waf_login` to:
  - navigate to the provider login or console URL
  - wait for DOM readiness and WAF completion indicators
  - read cookies from the WebView runtime cookie store with `cookies_for_url`, not only `document.cookie`
  - filter by required names
  - keep waiting until all required cookies are present or timeout
  - return a structured result with `found`, `missing`, `source`, and `expires_at`
- After cookie extraction, validate by making a lightweight request such as `/api/user/self` with merged account cookies and WAF cookies before retrying sign-in.
- Retry failed accounts once per provider after a successful provider-level cookie refresh.
- If required cookies are missing, show actionable UI copy instead of "未知错误".

Pros:

- Fits the current Tauri architecture.
- Avoids adding Playwright/browser dependencies to CCR.
- Keeps credentials in CCR's encrypted storage and WAF cache manager.
- Solves the screenshot failure directly.

Cons:

- Requires careful async handling around Tauri cookie APIs on Windows.
- Still cannot guarantee bypass for slider CAPTCHA or future bot-management changes.

### Approach B: Browser Profile / Playwright-Style External Helper

Embed or invoke a browser automation helper similar to public AnyRouter scripts.

Pros:

- Strong parity with existing automation scripts.
- Can use persistent profiles and richer wait logic.

Cons:

- Adds heavy runtime dependencies and packaging complexity.
- Higher anti-bot and account-risk footprint.
- Harder to fit CCR desktop security and verification boundaries.

### Approach C: Manual WAF Cookie Import First

Keep auto-recovery minimal and make the UI better at guiding manual cookie refresh.

Pros:

- Smallest implementation.
- Avoids fragile browser automation.

Cons:

- Does not satisfy the user expectation from the current "正在自动处理 WAF" flow.
- Leaves batch sign-in largely manual for AnyRouter.

## Flow Optimizations

- Do provider-level recovery before account-level retry: if 3 accounts share AnyRouter, open only one WAF window and reuse the refreshed provider cookies.
- Stop marking the flow as "all tasks complete" when all accounts still have unrecovered WAF failures; use a terminal state such as "需要手动处理 WAF".
- Add diagnostics to the modal:
  - detected challenge class: WAF / Cloudflare / JSON API error / auth expired
  - required WAF cookies: present / missing
  - whether retry used cached or freshly acquired WAF cookies
- Cache status should show cookie names and age without values.
- Add cooldown/rate limiting: do not repeatedly open WAF windows for the same provider during one batch or within a short time window after failure.
- Preserve one retry only by default to avoid account or IP risk.

## Proposed Tests

- Rust unit tests for cookie parsing/filtering:
  - all required cookies present
  - missing `acw_sc__v2`
  - HttpOnly/secure cookie included from cookie-store result
  - no cookie values appear in logs/errors
- Rust/Tauri command tests where feasible:
  - `open_waf_login` returns structured missing-cookie errors
  - cache write only happens after required cookies are present
- Frontend unit/smoke tests:
  - WAF recovery displays missing cookie names and provider name
  - retry result merge does not convert unrecovered WAF failures into success
  - one provider-level recovery is performed for multiple failed accounts
- Backend service tests:
  - cached WAF cookies are merged into pre-check and sign-in requests
  - WAF HTML response maps to `waf_blocked`
  - retry is not infinite

## Risks And Guardrails

- This should not attempt to defeat arbitrary CAPTCHA or force headless anti-detection. The goal is to reuse legitimate browser-acquired cookies for the user's configured provider.
- Do not log cookie values, auth headers, API keys, or decrypted account cookies.
- Do not store account email/password as part of this task unless explicitly scoped later.
- Respect the existing encrypted credential boundary and `WafCookieManager` cache.
- Keep the feature honest in UI copy: auto-recovery can fail when the provider changes WAF rules.

## Web Access Note

The `web-access` skill was loaded, but its advertised `scripts/check-deps.mjs` file is absent from the installed skill directory, so CDP browser automation was not used. Research used static public web sources instead.
