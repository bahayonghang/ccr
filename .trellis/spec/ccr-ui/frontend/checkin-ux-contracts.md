# Check-in UX Concurrency Contracts

> Frontend contracts for check-in batch concurrency, event-based job waiting, 4-state result display, and toast-only error surfacing.

---

## Scenario: Balance refresh concurrency governance

### 1. Scope / Trigger

- Trigger: changing batch balance refresh, the per-key queue, or refresh throttling in `ccr-ui/src/features/checkin/`.
- Applies to `lib/balanceRefreshQueue.ts` and `hooks/useCheckinState.ts` (`refreshAllBalances` / `refreshAccountBalance`).

### 2. Signatures

- `runPerKeySequential<T>(tasks: PerKeyTask<T>[], concurrency = BALANCE_REFRESH_CONCURRENCY): Promise<PromiseSettledResult<T>[]>` — same-key serial, cross-key parallel, results in input order (allSettled semantics).
- `shouldSkipBalanceRefresh(lastBalanceCheckAt: string | undefined, now?: number, minIntervalMs?: number): boolean`
- Constants: `BALANCE_REFRESH_CONCURRENCY = 5` (aligned with the backend check-in Semaphore), `BALANCE_REFRESH_MIN_INTERVAL_MS = 30_000`.

### 3. Contracts

- Batch refresh must never issue unbounded `Promise.allSettled` over accounts. The queue key is the provider `base_url` origin (fallback: `provider_id` when the URL fails to parse) so same-origin requests are strictly serial while different origins run in parallel up to the cap.
- Accounts whose `last_balance_check_at` is within 30s are skipped from batch refresh and the skipped count is surfaced via `uiStore.showInfo` (`checkin.info.balanceRefreshSkipped`). Manual single-account refresh is the force path and bypasses the throttle.
- Missing or invalid `last_balance_check_at` must NOT skip (refresh proceeds).

### 4. Validation & Error Matrix

- Same-key tasks overlapping in flight -> contract broken (guard test asserts per-key max concurrency of 1).
- More than `concurrency` key groups running at once -> contract broken.
- A rejected task must not abort other tasks; failures are collected and reported in one error toast.

### 5. Good/Base/Bad Cases

- Good: 20 accounts across 8 origins refresh with ≤5 concurrent requests, same-origin requests serialized.
- Base: a second "refresh all" within 30s skips already-fresh accounts and shows the skipped count toast.
- Bad: reintroducing `Promise.allSettled(accounts.map(...))` without the queue, or throttling the single-account refresh path.

### 6. Tests Required

- `cd ccr-ui && bun run test:smoke -- tests/checkin/checkin-balance-queue.smoke.test.ts`

### 7. Wrong vs Correct

#### Wrong

```typescript
await Promise.allSettled(enabledAccs.map((a) => queryCheckinBalance(a.id)));
```

#### Correct

```typescript
await runPerKeySequential(
  accountsToRefresh.map((account) => ({
    key: getAccountOriginKey(account),
    run: () => queryCheckinBalance<BalanceSnapshot>(account.id),
  })),
);
```

---

## Scenario: Event-based check-in job waiting (no polling)

### 1. Scope / Trigger

- Trigger: changing how the frontend waits for check-in job completion, including the WAF recovery retry path.
- Applies to `ccr-ui/src/features/checkin/lib/waitForCheckinJob.ts` (`waitForCheckinJobResult`) and `lib/checkinWafRecovery.ts` / `lib/checkinJob.ts`.

### 2. Signatures

- `waitForCheckinJobResult(jobId: string, initialSnapshot: CheckinJobSnapshot): Promise<CheckinJobSnapshot>` — resolves on `checkin:job-finished` / `checkin:job-timeout` for the matching `job_id`.
- Events: `checkin:job-delta` (progress), `checkin:job-finished`, `checkin:job-timeout`.

### 3. Contracts

