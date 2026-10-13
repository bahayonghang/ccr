# Local Rust Build Cache

## 1. Scope / Trigger

Apply when diagnosing or configuring an optional local mbx installation for the root Rust workspace or `ccr-ui/src-tauri`. Root `just build` and `just ui-build-backend` invoke ordinary Cargo; the machine's command environment owns automatic wrapping. Do not require mbx for other developers or CI, or wrap Bun/Vite as Rust compilation.

## 2. Signatures

```powershell
(Get-Command cargo).Source
mbx setup --status
mbx doctor --json
mbx settings get cache_dir
mbx cache dir
mbx adopt --dry-run . ./ccr-ui/src-tauri
mbx cache stats
mbx cache projects
```

## 3. Contracts

- `mbx setup` installs the Cargo shim; successful setup alone does not prove ordinary Cargo uses it. Effective PATH must resolve Cargo to that shim before Rustup's Cargo.
- On Windows inspect both Machine and User PATH; a Machine Rust path may win over a User shim path. Preserve unrelated entries and registry value types when repairing persistence. Back up private environment data outside Git.
- A desktop application retains its inherited environment. Validate a no-profile child with the persisted environment separately from temporary PATH overrides; document the required application restart.
- `cache_dir` / `MBX_CACHE_DIR` names the shared cache root. `mbx cache dir` prints its `actions` store. Managed outputs default to `<cache_dir>/targets`, unless `target.root` / `MBX_TARGET_ROOT` overrides this.
- Explicit Cargo target-directory settings can bypass managed target placement. Inspect `--target-dir`, `CARGO_TARGET_DIR` and `build.target-dir` before interpreting workspace links.
- Before migrating, resolve the root and Tauri target locations, check active builds, and validate absolute source/destination boundaries. Preserve old outputs before changing a cross-volume managed link. Unlinking a target link must never recursively delete its destination.

## 4. Validation & Error Matrix

| Observation | Meaning / required action |
| --- | --- |
| Setup current; doctor setup warns not active | Fix effective command resolution; reinstalling mbx does not correct PATH |
| Sandbox doctor says cache not writable | Repeat authorized diagnostic outside sandbox before attributing failure to disk ACL |
| D cache configured; target still resolves to C | Inspect prior managed link and actual build entry; changing config does not prove migration |
| `mbx adopt --dry-run` skips a target link | Do not treat it as migrated; preserve outputs before a supported relocation |
| Plain build is entirely Cargo Fresh | Confirms artifacts are current, not that mbx restored cached compilations |
| Doctor detects hard-link support on Windows | Read session restore counters before claiming hard-linked outputs; verified restores may still use copies |
| New terminal works; old IDE bypasses mbx | Inspect the IDE's inherited environment and restart it |

## 5. Good / Base / Bad Cases

- Good: no-profile Cargo resolves to mbx; doctor confirms writable configured cache; both workspace targets resolve under the intended root; session evidence shows compilation storage and reuse.
- Base: no mbx installed; ordinary repository Cargo recipes continue to work.
- Bad: force an unverified `RUSTC_WRAPPER`, overwrite Rustup's executable, commit an absolute personal cache directory to Cargo config, or change ACLs in response to a sandbox-only failure.

## 6. Tests Required

Use actual environment and builds rather than unit tests that mirror configuration. Assert setup/cache/config diagnostic results, each target's resolved location, successful `just build` and `just ui-build-backend`, and mbx activity attributed to those workspaces. Prove shared reuse with the same small crate and options in two fresh independent target directories; the second run must show mbx hits/restores, not merely Cargo Fresh. Preserve existing target directories during this probe.

## 7. Wrong vs Correct

Wrong: `mbx setup --status` succeeds, therefore every Cargo invocation and cache location is correct.

Correct: inspect Cargo resolution inside the process that builds, run doctor in the real permission context, resolve both target links, then obtain actual compilation and cache-hit evidence.

References: [setup](https://mr-boxington.jdx.dev/setup), [managed targets](https://mr-boxington.jdx.dev/managed-targets), [configuration](https://mr-boxington.jdx.dev/configuration). Verify behavior against the locally installed mbx version before applying a future migration.
