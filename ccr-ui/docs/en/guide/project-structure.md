# Project Structure

This document provides a detailed introduction to the overall structure of the CCR UI project and the purpose of each directory.

## 📁 Overall Project Structure

```
ccr-ui/
├── backend/                    # Rust backend service
│   ├── src/                   # Source code
│   │   ├── main.rs           # Application entry point
│   │   ├── config_reader.rs  # CCR config file reading
│   │   ├── models.rs         # Data model definitions
│   │   ├── claude_config_manager.rs  # Claude config management
│   │   ├── markdown_manager.rs       # Markdown file management
│   │   ├── plugins_manager.rs        # Plugin management
│   │   ├── settings_manager.rs       # Settings management
│   │   ├── handlers/         # HTTP request handlers
│   │   │   ├── mod.rs
│   │   │   ├── config.rs     # Config-related APIs
│   │   │   ├── command.rs    # Command execution APIs
│   │   │   ├── system.rs     # System info APIs
│   │   │   ├── version.rs    # Version management APIs
│   │   │   ├── mcp.rs        # MCP server management
│   │   │   ├── agents.rs     # Agent management
│   │   │   ├── plugins.rs    # Plugin management
│   │   │   └── slash_commands.rs # Slash command management
│   │   └── executor/         # Command executor
│   │       ├── mod.rs
│   │       └── cli_executor.rs # CLI command execution
│   ├── Cargo.toml            # Rust project config
│   ├── examples/             # Example config files
│   │   └── settings.example.json
│   └── README.md             # Backend documentation
├── frontend/                  # Vue 3 + Vite frontend application
│   ├── public/               # Static assets
│   │   └── vite.svg         # App icon
│   ├── src/                 # Source code
│   │   ├── main.ts          # App entry point
│   │   ├── App.vue          # Root component
│   │   ├── views/           # Page components
│   │   │   ├── HomeView.vue        # Dashboard homepage
│   │   │   ├── ConfigsView.vue     # Config management
│   │   │   ├── CommandsView.vue    # Command execution
│   │   │   ├── McpView.vue         # MCP server management
│   │   │   ├── AgentsView.vue      # Agent management
│   │   │   ├── PluginsView.vue     # Plugin management
│   │   │   ├── SlashCommandsView.vue # Slash command management
│   │   │   ├── SyncView.vue        # Cloud sync
│   │   │   ├── StatsView.vue       # Statistics analysis
│   │   │   └── ConverterView.vue   # Config converter
│   │   ├── components/      # Reusable components
│   │   │   ├── MainLayout.vue      # Main layout
│   │   │   ├── Navbar.vue          # Navigation bar
│   │   │   ├── CollapsibleSidebar.vue # Sidebar
│   │   │   ├── RightSidebar.vue    # Right sidebar
│   │   │   ├── StatusHeader.vue    # Status header
│   │   │   ├── HistoryList.vue     # History list
│   │   │   ├── VersionManager.vue  # Version manager
│   │   │   ├── ThemeToggle.vue     # Theme toggle
│   │   │   ├── UpdateModal.vue     # Update dialog
│   │   │   └── ConfigCard.vue      # Config card
│   │   ├── router/          # Vue Router config
│   │   │   └── index.ts
│   │   ├── stores/          # Pinia state management
│   │   │   ├── config.ts
│   │   │   ├── theme.ts
│   │   │   └── system.ts
│   │   ├── api/             # API client
│   │   │   └── client.ts
│   │   ├── types/           # TypeScript type definitions
│   │   │   └── index.ts
│   │   ├── styles/          # Global styles
│   │   │   └── main.css
│   │   └── utils/           # Utility functions
│   │       └── helpers.ts
│   ├── package.json        # Node.js project config
│   ├── vite.config.ts      # Vite config
│   ├── tailwind.config.js  # Tailwind CSS config
│   ├── postcss.config.js   # PostCSS config
│   ├── tsconfig.json       # TypeScript config
│   ├── .eslintrc.cjs       # ESLint config
│   └── README.md           # Frontend documentation
├── docs/                   # Project documentation
│   ├── .vitepress/         # VitePress config
│   │   └── config.ts
│   ├── backend/            # Backend docs
│   │   ├── api.md         # API reference
│   │   └── architecture.md # Architecture design
│   ├── frontend/           # Frontend docs
│   │   ├── api.md         # API calls
│   │   ├── development.md # Development guide
│   │   └── overview.md    # Frontend overview
│   ├── guide/              # User guide
│   │   ├── getting-started.md # Getting started
│   │   └── project-structure.md # Project structure
│   ├── index.md            # Docs homepage
│   ├── contributing.md     # Contributing guide
│   ├── faq.md             # FAQ
│   ├── package.json       # Docs build config
│   └── public/            # Docs static assets
│       ├── favicon.ico
│       └── logo.svg
├── clean-logs.sh           # Log cleanup script
├── justfile               # Just task config
├── .gitignore             # Git ignore file
├── ARCHITECTURE.md        # Architecture docs
└── README.md              # Project README
```

