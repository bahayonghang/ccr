# Release signing and provenance design

## Trust outcome

A release is official only when each applicable artifact has a platform or
publisher identity signature and the release bundle has verifiable build
provenance. Checksums remain useful for transport integrity but are not treated
as identity proof.

## Pipeline separation

- Pull requests run packaging and signature-verifier tests with fixtures; they
  never receive production signing material.
- Release jobs use environment-scoped secrets and least-privilege permissions.
  Missing signing identity fails the applicable release job before publication;
  the workflow never silently uploads an unsigned artifact under the same
  release channel.
- Unsigned development artifacts, if retained, use an explicit non-release name
  and cannot feed the updater.

## Platform signing

- macOS imports a temporary keychain identity, signs the Tauri bundle, submits
  notarization, staples the result, and verifies with `codesign` and
  `spctl`. Temporary keychain material is always deleted.
- Windows imports/uses the scoped code-signing identity, signs binaries and
  installers with SHA-256 plus an RFC3161 timestamp, then verifies with
  `signtool verify /pa /all`.
- VSIX packaging generates the vsce manifest/signature inputs, publishes with
  the authorized publisher identity, and verifies the returned/provided
  signature using the installed `vsce verify-signature` flow. Publisher
  authorization is checked before publish.

Secret names and external identity setup are documented, but secret values are
never echoed, persisted in artifacts, or accepted through pull-request input.

## Provenance and SBOM

Release artifacts are finalized before hashing. GitHub OIDC with minimal
`id-token: write` permission produces build provenance bound to artifact
digests. CycloneDX/SPDX SBOMs cover Rust, UI, and VS Code packages and are
included in the attested release bundle. Attestation actions are pinned to
reviewed full commit SHAs.

## Update policy

The updater consumes only a signed manifest whose digest and platform signature
verify before apply. If no updater is currently active, documentation and a
regression check freeze that state; future enablement must pass the same
verifier. A verification failure leaves the installed version untouched.

## External activation evidence

Repository tests can prove workflow shape, fail-closed secret handling,
verification commands, fixture signatures, and provenance wiring. Full
acceptance additionally requires real Apple, Windows, Marketplace publisher,
and GitHub release identities plus verification of artifacts from an actual
release run.

Current evidence (2026-07-26): the available GitHub token receives HTTP 403 for
Actions secret inventory and branch protection. No production signing identity
is discoverable in the working tree. This is an external activation boundary,
not a repository-side pass.

## Rollback

On signing/notarization/provenance failure, publication and update metadata are
withheld. Rollback never converts the same version/tag into an unsigned official
release.
