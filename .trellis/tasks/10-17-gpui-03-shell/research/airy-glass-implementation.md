# GPUI airy-glass implementation

Date: 2026-10-27. Resumed by user's explicit request to continue the existing GPUI Trellis task. This is the approved visual increment on the existing task tree; it does not complete parent acceptance or authorize commit/push/archive.

## Delivered boundary

- `ccr-desktop/DESIGN.md` was updated first to the approved static mint/lavender scene, single paper sheet, green actions/focus and independent status/output roles. Original system fonts, density, input, keyboard and domain constraints remain.
- `src/theme.rs` remains the single theme owner. Named app-specific palette roles map through the existing Kit `Theme::update`, keeping Base and Component synchronized. Reading/input/list and popover surfaces remain opaque. Button hover/active have their own green tones and independent readable action ink; success/warning/danger/info remain distinct roles.
- `src/shell/mod.rs` wraps the existing workspace in one static scene and one opaque, non-scrolling 20px sheet with one shadow/outline. Existing page entities, action callbacks, stable IDs, observers and focus handles are retained. The sheet inset caps at 12 logical px and consumes only spare width/height above the original minimum, so the scene margin disappears at the minimum. No new navigation breakpoint, OS-window transparency, timer, shader or render I/O.
- Existing shell interaction tests now verify query preservation through theme/language/navigation and the new dark action palette. New behavior tests verify opaque fallback under normal/minimum/below-minimum geometry and contrast of all actual theme reading surfaces, including disabled muted-on-base and all action inks/states. These geometry inputs are unit tests, not the user-deferred native size/DPI matrix.

## Material finding and honest fallback

See [capability research](airy-glass-capabilities.md). The exact locked rendering surface has no in-app backdrop sampling API; window Blurred/Mica and shadow blur do not satisfy that contract. Current native sheet therefore stays alpha=1. There is no speculative Glass flag, renderer patch, dependency upgrade or fabricated system-preference integration. Output purple is a dedicated semantic role, not a substitute for selected/unknown/status state.

## Before identity

The old release was built successfully with `just desktop-build` (4m16s), then copied before any product edit to `outputs/desktop/airy-glass/2026-10-27/ccr-gpui-before.exe`:

`42e3aadb2fd6b676aafbe1d4882426c3bdf3733496c910645e1ddeb38612e1a8`

`before-release-receipt.json` links `before-environment.json` with OS/GPU/driver/refresh/power/Rust, all original desktop source hashes, lock and closed fixture hashes. The before executable is immutable and available for same-method later comparison. Main owns its actual native observation. Current S1/S2 data surfaces/timing hooks are incomplete; no S0/simple screenshot or concurrent-build CPU sample is promoted to the full baseline.

## Verification

Raw logs are preserved under `outputs/desktop/airy-glass/2026-10-27/`.

| Command | Result | Evidence |
| --- | --- | --- |
| Initial default `just desktop-build` | FAIL: existing just shim could not create target process | `before-release-build.log` |
| Same recipe with actual installed just1.59 + matching Rust1.98 on child PATH | PASS exit0; old release frozen | `before-release-build-direct.log`, `before-release-receipt.json` |
| `just desktop-fmt` | PASS exit0; separate repair before validation | `fmt-repair.log` |
| `just desktop-check` | PASS exit0, locked all targets | `check.log` |
| `just desktop-lint` | PASS exit0, locked all targets and `-D warnings` | `lint.log` |
| `just desktop-test` | Running at first write; no pass assumed | `test.log` |

No global tool configuration or dependency/lock changes were made. Existing actual executables were selected through process-only PATH. Other concurrent owners retain their domain/shared/native-runtime files; this visual implementer does not edit those.

## Remaining acceptance

New native light/dark appearance, focus/confirm/loading/error/disabled and profile flows belong to the coordinator's new-artifact capture, not this implementer's headless tests. Real glass is unavailable on the inspected public rendering API. Native preference changes, OS UIA/screen reader, IME composition, macOS/Linux, full S1/S2 presented-frame/input/CPU/memory comparisons and parent budgets remain independently `NOT_RUN` until actually evidenced. Window/DPI matrices retain user-deferred status. No previous screenshots or historical PASS entries were overwritten.
