# Web API Reference

CCR includes a lightweight REST API server (launched via `ccr web`), listening on port **19527** by default.

> ⚠️ `ccr web` is the legacy lightweight API, primarily for scripting/CI programmatic access. For browser-based management, use `ccr ui` instead.

## Starting the Server

```bash
# Default (0.0.0.0:19527)
ccr web

# Custom port
ccr web --port 9000

# Don't auto-open browser
ccr web --no-browser
```

## API Endpoints Overview

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Home page (Web UI) |
| GET | `/api/configs` | List all configurations |
| POST | `/api/switch` | Switch configuration |
| POST | `/api/config` | Add configuration |
| PUT | `/api/config/:name` | Update configuration |
| DELETE | `/api/config/:name` | Delete configuration |
| PATCH | `/api/config/:name/enable` | Enable configuration |
| PATCH | `/api/config/:name/disable` | Disable configuration |
| GET | `/api/history` | Get operation history |
| POST | `/api/validate` | Validate configurations |
| POST | `/api/clean` | Clean backups |
| GET | `/api/settings` | Get settings |
| GET | `/api/settings/backups` | Get backup list |
| POST | `/api/settings/restore` | Restore settings |
| POST | `/api/export` | Export configurations |
| POST | `/api/import` | Import configurations |
| GET | `/api/sync/status` | Get sync status |
| POST | `/api/sync/config` | Set sync configuration |
| POST | `/api/sync/push` | Push to cloud |
| POST | `/api/sync/pull` | Pull from cloud |

## Endpoint Details

### Configuration Management

#### GET /api/configs

List all configurations.

```bash
curl http://localhost:19527/api/configs
```

**Response:**

```json
{
  "configs": [
    {
      "name": "anthropic",
      "description": "Anthropic Official API",
      "base_url": "https://api.anthropic.com",
      "auth_token": "sk-a...key",
      "model": "claude-sonnet-4-5-20250929",
      "small_fast_model": "claude-3-5-haiku-20241022",
      "is_current": true,
      "is_complete": true
    }
  ]
}
```

#### POST /api/switch

Switch the active configuration.

```bash
curl -X POST http://localhost:19527/api/switch \
  -H "Content-Type: application/json" \
  -d '{"config_name": "anyrouter"}'
```

**Request body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `config_name` | string | Yes | Target configuration name |

**Response:**

```json
{
  "success": true,
  "message": "Configuration switched successfully"
}
```

#### POST /api/config

Add a new configuration.

```bash
curl -X POST http://localhost:19527/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "name": "newconfig",
    "description": "New Configuration",
    "base_url": "https://api.example.com",
    "auth_token": "your-token",
    "model": "claude-sonnet-4-5-20250929"
  }'
```

#### PUT /api/config/:name

Update an existing configuration.

```bash
curl -X PUT http://localhost:19527/api/config/anthropic \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated Description",
    "auth_token": "new-token"
  }'
```

#### DELETE /api/config/:name

Delete a configuration.

```bash
curl -X DELETE http://localhost:19527/api/config/oldconfig
```

#### PATCH /api/config/:name/enable

Enable a configuration.

```bash
curl -X PATCH http://localhost:19527/api/config/myconfig/enable
```

#### PATCH /api/config/:name/disable

Disable a configuration.

```bash
curl -X PATCH http://localhost:19527/api/config/myconfig/disable
```

### History & Validation

#### GET /api/history

Get operation history.

```bash
curl http://localhost:19527/api/history?limit=20
```

**Query parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | number | 20 | Number of records to return |
| `type` | string | - | Filter by operation type |

#### POST /api/validate

Validate all configurations.

```bash
curl -X POST http://localhost:19527/api/validate
```

**Response:**

```json
{
  "valid": true,
  "errors": [],
  "warnings": []
}
```

#### POST /api/clean

Clean old backup files.

```bash
curl -X POST http://localhost:19527/api/clean \
  -H "Content-Type: application/json" \
  -d '{"days": 7, "dry_run": false}'
```

**Request body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `days` | number | Yes | Days to retain |
| `dry_run` | boolean | No | Preview only, don't delete |

### Settings Management

#### GET /api/settings

Get current settings.

```bash
curl http://localhost:19527/api/settings
```

#### GET /api/settings/backups

Get settings backup list.

```bash
curl http://localhost:19527/api/settings/backups
```

#### POST /api/settings/restore

Restore settings from a backup.

```bash
curl -X POST http://localhost:19527/api/settings/restore \
  -H "Content-Type: application/json" \
  -d '{"backup_name": "settings_20250101_120000.json"}'
```

### Import & Export

#### POST /api/export

Export configurations.

```bash
curl -X POST http://localhost:19527/api/export
```

#### POST /api/import

Import configurations.

```bash
curl -X POST http://localhost:19527/api/import \
  -H "Content-Type: application/json" \
  -d '{"merge": true, "backup": true}'
```

### Cloud Sync

#### GET /api/sync/status

Get WebDAV sync status.

```bash
curl http://localhost:19527/api/sync/status
```

#### POST /api/sync/config

Set WebDAV sync configuration.

```bash
curl -X POST http://localhost:19527/api/sync/config \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://dav.example.com/dav/",
    "username": "user",
    "password": "pass",
    "remote_path": "/ccr/"
  }'
```

#### POST /api/sync/push

Push local configurations to cloud.

```bash
curl -X POST http://localhost:19527/api/sync/push
```

#### POST /api/sync/pull

Pull configurations from cloud.

```bash
curl -X POST http://localhost:19527/api/sync/pull
```

## Security Notes

::: warning Security
The Web API listens on `0.0.0.0:19527` by default. For production use:
- Use `--host 127.0.0.1` to restrict to local access
- Use SSH port forwarding or VPN for remote access
- Use a reverse proxy (nginx/caddy) with HTTPS
- Restrict access IPs via firewall
:::

## Related Documentation

- [web command](/en/reference/commands/web) - Web server launch options
- [ui command](/en/reference/commands/ui) - Recommended full-stack web interface
- [Architecture](/en/reference/architecture) - CCR layered architecture
