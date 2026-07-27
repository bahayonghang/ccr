# Release signing and provenance verification

An official release contains only artifacts that passed platform signing, VSIX
publisher signing, and GitHub build provenance. `SHA256SUMS` proves transport
integrity; it does not prove publisher identity.

## Verify downloaded artifacts

Verify GitHub attestation and the centralized checksum manifest first:

```bash
gh attestation verify <artifact> --repo bahayonghang/ccr
sha256sum -c SHA256SUMS --ignore-missing
```

On macOS, verify the code signature, Gatekeeper assessment, and notarization
ticket:

```bash
codesign --verify --deep --strict --verbose=2 "CCR Desktop.app"
spctl --assess --type execute --verbose=2 "CCR Desktop.app"
xcrun stapler validate "CCR Desktop.app"
xcrun stapler validate CCR_Desktop.dmg
```

On Windows, verify executables and installers against the system trust chain and
the RFC3161 timestamp:

```powershell
signtool verify /pa /all /v .\ccr.exe
signtool verify /pa /all /v .\CCR_Desktop.msi
Get-AuthenticodeSignature .\ccr.exe
```

Verify a VSIX with the manifest and `.p7s` from the same release:

```bash
npx --yes @vscode/vsce@3.7.1 verify-signature \
  -i ccr-vscode-<version>.vsix \
  -m extension.signature.manifest \
  -s extension.signature.p7s
```

Stop installation or update when any verification fails. Keep the currently
installed version unchanged.

## Protected release environment

The workflow references these secret or variable names; values never live in
the repository:

| Platform | Secret / variable |
| --- | --- |
| Apple | `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` |
| Windows | `WINDOWS_CERTIFICATE_BASE64`, `WINDOWS_CERTIFICATE_PASSWORD`, `WINDOWS_CERTIFICATE_THUMBPRINT`, `WINDOWS_TIMESTAMP_URL` (variable) |
| VSIX | `VSCE_PAT`, `VSIX_SIGN_TOOL_PATH` (variable) |

The VSIX job runs only on a controlled self-hosted Linux runner carrying the
`vsix-signing` label. `VSIX_SIGN_TOOL_PATH` must identify the executable sign
tool managed by the publisher. A signing, verification, SBOM, OIDC attestation,
or Marketplace publication failure prevents the GitHub Release job from running.

## Updater state

The desktop application does not currently enable the Tauri updater.
`just release-security-check` rejects an updater dependency or configuration
until a signed-manifest and provenance verifier has regression coverage proving
that failure preserves the installed version. Development artifacts cannot use
official release names or feed an updater.

## Rotation, revocation, and rollback

For identity rotation, update the protected environment, retain the old public
certificate chain for historical verification, and validate the new identity
with an isolated release. On suspected compromise, disable the environment,
revoke the certificate or publisher token, and stop tag releases immediately.

Never replace a failed release with an unsigned artifact under the same version
or tag. Publish a fixed version under a new tag. If Marketplace publication
succeeds before the GitHub Release fails, record the partial publication, keep
updater metadata withheld, and resume only after GitHub-side verification passes.
