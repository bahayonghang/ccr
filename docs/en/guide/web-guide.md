# Web Interface Guide

CCR provides two web interfaces for visual configuration management:

1. **CCR UI Application** (`ccr ui`, recommended) - Full-featured Vue.js 3 + Axum application for browser-based usage
2. **Lightweight Web API** (`ccr web`, legacy) - RESTful API server kept for programmatic access and backward compatibility

::: warning Deprecation Notice
`ccr ui` is the recommended way to use CCR in a web browser.
The `ccr web` command is a **legacy** lightweight API server that will be gradually deprecated.
Use `ccr web` only when you specifically need the HTTP API or the old built-in web interface; for all new web usage, prefer `ccr ui`.
:::

## Web API Server (Legacy)

Built-in lightweight Axum API server for programmatic access and legacy web interface usage.

### Quick Start

```bash
# Start with default settings
ccr web

# Custom port
ccr web --port 9000

# Don't open browser automatically
ccr web --no-browser

# Custom port without browser
ccr web -p 8080 --no-browser
```

### Features

- 🌐 **14 RESTful API Endpoints**
- ⚡ **Smart Port Binding** - Auto-fallback if port occupied
- 🔒 **Concurrency Safe** - Same file locking as CLI
- 📊 **JSON Responses** - Standard REST API format
- 🎨 **Modern UI** - Glassmorphism design

### API Endpoints

#### Configuration Management

```bash
# List all configurations
GET /api/configs

# Switch configuration
POST /api/switch
Body: {"name": "anthropic-official"}

# Create configuration
POST /api/config
Body: {profile configuration}

# Update configuration
POST /api/config/{name}
Body: {updated fields}

# Delete configuration
DELETE /api/config/{name}
```

#### Settings & Backups

```bash
# Get current settings
GET /api/settings

# List backups
GET /api/settings/backups

# Restore backup
POST /api/settings/restore
Body: {"backup_name": "settings_20250101_120000.json.bak"}
```

#### Validation & History

```bash
# Validate all configs
POST /api/validate

# Get operation history
GET /api/history
```

#### Import/Export

```bash
# Export configurations
POST /api/export
Body: {"no_secrets": true}

# Import configurations
POST /api/import
Body: {import data}
```

#### Maintenance

```bash
# Clean old backups
POST /api/clean
Body: {"days": 7, "dry_run": false}
```

#### System Information

```bash
# Get system info
GET /api/system
```

### Example Usage

**Using curl:**

```bash
# Start server
ccr web --port 8080 --no-browser

# List configurations
curl http://localhost:8080/api/configs

# Switch configuration
curl -X POST http://localhost:8080/api/switch \
  -H "Content-Type: application/json" \
  -d '{"name": "anthropic-official"}'

# Get current settings
curl http://localhost:8080/api/settings

# Validate
curl -X POST http://localhost:8080/api/validate
```

**Using JavaScript:**

```javascript
// List configurations
const response = await fetch('http://localhost:8080/api/configs');
const configs = await response.json();
console.log(configs);

// Switch configuration
await fetch('http://localhost:8080/api/switch', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ name: 'anthropic-official' })
});
```

## CCR UI Application

Full-featured Vue.js 3 + Axum web application with visual dashboard.

### Quick Start

```bash
# Auto-download and start (first time)
ccr ui

# Custom ports
ccr ui --port 3000 --backend-port 8081
```

### Features

- ⚛️ **Vue.js 3 Frontend** - Modern reactive UI
- 🦀 **Axum Backend** - High-performance Rust server
- 🖥️ **Visual Dashboard** - Interactive configuration management
- 💻 **Command Executor** - Execute CCR commands with visual output
- 📊 **Syntax Highlighting** - Terminal-style output
- ⚡ **Real-time Execution** - Async command execution
- 🔄 **Auto-Download** - Downloads from GitHub on first use

### First Time Setup

```bash
ccr ui

# Prompts:
# 💬 CCR UI is a full Vue.js 3 + Axum Web application
#    It will be downloaded to:
#    /home/user/.ccr/ccr-ui/
#
# ❓ Download CCR UI from GitHub now? [Y/n]: y
# 📦 Cloning repository...
# ⏳ Downloading (this may take a few minutes)...
# ✅ CCR UI download complete
#
# [Auto-checking dependencies and starting...]
```

### Three-Tier Detection

1. **Development Environment** - `ccr-ui/` in current/parent directory
2. **User Directory** - `~/.ccr/ccr-ui/` (daily use)
3. **GitHub Download** - Auto-download on first use

### UI Features

#### Dashboard Tab
- Configuration overview
- Quick profile switching
- Current status display
- Recent operations

#### Configurations Tab
- List all profiles
- Add/edit/delete profiles
- Profile validation
- Switch with one click

