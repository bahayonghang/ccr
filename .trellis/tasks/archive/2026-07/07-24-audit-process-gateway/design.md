# ProcessGateway and capability governance design

## Ownership boundary

All desktop-spawned processes move behind `ProcessGateway`. Callers submit a
closed capability and typed arguments, never a raw executable string. The
gateway owns resolution, environment filtering, spawn, output transport,
timeout, cancellation, process-tree cleanup, audit metadata, and terminal
state.

## Capability model

`ProcessDescriptor` contains a stable capability ID, `TrustedExecutable`,
argument validator, environment allowlist, timeout, per-stream byte limit,
concurrency class, cancellation policy, and audit policy.

`TrustedExecutable` distinguishes packaged CCR, development CCR, system
OpenSSH tools, and explicitly supported package managers. Production CCR
resolution is an absolute packaged path validated against a build-produced
hash manifest. PATH fallback is disabled. Development fallback requires an
explicit development-mode setting and remains constrained to the repository
build path.

Version output remains a compatibility check, not an identity proof.

## Foreground execution

Foreground execution uses piped readers rather than `Command::output`. Each
stream is capped at 1 MiB by default, a command descriptor may lower the cap,
and a 60-second default timeout applies unless a reviewed descriptor overrides
it. Output beyond the cap is counted/truncated and the process is terminated
according to policy. The result reports exit, timeout, truncation, byte counts,
and duration without unbounded allocation.

## Background execution and events

Readers feed a bounded channel of 256 delta batches. Lines are accumulated up
to a size/count threshold and emitted at no more than 20 Hz. Progress events
contain `{job_id, seq, channel, lines, dropped_count}`; full snapshots are
returned only by query and terminal events.

Job retention uses `VecDeque` for O(1) eviction. Queue saturation follows the
descriptor's backpressure policy and always increments an observable dropped
counter; it cannot allocate an unbounded backlog.

## Process-tree lifecycle

Unix children start in a dedicated process group. Windows children are assigned
to a Job Object configured to terminate descendants. Cancellation sends a
graceful signal, waits, escalates after a bounded grace period, reaps the direct
child, confirms the managed tree is gone, and only then marks terminal state.

The owned-process registry records PID plus creation identity/capability.
Unknown port owners are reported but never killed. OAuth port release can stop
only a matching live registry entry.

## URL capability

External navigation parses the URL and matches a per-OAuth-provider HTTPS host
allowlist. Loopback HTTP is permitted only for the fixed callback origin.
User-info credentials, file/custom schemes, malformed hosts, and unexpected
ports are rejected before platform open commands run.

## Compatibility and rollout

Capabilities migrate one at a time behind the same facade so UI command names
remain stable. The old raw helpers are deleted when the final caller moves;
there is no generic escape hatch. A capability can be disabled to roll back,
but production PATH fallback, arbitrary URL opening, and unowned PID killing
are never restored.
