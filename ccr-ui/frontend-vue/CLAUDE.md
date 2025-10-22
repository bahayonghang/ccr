# CCR UI Frontend Module

[Root Directory](../../CLAUDE.md) > [ccr-ui](../CLAUDE.md) > **frontend-vue**

## Change Log (Changelog)
- **2025-10-22 00:04:36 CST**: Initial module documentation created

## Module Responsibilities

The frontend module is a Vue.js 3 single-page application providing:

1. **Visual Dashboard** - Unified interface for managing multiple AI CLI tools
2. **Configuration Management** - Visual editor for configs
3. **Command Execution** - Execute CCR commands with visual feedback
4. **Multi-Platform Support** - Claude, Codex, Gemini, Qwen, iFlow
5. **Liquid Glass Design** - Modern glassmorphism UI with smooth animations

Key Features:
- Responsive design (mobile, tablet, desktop)
- Dark/light theme support
- Real-time API communication
- State management with Pinia
- TypeScript for type safety

## Entry and Startup

### Main Entry Point

**File**: `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/main.ts`

**Startup Sequence**:
```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './styles/index.css'

const app = createApp(App)

// 1. Add Pinia (state management)
app.use(createPinia())

// 2. Add Vue Router
app.use(router)

// 3. Mount app
app.mount('#app')
```

### Development Server

```bash
cd /home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue

# Install dependencies
npm install

# Start dev server (port 3000)
npm run dev

# Build production
npm run build

# Preview production build
npm run preview
```

### Module Structure

```
frontend-vue/
├── src/
│   ├── main.ts               - Entry point
│   ├── App.vue               - Root component
│   ├── views/                - Page components (40+ files)
│   │   ├── HomeView.vue
│   │   ├── ConfigsView.vue
│   │   ├── CommandsView.vue
│   │   ├── ClaudeCodeView.vue
│   │   ├── CodexView.vue
│   │   ├── CodexProfilesView.vue
│   │   ├── CodexMcpView.vue
│   │   ├── GeminiCliView.vue
│   │   ├── QwenView.vue
│   │   ├── IflowView.vue
│   │   ├── McpView.vue
│   │   ├── AgentsView.vue
│   │   ├── SlashCommandsView.vue
│   │   ├── PluginsView.vue
│   │   ├── ConverterView.vue
│   │   └── SyncView.vue
│   ├── components/           - Reusable components
│   │   ├── Button.vue
│   │   ├── Card.vue
│   │   ├── Input.vue
│   │   ├── Table.vue
│   │   ├── Navbar.vue
│   │   ├── MainLayout.vue
│   │   ├── ThemeToggle.vue
│   │   ├── ConfigCard.vue
│   │   ├── DetailField.vue
│   │   ├── HistoryList.vue
│   │   ├── UpdateModal.vue
│   │   ├── VersionManager.vue
│   │   ├── StatusHeader.vue
│   │   ├── RightSidebar.vue
│   │   └── CollapsibleSidebar.vue
│   ├── router/               - Vue Router config
│   │   └── index.ts
│   ├── store/                - Pinia stores
│   │   ├── index.ts
│   │   └── theme.ts
│   ├── api/                  - API client
│   │   └── client.ts
│   ├── types/                - TypeScript types
│   │   └── index.ts
│   └── styles/               - Global styles
│       └── index.css
├── public/                   - Static assets
├── index.html                - HTML template
├── package.json              - Dependencies
├── tsconfig.json             - TypeScript config
├── vite.config.ts            - Vite config
└── tailwind.config.js        - Tailwind CSS config
```

## External Interfaces

### Vue Router Routes

```typescript
const routes = [
  { path: '/', component: HomeView },

  // CCR Config Management
  { path: '/configs', component: ConfigsView },
  { path: '/commands', component: CommandsView },
  { path: '/converter', component: ConverterView },
  { path: '/sync', component: SyncView },

  // Claude Code
  { path: '/claude', component: ClaudeCodeView },
  { path: '/mcp', component: McpView },
  { path: '/agents', component: AgentsView },
  { path: '/slash-commands', component: SlashCommandsView },
  { path: '/plugins', component: PluginsView },

  // Codex
  { path: '/codex', component: CodexView },
  { path: '/codex/profiles', component: CodexProfilesView },
  { path: '/codex/mcp', component: CodexMcpView },
  { path: '/codex/agents', component: CodexAgentsView },
  { path: '/codex/slash-commands', component: CodexSlashCommandsView },
  { path: '/codex/plugins', component: CodexPluginsView },

  // Gemini CLI
  { path: '/gemini-cli', component: GeminiCliView },
  { path: '/gemini-cli/mcp', component: GeminiMcpView },
  { path: '/gemini-cli/agents', component: GeminiAgentsView },
  { path: '/gemini-cli/slash-commands', component: GeminiSlashCommandsView },
  { path: '/gemini-cli/plugins', component: GeminiPluginsView },

  // Qwen
  { path: '/qwen', component: QwenView },
  { path: '/qwen/mcp', component: QwenMcpView },
  { path: '/qwen/agents', component: QwenAgentsView },
  { path: '/qwen/slash-commands', component: QwenSlashCommandsView },
  { path: '/qwen/plugins', component: QwenPluginsView },

  // iFlow
  { path: '/iflow', component: IflowView },
  { path: '/iflow/mcp', component: IflowMcpView },
  { path: '/iflow/agents', component: IflowAgentsView },
  { path: '/iflow/slash-commands', component: IflowSlashCommandsView },
  { path: '/iflow/plugins', component: IflowPluginsView },
]
```

