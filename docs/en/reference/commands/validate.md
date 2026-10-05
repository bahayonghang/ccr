# validate - Configuration Validation

`ccr validate` reads Claude, Codex, and Grok profile documents and existing runtime settings. The command does not initialize files, repair current markers, create backups, or create log directories.

## Usage

```bash
ccr validate
```

## Scope

Profiles use `<CCR_ROOT>/platforms/<platform>/profiles.toml`. The registry uses `<CCR_ROOT>/config.toml`. Runtime paths follow the existing Claude, Codex, and Grok path resolvers.

Each platform uses the same domain validator as profile application. Claude subscription profiles do not require API-key fields. Codex validates the selected OpenAI or provider auth mode. Grok validates official session profiles and third-party API-key or environment-key profiles separately.

A disabled profile can remain in the document. The command reports a warning for an inactive disabled profile and an error for a disabled current profile. A simplified document without a current marker and an existing empty document remain inactive.

Claude selects an existing file-marker target before an existing registry-marker target. A stale or conflicting marker produces `warning/invalid` when a valid candidate exists. Diagnostics use the selected candidate without repairing either marker. A selected disabled profile still returns `90`; diagnostics do not select another enabled profile. Explicit markers with no existing candidate return `62`. Codex and Grok retain their current selection rules.

Runtime file checks do not depend on profile presence. Existing corrupt or unreadable runtime files still produce errors. For a valid current profile, diagnostics classify runtime read and syntax errors before checking the operational state.

## Exit Codes

| Condition | Category | Exit code |
| --- | --- | --- |
| Valid configuration, or warnings only | `valid`, `missing`, `inactive`, `disabled` | `0` |
| Invalid profile or disabled current profile | `invalid`, `disabled` | `90` |
| Invalid profile TOML syntax or field type | `corrupt` | `14` |
| Profile or runtime read failure | `unreadable` | `51` |
| Invalid existing runtime JSON or TOML | `corrupt` | `40` or `41` |
| Explicit current target is missing and the domain rules permit no valid fallback | `invalid` | `62` |

Other failures retain the existing `CcrError` code. When several errors occur, the first error in registry, Claude, Codex, Grok order determines the exit code. Error reports no longer return a successful process status.

A missing path differs from denied access, a sharing violation, or a directory at a file path. Read errors are not treated as an unconfigured profile.

## Output and Boundaries

The terminal report contains severity, category, platform, and target. Reports omit credentials, source documents, and rejected field values. Diagnostics do not apply profiles.

Use [doctor](./doctor) for the environment, runtime auth, conflicts, and optional online checks. Grok owns official session authentication. Offline diagnostics do not prove that a session can log in. Gemini and Droid retain compatibility adapters; their auth/profile command support has not expanded.
