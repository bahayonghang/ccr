# Getting Started

This guide will help you quickly set up and run the CCR UI project.

## System Requirements

Before getting started, please ensure your system meets the following requirements:

### Required Components

- **Rust 1.70+** (including Cargo)
- **Node.js 18+** (including npm)
- **CCR** installed and available in PATH

### Recommended Tools

- **Just** - Command runner to simplify workflow
  - Install: `cargo install just` or `brew install just`
  - Repository: https://github.com/casey/just

## Installation Steps

### 🚀 Quick Installation (Using Just - Recommended)

```bash
cd ccr-ui

# Check system requirements
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

## Development Environment Startup

### 🚀 Quick Start (Using Just)

```bash
cd ccr-ui

# 🌟 Super simple: Just one letter!
just s               # Start development environment (s = start)

# Or one-click fully automatic (check + install + dev)
just quick-start

# View help (when you don't know what command to use)
just                 # Display common commands help
just --list          # Display all 40+ commands
```

**Simplified Commands Quick Reference:**
- `just s` = Start development
- `just i` = Install dependencies  
- `just b` = Build production
- `just c` = Check code
- `just t` = Run tests
- `just f` = Format code

### 📝 Manual Startup

#### Start Backend Server

```bash
cd backend
cargo run

# Or specify custom port
cargo run -- --port 8081

# Or use Just
just dev-backend
```

The backend server starts by default at `http://127.0.0.1:8081`.

#### Start Frontend Development Server

```bash
cd frontend
npm run dev

# Or use Just
just dev-frontend
```

The frontend development server starts at `http://localhost:5173` with hot reload support.

## Verify Installation

### 1. Check Backend Service

Visit `http://127.0.0.1:8081/api/system/info` which should return system information:

```json
{
  "status": "ok",
  "version": "0.1.0",
  "system": {
    "os": "linux",
    "arch": "x86_64",
    "cpu_count": 8,
    "username": "user"
  }
}
```

### 2. Check Frontend Application

Visit `http://localhost:5173` and you should see the CCR UI main interface.

### 3. Test CCR Integration

Try executing the `ccr list` command in the application, you should be able to see the current configuration list.

## Common Issues

### Port Conflict

If you encounter a port conflict, you can modify the port:

```bash
# Backend
cargo run -- --port 8082

# Frontend (modify vite.config.ts)
export default defineConfig({
  server: {
    port: 5174
  }
})
```

### CCR Command Unavailable

Ensure CCR is correctly installed:

```bash
# Check if CCR is in PATH
which ccr

# Test CCR command
ccr --version
```

### Dependency Installation Failed

If you encounter dependency installation issues:

```bash
# Clean cache
cargo clean
npm cache clean --force

# Reinstall
just install
```

## Next Steps

Now that you've successfully run CCR UI, you can:

1. View [Project Structure](/en/guide/project-structure) to understand code organization
2. Read [Frontend Development Guide](/en/frontend/development) to learn frontend development
3. Check [Backend Architecture](/en/backend/architecture) to understand backend design
4. Reference [API Documentation](/en/backend/api) for interface details

## Get Help

If you encounter issues during installation or usage:

- Check [FAQ](/en/faq) for common questions and answers
- Submit issues on [GitHub Issues](https://github.com/your-username/ccr/issues)
- View the project's [Contributing Guide](/en/contributing) to learn how to participate in development

