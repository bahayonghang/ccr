# CLI Output Presentation

## 1. Scope / Trigger

Read this contract before changing human CLI result lines, fields, progress,
warnings, errors, or command suggestions. The shared renderer belongs to
`ccr-core`; handlers choose the message and keep the existing business branch.
Doctor uses the same status renderer and keeps its typed report and stdout.

Tables, Clap help, interactive questions, TUI, desktop, and VS Code layouts
retain their own contracts. File logging, redaction, and the log bridge follow
[Logging Contracts](../../ccr-core/backend/logging-contracts.md).

## 2. Signatures

```rust
// ccr_core::core::logging
pub enum OutputStatus { Success, Warning, Error, Step, Skipped }
ColorOutput::configure_cli_output();
ColorOutput::format_status(status: OutputStatus, msg: &str, is_terminal: bool) -> String;
ColorOutput::success(msg: &str);
ColorOutput::info(msg: &str);
ColorOutput::warning(msg: &str);
ColorOutput::error(msg: &str);
ColorOutput::step(msg: &str);
ColorOutput::key_value(key: &str, value: &str, indent: usize);

// ccr_cli::commands::common
print_next_steps(steps: &[(&str, &str)]);
```

The existing public methods retain their signatures. `format_status` returns
text without writing a stream or selecting an exit code. Pass the terminal
capability of the stream that will receive the text.

## 3. Contracts

### Status, fields, and suggestions

| Meaning | Terminal | Plain text | Existing stream |
| --- | --- | --- | --- |
| Completed operation | `✓ <message>` | `成功: <message>` | stdout |
| Warning | `! 警告: <message>` | `警告: <message>` | stdout |
| Error | `× 错误: <message>` | `错误: <message>` | stderr |
| Progress | `→ <message>` | `进度: <message>` | stdout |
| Skipped check | `- 跳过: <message>` | `跳过: <message>` | selected by caller |
| Information, count, cancellation, normal empty state | Original text without a status prefix | Same text | stdout |

- Pass plain message text to shared status functions. Remove an embedded
  duplicate status marker or whole-message color from the migrated message.
- Style only the shared status marker and field name. Keep the message body
  and required field values readable without color.
- A simple operation has one result line, adjacent indented fields, and an
  optional suggestion block. Do not add a leading blank line to the result.
- `key_value` preserves the value. Each continuation line has `indent + 2`
  spaces. Handlers omit absent optional fields instead of inventing values.
- Empty suggestions print no heading or whitespace. Nonempty suggestions
  have one blank line, `下一步`, a two-space action line, and a four-space
  command line. Commands and descriptions occupy separate lines.
- A successful operation normally offers one action and at most two. A
  failure offers an action allowed by the existing branch. Cancellation
  does not add a success result or success suggestions.
- Use an existing validated account name when available. Codex and Claude
  overwrite suggestions use `ccr <platform> auth save --force -- <name>` and
  retain the overwrite consequence. The `--` delimiter supports a leading
  hyphen. An unknown name uses a list or help command.
- Do not add shell prompt characters, outer quotes, placeholder names,
  passwords, or keys to a copyable suggestion. Do not truncate fields or
  suggestion commands for terminal width.

Doctor maps `Ok/Warn/Fail/Skip` to `Success/Warning/Error/Skipped` and prints
the formatted line to stdout. `DoctorStatus::label`, report serialization,
summary counts, details, recommendations, and read-only behavior remain
unchanged. A skipped check remains distinguishable from a passed check.

### Output capabilities

`crates/ccr/src/main.rs` calls `configure_cli_output()` once before Clap
parsing. Exact `TERM=dumb` sets the existing `colored` override to false.
Other modes do not set a global override. Per-message rendering never changes
global color state.

| Environment / target | Status characters | Shared styling |
| --- | --- | --- |
| TTY, `TERM` other than `dumb` | Symbols | Existing `colored` policy |
| TTY with `NO_COLOR`, no force | Symbols | None |
| Captured target stream, no force | Plain status words | None |
| Exact `TERM=dumb`, including force | Plain status words | None |
| `CLICOLOR_FORCE` present and not `0`, non-dumb | Determined by target TTY | Existing force policy, including priority over `NO_COLOR` |
| stdout and stderr have different TTY capabilities | Each target chooses its own characters | Existing library policy; no new override to force stderr color |

