# CCR UI - Full-Stack Web Application

A modern full-stack web application for managing CCR (Claude Code Configuration Switcher) with a Vue 3 frontend and Rust Axum backend.

## ⚡ 超快速开始

```bash
cd ccr-ui
just s    # 启动开发环境（就这么简单！）
```

**第一次使用？** 运行 `just quick-start` 自动完成所有设置。

**不知道用什么命令？** 直接运行 `just` 查看帮助！

---

## Features

- **Config Management**: View, switch, and validate CCR configurations
- **Command Executor**: Execute any CCR command with a visual interface
- **Multi-Folder WebDAV Sync**:
  - Independent folder management (add/remove/enable/disable)
  - Individual folder operations (push/pull/status)
  - Batch operations for all enabled folders
  - Support for multiple sync destinations (Nutstore, Nextcloud, ownCloud)
  - Smart filtering (excludes backups, locks, temp files)
- **Real-time Output**: See command output in a terminal-style display
- **Multi-page Navigation**: Easy switching between different functionalities

## Architecture

```
ccr-ui/
├── backend/           # Rust Axum server
│   ├── src/
│   │   ├── main.rs    # Server entry point
│   │   ├── executor/  # CLI subprocess executor
│   │   ├── handlers/  # API route handlers
│   │   └── models/    # Request/response models
│   └── Cargo.toml
├── frontend/          # Vue 3 + Vite application (TypeScript)
│   ├── src/
│   │   ├── App.vue    # Root Vue component
│   │   ├── main.ts    # Application entry point
│   │   ├── components/# Reusable Vue components
│   │   ├── views/     # Page components
│   │   ├── router/    # Vue Router configuration
│   │   ├── store/     # Pinia state management
│   │   └── api/       # API client & utilities
│   ├── package.json
│   └── vite.config.ts
└── README.md
```

## Prerequisites

### Required
- **Rust 1.70+** (with Cargo)
- **Node.js 18+** (with npm)
- **CCR** installed and available in PATH

### Optional but Recommended
- **Just** - Command runner for simplified workflow
  - Install: `cargo install just` or `brew install just`
  - Repo: https://github.com/casey/just

## Installation

### 🚀 Quick Install (Using Just - Recommended)

```bash
cd ccr-ui

# Check prerequisites
just check-prereqs

# Install all dependencies
just install
```

### 📦 Manual Installation

#### 1. Install Backend Dependencies

```bash
cd backend
cargo build --release
```

#### 2. Install Frontend Dependencies

```bash
cd frontend
npm install
```

## Development

### 🚀 Quick Start (Using Just)

```bash
cd ccr-ui

# 🌟 超简单：只需一个字母！
just s               # 启动开发环境（s = start）

# 或者一键全自动（检查 + 安装 + 开发）
just quick-start

# 查看帮助（不知道用什么命令时）
just                 # 显示常用命令帮助
just --list          # 显示所有40+命令
```

**简化命令速查：**
- `just s` = 启动开发
- `just i` = 安装依赖  
- `just b` = 构建生产版本
- `just c` = 检查代码
- `just t` = 运行测试
- `just f` = 格式化代码

### 📝 Manual Start

#### Start Backend Server

```bash
cd backend
cargo run

# Or with custom port
cargo run -- --port 8081

# Or using Just
just dev-backend
```

The backend will start on `http://127.0.0.1:8081` by default.

#### Start Frontend Dev Server

```bash
cd frontend
npm run dev

# Or using Just
just dev-frontend
```

The frontend will start on `http://localhost:5173` with Vite's fast hot module replacement.

## Production Build

### 🏗️ Using Just (Recommended)

```bash
cd ccr-ui

# Build both backend and frontend
just build

# Full CI workflow (check + test + build)
just ci

# Run production backend
just run-prod
```

### 📦 Manual Build

#### Build Backend

```bash
cd backend
cargo build --release
```

The binary will be at `target/release/ccr-ui-backend`.

#### Build Frontend

```bash
cd frontend
npm run build
```

The built files will be in `frontend/dist/`.

## 🖥️ Tauri Desktop Application

CCR UI also supports running as a native desktop application using Tauri 2.0! This provides better performance and native OS integration.

### Prerequisites for Desktop

In addition to the web version requirements:
- **Rust 1.70+** (already required for backend)
- **System Dependencies**:
  - **Linux**: `libwebkit2gtk-4.0-dev`, `build-essential`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio C++ Build Tools

### 🚀 Quick Start (Desktop)

```bash
cd ccr-ui

# Start Tauri development mode (opens desktop window)
just tauri-dev

# Build desktop application
just tauri-build

# Check Tauri environment
just tauri-check
```

### Available Tauri Commands

