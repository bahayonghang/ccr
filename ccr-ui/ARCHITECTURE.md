# CCR UI Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         User Browser                         │
│                    http://localhost:5173                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ HTTP Requests
                         │ (Proxied by Vite)
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React + Vite)                   │
│                                                               │
│  ┌────────────┐  ┌─────────────┐  ┌──────────────────────┐ │
│  │  Navbar    │  │  Sidebar    │  │  Main Content Area   │ │
│  └────────────┘  └─────────────┘  └──────────────────────┘ │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              React Router                            │   │
│  │  ┌────────────────┐     ┌───────────────────────┐  │   │
│  │  │ /configs       │     │ /commands             │  │   │
│  │  │ ConfigManage   │     │ CommandExecutor       │  │   │
│  │  └────────────────┘     └───────────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              API Client (Axios)                      │   │
│  │  • listConfigs()      • executeCommand()            │   │
│  │  • switchConfig()     • listCommands()              │   │
│  │  • validateConfigs()  • getCommandHelp()            │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ /api/* requests
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                 Backend (Actix Web + Rust)                   │
│                   http://localhost:8081                      │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                  HTTP Router                         │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │   │
│  │  │Config Handler│  │Command Handler│ │Sys Handler│ │   │
│  │  └──────────────┘  └──────────────┘  └───────────┘ │   │
│  └──────────────────────────────────────────────────────┘   │
│                         │                                     │
│                         │ Uses                                │
│                         ▼                                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │            CLI Executor (Tokio Process)              │   │
│  │  • Spawns 'ccr' subprocess                           │   │
│  │  • Captures stdout/stderr                            │   │
│  │  • Handles timeout (30s)                             │   │
│  │  • Returns CommandOutput                             │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Subprocess spawn
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   CCR CLI Binary                             │
│                 (Installed in PATH)                          │
│                                                               │
│  Executes commands like:                                     │
│  • ccr list                                                  │
│  • ccr switch <name>                                         │
│  • ccr validate                                              │
│  • ccr history                                               │
│  • etc.                                                      │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Reads/Writes
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    File System                               │
│                                                               │
│  ~/.ccs_config.toml          Configuration source            │
│  ~/.claude/settings.json     Claude Code settings            │
│  ~/.claude/ccr_history.json  Operation history               │
│  ~/.claude/backups/          Backup files                    │
└─────────────────────────────────────────────────────────────┘
```

## Request Flow Examples

### 1. Config Management Flow

```
User clicks "Switch to config"
         │
         ▼
┌─────────────────────┐
│  ConfigManagement   │  onClick(handleSwitch)
│  Component          │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│  API Client         │  switchConfig(configName)
└─────────┬───────────┘
          │
          │ POST /api/switch
          │ { "config_name": "anthropic" }
          ▼
┌─────────────────────┐
│  Config Handler     │  handle_switch_config()
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│  CLI Executor       │  execute_command(["switch", "anthropic"])
└─────────┬───────────┘
          │
          │ spawn subprocess
          ▼
┌─────────────────────┐
│  CCR CLI            │  ccr switch anthropic
└─────────┬───────────┘
          │
          │ Updates files
          ▼
┌─────────────────────┐
│  File System        │  ~/.claude/settings.json
└─────────────────────┘
          │
          │ Returns output
          ▼
      Success/Error
          │
          ▼
  Display notification to user
```

### 2. Command Execution Flow

```
User selects command "list" and clicks "Execute"
         │
         ▼
┌─────────────────────┐
│  CommandExecutor    │  handleExecute()
│  Component          │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│  API Client         │  executeCommand({ command: "list", args: [] })
└─────────┬───────────┘
          │
          │ POST /api/command/execute
          │ { "command": "list", "args": [] }
          ▼
┌─────────────────────┐
│  Command Handler    │  execute_command()
└─────────┬───────────┘
          │
          │ Validates command against allowed list
          ▼
┌─────────────────────┐
│  CLI Executor       │  execute_command(["list"])
└─────────┬───────────┘
          │
          │ spawn subprocess
          │ capture stdout/stderr
          │ measure duration
          ▼
┌─────────────────────┐
│  CCR CLI            │  ccr list
└─────────┬───────────┘
          │
          │ Returns output
          ▼
┌─────────────────────┐
│  CommandResponse    │  { success, output, error, exit_code, duration_ms }
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│  CommandExecutor    │  Display in terminal with syntax highlighting
│  Output Display     │
└─────────────────────┘
```

## Component Hierarchy

### Frontend Components

```
App (Router + Layout)
├── Navbar
├── Sidebar
│   └── Navigation Links
└── Main Content
    ├── ConfigManagement Page
    │   ├── Stats Panel
    │   ├── Action Buttons
    │   └── Config Cards (grid)
    │       └── ConfigCard
    │           ├── Config Info
    │           ├── Details
    │           └── Switch Button
    │
    └── CommandExecutor Page
        ├── Command List (sidebar)
        │   └── Command Buttons
        ├── Command Info Panel
        │   ├── Description
        │   ├── Usage
        │   └── Examples
        ├── Execution Panel
        │   ├── Args Input
        │   └── Execute Button
        └── Output Display
            ├── Status Bar
            │   ├── Exit Code
            │   └── Duration
            ├── Action Buttons
            │   ├── Copy
            │   └── Clear
            └── Syntax Highlighted Output
```

### Backend Modules

```
main.rs (Server Setup)
├── handlers/
│   ├── config.rs
│   │   ├── list_configs()
│   │   ├── switch_config()
│   │   ├── validate_configs()
│   │   ├── clean_backups()
│   │   ├── export_config()
│   │   ├── import_config()
│   │   ├── get_history()
│   │   └── CRUD stubs
│   ├── command.rs
│   │   ├── execute_command()
│   │   ├── list_commands()
│   │   └── get_command_help()
│   └── system.rs
│       └── get_system_info()
├── executor/
│   └── mod.rs
│       ├── execute_command()
│       └── execute_command_with_timeout()
└── models.rs
    ├── ApiResponse<T>
    ├── CommandRequest/Response
    ├── ConfigItem/ListResponse
    ├── HistoryEntry/Response
    └── SystemInfoResponse
```

## Data Flow

### Config Data Flow

```
CCR Config File          Backend API           Frontend State
                                               
~/.ccs_config.toml  →   GET /api/configs  →   useState<ConfigItem[]>
                         ↓                      ↓
                    ConfigListResponse      Display in UI
                         ↓                      ↓
                    POST /api/switch       User clicks switch
                         ↓                      ↓
Update settings.json ←  CCR CLI           Refresh configs
```

### Command Execution Data Flow

```
User Input           Backend Processing          Output Display
                                                
Command: "list"  →   POST /api/command/execute  →  Loading state
Args: []              ↓                             ↓
                 Validate command                   ↓
                      ↓                             ↓
                 Spawn subprocess                   ↓
                      ↓                             ↓
                 ccr list                           ↓
                      ↓                             ↓
                 Capture output                     ↓
                      ↓                             ↓
                 CommandResponse  →  Display with syntax highlighting
                 {                       Exit code: 0
                   success: true,        Duration: 45ms
                   output: "...",        Output: [colored text]
                   exit_code: 0,
                   duration_ms: 45
                 }
```

## Security Architecture

### Input Validation

```
Frontend                Backend                 CCR CLI
                                               
User Input  →  Basic validation  →  Command whitelist  →  Subprocess
                (required fields)   (13 allowed cmds)     (no shell)
                                    ↓
                             Argument sanitization
                             (array, not string)
                                    ↓
                             tokio::process::Command
                             (safe execution)
```

### CORS Configuration

```
Development:
Frontend (5173) ←→ Backend (8081)
                   ↑
                   CORS: allow_any_origin()

Production:
Frontend (static) → Backend (same origin)
                    ↑
                    CORS: restrictive
```

## Deployment Architecture

### Development

```
┌─────────────┐     ┌─────────────┐
│  Frontend   │     │  Backend    │
│  Vite Dev   │────→│  cargo run  │
│  Port 5173  │     │  Port 8081  │
└─────────────┘     └─────────────┘
      │                    │
      │                    │
      ▼                    ▼
  Hot Reload           Watch Mode
```

### Production

```
┌─────────────┐     ┌──────────────────┐
│  Frontend   │     │  Backend Binary  │
│  Static     │────→│  Release Build   │
│  Files      │     │                  │
└─────────────┘     └──────────────────┘
      │                     │
      ▼                     ▼
  Web Server          Standalone Binary
  (nginx/caddy)       ./ccr-ui-backend
```

## Technology Stack Details

### Backend Stack

```
Actix Web 4.9
    ├── actix-cors (CORS middleware)
    ├── actix-web-codegen (macros)
    └── tokio (async runtime)
        ├── tokio::process (subprocess)
        └── tokio::time (timeout)

Serialization
    ├── serde (traits)
    └── serde_json (JSON)

Utilities
    ├── chrono (timestamps)
    ├── anyhow (error handling)
    ├── thiserror (custom errors)
    ├── clap (CLI args)
    ├── whoami (system info)
    └── num_cpus (CPU info)
```

### Frontend Stack

```
React 18
    ├── react-dom (rendering)
    └── react-router-dom (routing)

TypeScript 5.6
    └── Type definitions

Build Tools
    ├── Vite 5.4 (dev server + bundler)
    ├── esbuild (fast compilation)
    └── rollup (production bundling)

Styling
    ├── Tailwind CSS 3.4
    ├── PostCSS
    └── Autoprefixer

HTTP & Icons
    ├── Axios 1.7 (API client)
    ├── Lucide React 0.454 (icons)
    └── React Syntax Highlighter 15.6
```

## Performance Characteristics

### Backend
- **Async I/O**: Non-blocking request handling
- **Subprocess Pool**: One subprocess per request
- **Timeout**: 30 seconds max per command
- **Memory**: ~10MB base + subprocess overhead

### Frontend
- **Bundle Size**: ~300KB (minified + gzipped)
- **Initial Load**: <1 second
- **Hot Reload**: <100ms
- **API Calls**: Axios with timeout

## Error Handling Flow

```
Error Source         Detection              Recovery
                                           
CCR Binary Not Found  →  Executor check  →  Display error message
                                           
Command Timeout      →  30s timeout     →  Kill process, notify user
                                           
Invalid Command      →  Whitelist check →  Return 400 error
                                           
Network Error        →  Axios intercept →  Show retry option
                                           
Parse Error          →  JSON validation →  Show raw output
```

---

**Note**: This architecture is designed to be:
- **Independent**: Doesn't modify existing CCR code
- **Safe**: All commands validated and executed in subprocesses
- **Extensible**: Easy to add new commands/features
- **Maintainable**: Clear separation of concerns

