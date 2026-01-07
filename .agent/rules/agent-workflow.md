---
trigger: always_on
glob: "**/*"
description: "Workflow rules for the AI Agent: Planning, Safety, and Execution."
---

# Agent Workflow & Behavior

## 1. Planning Mode (Mandatory)
- **Think First:** Before writing any code, analyze the request and existing context.
- **Plan Artifacts:** For complex tasks (features, refactors), explicitly list the steps:
  1.  **Analyze:** What files are affected?
  2.  **Design:** What changes are needed? (Signatures, Data Structures)
  3.  **Verify:** How will this be tested?
- **User Confirmation:** If the plan involves significant architectural changes or deletion, ask for confirmation.

## 2. Context Awareness
- **Read Before Write:** Always read relevant files (`read_file`, `search_file_content`) to understand local conventions before modifying.
- **Directory Structure:** Respect the `src/` (Rust) vs `ccr-ui/` (Frontend) separation.

## 3. Safety & Integrity
- **Atomic Operations:** When possible, keep file writes atomic.
- **No Broken Builds:** Ensure the project compiles after changes.
- **Secrets:** NEVER output or commit secrets (API keys, env vars).

## 4. Tool Usage
- **Justfile:** Always prefer `just <command>` over raw `cargo` or `npm` commands.
  - Build: `just build`
  - Test: `just test`
  - Lint: `just lint`