### API Client (`src/api/client.ts`)

```typescript
import axios from 'axios'

const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8081',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Claude Code APIs
export const claudeApi = {
  listMcpServers: () => apiClient.get('/api/mcp'),
  addMcpServer: (data: any) => apiClient.post('/api/mcp', data),
  updateMcpServer: (name: string, data: any) =>
    apiClient.put(`/api/mcp/${name}`, data),
  deleteMcpServer: (name: string) =>
    apiClient.delete(`/api/mcp/${name}`),
  toggleMcpServer: (name: string) =>
    apiClient.put(`/api/mcp/${name}/toggle`),
  // ... agents, slash commands, plugins, config
}

// Codex APIs
export const codexApi = {
  listProfiles: () => apiClient.get('/api/codex/profiles'),
  // ... similar pattern
}

// System APIs
export const systemApi = {
  getInfo: () => apiClient.get('/api/system/info'),
  getVersion: () => apiClient.get('/api/version'),
}

// Command execution
export const commandApi = {
  execute: (command: string, args: string[]) =>
    apiClient.post('/api/command/execute', { command, args }),
}
```

### Component Props and Events

**Button Component**:
```vue
<script setup lang="ts">
interface Props {
  variant?: 'primary' | 'secondary' | 'danger'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  loading: false,
})

const emit = defineEmits<{
  click: []
}>()
</script>
```

**Card Component**:
```vue
<script setup lang="ts">
interface Props {
  title?: string
  subtitle?: string
  glass?: boolean  // Glassmorphism effect
}
</script>
```

## Key Dependencies and Configuration

### Core Dependencies (package.json)

```json
{
  "dependencies": {
    "vue": "^3.5.22",              // Core framework
    "vue-router": "^4.4.5",        // Routing
    "pinia": "^2.2.6",             // State management
    "axios": "^1.7.9",             // HTTP client
    "lucide-vue-next": "^0.468.0", // Icons
    "tailwindcss": "^3.4.17"       // Styling
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.2.1",  // Vite plugin
    "vite": "^7.1.11",                // Build tool
    "typescript": "^5.7.3",           // Type checking
    "vue-tsc": "^2.2.0",              // Vue TypeScript compiler
    "eslint": "^9.19.0"               // Linting
  }
}
```

### Vite Configuration (vite.config.ts)

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8081',
        changeOrigin: true,
      },
    },
  },
})
```

### Tailwind Configuration (tailwind.config.js)

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Liquid glass theme colors
        primary: {
          50: '#f0f9ff',
          // ... color palette
        },
      },
      backdropBlur: {
        xs: '2px',
      },
    },
  },
  plugins: [],
  darkMode: 'class',
}
```

### TypeScript Configuration (tsconfig.json)

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

### Environment Variables

**.env.development**:
```
VITE_API_BASE_URL=http://localhost:8081
VITE_APP_TITLE=CCR UI
```

**.env.production**:
```
VITE_API_BASE_URL=/api  # Relative path for production
VITE_APP_TITLE=CCR UI
```

## Data Models (TypeScript)

### Types Definition (`src/types/index.ts`)

```typescript
// MCP Server
export interface McpServer {
  name: string
  command: string
  args: string[]
  env?: Record<string, string>
  enabled?: boolean
}

// Agent
export interface Agent {
  name: string
  description: string
  instructions: string
  enabled: boolean
}

// Slash Command
export interface SlashCommand {
  name: string
  description: string
  command: string
  enabled: boolean
}

// Plugin
export interface Plugin {
  name: string
  description?: string
  enabled: boolean
  config: any
}

// Config Section
export interface ConfigSection {
  description?: string
  base_url: string
  auth_token: string
  model: string
  small_fast_model?: string
}

// System Info
export interface SystemInfo {
  hostname: string
  os: string
  cpu_count: number
  total_memory: number
  used_memory: number
}

// Command Result
export interface CommandResult {
  stdout: string
  stderr: string
  exit_code: number
}
```

