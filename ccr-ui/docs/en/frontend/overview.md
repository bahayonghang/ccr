# Frontend Overview

CCR UI frontend is a modern single-page application built with Vue 3, TypeScript, and Vite.

## Tech Stack

### Core Framework

- **Vue 3.5.22** - Progressive JavaScript framework
  - Composition API for better code organization
  - Reactive data binding
  - Component-based architecture

- **Vite 7.1.11** - Next-generation build tool
  - Lightning-fast cold start
  - Hot module replacement (HMR)
  - Optimized production builds

- **TypeScript 5.7.3** - Type-safe JavaScript
  - Static type checking
  - Enhanced IDE support
  - Better code maintainability

### State Management

- **Vue Router 4.4.5** - Official routing solution
  - Nested routes
  - Lazy loading
  - Navigation guards

- **Pinia 2.2.6** - State management library
  - Type-safe stores
  - DevTools integration
  - Modular store design

### Styling

- **Tailwind CSS 3.4.17** - Utility-first CSS framework
  - Rapid prototyping
  - Consistent design system
  - Dark mode support

- **PostCSS** - CSS preprocessing
  - Autoprefixer
  - Custom properties
  - Modern CSS features

### HTTP Client

- **Axios 1.7.9** - Promise-based HTTP client
  - Request/response interceptors
  - Automatic transforms
  - Error handling

### Icons

- **Lucide Vue Next 0.468.0** - Modern icon library
  - Consistent design
  - Tree-shakeable
  - Customizable

## Project Structure

```
frontend/
├── src/
│   ├── main.ts              # Application entry
│   ├── App.vue              # Root component
│   ├── views/               # Page components
│   │   ├── HomeView.vue    # Dashboard
│   │   ├── ConfigsView.vue # Config management
│   │   ├── McpView.vue     # MCP servers
│   │   └── ...
│   ├── components/          # Reusable components
│   │   ├── MainLayout.vue
│   │   ├── Navbar.vue
│   │   └── ...
│   ├── router/             # Route configuration
│   ├── stores/             # Pinia stores
│   ├── api/                # API client
│   ├── types/              # TypeScript types
│   ├── styles/             # Global styles
│   └── utils/              # Utility functions
├── public/                 # Static assets
├── index.html             # HTML template
├── vite.config.ts         # Vite configuration
├── tailwind.config.js     # Tailwind configuration
├── tsconfig.json          # TypeScript configuration
└── package.json           # Dependencies
```

## Key Features

### Dashboard Homepage

- System monitoring (CPU, memory)
- Feature module cards
- Quick navigation
- Real-time status

### Configuration Management

- List all CCR configurations
- Switch between configurations
- Validate configurations
- Real-time updates

### MCP Server Management

- Add/edit/delete MCP servers
- Enable/disable servers
- Test server connections
- View server logs

### Agent Management

- Configure AI agents
- Bind tools to agents
- Set agent parameters
- Manage agent lifecycle

### Statistics & Analytics

- Cost tracking
- Token usage statistics
- Trend analysis
- Multi-dimensional reports

### Theme System

- Light/dark mode
- Automatic theme detection
- Smooth transitions
- Persistent preferences

## Development

### Start Development Server

```bash
npm run dev
```

Visit `http://localhost:5173`

### Build for Production

```bash
npm run build
```

Output in `dist/` directory

### Type Checking

```bash
npm run type-check
```

### Linting

```bash
npm run lint
```

## Component Guidelines

### Component Structure

```vue
<script setup lang="ts">
// 1. Imports
import { ref, computed } from 'vue'
import type { MyType } from '@/types'

// 2. Props & Emits
interface Props {
  data: MyType
}
const props = defineProps<Props>()
const emit = defineEmits<{
  update: [value: MyType]
}>()

// 3. State
const localState = ref('')

// 4. Computed
const computed Value = computed(() => props.data.value)

// 5. Methods
function handleUpdate() {
  emit('update', props.data)
}
</script>

<template>
  <!-- Template content -->
</template>

<style scoped>
/* Component styles */
</style>
```

### Naming Conventions

- **Components**: PascalCase (e.g., `ConfigCard.vue`)
- **Files**: kebab-case (e.g., `config-card.ts`)
- **Variables**: camelCase (e.g., `configData`)
- **Constants**: UPPER_SNAKE_CASE (e.g., `API_BASE_URL`)

