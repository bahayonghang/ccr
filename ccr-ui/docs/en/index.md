---
layout: home

hero:
  name: "CCR UI"
  text: "Modern AI Configuration Management Platform"
  tagline: "Full-stack multi-CLI tool configuration management web application built with Vue 3 + Rust"
  image:
    src: /logo.svg
    alt: CCR UI Logo
  actions:
    - theme: brand
      text: Get Started
      link: /en/guide/getting-started
    - theme: alt
      text: View Source
      link: https://github.com/bahayonghang/ccr

features:
  - icon: ⚡
    title: Multi-interface
    details: CLI-first; full CCR UI for browser/desktop, legacy Axum API for scripts, Tauri desktop build ready.
  - icon: 🔧
    title: Complete config control
    details: Visual list/switch/validate/import-export/history/backup aligned with CCR CLI.
  - icon: ☁️
    title: WebDAV multi-folder sync
    details: Register/enable folders, batch or single push/pull/status, interactive allow-list, smart filters for backups/history/locks/ui.
  - icon: 🎯
    title: Multi-CLI coverage
    details: Claude, Codex, Gemini, Qwen, iFlow (planned) dashboards plus command execution center and converter.
  - icon: 🛠️
    title: Dev-friendly
    details: Just tasks, hot reload, TypeScript, ESLint/Prettier, VitePress docs; backend reuses CCR crate.
  - icon: 📊
    title: Stats & analytics
    details: Cost/usage dashboards with time/model/project slices, JSON/CSV export, dark/light themes.
---

## Project Highlights

CCR UI is a modern full-stack web application specifically designed for managing multiple AI CLI tool configurations. It combines a user-friendly frontend interface with high-performance backend processing, providing developers with a powerful and intuitive configuration management platform.

### 🎨 Frontend Stack

- **Vue 3.5.22** - Progressive JavaScript framework with Composition API and reactive data binding
- **Vite 7.1.11** - Next-generation frontend build tool with lightning-fast cold start and HMR
- **Vue Router 4.4.5** - Official Vue.js router with nested routes and lazy loading support
- **Pinia 2.2.6** - Vue state management library, type-safe and DevTools friendly
- **TypeScript 5.7.3** - Type-safe JavaScript superset
- **Tailwind CSS 3.4.17** - Utility-first CSS framework
- **Lucide Vue Next 0.468.0** - Modern icon library
- **Axios 1.7.9** - Powerful HTTP client

### ⚙️ Backend Stack

- **Rust 2024 Edition** - System programming language, safe and high-performance
- **Axum 0.7** - Modern async web framework
- **Tokio** - Async runtime
- **Serde** - Serialization and deserialization framework
- **Tower** - Middleware and service abstraction

### 📋 Core Features

#### 🏠 Dashboard Homepage
- **Feature Module Navigation** - 8 main feature module cards
- **System Monitoring** - Real-time CPU and memory usage display
- **Quick Access** - One-click navigation to various CLI tool configuration pages

#### 🔵 Claude Code Configuration Management
- **Configuration Management** - View, switch, validate CCR configurations
- **Cloud Sync** - WebDAV cloud configuration synchronization
- **MCP Servers** - Model Context Protocol server management
- **Slash Commands** - Custom command management
- **Agents** - AI Agent configuration and tool binding
- **Plugin Management** - Plugin enable/disable and configuration

#### 🎯 Multi CLI Tool Support
- **Codex** - MCP servers, Profiles, basic configuration
- **Gemini CLI** - Google Gemini AI configuration management
- **Qwen** - Alibaba Tongyi Qianwen configuration management
- **IFLOW** - Internal workflow configuration

#### 🛠️ Other Features
- **Command Execution Center** - Unified CLI command execution interface
- **Configuration Converter** - Cross-CLI tool configuration format conversion
- **History** - Complete operation audit log
- **Real-time Output** - Terminal-style command output display

#### 📊 Statistics & Cost Analysis
- **Cost Tracking** - Precisely record cost and token usage for each API call
- **Multi-dimensional Statistics** - Statistics grouped by time range (today/this week/this month), model, and project
- **Visual Dashboard** - 4 overview cards (total cost, API calls, input/output tokens)
- **Trend Analysis** - Cost trend charts and top session queries
- **Data Export** - Support JSON/CSV format export for statistical reports
- **Real-time Refresh** - One-click refresh of latest data
- **Responsive Design** - Support dark mode and mobile devices

### 🎨 Design Features

- **Glassmorphism Style** - Semi-transparent background + blur effects
- **Gradient Colors** - Unique gradient color scheme for each CLI tool
- **Smooth Animations** - Card hover, arrow movement, gradient flow
- **Responsive Layout** - Adapts to desktop, tablet, mobile, and other devices

## Quick Start

```bash
# Launch via CCR CLI (auto-detect or download UI)
ccr ui -p 3000 --backend-port 38081

# Source development
git clone https://github.com/bahayonghang/ccr.git
cd ccr/ccr-ui
just s                    # start frontend (5173) + backend (38081)
# manual:
#   cargo run --manifest-path backend/Cargo.toml -- --port 38081
#   (new shell) cd frontend && npm install && npm run dev
```

Visit `http://localhost:5173` to start using CCR UI.

## Documentation Navigation

- [Getting Started](/en/guide/getting-started)
- [Project Structure](/en/guide/project-structure)
- [Frontend Reference](/en/reference/frontend/overview)
- [Backend Reference](/en/reference/backend/architecture)
- [Contributing](/en/contributing)
- [FAQ](/en/faq)

## Feature Preview

### 📊 Dashboard Homepage

The new Dashboard homepage provides:
- Real-time system status monitoring (CPU, memory, system info)
- 9 feature module cards with tech-inspired design (including new statistics analysis)
- Dynamic gradient background effects
- One-click quick access to various features

### 🔵 CLI Tool Homepage

Each CLI tool has its own dedicated homepage:
- Clear feature category display
- Unique color scheme
- Quick navigation for sub-features
- Convenient back to home button

### 🎯 Feature Pages

All original features are preserved:
- Configuration management (list, switch, validate)
- MCP server management
- Agent configuration
- Plugin management
- Slash commands management
- Cloud sync functionality
- **📊 Statistics Analysis** (new) - Complete cost tracking and usage statistics dashboard

---

<div style="text-align: center; margin-top: 2rem; padding: 1rem; background: var(--vp-c-bg-soft); border-radius: 8px;">
  <p>🚀 <strong>Start exploring the powerful features of CCR UI!</strong></p>
  <p>A modern configuration management platform supporting multiple AI CLI tools, providing complete configuration management and cloud sync capabilities.</p>
  <p>If you encounter any issues while using it, feel free to check our <a href="/en/faq">FAQ</a> or submit an <a href="https://github.com/bahayonghang/ccr/issues">Issue</a>.</p>
</div>