## 🔧 Backend Structure Details

### Core Files

| File | Purpose | Description |
|------|---------|-------------|
| `main.rs` | App entry | Start HTTP server, configure routes and middleware |
| `models.rs` | Data models | Define request/response data structures |
| `config_reader.rs` | Config reading | Read and parse config files |

### Handlers Module (handlers/)

```
handlers/
├── mod.rs          # Module exports
├── config.rs       # Config management APIs
│   ├── GET /api/configs           # Get config list
│   ├── POST /api/configs/switch   # Switch config
│   └── POST /api/configs/validate # Validate config
├── command.rs      # Command execution APIs
│   ├── POST /api/commands/execute # Execute command
│   └── GET /api/commands/list     # Get command list
└── system.rs       # System info APIs
    └── GET /api/system/info       # Get system info
```

### Executor Module (executor/)

```
executor/
├── mod.rs          # Module exports
└── cli_executor.rs # CLI command executor
    ├── execute_ccr_command()      # Execute CCR command
    ├── execute_arbitrary_command() # Execute arbitrary command
    └── Timeout handling, error handling, etc.
```

## ⚛️ Frontend Structure Details (Vue 3 + Vite)

### Vue Application Structure

```
src/
├── main.ts                # App entry
├── App.vue                # Root component
└── views/                 # Page components
```

### Component Architecture

```
src/components/
├── layout/               # Layout components
│   ├── Navbar.vue       # Top navigation bar
│   └── CollapsibleSidebar.vue # Collapsible sidebar
├── sidebar/              # Sidebar components
│   ├── LeftSidebar.vue  # Left sidebar
│   └── RightSidebar.vue # Right sidebar
├── history/              # History components
│   └── HistoryList.vue  # History list
└── ui/                   # Basic UI components
    └── ThemeToggle.vue  # Theme toggle button
```

### Libraries and Tools

```
src/lib/
├── api/                  # API client
│   └── client.ts        # HTTP client config
│       ├── Axios instance config
│       ├── Request/response interceptors
│       ├── Error handling
│       └── API route proxy
└── types/                # TypeScript type definitions
    └── index.ts         # Common type definitions
```

### Routes and Pages

Vue Router route configuration:

| Route Path | Component | Description |
|-----------|-----------|-------------|
| `/` | `HomeView.vue` | Dashboard homepage |
| `/configs` | `ConfigsView.vue` | Config management |
| `/commands` | `CommandsView.vue` | Command execution |
| `/mcp` | `McpView.vue` | MCP server management |
| `/agents` | `AgentsView.vue` | Agent management |
| `/plugins` | `PluginsView.vue` | Plugin management |
| `/sync` | `SyncView.vue` | Cloud sync |
| `/stats` | `StatsView.vue` | Statistics analysis |

## 📚 Documentation Structure

### VitePress Configuration

