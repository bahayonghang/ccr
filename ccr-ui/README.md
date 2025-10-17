# CCR UI - Full-Stack Web Application

A modern full-stack web application for managing CCR (Claude Code Configuration Switcher) with a React frontend and Actix Web backend.

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
- **WebDAV Sync**: Push/pull configs to cloud storage (Nutstore, Nextcloud, ownCloud)
- **Real-time Output**: See command output in a terminal-style display
- **Multi-page Navigation**: Easy switching between different functionalities

## Architecture

```
ccr-ui/
├── backend/           # Actix Web server (Rust)
│   ├── src/
│   │   ├── main.rs    # Server entry point
│   │   ├── executor/  # CLI subprocess executor
│   │   ├── handlers/  # API route handlers
│   │   └── models/    # Request/response models
│   └── Cargo.toml
├── frontend/          # Next.js application (TypeScript)
│   ├── src/
│   │   ├── app/       # Next.js App Router pages
│   │   ├── components/# Reusable components
│   │   └── lib/       # API client & utilities
│   ├── package.json
│   └── next.config.mjs
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

The frontend will start on `http://localhost:5173` with auto-reload.

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
- `GET /api/sync/status` - Get sync configuration status
- `POST /api/sync/push` - Upload config to cloud (body: `{force: boolean}`)
- `POST /api/sync/pull` - Download config from cloud (body: `{force: boolean}`)
- `GET /api/sync/info` - Get sync feature information
- `POST /api/sync/config` - Configure WebDAV sync (not supported in web API)

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
- **Actix Web 4.9** - Fast async web framework
- **Tokio 1.42** - Async runtime
- **Serde** - Serialization
- **Chrono** - Date/time handling

### Frontend
- **Next.js 16 (canary)** - React framework with App Router
- **React 19** - UI library
- **TypeScript 5.6** - Type safety
- **Tailwind CSS 3.4** - Styling
- **Axios** - HTTP client
- **Lucide React** - Icons
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