`NO_COLOR` affects styling only. Keep the installed library's treatment of
empty environment values. Startup dumb-mode suppression also covers retained
colored titles, separators, tables, and confirmation text. Independent
logging and external-tool output retain their existing contracts.

### Machine and business boundaries

JSON branches serialize the existing DTO before human rendering. Preserve
the JSON fields, values, streams, and exit codes. Explicit force color must
not decorate JSON. Preserve existing confirmation, overwrite, masking,
service-call, storage, backup, and account-switch behavior. Presentation must
not add credential reads, account checks, process checks, or network calls.
An existing error branch that returns `Ok` keeps its exit behavior unless a
separate task authorizes the business change.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Optional email or description absent | Omit the field; do not print an empty value |
| Save succeeds | Result and masked fields; suggest listing accounts |
| Duplicate name | Existing error/exit behavior; safe overwrite command and consequence |
| Unknown, logged-out, unsupported, or provider-key state | Existing state text and permitted action only |
| Partial import or repair | Keep per-item outcomes and partial-result semantics |
| Delete cancelled | Neutral cancellation; preserve the stored account |
| Doctor failure | Error marker on stdout; preserve the binary's failure exit code |
| `TERM=dumb` with forced color | No shared ANSI styling or decorative status symbols |
| JSON command with forced color | Parseable existing DTO; no human suggestion block |

## 5. Good / Base / Bad Cases

- Good: `✓ 已保存账号 teacher`, then `  邮箱: tea***@example.test`, then
  `下一步` with `ccr codex auth list` on its own command line.
- Base: captured save output starts with `成功: 已保存账号 teacher`. A normal
  empty list uses ordinary text. A multiline description retains each line.
- Bad: labeling a count or cancellation as success, repeating a success emoji
  inside `success`, or offering an OAuth save action for a provider key.

## 6. Tests Required

Use Cargo's default parallelism and `--skip export_bindings`. Process-global
environment changes use the existing crate-local fixture lock. Prefer child
process environment overrides for output-mode tests.

Windows `dirs::home_dir()` uses the system Known Folder path. Set child
`CCR_ROOT` for logger isolation and explicit Claude/Codex paths plus a temporary
Gemini home for Doctor. Follow [logging paths](../../ccr-core/backend/logging-contracts.md)
and [diagnostic paths](./diagnostics-contract.md). `CCR_LOG_LEVEL=off` filters
events but does not stop file-writer initialization or old-log cleanup.
Shared-output and Doctor-renderer test executables do not initialize these
services; record that narrower scope. Do not count ignored tests as PASS.

- `cargo test -p ccr-core core::logging -- --skip export_bindings`: status
  text, message styling, information, and multiline fields.
- `cargo test -p ccr-core --test output_presentation -- --skip export_bindings`:
  both streams, TTY character selection, `NO_COLOR`, force precedence, dumb
  startup suppression, retained helpers, and formatter stream purity.
- `cargo test -p ccr-cli commands::common::feedback -- --skip export_bindings`:
  empty blocks, command separation, and complete multiline/long text.
- `cargo test -p ccr --test commands -- output_presentation --skip export_bindings`:
  synthetic Auth and non-Auth data, JSON, stream, exit, masking, and suggestion
  assertions. Run the affected existing command regressions as well.
- For a migration across shared consumers, run the affected crate tests,
  `just version-check`, `just fmt-check`, `just lint-strict`, and `just ci`.
  Changed command documentation requires its build and audit.
- Capture the actual Windows CLI in 40/80/120-column terminals on dark and
  light backgrounds, in normal/`NO_COLOR`/dumb modes. Check required fields,
  warning/error/skip distinction, multiline text, full commands, and
  independently redirected streams. Keep process, native, OS, and hosted
  evidence separate. Mark a missing check `NOT_RUN` or `UNVERIFIED`.

## 7. Wrong vs Correct

Wrong:

```rust
ColorOutput::success(&format!("✓ 共 {} 个账号", count).green().to_string());
ColorOutput::info("使用 'ccr codex auth switch <名称>' 切换账号");
```

Correct:

```rust
ColorOutput::info(&format!("共 {} 个账号", count));
print_next_steps(&[("查看当前账号", "ccr codex auth current")]);
```

A presentation rollback restores only the approved renderer, handler text,
tests, documentation, and spec changes. Do not restore account files or run
an authentication command to roll back display text.