```
docs/.vitepress/
├── config.ts          # Site config
│   ├── Navigation config
│   ├── Sidebar config
│   ├── Theme config
│   └── Search config
└── theme/             # Custom theme (optional)
    ├── index.ts       # Theme entry
    └── components/    # Custom components
```

### Documentation Content

```
docs/
├── guide/             # User guide
│   ├── getting-started.md    # Getting started
│   ├── project-structure.md # Project structure
│   ├── development-setup.md # Development setup
│   └── build-deploy.md      # Build and deploy
├── frontend/          # Frontend docs
│   ├── overview.md           # Overview
│   ├── tech-stack.md        # Tech stack
│   ├── development.md       # Development guide
│   ├── components.md        # Component docs
│   ├── api.md              # API reference
│   ├── styling.md          # Styling guide
│   └── testing.md          # Testing guide
├── backend/           # Backend docs
│   ├── architecture.md      # Architecture design
│   ├── tech-stack.md       # Tech stack
│   ├── development.md      # Development guide
│   ├── api.md             # API docs
│   ├── models.md          # Data models
│   ├── error-handling.md  # Error handling
│   └── deployment.md      # Deployment guide
├── contributing.md    # Contributing guide
├── changelog.md       # Changelog
├── faq.md            # FAQ
└── index.md          # Docs homepage
```

## 🛠️ Configuration Files

### Backend Configuration

#### Cargo.toml
```toml
[package]
name = "ccr-ui-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.9"      # Web framework
tokio = "1.42"         # Async runtime
serde = "1.0"          # Serialization
anyhow = "1.0"         # Error handling
# ... other dependencies
```

### Frontend Configuration

#### package.json
```json
{
  "name": "ccr-ui-frontend",
  "version": "0.1.0",
  "dependencies": {
    "vue": "^3.5.22",
    "vue-router": "^4.4.5",
    "vite": "^7.1.11",
    "typescript": "^5.7.3"
  },
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  }
}
```

## 🔄 Data Flow

### Request Flow

```
User Action → Frontend Component → API Client → Backend Handler → CLI Executor → CCR Command
                                                                    ↓
User Interface ← Frontend Component ← API Response ← Backend Response ← Command Result ← Command Output
```

### File Relationships

```
Frontend Page Components → Use → UI Components
     ↓
Call API Services → Through → HTTP Client
     ↓
Request Backend APIs → Handler → Executor → CCR CLI
```

## 📦 Build Artifacts

### Frontend Build

```
frontend/dist/
├── assets/          # Built assets
│   ├── *.js        # JavaScript files
│   ├── *.css       # CSS files
│   └── *.svg       # SVG icons
└── index.html      # Entry HTML
```

### Backend Build

```
backend/target/release/
└── ccr-ui-backend    # Executable
```

## 🚀 Deployment Structure

### Development Environment

```
Development:
├── Frontend dev server (localhost:5173) - Vite + Vue 3
├── Backend dev server (localhost:8081) - Axum (Rust)
└── Docs dev server (localhost:5174) - VitePress
```

### Production Environment

```
Production:
├── Static file server (Nginx/Caddy) - Frontend SPA
├── Backend API server (Rust binary)
└── Docs site (static deployment)
```

## 📋 Development Workflow

### 1. New Feature Development

```
1. Define data models in backend/src/models.rs
2. Add API handlers in backend/src/handlers/
3. Define frontend types in frontend/src/types/
4. Add API client in frontend/src/api/
5. Develop UI components in frontend/src/components/
6. Integrate page functionality in frontend/src/views/
7. Update related docs in docs/
```

### 2. Testing Process

```
1. Backend unit tests: cargo test
2. Frontend unit tests: npm test
3. Integration tests: Start full app for testing
4. Docs tests: Verify docs build and links
```

### 3. Deployment Process

```
1. Backend build: cargo build --release
2. Frontend build: npm run build
3. Docs build: npm run docs:build
4. Deploy to target environment
```

This project structure follows best practices for frontend-backend separation, with clear responsibilities for each module, making it easy to develop, test, and maintain.

