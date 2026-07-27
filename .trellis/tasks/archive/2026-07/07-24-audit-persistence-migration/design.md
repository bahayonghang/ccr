# Persistence and migration correctness design

## Atomic writer policy

`AsyncAtomicWriter` gains an explicit options value instead of inferring
confidentiality from a path:

- confidentiality: normal or secret;
- permission policy: preserve existing mode/ACL, otherwise create secret files
  as owner-only;
- durability: data-only or data-plus-parent-directory.

The existing constructor remains for non-secret compatibility. Credential,
profile, auth, and settings callers must use the explicit secret constructor.
The writer creates the temporary file with the final restrictive policy before
writing, fsyncs it, atomically replaces the target, reapplies/verifies preserved
metadata when the platform requires it, then fsyncs the parent directory.

On Unix, a new secret target is `0600`; replacement preserves an existing mode
that is at least as restrictive. On Windows, the target security descriptor is
captured and preserved across replacement. If that cannot be guaranteed, the
write fails closed and leaves the old target intact.

## Migration transaction framework

Each migration is expressed as a function over a transaction and registered
with version, name, postcondition, and optional preflight/backup policy.
`apply_migration` performs:

1. already-applied check;
2. pre-migration durable database backup when required;
3. begin immediate transaction;
4. schema/data work;
5. postcondition validation;
6. marker insertion with structured counts;
7. commit.

Any error before commit rolls back schema, data, rejection rows, and marker.
Non-transactional operations are explicitly split into prepare/commit phases
and are not hidden inside the helper.

## v3 repair migration

The next available migration version audits every v3 candidate. Each row must
end in exactly one state: successfully repaired or recorded in
`migration_rejections` with version, stable row identity, coded error, and
timestamp. Row decoding and update failures propagate; malformed business JSON
is recorded rather than silently discarded.

The postcondition proves:

- processed = repaired + rejected;
- no candidate disappeared from accounting;
- remaining inconsistent rows correspond exactly to rejection records;
- rerunning after success is a no-op.

The marker stores these counts. Existing version 3 history is not rewritten.

## Verification and failure model

Migration tests use historical fixture copies and failpoints before marker,
after schema changes, during backfill, during rejection insert, and during
postcondition validation. Each failed run must preserve the pre-run database
or be restorable from the verified backup; a second successful run must be
idempotent and pass `PRAGMA integrity_check`.

## Compatibility and rollback

- Public `CcrError` remains frozen; migration-specific detail stays in
  `MigrationError` and structured logs without sensitive row content.
- Existing non-secret writer callers retain their current API until migrated.
- Repair rollback restores the pre-migration backup only after integrity and
  identity checks; it never overwrites a newer database silently.