- No `setTimeout`/interval polling loops for job status. Terminal state arrives via Tauri events; after listeners attach, exactly one `getCheckinJobStatus` reconciliation call covers events fired before attachment.
- Events for other job ids and non-terminal snapshots must be ignored; listeners are removed on settle.
- The job-start failure path reports through a toast callback (`notifyJobStartFailed`), never `alert()`.

### 4. Validation & Error Matrix

- Event fired before listener attach -> reconciliation snapshot resolves the wait.
- Listener setup or reconciliation throws -> promise rejects after unlisteners are cleaned up.
- Reintroducing a `for { sleep(500ms) }` poll -> contract broken (`checkin-waf-event-wait.smoke.test.ts` asserts zero timers).

### 6. Tests Required

- `cd ccr-ui && bun run test:smoke -- tests/checkin/checkin-waf-event-wait.smoke.test.ts tests/checkin/checkin-state.smoke.test.ts tests/checkin/checkin-progress-modal.smoke.test.tsx`

### 7. Wrong vs Correct

#### Wrong

```typescript
for (let attempt = 0; attempt < 240; attempt += 1) {
  await new Promise((resolve) => setTimeout(resolve, 500));
  snapshot = await getCheckinJobStatus(jobId);
}
```

#### Correct

```typescript
unlisteners.push(
  await listen("checkin:job-finished", (e) => settle(e.payload)),
);
unlisteners.push(await listen("checkin:job-timeout", (e) => settle(e.payload)));
settle(await getCheckinJobStatus(jobId)); // 一次对账，覆盖事件先于监听的窗口
```

---

## Scenario: 4-state result display and toast-only errors

### 1. Scope / Trigger

- Trigger: changing check-in result panels, summaries, record status rendering, or error surfacing in checkin views.
- Applies to `CheckinView.tsx`, `CheckinRecordsTab.tsx`, `useCheckinState.ts`, and the `checkin.*` i18n namespace (`zh-CN` + `en-US`).

### 2. Signatures

- Result grouping computeds: `successCheckinResults` / `alreadyCheckedInResults` / `failedCheckinResults` / `skippedCheckinResults`.
- `getSkipReasonText(skipReason?: string)` maps `account_disabled` / `provider_disabled` / `provider_unsupported` to `checkin.skipReasons.*`; `getSkippedDetail(item)` falls back to `item.message` then `checkin.detail.skipped`.
- Frontend summary builder (`buildCheckinSummary`) counts `skipped` separately; `CheckinSummary.skipped` stays optional for old payload compatibility.

### 3. Contracts

- Result panels and summaries render four groups: success / already_checked_in / failed / skipped. Skipped is never counted or styled as failure.
- For persisted records, `skip_reason` arrives through the `error_code` column (backend 4-state contract); record reason rendering must map it for `status === 'skipped'`.
- All check-in error surfacing goes through `uiStore` toasts (`showError` / `showInfo` / `showSuccess`); `alert(` count in check-in related sources must stay 0.
- `cookie_expired` failures expose an "更新 Cookie" quick-fix entry (result cards via `openAccountCookieFix`, record rows via the `update-cookie` event) that opens the account editor with the cookies field focused (`pendingEditAccountId` prop on `CheckinAccountsTab`, consumed via `pending-edit-consumed`).
- New `checkin.*` i18n keys must land in `zh-CN.ts` and `en-US.ts` in the same change.

### 5. Good/Base/Bad Cases

- Good: a `skipped` result with `skip_reason: 'provider_unsupported'` renders in the skipped group with the mapped zh/en copy.
- Base: old backend payloads without `summary.skipped` render without a skipped badge.
- Bad: lumping skipped into failed counts, or adding a new `alert()` in checkin code.

### 6. Tests Required

- `cd ccr-ui && bun run test:smoke -- tests/checkin/checkin-cookie-fix.smoke.test.tsx tests/checkin/checkin-accounts-tab.smoke.test.tsx`
- `cd ccr-ui && bun run test:i18n`
- `rg -n "alert\(" ccr-ui/src/features/checkin` must return nothing.
