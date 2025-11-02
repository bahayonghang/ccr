# Contributing Guide

Thank you for your interest in the CCR UI project! We welcome all forms of contributions, including but not limited to code, documentation, testing, issue reports, and feature suggestions.

## 🤝 How to Contribute

### Contribution Types

We welcome the following types of contributions:

- **🐛 Bug Fixes** - Fix issues in existing features
- **✨ New Features** - Add new functionality
- **📚 Documentation Improvements** - Improve documentation quality and completeness
- **🧪 Test Enhancements** - Add or improve test cases
- **🎨 UI/UX Improvements** - Improve user interface and experience
- **⚡ Performance Optimization** - Improve application performance
- **🔧 Tooling Improvements** - Improve development tools and processes

### Contribution Process

1. **Fork the Project** - Fork this project on GitHub
2. **Create a Branch** - Create a new branch for your contribution
3. **Develop and Test** - Implement your changes and ensure tests pass
4. **Submit PR** - Create a Pull Request and describe your changes
5. **Code Review** - Wait for maintainers to review your code
6. **Merge** - After review approval, your code will be merged

## 🚀 Development Environment Setup

### System Requirements

- **Rust 1.70+** (including Cargo)
- **Node.js 18+** (including npm)
- **Git** version control
- **CCR** installed and available in PATH

### Clone the Project

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/ccr.git
cd ccr/ccr-ui

# Add upstream repository
git remote add upstream https://github.com/ORIGINAL_OWNER/ccr.git
```

### Install Dependencies

```bash
# Using Just (recommended)
just install

# Or install manually
cd backend && cargo build
cd ../frontend && npm install
cd ../docs && npm install
```

### Start Development Environment

```bash
# Start complete development environment
just dev

# Or start separately
just dev-backend    # Start backend server
just dev-frontend   # Start frontend dev server
just dev-docs       # Start docs server
```

## 📝 Coding Standards

### General Guidelines

- Write code comments and documentation in English
- Keep code concise and readable
- Follow the existing code style in the project
- Add appropriate tests for new features
- Update related documentation

### Rust Backend Guidelines

#### Code Style

```rust
// ✅ Recommended function naming and structure
pub async fn get_configs() -> Result<Vec<Config>, ApiError> {
    // Function implementation
}

// ✅ Recommended error handling
match execute_ccr_command("list", &[]).await {
    Ok(output) => {
        if output.success {
            // Handle success
        } else {
            // Handle command execution failure
        }
    }
    Err(e) => {
        // Handle error
        return Err(ApiError::from(e));
    }
}

// ✅ Recommended struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub description: Option<String>,
    pub settings: Settings,
}
```

#### Formatting

```bash
# Format code
cargo fmt

# Check code style
cargo clippy

# Run tests
cargo test
```

#### Error Handling

- Use `Result<T, E>` for operations that might fail
- Use `anyhow::Result` for complex error scenarios
- Use `thiserror` to define custom error types
- Provide meaningful error messages

### Vue Frontend Guidelines

#### Component Style

```vue
<!-- ✅ Recommended component structure -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Config } from '@/types'

// Props definition
interface Props {
  config: Config
  editable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  editable: false
})

// Emits definition
const emit = defineEmits<{
  update: [config: Config]
  delete: [id: string]
}>()

// Reactive state
const isEditing = ref(false)

// Computed properties
const displayName = computed(() => 
  props.config.name || 'Unnamed Config'
)
</script>

<template>
  <div class="config-card">
    <h3>{{ displayName }}</h3>
    <!-- Component content -->
  </div>
</template>

<style scoped>
.config-card {
  @apply p-4 rounded-lg bg-white dark:bg-gray-800;
}
</style>
```

#### TypeScript Usage

```typescript
// ✅ Recommended type definitions
interface Config {
  id: string
  name: string
  description?: string
  settings: Settings
  createdAt: Date
}

// ✅ Recommended function signatures
async function fetchConfigs(): Promise<Config[]> {
  // Function implementation
}

// ✅ Use type guards
function isConfig(obj: unknown): obj is Config {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    'id' in obj &&
    'name' in obj
  )
}
```

#### Formatting

```bash
# Type checking
npm run type-check

# Lint checking
npm run lint

# Fix lint issues
npm run lint:fix

# Format code
npm run format
```

## 🧪 Testing

### Backend Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run with code coverage
cargo tarpaulin --out Html
```

### Frontend Tests

```bash
# Run all tests
npm test

# Run in watch mode
npm test:watch

# Generate coverage report
npm run test:coverage
```

## 📚 Documentation

### Documentation Types

