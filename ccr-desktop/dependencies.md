# Direct dependency decision — 2026-10-17

Checked the official [crates.io API](https://crates.io/api/v1/crates/gpui-kit) on the implementation day.
`max_stable_version` and `newest_version` both returned **0.7.3**, published
2026-10-12, not yanked, Apache-2.0. Raw registry response is retained at
`outputs/desktop/foundation/gpui-kit-registry.json` (generated local evidence).
This independently rechecks the planning snapshot.

| Direct dependency | Latest stable candidate | Selected | Features | Declared upstream MSRV | Reason / source |
| --- | --- | --- | --- | --- | --- |
| gpui-kit (normal) | 0.7.3 | =0.7.3 | default-features=false; component, assets | Not declared (`rust_version: null`) | Single application facade pins matching GPUI/Base/Component/platform family. [Registry](https://crates.io/crates/gpui-kit/0.7.3), [manifest](https://docs.rs/crate/gpui-kit/0.7.3/source/Cargo.toml) |
| gpui-kit (dev) | 0.7.3 | =0.7.3 | component, assets, test-support | Not declared | Same facade; test-support only under dev-dependencies for actual control interaction tests. [Testing contract](https://docs.rs/crate/gpui-kit/0.7.3/source/TESTING.md) |

There are no other direct external or domain dependencies in the foundation probe.
The lockfile is application-owned and must be committed with the manifest. Registry versions and
checksums were pinned there; the foundation used no git main, Cargo patch or local source override.
The narrowly scoped task 02 Windows adapter exception is documented below.
`cargo generate-lockfile` resolved 826 packages to the latest Rust 1.95 compatible requirements.
That resolver choice is not execution evidence for the declared MSRV.

The Kit manifest binds the gpui-pre **0.3.9** family and Base / Component / Assets **0.7.3**.
Normal target `cargo tree --locked --edges normal` contains neither Tauri nor GPUI Fast.
Cargo's lockfile / metadata also retains optional GPUI Fast packages; their presence alone does not
mean the normal application enables that backend. `gpui-fast`, profiler, inspector, speech,
tree-sitter languages, WebView and JS extension features are not selected by this application.
Component's own fixed transitive parsers / editor dependencies still appear in the lock graph.

MSRV policy: this independent application declares Rust **1.95**, matching repository crates,
with development compiler **1.98.0** inherited from `../rust-toolchain.toml`. Upstream Kit does
not declare an MSRV, so only an actual `cargo +1.95 check --locked --all-targets` establishes
compatibility for the tested platform. See task evidence for actual results and missing platforms.
Development compiler success must not be substituted for MSRV evidence.

The upstream snapshot and defaults for platform text rendering are retained, including Kit's
macOS font-kit configuration. [Upstream setup](https://gpui-kit.com/docs/getting-started/).
Latest compatible transitive selections, platform libraries, licensing, audit warnings and
unavailable targets are recorded with commands in the task evidence; no audit ignore is added.


## Shared application service dependencies — 2026-10-18

Official registry candidate metadata is retained under
`outputs/desktop/services/{tokio,tokio-util,async-channel}-registry.json`.
The new `ccr-app` package has MSRV 1.95, matching its domain owners.
It has no GPUI, Tauri, umbrella `ccr`, or WebView dependency. Desktop consumes it
as a path dependency; root CLI default-members are unchanged.

| New direct dependency | Latest stable candidate | Selected / lock | Features | Upstream MSRV | Source / exception |
| --- | --- | --- | --- | --- | --- |
| `ccr-app` (desktop / Tauri) | local workspace 7.5.0 | local path 7.5.0 | none | 1.95 | `crates/ccr-app/Cargo.toml`; shared configuration list and bounded runtime |
| `ccr-config`, `ccr-core` (ccr-app) | local workspace 7.5.0 | local path 7.5.0 | existing domain defaults | 1.95 | Domain parsing, persistence, errors and Secret owner; no alternative DB |
| `async-channel` (ccr-app) | 2.5.0 | 2.5.0 in all three locks | default `std` | 1.60 | [Registry](https://crates.io/crates/async-channel/2.5.0); bounded receiver compatible with GPUI executor |
| `tokio` (ccr-app) | 1.53.3 | workspace requirement 1.53.1; root/Tauri locks 1.53.1, desktop lock 1.53.3 | workspace features (`fs`, `io-util`, `macros`, `net`, `process`, `rt-multi-thread`, `signal`, `sync`, `time`); dev adds `test-util` | 1.71 | [Registry](https://crates.io/crates/tokio/1.53.3); preserve existing root/Tauri resolution to avoid unrelated upgrade |
| `tokio-util` (ccr-app) | 0.7.20 | workspace requirement 0.7.19; root/Tauri locks 0.7.19, desktop lock 0.7.20 | `rt` | 1.71 (0.7.19), 1.85 (0.7.20) | [Registry](https://crates.io/crates/tokio-util/0.7.20); existing cancellation/task tracker compatible release retained in root/Tauri |
| `tempfile` (ccr-app dev only) | 3.27.0 | workspace requirement/lock 3.27.0 | default `getrandom` | 1.63 | [Official registry API](https://crates.io/api/v1/crates/tempfile), independently checked 2026-10-18; raw response `outputs/desktop/services/tempfile-registry.json`; no new global installation |

Adding domain crates initially made Cargo's MSRV fallback resolver downgrade
existing desktop transitive choices: font-types 0.12.7, h2 0.4.20,
libredox 0.1.25, open 5.4.4, redox_users 0.5.3, rustls-native-certs 0.8.5,
schannel 0.1.30, zbus_names 4.3.5, zvariant 5.15.1, zvariant_derive 5.15.1.
All ten were restored to the foundation selections using precise Cargo updates;
no manifest pin or permanent resolver override was added. Only newly needed
shared-domain dependencies were resolved. Final `--locked` Windows metadata
passes; native and platform acceptance are recorded separately.

## Windows accessibility adapter exception — 2026-10-18

Only this independent desktop workspace patches `accesskit_windows` **0.34.0**
to `vendor/accesskit_windows`. Its edition 2024, MSRV **1.85**, public API,
features and dependency requirements remain unchanged. Registry archive SHA-256:
`106c2b961215864d1c2e703ee63269c25c4e80a577ffb2c1017b9c17dcdf83a1`.
The crate's own VCS metadata pins upstream commit
`c88605b96d04431f9c3c792464a0f2f253480e94`, `platforms/windows`.
MIT, Apache-2.0 and Chromium-derived license notices are retained in the vendor directory.

The selected GPUI Windows 0.3.9 family requires the compatible `0.34` adapter.
The official crates.io API was rechecked on 2026-10-18: latest stable is 0.35.1,
and the only non-yanked compatible `0.34.*` release is 0.34.0 (declared MSRV 1.85).
Raw response: `outputs/desktop/services/accesskit-close/accesskit-windows-registry.json`.
Upstream 0.35.1 is a newer candidate with changed AccessKit dependencies; its
changelog does not establish a fix, and inspected upstream main still unwraps
`ClientToScreen`. Exact findings and sources are in task 02
`research/native-close-race.md`. The local exception fixes the actual invalid-HWND
panic observed during Alt+F4: fallible coordinate conversion propagates
`UIA_E_ELEMENTNOTAVAILABLE` through four existing provider methods, retaining
normal accessibility rather than disabling it. No made-up coordinates or panic
catch is used in production. `vendor/accesskit_windows/CCR-PATCH.md` records the
source, focused diff and reproducible no-window test gate.

Only the adapter lock entry changes from registry checksum to a repository path;
all other locked package versions/sources/dependency lists remain unchanged.
Root CLI and Tauri workspaces do not inherit this `[patch.crates-io]` entry.

## Command favorites/history domain reuse — 2026-10-18

| New direct dependency | Candidate / selected | Features | MSRV | Source / reason |
| --- | --- | --- | --- | --- |
| `ccr-db` | local 7.5.0 / path 7.5.0 | existing domain defaults; no custom feature | 1.95 | `crates/ccr-db/Cargo.toml`; sole desktop UI-state SQL, UUID and persistence owner |
| `ccr-core` | local 7.5.0 / path 7.5.0 | existing domain defaults | 1.95 | `crates/ccr-core/Cargo.toml`; reuse existing text redaction before native projection |
| `tempfile` (desktop dev) | 3.27.0 / =3.27.0 | defaults | 1.63 | Reuses the implementation-day official registry verification and source response in the shared-service table above; private file database tests, no global HOME mutation |

Adding ccr-db requires its existing crypto/YAML dependencies (including the domain's
deprecated serde_yaml 0.9.34 release); this task does not replace the owner's
serialization or encryption choices. Cargo initially downgraded seven already
locked optional platform packages under MSRV fallback. Precise updates restore
font-types 0.12.7, libredox 0.1.25, open 5.4.4, redox_users 0.5.3,
zbus_names 4.3.5, zvariant 5.15.1 and zvariant_derive 5.15.1.
No permanent resolver override, root/Tauri lock update or unrelated dependency
upgrade is introduced. Locked checks are recorded in task 03 domain evidence.


## Configuration / Profiles domain reuse — 2026-10-18

The implementation-day official crates.io responses are retained in
`outputs/desktop/profiles/services/{toml,serde_json,walkdir}-registry.json`.
These are current registry checks, rather than the planning snapshot.

| New direct dependency | Candidate / selected | Features | Declared MSRV | Source / reason |
| --- | --- | --- | --- | --- |
| `ccr-config` (desktop) | local 7.5.0 / path 7.5.0 | existing defaults | 1.95 | `crates/ccr-config/Cargo.toml`; sole profile catalog, typed patches, validation, canonical source, backup/import/export owner |
| `ccr-cli` (desktop) | local 7.5.0 / path 7.5.0 | default empty features; no TUI/TS enabled | 1.95 | `crates/ccr-cli/Cargo.toml`; existing platform/apply/off/operation journal and security owner |
| `ccr-codex` (desktop) | local 7.5.0 / path 7.5.0 | existing defaults | 1.95 | `crates/ccr-codex/Cargo.toml`; existing runtime/auth normalization and explicit environment export |
| `ccr-store` (desktop) | local 7.5.0 / path 7.5.0 | existing defaults | 1.95 | `crates/ccr-store/Cargo.toml`; configuration operation history, separate from command UI recents |
| `serde_json` (desktop + ccr-app) | 1.0.154 / =1.0.154 | default std; domain-unified features retained | 1.71 | [Official registry](https://crates.io/api/v1/crates/serde_json); existing Tauri platform-specific field decoders extracted for both adapters |
| `toml` (desktop + ccr-app) | 1.1.8+spec-1.1.0 / =1.1.8 | default std/serde/parse/display; existing unified preserve_order retained | 1.85 | [Official registry](https://crates.io/api/v1/crates/toml); existing domain typed patch values and safe syntax-location validation; no replacement profile parser/persistence |
| `walkdir` (ccr-app) | 2.5.0 / =2.5.0 | no optional features | Not declared | [Official registry](https://crates.io/api/v1/crates/walkdir); shared existing depth-3 .bak/.backup cleanup policy with explicit managed root |

Only `ccr-app` adds the three direct external dependencies in the root graph.
The desktop graph already had serde_json 1.0.154, toml 1.1.8 and walkdir 2.5.0.
The root/Tauri shared graphs previously locked serde_json 1.0.151 and toml 1.1.4;
selecting the current compatible stable candidates updates those two plus the
four required TOML family packages (serde_spanned, toml_datetime, toml_parser,
toml_writer). No unrelated package version/source/checksum was removed or upgraded.
`outputs/desktop/profiles/services/lock-delta.json` retains the three-way comparison.
Tauri dependency-name version qualifiers change with the TOML family as expected.
The desktop graph adds 44 previously absent packages required by the actual
existing domain crates and does not remove or upgrade existing package records.

No ccr umbrella, Tauri, GPUI or WebView dependency is introduced into ccr-app.
The root CLI default-members and legacy public recipes remain unchanged.
Root and Tauri verification after the first runs uses the project-resolved real
Cargo/rustc 1.98.0 with its bin directory prepended only in the child process;
`outputs/desktop/profiles/services/toolchain.json` records exact paths/versions.
Desktop verification continues through its normal project-local Cargo resolver.
MSRV 1.95 and final native acceptance remain separate checks recorded in task04.
