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
- [ ] Configure the Marketplace publisher identity and verify a published VSIX
  signature.
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
| `actionlint .github/workflows/release.yml` | PASS with declared `vsix-signing` self-hosted runner label |
| `just ci-governance-check` | PASS: 47 immutable action references; dependency and command inventory gates pass |
| `just vscode-ci` | PASS: 50/50 tests and development VSIX packaging |
| `just ui-check` | PASS: backend/lint/type-check; 102 smoke files / 457 tests |
| docs build + audit | PASS |
| `just version-check` | BLOCKED by unrelated parallel `7.0.0` metadata: `ccr-ui/README.md` lacks `version-7.0.0`; version values themselves are aligned |
| GitHub `release` environment | EXTERNAL BLOCK: environment GET and secret list return HTTP 404 |
| Repository secrets inventory | EXTERNAL BLOCK: HTTP 403, token lacks Actions secrets permission |
| Real Apple/Windows/VSIX signatures and OIDC provenance | NOT RUN: no certificate, publisher sign-tool/identity, protected environment, or authorized release run |

This is a verified repository-side checkpoint only. Do not archive this task or
mark its acceptance criteria complete until the external activation rows pass
against actual release artifacts.

## Rollback checks

- A failed signer/notarizer/attestor must publish no update manifest.
- No job may print or archive decoded certificate/key material.
- Rotation documentation must preserve verification of already released
  artifacts while revoking future use of compromised identities.
