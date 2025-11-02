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
      link: https://github.com/your-username/ccr

features:
  - icon: ⚡
    title: Modern Tech Stack
    details: Frontend built with Vue 3.5 + Vite 7.1 + TypeScript + Tailwind CSS, backend powered by Rust + Axum, delivering ultimate development experience and runtime performance
  - icon: 🎯
    title: Intuitive User Interface
    details: Tech-inspired glassmorphism design with dark/light theme support, Dashboard homepage displays all feature modules at a glance
  - icon: 🔧
    title: Powerful Configuration Management
    details: Support viewing, switching, validating, and cloud syncing CCR configurations, real-time command execution results, and history tracking
  - icon: 🚀
    title: Multi CLI Tool Support
    details: Unified management of configurations and services for multiple AI CLI tools including Claude Code, Codex, Gemini CLI, Qwen, and IFLOW
  - icon: 📱
    title: Responsive Design
    details: Fully responsive design supporting desktop and mobile access, with smooth animations and hover interactions
  - icon: 🛠️
    title: Developer Friendly
    details: Complete development toolchain with Vite hot reload, TypeScript type checking, ESLint + Prettier for optimal development experience
  - icon: ☁️
    title: Cloud Synchronization
    details: Support WebDAV cloud configuration sync and automatic backups, ensuring configuration files are safe and secure
  - icon: 🔌
    title: Plugin Ecosystem
    details: Rich extension functionality management including MCP servers, Agents, Plugins, and Slash Commands
  - icon: 📊
    title: Statistics & Cost Analysis
    details: Complete API usage statistics and cost tracking system, real-time monitoring of costs, token usage, with multi-dimensional analysis by time/model/project
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
# Clone the project
git clone https://github.com/your-username/ccr.git
cd ccr/ccr-ui

# Quick start with Just (recommended)
just s

# Or start manually
cd backend && cargo run &
cd frontend && npm run dev
```

Visit `http://localhost:5173` to start using CCR UI.

## Documentation Navigation

- [Getting Started](/en/guide/getting-started) - Learn how to install and run the project
- [Project Structure](/en/guide/project-structure) - Detailed project architecture documentation
- [Frontend Documentation](/en/frontend/overview) - Vue 3 frontend development guide
- [Backend Documentation](/en/backend/architecture) - Rust backend architecture documentation
- [Contributing Guide](/en/contributing) - How to participate in project development
- [FAQ](/en/faq) - Frequently Asked Questions

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
  <p>If you encounter any issues while using it, feel free to check our <a href="/en/faq">FAQ</a> or submit an <a href="https://github.com/your-username/ccr/issues">Issue</a>.</p>
</div>