## API Integration

### API Client Setup

```typescript
// src/api/client.ts
import axios from 'axios'

const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8081',
  timeout: 10000,
})

// Request interceptor
apiClient.interceptors.request.use(
  (config) => {
    // Add auth token if needed
    return config
  },
  (error) => Promise.reject(error)
)

// Response interceptor
apiClient.interceptors.response.use(
  (response) => response.data,
  (error) => {
    // Handle errors
    return Promise.reject(error)
  }
)

export default apiClient
```

### Making API Calls

```typescript
// src/api/configs.ts
import apiClient from './client'
import type { Config } from '@/types'

export const configsApi = {
  async list(): Promise<Config[]> {
    return apiClient.get('/api/configs')
  },
  
  async switch(name: string): Promise<void> {
    return apiClient.post('/api/configs/switch', { name })
  },
  
  async validate(config: Config): Promise<boolean> {
    return apiClient.post('/api/configs/validate', config)
  },
}
```

## State Management

### Store Structure

```typescript
// src/stores/config.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { configsApi } from '@/api/configs'
import type { Config } from '@/types'

export const useConfigStore = defineStore('config', () => {
  // State
  const configs = ref<Config[]>([])
  const currentConfig = ref<Config | null>(null)
  const loading = ref(false)

  // Getters
  const hasConfigs = computed(() => configs.value.length > 0)
  
  // Actions
  async function fetchConfigs() {
    loading.value = true
    try {
      configs.value = await configsApi.list()
    } finally {
      loading.value = false
    }
  }

  async function switchConfig(name: string) {
    await configsApi.switch(name)
    await fetchConfigs()
  }

  return {
    configs,
    currentConfig,
    loading,
    hasConfigs,
    fetchConfigs,
    switchConfig,
  }
})
```

## Styling Guidelines

### Tailwind Utilities

```vue
<template>
  <!-- Use Tailwind utility classes -->
  <div class="p-4 bg-white dark:bg-gray-800 rounded-lg shadow-md">
    <h2 class="text-xl font-bold text-gray-900 dark:text-white">
      Title
    </h2>
    <p class="mt-2 text-gray-600 dark:text-gray-300">
      Description
    </p>
  </div>
</template>
```

### Custom Styles

```vue
<style scoped>
.custom-component {
  /* Use @apply for common patterns */
  @apply flex items-center gap-2;
  
  /* Custom CSS when needed */
  transition: all 0.3s ease;
}

.custom-component:hover {
  @apply shadow-lg;
}
</style>
```

## Testing

### Unit Tests

```typescript
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import ConfigCard from '@/components/ConfigCard.vue'

describe('ConfigCard', () => {
  it('renders correctly', () => {
    const wrapper = mount(ConfigCard, {
      props: {
        config: {
          name: 'test-config',
          description: 'Test description'
        }
      }
    })
    
    expect(wrapper.text()).toContain('test-config')
  })
})
```

## Performance Optimization

### Code Splitting

```typescript
// router/index.ts
const routes = [
  {
    path: '/configs',
    component: () => import('../views/ConfigsView.vue')
  }
]
```

### Lazy Loading Components

```vue
<script setup lang="ts">
import { defineAsyncComponent } from 'vue'

const HeavyComponent = defineAsyncComponent(() =>
  import('./HeavyComponent.vue')
)
</script>
```

## Best Practices

1. **Type Safety**: Always use TypeScript types
2. **Composition API**: Prefer Composition API over Options API
3. **Component Size**: Keep components small and focused
4. **Props Validation**: Define clear prop types
5. **Event Naming**: Use descriptive event names
6. **State Management**: Use Pinia for shared state
7. **Error Handling**: Always handle API errors
8. **Loading States**: Show loading indicators
9. **Accessibility**: Use semantic HTML and ARIA attributes
10. **Performance**: Lazy load heavy components

## Further Reading

- [Development Guide](/en/frontend/development)
- [Component Documentation](/en/frontend/components)
- [API Reference](/en/frontend/api)
- [Styling Guide](/en/frontend/styling)
- [Testing Guide](/en/frontend/testing)