### Pinia Store (`src/store/theme.ts`)

```typescript
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export const useThemeStore = defineStore('theme', () => {
  const isDark = ref(false)

  // Initialize from localStorage
  const stored = localStorage.getItem('theme')
  if (stored) {
    isDark.value = stored === 'dark'
  } else {
    isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }

  // Apply theme class
  const applyTheme = () => {
    if (isDark.value) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }

  // Watch for changes
  watch(isDark, () => {
    localStorage.setItem('theme', isDark.value ? 'dark' : 'light')
    applyTheme()
  })

  // Initialize
  applyTheme()

  const toggleTheme = () => {
    isDark.value = !isDark.value
  }

  return { isDark, toggleTheme }
})
```

## Testing and Quality

### Type Checking

```bash
# Run TypeScript type checking
npm run type-check
```

### Linting

```bash
# Run ESLint
npm run lint

# Fix lint issues
npm run lint -- --fix
```

### Building

```bash
# Development build
npm run dev

# Production build
npm run build

# Preview production build
npm run preview
```

### Component Testing (if configured)

```bash
# Run component tests
npm run test

# Run with coverage
npm run test:coverage
```

## Styling Guide

### Liquid Glass Design System

**Glass Card**:
```vue
<div class="
  backdrop-blur-md
  bg-white/10
  dark:bg-gray-900/30
  border border-white/20
  rounded-2xl
  shadow-xl
  p-6
">
  <!-- Content -->
</div>
```

**Button Styles**:
```vue
<button class="
  px-6 py-3
  bg-gradient-to-r from-blue-500 to-purple-600
  hover:from-blue-600 hover:to-purple-700
  text-white
  rounded-xl
  shadow-lg
  transition-all duration-300
  hover:scale-105
  active:scale-95
">
  Click Me
</button>
```

**Theme Toggle**:
```vue
<button
  @click="toggleTheme"
  class="
    p-2 rounded-lg
    bg-gray-200 dark:bg-gray-700
    hover:bg-gray-300 dark:hover:bg-gray-600
    transition-colors
  "
>
  <SunIcon v-if="isDark" />
  <MoonIcon v-else />
</button>
```

## Frequently Asked Questions (FAQ)

### Q: How do I add a new view?

A:
1. Create view file in `src/views/NewView.vue`
2. Add route in `src/router/index.ts`
3. Add navigation link in `src/components/Navbar.vue`
4. Implement component logic and template

### Q: How do I make API calls?

A:
```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { claudeApi } from '@/api/client'

const mcpServers = ref([])
const loading = ref(false)

const fetchServers = async () => {
  loading.value = true
  try {
    const response = await claudeApi.listMcpServers()
    mcpServers.value = response.data
  } catch (error) {
    console.error('Failed to fetch servers:', error)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchServers()
})
</script>
```

### Q: How do I use Pinia store?

A:
```vue
<script setup lang="ts">
import { useThemeStore } from '@/store/theme'

const themeStore = useThemeStore()

// Access state
const isDark = computed(() => themeStore.isDark)

// Call actions
const toggle = () => {
  themeStore.toggleTheme()
}
</script>
```

### Q: How do I handle form validation?

A:
```vue
<script setup lang="ts">
import { ref, computed } from 'vue'

const formData = ref({
  name: '',
  command: '',
})

const errors = ref<Record<string, string>>({})

const validateForm = () => {
  errors.value = {}

  if (!formData.value.name) {
    errors.value.name = 'Name is required'
  }

  if (!formData.value.command) {
    errors.value.command = 'Command is required'
  }

  return Object.keys(errors.value).length === 0
}

const submit = () => {
  if (!validateForm()) return
  // Submit logic
}
</script>
```

### Q: How do I customize the theme?

A: Edit `tailwind.config.js` to customize colors, spacing, etc. The liquid glass effect uses:
- `backdrop-blur` utilities
- `bg-white/10` for semi-transparent backgrounds
- `border-white/20` for subtle borders
- `rounded-2xl` for rounded corners
- `shadow-xl` for depth

## Related File List

### Source Code
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/main.ts` - Entry point
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/App.vue` - Root component
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/views/` - Page components (40+ files)
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/components/` - Reusable components (15+ files)
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/router/index.ts` - Router config
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/store/` - Pinia stores
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/api/client.ts` - API client
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/src/types/index.ts` - TypeScript types

### Configuration
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/package.json` - Dependencies
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/vite.config.ts` - Vite config
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/tsconfig.json` - TypeScript config
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/tailwind.config.js` - Tailwind config
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/.gitignore` - Git ignore

### Build Output
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/dist/` - Production build (ignored)
- `/home/lyh/Documents/Github/ccr/ccr-ui/frontend-vue/node_modules/` - Dependencies (ignored)