| Command | Description |
|---------|-------------|
| `just tauri-dev` | Start Tauri development mode (desktop app) |
| `just tauri-build` | Build production desktop app |
| `just tauri-build-debug` | Build debug version (faster, with symbols) |
| `just tauri-check` | Check Tauri environment and configuration |
| `just tauri-check-all` | Full check (TypeScript + Rust) |
| `just tauri-check-rust` | Check Tauri Rust code only |
| `just tauri-clippy` | Run Rust linter (Clippy) |
| `just tauri-fmt` | Format Tauri Rust code |
| `just tauri-test` | Run Tauri tests |
| `just tauri-clean` | Clean Tauri build artifacts |

### Build Artifacts

After running `just tauri-build`, you'll find platform-specific installers:

**Linux**:
- `.deb` - Debian/Ubuntu package
- `.AppImage` - Universal Linux package

**macOS**:
- `.dmg` - Disk image installer
- `.app` - Application bundle

**Windows**:
- `.msi` - Windows installer
- `.exe` - Standalone executable

All artifacts are located in: `frontend/src-tauri/target/release/bundle/`

### Desktop vs Web Mode

The application automatically detects the runtime environment:
- **Desktop mode**: Uses Tauri invoke (< 1ms, 50x faster)
- **Web mode**: Uses HTTP API (20-50ms)

Both modes share the same Vue.js frontend with automatic backend switching!

For detailed Tauri documentation, see [`frontend/README.md`](frontend/README.md) and [`frontend/README.dev.md`](frontend/README.dev.md).

## API Endpoints

### Config Management
- `GET /api/configs` - List all configurations
- `POST /api/switch` - Switch to a configuration
- `POST /api/validate` - Validate configurations
- `POST /api/clean` - Clean old backups
- `POST /api/export` - Export configurations
- `POST /api/import` - Import configurations
- `GET /api/history` - Get operation history
- `GET /api/system` - Get system information

### Command Execution
- `POST /api/command/execute` - Execute a CCR command
- `GET /api/command/list` - List available commands
- `GET /api/command/help/{command}` - Get help for a command

### WebDAV Sync

#### Basic Sync Operations
- `GET /api/sync/status` - Get sync configuration status
- `POST /api/sync/push` - Upload config to cloud (body: `{force: boolean}`)
- `POST /api/sync/pull` - Download config from cloud (body: `{force: boolean}`)
- `GET /api/sync/info` - Get sync feature information
- `POST /api/sync/config` - Configure WebDAV sync (not supported in web API)

#### Multi-Folder Management
- `GET /api/sync/folders` - List all registered sync folders
- `POST /api/sync/folders` - Add new sync folder (body: `{name, local_path, remote_path?, description?}`)
- `DELETE /api/sync/folders/:name` - Remove folder registration (keeps local files)
- `GET /api/sync/folders/:name` - Get folder details
- `PUT /api/sync/folders/:name/enable` - Enable folder for sync
- `PUT /api/sync/folders/:name/disable` - Disable folder (keeps config)

#### Folder-Specific Operations
- `POST /api/sync/folders/:name/push` - Push specific folder (body: `{force: boolean}`)
- `POST /api/sync/folders/:name/pull` - Pull specific folder (body: `{force: boolean}`)
- `GET /api/sync/folders/:name/status` - Get folder sync status

#### Batch Operations
- `POST /api/sync/all/push` - Push all enabled folders (body: `{force: boolean}`)
- `POST /api/sync/all/pull` - Pull all enabled folders (body: `{force: boolean}`)
- `GET /api/sync/all/status` - Get status of all folders

## Usage

### Config Management Page

1. View all available configurations
2. See which configuration is currently active
3. Switch between configurations with one click
4. Validate all configurations

### Command Executor Page

1. Select a command from the list (13 available commands)
2. Enter optional arguments
3. Execute the command
4. View output in real-time with syntax highlighting
5. Copy output or clear it

### Multi-Folder Sync Page

The sync page provides comprehensive WebDAV synchronization with multi-folder management:

#### WebDAV Configuration
- View current WebDAV configuration status
- Server URL, username, and remote path
- Connection test status

#### Folder Management
**Add New Folder**:
1. Enter folder name (e.g., `claude`, `gemini`)
2. Specify local path (e.g., `~/.claude`, `~/.gemini`)
3. Optional: Set custom remote path
4. Optional: Add description
5. Click "Add Folder" to register

**Manage Folders**:
- **Enable/Disable**: Toggle folder participation in batch operations
- **Push**: Upload individual folder to cloud
- **Pull**: Download individual folder from cloud
- **Status**: Check folder sync status
- **Remove**: Delete folder registration (keeps local files)

#### Batch Operations
- **Push All**: Upload all enabled folders in one operation
- **Pull All**: Download all enabled folders in one operation
- **Status All**: View status of all registered folders

