# SSH hardening implementation plan

## Ordered work

- [ ] Introduce validated host/user/identity/remote-path types and hostile-corpus
  unit/property tests.
- [ ] Centralize OpenSSH argument construction with app-owned known-hosts,
  strict checking, batch mode, timeouts, and option boundaries.
- [ ] Add the backend host-key challenge registry with TTL and single-use
  confirmation; remove the renderer fingerprint write API.
- [ ] Make `connect_internal` perform the real nonce handshake before updating
  state; classify new/match/mismatch and trust failures explicitly.
- [ ] Implement SFTP read and staged atomic write operations; remove shell-based
  write construction and retain no unsafe compatibility path.
- [ ] Update Tauri DTOs/front-end calls for `challenge_id` and typed trust
  outcomes.
- [ ] Add fake OpenSSH fixtures for key-new/match/mismatch, challenge replay,
  hostile paths, option injection, upload failure, and rename failure.
- [ ] Update SSH/security specs with the finalized trust and transport contract.

## Focused validation

```powershell
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml ssh -- --test-threads=1
just frontend-check-quick
just lint-strict
just test
```

Run platform integration coverage where available for Windows, Linux, and
macOS. Record unavailable platform evidence as unverified rather than passed.

## Rollback checks

- Disabling remote writes must not modify local profiles.
- Host mismatch must never fall through to network-error retry logic.
- No command line, event, or audit record may contain private-key or password
  material.
