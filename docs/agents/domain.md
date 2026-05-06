# Domain Docs

This repo uses a multi-context domain-doc layout. Engineering skills should use the context that matches the area being changed.

## Before Exploring

Read these files when they exist:

- `CONTEXT-MAP.md` at the repo root. It points to each context-specific `CONTEXT.md`.
- The relevant context's `CONTEXT.md`.
- `docs/adr/` for system-wide architectural decisions.
- Context-scoped ADRs near the relevant context, such as `crates/<crate>/docs/adr/`, `ccr-ui/docs/adr/`, or `ccr-vscode/docs/adr/`.

If any of these files do not exist, proceed silently. Do not flag their absence or suggest creating them upfront. Producer workflows such as `/grill-with-docs` create them lazily when terms or decisions are resolved.

## Expected Layout

```text
/
|-- CONTEXT-MAP.md
|-- docs/adr/
|   `-- 0001-system-wide-decision.md
|-- crates/
|   `-- <crate>/
|       |-- CONTEXT.md
|       `-- docs/adr/
|-- ccr-ui/
|   |-- CONTEXT.md
|   `-- docs/adr/
`-- ccr-vscode/
    |-- CONTEXT.md
    `-- docs/adr/
```

## Use the Glossary Vocabulary

When output names a domain concept in an issue title, refactor proposal, hypothesis, or test name, use the term as defined in the relevant `CONTEXT.md`. Avoid drifting to synonyms that the glossary explicitly rejects.

If the concept is missing from the glossary, either reconsider whether the project already uses a better term or note the gap for a future domain-doc producer workflow.

## Flag ADR Conflicts

If output contradicts an existing ADR, surface the conflict explicitly instead of silently overriding it.
