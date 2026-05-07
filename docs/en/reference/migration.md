# Migration Guide

This page explains the move from the old global platform/profile routing model to the explicit Claude Runtime / Codex Runtime model.

## Command migration quick map

| Legacy command | Current path | Notes |
|---|---|---|
| `ccr switch <name>` | `ccr claude profile switch <name>` / `ccr codex profile switch <name>` | no more implicit platform inference |
| `ccr <name>` | same mapping | shortcut retired |
| `ccr platform switch <platform>` | no longer the main auth/profile path | use explicit profile/auth commands |
| `ccr platform current` | `ccr current` | inspect dual runtime state |
| `ccr platform profile ...` | `ccr claude profile ...` / `ccr codex profile ...` | explicit platform-scoped profile commands |

## Registry migration

- older files may still contain `default_platform` / `current_platform`
- CCR still reads them for backward compatibility
- the routing truth is now each platform entry's `current_profile`
