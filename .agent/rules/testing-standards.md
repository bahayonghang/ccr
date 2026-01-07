---
trigger: always_on
glob: "**/*.{rs,ts,vue}"
description: "Requirements for testing and quality assurance."
---

# Testing Standards

## 1. Requirement
- **New Features:** Must include accompanying unit or integration tests.
- **Bug Fixes:** Must include a regression test that fails without the fix and passes with it.

## 2. Rust Testing
- **Unit Tests:** Place in `#[cfg(test)] mod tests` at the bottom of the file.
- **Integration Tests:** Place in `tests/` directory.
- **Mocks:** Use strictly typed mocks where external dependencies are involved.

## 3. Frontend Testing
- **Unit:** Use `vitest` for logic and component rendering tests.
- **E2E:** (If applicable) Use appropriate E2E framework as configured in `package.json`.

## 4. Verification
- Run `just check` or `just test` before declaring a task complete.