1. **Code Comments** - Inline code comments
2. **API Documentation** - Backend API endpoint documentation
3. **Component Documentation** - Frontend component documentation
4. **User Guide** - User usage guide
5. **Development Documentation** - Development guide

### Documentation Standards

- Write in English
- Provide clear examples
- Keep documentation up to date
- Use Markdown format

### Build Documentation

```bash
# Start docs dev server
just dev-docs

# Build docs
just build-docs

# Preview built docs
just preview-docs
```

## 🔄 Git Workflow

### Branch Naming

```bash
# Feature branch
feature/add-new-config-option

# Bug fix branch
fix/config-loading-error

# Documentation branch
docs/update-api-docs

# Performance optimization branch
perf/optimize-config-loading
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/) standard:

```bash
# Format
<type>(<scope>): <subject>

# Examples
feat(backend): add config validation endpoint
fix(frontend): resolve theme toggle issue
docs(guide): update installation instructions
test(backend): add config manager tests
refactor(frontend): simplify config store logic
perf(backend): optimize database queries
style(frontend): fix component formatting
chore(deps): update dependencies
```

### Pull Request

1. **Title**
   - Clear and concise description of changes
   - Follow Conventional Commits format

2. **Description**
   - Detailed explanation of changes
   - Related issue numbers (#123)
   - Screenshots (if UI changes)
   - Testing instructions

3. **Checklist**
   ```markdown
   - [ ] Code follows project style guidelines
   - [ ] Added/updated tests
   - [ ] All tests pass
   - [ ] Updated documentation
   - [ ] No breaking changes (or marked as such)
   - [ ] Added changelog entry
   ```

## 🏗️ Project Structure

### Backend

```
backend/
├── src/
│   ├── main.rs           # Application entry
│   ├── models.rs         # Data models
│   ├── handlers/         # Request handlers
│   ├── executor/         # Command executor
│   └── ...
├── Cargo.toml           # Dependencies
└── tests/               # Tests
```

### Frontend

```
frontend/
├── src/
│   ├── main.ts          # Application entry
│   ├── App.vue          # Root component
│   ├── views/           # Page components
│   ├── components/      # Reusable components
│   ├── router/          # Route config
│   ├── stores/          # State management
│   ├── api/             # API client
│   └── types/           # Type definitions
└── package.json         # Dependencies
```

## 🎯 Common Tasks

### Add New API Endpoint

1. **Backend**
   ```rust
   // Add handler in handlers/
   pub async fn new_endpoint(
       State(state): State<AppState>,
   ) -> Result<Json<Response>, ApiError> {
       // Implementation
   }
   
   // Add route in main.rs
   .route("/api/new-endpoint", get(new_endpoint))
   ```

2. **Frontend**
   ```typescript
   // Add API call in api/client.ts
   export async function callNewEndpoint() {
     const response = await axios.get('/api/new-endpoint')
     return response.data
   }
   ```

### Add New Page

1. **Create component**
   ```bash
   touch frontend/src/views/NewPageView.vue
   ```

2. **Add route**
   ```typescript
   // In router/index.ts
   {
     path: '/new-page',
     name: 'NewPage',
     component: () => import('../views/NewPageView.vue')
   }
   ```

3. **Add navigation**
   ```vue
   <!-- In Navbar.vue or sidebar -->
   <router-link to="/new-page">New Page</router-link>
   ```

## 🐛 Bug Reports

When reporting a bug, please include:

1. **Description** - Clear description of the bug
2. **Reproduction Steps** - Detailed steps to reproduce
3. **Expected Behavior** - What should happen
4. **Actual Behavior** - What actually happens
5. **Environment**
   - OS: [e.g., macOS 13.0]
   - Rust version: [e.g., 1.70.0]
   - Node.js version: [e.g., 18.0.0]
6. **Screenshots** - If applicable
7. **Logs** - Relevant error logs

## ✨ Feature Requests

When requesting a feature, please include:

1. **Problem Description** - What problem does this solve?
2. **Proposed Solution** - How should it work?
3. **Alternatives** - Other possible solutions
4. **Additional Context** - Any other relevant information

## 📞 Communication

- **GitHub Issues** - Bug reports and feature requests
- **Pull Requests** - Code contributions
- **Discussions** - General discussions and questions

## 📜 Code of Conduct

- Be respectful and considerate
- Welcome diverse perspectives
- Focus on constructive feedback
- Help create a positive community

## 🙏 Thank You

Thank you for contributing to CCR UI! Every contribution, big or small, is valuable and appreciated. Together, we can make this project better!

---

**Questions?** Feel free to ask in [GitHub Discussions](https://github.com/your-username/ccr/discussions) or open an issue.

