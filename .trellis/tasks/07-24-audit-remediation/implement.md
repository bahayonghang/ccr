# Audit remediation parent execution plan

## Child delivery loop

- [ ] Review and activate the next child only after its PRD/design/implementation
  plan and manifests pass validation.
- [ ] Load relevant Trellis specs before edits; implement the complete child
  contract and add requirement-linked regression tests.
- [ ] Run narrow tests, subsystem gates, and the child check review; fix every
  finding before commit.
- [ ] Update affected specs, make an atomic Chinese Conventional Commit with
  emoji, archive the child, record the work commit in the journal, and update
  the parent evidence matrix.

## Ordered children

- [x] `07-24-audit-install-plan-handle` (`b444b459`; focused install,
  bindings, frontend, lint, and workspace tests passed)
- [x] `07-24-audit-ssh-hardening` (`19cef4b2`; SSH/Tauri/frontend, lint,
  and workspace tests passed)
- [x] `07-24-audit-webdav-sync` (`0e58e9e9`; sync/Tauri/Vitest, lint,
  frontend, and workspace tests passed)
- [x] `07-24-audit-persistence-migration` (`3a3c9c55`; Windows/WSL2 secret
  writer, migration, ccr-db, CLI/Codex, lint, and workspace tests passed)
- [x] `07-24-audit-process-gateway` (`e5892e04`; gateway, Windows process
  tree, Tauri, frontend, lint, and workspace tests passed; PR #42 Linux,
  Windows, and macOS hosted process gates passed)
- [x] `07-24-audit-ci-governance` (`691fd0d5`, `bb46226b`, `7e7c4514`,
  `158b007c`, `6951839f`, `09acd6f2`, `133842b3`; 四个稳定 contexts 与
  Linux/Windows/macOS hosted matrix 通过；`main`/`dev` strict required
  protection 已配置并回读)
- [x] `07-24-audit-typed-ipc`（实现 `3de89558`、证据 `b381e1ad`、
  归档 `f8201d42`、journal `de6deaf1`；metadata 315/315、typed 252/315、
  精确单一声明 252/252，以及 runtime policy/ACL/confirmation/timeout
  ownership 全部验证通过）
- [x] `07-24-audit-p3-cleanup` (`a4e9dd3f`; facade deprecation、umbrella
  dependency guard、职责拆分契约、UTF-8/JSON 格式门禁；public API/doctest、
  scripts、migration、fmt、lint、workspace tests passed；`version-check` 仅被
  并行 `ccr-ui/README.md` 版本事实阻塞)

## Final integration

- [x] Re-run every child security regression target against the integrated tree.
- [x] Generate the 35-finding and quantitative before/target evidence matrix.
- [x] Run `just version-check`, `just fmt-check`, subsystem gates, and `just ci`;
  `version-check` 的唯一失败为任务外 `ccr-ui/README.md` 缺少 `version-7.0.0`。
- [x] Inspect final tracked/untracked diff and prove unrelated pre-existing work
  was neither overwritten nor accidentally included.
- [x] Record the superseding unsigned-release decision: delete the
  `release-signing` child, revert signing/provenance gates, retain SHA-256 as an
  integrity-only mechanism, keep the updater disabled, and classify P2-14 as
  `ACCEPTED_RISK` rather than `PASS`.
- [x] Commit an integration/spec evidence checkpoint without absorbing unrelated
  work.
- [ ] After final local validation passes, archive the parent and journal all
  work commits. Do not merge PR #43, create a tag/release, or push further
  changes.