#### Commands Tab
- Execute any CCR command
- Real-time output display
- Command history
- Syntax highlighting

#### History Tab
- Operation timeline
- Filter by type/date
- Export history
- Detailed operation info

#### System Tab
- System information
- File paths
- CCR version
- Platform status

### Manual ccr-ui Management

```bash
cd ~/.ccr/ccr-ui

# Or if in development
cd ccr-ui

# Available commands
just s              # Start development
just i              # Install dependencies
just b              # Build production
just c              # Check code
just t              # Run tests
just f              # Format code
just quick-start    # Full setup
```

## CLI vs TUI vs Web vs CCR UI

| Feature | CLI | TUI | Web API (`ccr web`, legacy) | CCR UI (`ccr ui`, recommended) |
|---------|-----|-----|-----------------------------|---------------------------------|
| **Best For** | Scripting, automation | Quick visual mgmt | Programmatic access, legacy flows | Full visual experience, daily management |
| **Interface** | Command line | Terminal UI | REST API | Web browser |
| **Ports** | N/A | N/A | 8080 (default) | 3000 + 8081 (default) |
| **Dependencies** | None | None | None | Node.js (auto-checked) |
| **Profile Management** | ✅ | ✅ | ✅ | ✅ |
| **Visual Dashboard** | ❌ | Limited | ❌ | ✅ |
| **Command Execution** | ✅ | Limited | ✅ (API) | ✅ (UI) |
| **Real-time Output** | ✅ | ❌ | ✅ | ✅ |
| **Statistics** | ✅ | ✅ | ✅ | ✅ (Visual) |

### When to Use Each

**CLI** (`ccr <command>`):
- Scripting and automation
- CI/CD pipelines
- Quick operations
- SSH sessions

**TUI** (`ccr tui`):
- Quick visual review
- Keyboard-driven workflow
- No browser needed
- Terminal-only environment

**Web API** (`ccr web`, legacy):
- Programmatic integration
- Custom frontends
- Microservices architecture
- Headless automation

**CCR UI** (`ccr ui`, recommended):
- Visual management
- Learning CCR features
- Team demonstrations
- Comprehensive overview

## Integration Examples

### With Custom Frontend

```javascript
// React component example
import { useState, useEffect } from 'react';

function ConfigSwitcher() {
  const [configs, setConfigs] = useState([]);
  const API_BASE = 'http://localhost:8080/api';

  useEffect(() => {
    fetch(`${API_BASE}/configs`)
      .then(r => r.json())
      .then(setConfigs);
  }, []);

  const switchConfig = async (name) => {
    await fetch(`${API_BASE}/switch`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name })
    });
    alert(`Switched to ${name}`);
  };

  return (
    <div>
      {configs.map(cfg => (
        <button key={cfg.name} onClick={() => switchConfig(cfg.name)}>
          {cfg.name}
        </button>
      ))}
    </div>
  );
}
```

### With Python

```python
import requests

API_BASE = "http://localhost:8080/api"

# List configurations
configs = requests.get(f"{API_BASE}/configs").json()
print(configs)

# Switch configuration
response = requests.post(
    f"{API_BASE}/switch",
    json={"name": "anthropic-official"}
)
print(response.json())

# Get history
history = requests.get(f"{API_BASE}/history").json()
print(history)
```

### In CI/CD

```yaml
# GitHub Actions example
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Start CCR Web API
        run: ccr web --port 8080 --no-browser &
        
      - name: Switch to production config
        run: |
          curl -X POST http://localhost:8080/api/switch \
            -H "Content-Type: application/json" \
            -d '{"name": "production"}'
      
      - name: Run deployment
        run: ./deploy.sh
```

## Troubleshooting

### Port Already in Use

```bash
# Web API auto-fallback
ccr web  # Tries 8080, 8081, 8082...

# Or specify port manually
ccr web --port 9000

# CCR UI custom ports
ccr ui --port 5000 --backend-port 9001
```

### Cannot Access from Remote

Web API only binds to localhost by default for security.

**For development (not production):**
```bash
# Not implemented yet - use SSH port forwarding
ssh -L 8080:localhost:8080 user@remote-server
```

### Browser Not Opening

```bash
# Use --no-browser and open manually
ccr web --no-browser
# Then open: http://localhost:8080

ccr ui --no-browser
# Then open: http://localhost:3000
```

## Security Notes

- 🔒 Web interfaces run on localhost only
- 🔒 No authentication (local access only)
- 🔒 Same file locking as CLI
- ⚠️ Don't expose to internet without proxy/auth
- ⚠️ API tokens visible in responses (masked in display)

## See Also

- [Quick Start](./quick-start) - Get started with CCR
- [Command Reference](./commands/) - All CLI commands
- [Configuration Guide](./configuration) - Advanced config
- [Examples](./examples/) - Practical examples