#### Smart Filtering
Automatic exclusion of:
- Backup directories (`backups/`)
- Lock files (`.locks/`)
- Temporary files (`*.tmp`, `*.bak`)
- System files (`.DS_Store`, `Thumbs.db`)

#### Supported Services
- ✅ Nutstore (坚果云)
- ✅ Nextcloud
- ✅ ownCloud
- ✅ Any standard WebDAV server

#### Example Workflow
1. Configure WebDAV connection (use CLI: `ccr sync config`)
2. Add folders: claude, gemini, custom configs
3. Enable folders you want to sync
4. Use batch operations for quick sync across all folders
5. Or use individual operations for selective sync

## Available Commands

- `init` - Initialize configuration file
- `list` - List all configurations
- `current` - Show current configuration
- `switch` - Switch to a configuration
- `validate` - Validate configurations
- `optimize` - Optimize configuration file
- `history` - Show operation history
- `clean` - Clean old backups
- `export` - Export configurations
- `import` - Import configurations
- `update` - Update CCR
- `version` - Show version information

## Technologies

### Backend
- **Axum 0.7** - Fast async web framework for Rust
- **Tokio 1.42** - Async runtime
- **Serde** - Serialization
- **Chrono** - Date/time handling

### Frontend
- **Vue 3.5** - Progressive JavaScript framework
- **Vite 7.1** - Next generation frontend tooling
- **Vue Router 4.4** - Official router for Vue.js
- **Pinia 2.2** - Vue state management
- **TypeScript 5.7** - Type safety
- **Tailwind CSS 3.4** - Styling
- **Axios 1.7** - HTTP client
- **Lucide Vue Next** - Icons
- **ANSI to HTML** - Terminal output rendering

## Configuration

### Backend Port

Default: `8081`

Change via command line:
```bash
cargo run -- --port 3000
```

### Frontend API Configuration

The frontend connects to the backend API. Configure in `src/lib/api/config.ts` if needed:

```typescript
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8081';
```

For production, set the `NEXT_PUBLIC_API_URL` environment variable.

## Security Notes

- The backend executes CCR commands as subprocesses
- All commands are validated against an allowed list
- No shell injection vulnerabilities (proper argument passing)
- CORS is enabled for local development
- Recommended to run on localhost only

## Troubleshooting

### Backend won't start
- Ensure CCR is installed: `ccr version`
- Check if port 8081 is available
- View logs for error messages

### Frontend can't connect to backend
- Ensure backend is running on port 8081
- Check browser console for CORS errors
- Verify API configuration in `src/lib/api/config.ts`

### Commands fail to execute
- Ensure CCR is in your PATH
- Check that CCR version is compatible
- View command output for specific errors

## Development Tips

### Hot Reload
Both frontend and backend support hot reload during development:
- Frontend: Next.js with Turbopack auto-reloads on file changes
- Backend: Use `just watch-backend` for auto-restart (requires `cargo-watch`)

### Debugging
- Frontend: Use browser DevTools
- Backend: Set `RUST_LOG=debug` for verbose logging

### Testing

**Using Just:**
```bash
just test          # Run all tests
just check         # Check code without building
just fmt           # Format code
just clippy        # Run Clippy linter
```

**Manual:**
```bash
# Backend tests
cd backend
cargo test

# Frontend lint
cd frontend
npm run lint
```

### Common Just Commands

**🌟 超简化版（推荐）：**
```bash
just        # 显示帮助
just s      # 启动开发（最常用！）
just i      # 安装依赖
just b      # 构建生产版本
just c      # 检查代码
just t      # 运行测试
just f      # 格式化代码
```

**📋 完整版：**
```bash
just help            # 显示帮助
just --list          # 显示所有命令（40+）
just info            # 显示项目信息
just check-prereqs   # 检查前置条件
just install         # 安装依赖
just dev             # 启动开发环境
just build           # 构建生产版本
just test            # 运行测试
just fmt             # 格式化代码
just clean           # 清理构建产物
just update          # 更新依赖
just quick-start     # 一键启动（检查+安装+开发）
```

**💡 速查表：**
| 简化 | 完整 | 说明 |
|-----|-----|------|
| `s` | `dev` | 启动开发 |
| `i` | `install` | 安装依赖 |
| `b` | `build` | 构建 |
| `c` | `check` | 检查 |
| `t` | `test` | 测试 |
| `f` | `fmt` | 格式化 |

查看所有命令：`just --list` 或查看 `SIMPLE_USAGE.md`

## License

MIT License - Same as CCR project

## Contributing

This is part of the CCR project. For contributions, see the main CCR repository.

## Related Projects

- **CCR** - Main CLI tool (Rust implementation)
- **CCS** - Shell version compatibility
- **CCR Tauri** - Desktop application

---

**Version**: 0.1.0  
**Last Updated**: 2025-01-13

