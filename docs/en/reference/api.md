# Web API Reference

This page only covers the legacy HTTP routes exposed by `ccr web`. For the main graphical product surface, see [UI Overview](/en/guide/ui-overview).

## Start the Service

```bash
ccr web
ccr web --host 127.0.0.1 --port 19527 --no-browser
```

Current defaults:

- Host: `127.0.0.1`
- Port: `19527`

## Route Groups

### Static Surface

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/` | Legacy HTML shell |
| GET | `/style.css` | Static stylesheet |
| GET | `/script.js` | Static script |

### Configuration

| Method | Path |
|--------|------|
| GET | `/api/configs` |
| POST | `/api/switch` |
| POST | `/api/config` |
| GET | `/api/config/{name}` |
| PUT | `/api/config/{name}` |
| DELETE | `/api/config/{name}` |
| PATCH | `/api/config/{name}/enable` |
| PATCH | `/api/config/{name}/disable` |
| POST | `/api/export` |
| POST | `/api/import` |

### Codex Profiles

| Method | Path |
|--------|------|
| GET | `/api/codex/profiles` |
| POST | `/api/codex/profiles` |
| PUT | `/api/codex/profiles/{name}` |
| DELETE | `/api/codex/profiles/{name}` |

### System and Settings

| Method | Path |
|--------|------|
| GET | `/api/history` |
| POST | `/api/validate` |
| POST | `/api/clean` |
| GET | `/api/settings` |
| GET | `/api/settings/backups` |
| POST | `/api/settings/restore` |
| GET | `/api/system` |
| POST | `/api/reload` |

### Stats and Cost

| Method | Path |
|--------|------|
| GET | `/api/stats/provider-usage` |
| GET | `/api/stats/cost/summary` |
| GET | `/api/stats/cost/details` |
| GET | `/api/stats/cost/export` |
| GET | `/api/stats/cost/by-model` |
| GET | `/api/budget/status` |
| POST | `/api/budget/set` |
| POST | `/api/budget/reset` |
| GET | `/api/pricing/list` |
| POST | `/api/pricing/set` |
| DELETE | `/api/pricing/remove/{model}` |
| POST | `/api/pricing/reset` |

### Platforms and Sync

| Method | Path |
|--------|------|
| GET | `/api/platforms` |
| POST | `/api/platforms/switch` |
| GET | `/api/sync/status` |
| POST | `/api/sync/config` |
| POST | `/api/sync/push` |
| POST | `/api/sync/pull` |

## Minimal Examples

### List configurations

```bash
curl http://127.0.0.1:19527/api/configs
```

### Switch platform

```bash
curl -X POST http://127.0.0.1:19527/api/platforms/switch
```

### Read system info

```bash
curl http://127.0.0.1:19527/api/system
```

## Notes

- This page intentionally does not document nonexistent `/api/provider-health/*` routes.
- The route source of truth is `crates/ccr/src/web/server.rs`.
- If you need page-level UX guidance, go back to [UI Overview](/en/guide/ui-overview); if you need startup semantics, see [`ccr web`](/en/reference/commands/web).
