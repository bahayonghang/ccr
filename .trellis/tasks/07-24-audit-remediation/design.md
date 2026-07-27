# Audit remediation parent design

## Role

This parent is a governance and integration task. It owns the verified finding
set, child mapping, execution order, cross-child invariants, and final evidence
matrix. It does not own product-code implementation.

## Execution graph

1. Release blockers: install handle, SSH hardening, WebDAV sync, persistence
   and migration.
2. Stability/governance: ProcessGateway and CI governance. Their quick fixes
   may land earlier, but full acceptance follows release-blocker contracts.
3. Contract expansion: typed IPC consumes the stabilized install/process/sync/
   SSH APIs.
4. Supply chain: release signing consumes final CI/release artifact definitions.
5. P3 cleanup consumes the final module boundaries and version-7 facade state.

Children are independently started, checked, committed, and archived. The
parent task map is updated immediately after each archive with its work commit
and verification evidence.

## Cross-child invariants

- Renderer/user-controlled data never becomes an executable, shell fragment,
  filesystem escape, or unowned process capability.
- Secret masking, lock order, backup-before-destructive-change, atomic replace,
  restrictive permissions, and parent durability are preserved.
- Generated typed contracts and command metadata have one source of truth.
- A local test cannot substitute for required cross-platform, hosted CI,
  branch-protection, or real signing evidence.
- `FAIL` and `UNVERIFIED` findings remain open; absence of a reproduced failure
  is not evidence of closure.

## Integration evidence

The parent maintains a finding-to-proof matrix for all 35 IDs. Each row names
the code change, focused regression test, child work commit, platform/hosted
evidence when required, and current status. Final closure requires all P1 and
in-scope P2/P3 requirements to be `PASS`, all children archived, a clean scoped
working tree, and a successful `just ci` on the final integrated commit.

External signing and GitHub repository settings are recorded separately from
repository-side implementation. They can close only with authoritative remote
state and real artifact verification under the user-selected completion model.

## Rollback

Each child keeps a capability-sized rollback. Parent rollback reverts only the
failed child commit(s) and reruns the integration matrix; it does not restore a
known security vulnerability as a compatibility shortcut.
