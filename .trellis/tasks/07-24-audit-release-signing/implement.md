# Release signing and provenance implementation plan

## Repository-side work

- [x] Inventory the actual release artifacts and updater state; define exact
  environment-scoped secret names and least-privilege job permissions.
- [ ] Add macOS keychain import/sign/notarize/staple/verify steps with guaranteed
  cleanup and fixture-backed verifier tests. Workflow steps and cleanup are
  implemented; a real signed/notarized fixture is not available locally.
- [ ] Add Windows Authenticode sign/timestamp/verify steps for binaries and
  installers with fixture-backed verifier tests. Workflow steps are
  implemented; a trusted Authenticode fixture is not available locally.
- [ ] Add VSIX publisher authorization, manifest/signature production, publish,
  and `vsce verify-signature` verification. The fail-closed path is wired but
  requires the external publisher sign-tool and identity.
- [x] Generate multi-ecosystem SBOMs and OIDC build provenance after final
  artifact creation; bind all release digests to the attestation.
- [x] Add/update updater verification so signature/provenance failure preserves
  the installed version; otherwise assert automatic update remains disabled.
- [x] Make missing identities fail closed before release publication and prove
  pull-request jobs cannot access production secrets.
- [x] Document user verification commands, identity rotation, incident revoke,
  and failed-release rollback.

## External activation

- [ ] Configure Apple Developer ID/notarization identity in the protected
  release environment and verify a real signed/stapled artifact.
- [ ] Configure Windows code-signing identity/timestamp access and verify a real
  signed installer on Windows.
- [ ] Configure the Marketplace publisher identity and a protected
  `[self-hosted, linux, vsix-signing]` runner with the publisher-managed
  sign-tool, then verify a published VSIX signature.
- [ ] Grant/verify GitHub OIDC and protected-environment policy, run a release,
  and verify provenance against downloaded artifact digests.

## Validation

```powershell
just version-check
just vscode-ci
just ui-check
just ci
```

Additionally run each platform verifier against actual release artifacts. A
fixture or dry run proves implementation behavior but does not satisfy external
activation acceptance.

## Repository-side evidence (2026-07-27)

| Evidence | Result |
| --- | --- |
| `just release-security-check` | PASS: 6/6 policy tests; updater disabled; centralized publication fail closed |
| `actionlint .github/workflows/release.yml` | PASS with actionlint 1.7.12; release workflow has no syntax or expression findings |
| `just ci-governance-check` | PASS: 52 immutable action references; release/dependency/command inventory gates pass |
| `just vscode-ci` | PASS: 50/50 tests and development VSIX packaging; unsigned development VSIX is not acceptance evidence |
| `just ui-check` | PASS: backend/lint/type-check; 104 smoke files / 464 tests; 6 unrelated generated whitespace hunks isolated then restored byte-for-byte |
| docs build + audit | PASS |
| `just version-check` | BLOCKED by unrelated parallel `7.0.0` metadata: `ccr-ui/README.md` lacks `version-7.0.0`; version values themselves are aligned |
| `just ci` | PASS: all 12 steps green in 03:53.493, including workspace tests, release build, audit, bindings drift, 104/464 frontend smoke tests, docs, and VS Code packaging |
| GitHub `release` environment | PARTIAL EXTERNAL PASS: environment exists; custom deployment policy allows only `v*` tags; repository/environment secrets and variables are all empty |
| Required branch protection | PASS: `main`/`dev` use strict required checks with admin enforcement; Root/Vue/Tauri/VS Code contexts are bound to app `15368`; force-push and deletion are disabled |
| Hosted regression PR #43 | PASS at head `94eda6d0`: Root `30259859698`, Tauri `30259859694`, Frontend `30259859557`, VS Code `30259859538`; all four required contexts passed, with Tauri Linux/Windows/macOS and gateway coverage successful. This PR did not run a tag release or receive signing identities |
| Repository/environment secrets inventory | EXTERNAL BLOCK: authoritative keyring OAuth inventory reports 0 repository secrets, 0 release-environment secrets, 0 repository variables, and 0 release-environment variables |
| Actions self-hosted runner inventory | EXTERNAL BLOCK: authoritative repository runner inventory reports 0 runners; the VSIX job requires protected labels `[self-hosted, linux, vsix-signing]` |
| Latest real release | NO-GO: `v6.5.0` run `29002291872` predates signing workflow; assets expose checksums only and the sampled artifact digest has no GitHub attestation (HTTP 404) |
| Real Apple/Windows/VSIX signatures and OIDC provenance | NOT RUN: all 12 local identity variables are absent, remote signing inventories and runner inventory are empty, and no post-remediation tag release exists |

This is a verified repository-side checkpoint only. Do not archive this task or
mark its acceptance criteria complete until the external activation rows pass
against actual release artifacts.

The first aggregate `just ci` attempt reached the frontend smoke suite and all
104 files / 464 tests passed, but Vitest exited nonzero on two transient worker
teardown rejections (`onUserConsoleLog`). The focused usage-store suite, a
standalone full smoke rerun, and the final full `just ci` all passed without a
code change, so no runner workaround was introduced without a reproducible
failure.

## Rollback checks

- A failed signer/notarizer/attestor must publish no update manifest.
- No job may print or archive decoded certificate/key material.
- Rotation documentation must preserve verification of already released
  artifacts while revoking future use of compromised identities.
