# Changelog - Neo-Terminal UI Overhaul

## [Unreleased] - Neo-Terminal Edition

### 🎨 Design System & Aesthetics
- **New Visual Language**: Implemented "Neo-Terminal" aesthetic featuring deep dark backgrounds (`slate-950`), glassmorphism, and neon accents.
- **Design Tokens**: Standardized all colors, spacing, and typography in `src/styles/tokens.ts` and `src/styles/theme.css`.
- **Typography**: Adopted `Inter` for UI and `JetBrains Mono` for code/data, improving readability and technical feel.
- **Animations**: Added GPU-accelerated micro-interactions (hover scales, glow effects) and page transitions (`fade-slide`, `scale-in`).

### 🧩 Components
- **Button**: Completely rewritten with variants: `primary` (neon glow), `ghost`, `glass`, `outline`. Added loading states and icon support.
- **Card**: New generic `Card` component replacing ad-hoc divs. Supports `elevated`, `glass`, and `neo` variants with optional glow effects.
- **Input**: Refactored text inputs with floating labels, focus glow rings, and glass backgrounds.
- **Modals**: Unified `EditConfigModal` and `AddConfigModal` to use the new `Card`-based modal shell with backdrop blur and animated entry.
- **Spinner**: New reusable `Spinner` component for consistent loading states.
- **AnimatedBackground**: specific component for the signature "breathing orbs" background effect.

### ⚡ Performance & Engineering
- **CSS Variables**: Migrated from hardcoded hex values to CSS variables for runtime theming capability.
- **Virtualization**: Retained and optimized `vue-virtual` in `HistoryList` for high-performance rendering of large datasets.
- **Lazy Loading**: ensured heavy components like Modals are loaded effectively (via v-if).
- **Type Safety**: Improved TypeScript typing across `ConfigsView` and API interactions.

### 🏠 Pages
- **HomeView**: Redesigned as a central command hub. Removed legacy "Guofeng" styles. Added quick action grid and system status summary.
- **ConfigsView**: Converted to a dashboard layout. Removed inline styles in favor of Tailwind utility classes.
- **Layout**: `MainLayout` now features a collapsible, glass-effect sidebar with smoother transitions.

### 🐛 Fixes
- Fixed inconsistent spacing in configuration lists.
- Resolved z-index issues with modal backdrops.
- Standardized form validation visual feedback.
