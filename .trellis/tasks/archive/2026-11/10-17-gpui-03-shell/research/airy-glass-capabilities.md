# GPUI airy-glass capability preflight

Date: 2026-10-27. Exact locked stack: `gpui-kit` / `gpui-component` / `gpui-base` 0.7.3, normal `gpui-pre` / Windows backend 0.3.9. No dependency, vendor or lock change. Source root below is `C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.

## Boundary and conclusion

The existing shell lacks the approved mint/lavender field, single paper sheet and semantic green action palette. These live in `ccr-desktop/DESIGN.md`, `src/theme.rs` and the existing `Shell::render`; the current page entities, domain services, keyboard owners and fixed row sizes remain intact. Meaningful tests belong in the existing theme/shell targets. No business API, persistence, OS wallpaper material, dependency upgrade or renderer patch is part of this visual change.

The inspected locked renderer exposes no in-app backdrop-sampling blur API. Therefore the deliverable on this stack is the approved **opaque fallback**, not an imitation glass effect. `WindowBackgroundAppearance::Blurred` is a different OS-window capability and must not be substituted. Real glass requires a separately reviewed renderer/dependency change; this preflight does not authorize one.

## Exact source evidence

| Source | Finding |
| --- | --- |
| `gpui-pre-0.3.9/src/color.rs:860-898` | `linear_gradient(angle, from, to)` and `linear_color_stop` provide the static two-stop scene. This is background paint, not blur. |
| `gpui-pre-0.3.9/src/style.rs:349-390` | `BoxShadow` has offset/color/blur/spread fields. Its blur concerns a shadow, not background sampling. |
| `gpui-pre-0.3.9/src/scene.rs:84-157`, `:199-220` | Primitive insertion/batching covers shadow, quad, path, underline, sprite and surface; no backdrop filter primitive or sampling control. |
| `gpui-pre-0.3.9/src/platform.rs:2764-2786` | Window background modes describe OS-window opacity/transparency/blur and Mica. They do not define per-element backdrop blur. |
| `gpui-pre-windows-0.3.9/src/window.rs:915-941` | Blurred maps to window composition attributes; Mica maps to DWM. These are expressly outside the approved in-app sheet contract. |
| `gpui-pre-0.3.9/src/window.rs:2419`, `src/app/context.rs:609` | Other `blur` matches concern keyboard focus, not rendering. |
| `gpui-component-0.7.3/src/theme/mod.rs:278-337`, `:354-399` | Keep one Theme owner, resolve System through native appearance, apply palette in a separate `Theme::update` after mode changes so Base and Component synchronize. |

Searches of exact GPUI, Windows/platform, Kit and Component sources found no exposed reduced-transparency/high-contrast preference API. This is an API-surface finding, not proof that the operating systems lack those settings. The safe production choice is opaque; native system-setting read/change verification remains `NOT_RUN`. No settings are changed during preflight.

| OS | In-app blur on inspected public stack | Preference API / native system change | Native solid | Native glass |
| --- | --- | --- | --- | --- |
| Windows | `UNSUPPORTED` by inspected rendering surface | No exposed API found / `NOT_RUN` | `NOT_RUN` | `UNSUPPORTED` |
| macOS | No common public API found; native backend `NOT_RUN` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| Linux | No common public API found; native backend `NOT_RUN` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |

An opaque sheet inherently avoids transparency even when the system preference cannot be observed. This does not claim system-setting integration passed. Window sizes/DPI remain user-deferred.

## Before-build environment and startup diagnosis

The default `just` shim failed to create its configured `current/just.exe` process. The existing real `C:/Users/lyh/scoop/apps/just/1.59.0/just.exe` works. The retry uses only child-process PATH prefixes for that executable and matching Rust 1.98 toolchain; no installation or global config mutation. Original failure and exact locked release retry logs remain in `outputs/desktop/airy-glass/2026-10-27/`.

Before the build, exact GPUI registry source directories were absent although the old debug artifact referenced them. Main's old-debug fixture launch failed before creating a window: DirectX recovery -> `Recreating GPU state for DirectWrite` -> `os error 3`; see `outputs/desktop/profiles/native/resume-20261027-core/stderr.log` and receipt. The locked build restored those sources through normal Cargo resolution.

Root-cause chain: `gpui-pre-windows-0.3.9/src/direct_write.rs:85-153` creates color-glyph vertex/pixel shaders; `:1295-1300` wraps this in GPU recovery. In **debug builds**, `directx_renderer.rs:1801-1867` calls `canonicalize()` on compile-time `CARGO_MANIFEST_DIR/src/color_text_raster.hlsl`, then `D3DCompileFromFile`. Missing source directories can therefore cause this exact path-not-found diagnostic. Release uses embedded shader bytes (`:1868-1869`). Main confirmed the unchanged debug hash opens the correct native fixture after source restoration (PID 31156, actual matching window observed). This supports the missing-development-source diagnosis, not a driver failure or shipped visual regression. No vendor patch was needed.

Before environment/source/closed-fixture hashes were captured in `outputs/desktop/airy-glass/2026-10-27/before-environment.json`; this records OS/GPU/driver/refresh/power/Rust and lock identity. It is not input/frame timing or S1/S2 evidence.

## Evidence boundaries

- Pre-change release build and preservation: pending at first write; coordinator must consume the final receipt before product edits.
- S1/S2 performance, presented frame timing, cross-platform native output, OS UIA/IME: `NOT_RUN` here. A release build alone does not satisfy a baseline.
- No native app automation is performed by this implementer; main owns real-window observation and acceptance.
