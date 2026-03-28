import type { Config } from 'tailwindcss'
import plugin from 'tailwindcss/plugin'

export default {
  darkMode: ['class', '[data-theme="dark"]'],
  content: [
    './index.html',
    './src/App.vue',
    './src/main.ts',
    './src/api/**/*.{js,ts}',
    './src/components/**/*.{vue,js,ts,jsx,tsx}',
    './src/composables/**/*.{js,ts,jsx,tsx}',
    './src/config/**/*.{js,ts}',
    './src/layouts/**/*.{vue,js,ts,jsx,tsx}',
    './src/router/**/*.{js,ts}',
    './src/stores/**/*.{js,ts}',
    './src/utils/**/*.{js,ts}',
    './src/views/**/*.{vue,js,ts,jsx,tsx}',
  ],
  corePlugins: {
    // We ship our own reset + base styles in `src/styles/base.css` and keep
    // Tailwind's generated CSS as lean as possible for startup performance.
    preflight: false,
  },
  theme: {
    // Override default font family to use our tokens
    fontFamily: {
      sans: ['var(--font-sans)', 'ui-sans-serif', 'system-ui'],
      brand: ['var(--font-brand)', 'var(--font-sans)', 'ui-sans-serif', 'system-ui'],
      mono: ['var(--font-mono)', 'ui-monospace', 'monospace'],
    },
    fontWeight: {
      thin: '400',
      extralight: '400',
      light: '400',
      normal: '400',
      medium: '500',
      semibold: '500',
      bold: '500',
      extrabold: '500',
      black: '500',
    },
    extend: {
      transitionProperty: {
        // Unified interactive transitions to avoid many one-off `transition-[...]` utilities.
        // This reduces generated CSS size and keeps motion language consistent.
        interactive: 'color, background-color, border-color, box-shadow, transform, opacity',
      },
      colors: {
        bg: {
          base: 'rgb(var(--color-bg-base-rgb) / <alpha-value>)',
          elevated: 'rgb(var(--color-bg-elevated-rgb) / <alpha-value>)',
          surface: 'rgb(var(--color-bg-surface-rgb) / <alpha-value>)',
          overlay: 'rgb(var(--color-bg-overlay-rgb) / <alpha-value>)',
        },
        text: {
          primary: 'rgb(var(--color-text-primary-rgb) / <alpha-value>)',
          secondary: 'rgb(var(--color-text-secondary-rgb) / <alpha-value>)',
          muted: 'rgb(var(--color-text-muted-rgb) / <alpha-value>)',
          ghost: 'rgb(var(--color-text-ghost-rgb) / <alpha-value>)',
          disabled: 'rgb(var(--color-text-disabled-rgb) / <alpha-value>)',
          inverted: 'rgb(var(--color-text-inverted-rgb) / <alpha-value>)',
        },
        accent: {
          primary: 'rgb(var(--color-accent-primary-rgb) / <alpha-value>)',
          secondary: 'rgb(var(--color-accent-secondary-rgb) / <alpha-value>)',
          success: 'rgb(var(--color-success-rgb) / <alpha-value>)',
          warning: 'rgb(var(--color-warning-rgb) / <alpha-value>)',
          danger: 'rgb(var(--color-danger-rgb) / <alpha-value>)',
          info: 'rgb(var(--color-info-rgb) / <alpha-value>)',
        },
        platform: {
          claude: 'rgb(var(--color-platform-claude-rgb) / <alpha-value>)',
          codex: 'rgb(var(--color-platform-codex-rgb) / <alpha-value>)',
          gemini: 'rgb(var(--color-platform-gemini-rgb) / <alpha-value>)',
          qwen: 'rgb(var(--color-platform-qwen-rgb) / <alpha-value>)',
          iflow: 'rgb(var(--color-platform-iflow-rgb) / <alpha-value>)',
        },
        border: {
          subtle: 'rgb(var(--color-border-subtle-rgb) / <alpha-value>)',
          default: 'rgb(var(--color-border-default-rgb) / <alpha-value>)',
          strong: 'rgb(var(--color-border-strong-rgb) / <alpha-value>)',
          interactive: 'rgb(var(--color-border-interactive-rgb) / <alpha-value>)',
        },
        premium: {
          pink: 'var(--color-premium-pink)',
          blue: 'var(--color-premium-blue)',
        },
      },
      backgroundImage: {
        'premium-gradient': 'linear-gradient(135deg, var(--color-premium-pink) 0%, var(--color-premium-blue) 100%)',
      },
      spacing: {
        px: '1px',
        1: 'var(--space-1)',
        2: 'var(--space-2)',
        3: 'var(--space-3)',
        4: 'var(--space-4)',
        5: 'var(--space-5)',
        6: 'var(--space-6)',
        8: 'var(--space-8)',
        10: 'var(--space-10)',
        12: 'var(--space-12)',
        16: 'var(--space-16)',
        20: 'var(--space-20)',
      },
      borderRadius: {
        sm: 'var(--radius-sm)',
        md: 'var(--radius-md)',
        lg: 'var(--radius-lg)',
        xl: 'var(--radius-xl)',
      },
      boxShadow: {
        'glow-primary': 'var(--shadow-glow-primary)',
        'glow-success': 'var(--shadow-glow-success)',
        'glow-danger': 'var(--shadow-glow-danger)',
      },
      transitionDuration: {
        fast: 'var(--duration-fast)',
        normal: 'var(--duration-normal)',
        slow: 'var(--duration-slow)',
      },
      transitionTimingFunction: {
        DEFAULT: 'var(--ease-default)',
        in: 'var(--ease-in)',
        out: 'var(--ease-out)',
        spring: 'var(--ease-spring)',
      },
      // Custom Animation Keyframes (GPU Optimized)
      keyframes: {
        'fade-in': {
          '0%': { opacity: '0', transform: 'translateZ(0)' },
          '100%': { opacity: '1', transform: 'translateZ(0)' },
        },
        'slide-up': {
          '0%': { opacity: '0', transform: 'translate3d(0, 10px, 0)' },
          '100%': { opacity: '1', transform: 'translate3d(0, 0, 0)' },
        },
        'scale-in': {
          '0%': { opacity: '0', transform: 'scale(0.95) translateZ(0)' },
          '100%': { opacity: '1', transform: 'scale(1) translateZ(0)' },
        },
        'neko-press': {
          '0%': { transform: 'scale(1)' },
          '40%': { transform: 'scale(0.92)' },
          '70%': { transform: 'scale(1.04)' },
          '100%': { transform: 'scale(1)' },
        },
        'neko-ear-wiggle': {
          '0%, 100%': { transform: 'rotate(0deg)' },
          '20%': { transform: 'rotate(-8deg)' },
          '40%': { transform: 'rotate(6deg)' },
          '60%': { transform: 'rotate(-4deg)' },
          '80%': { transform: 'rotate(2deg)' },
        },
        'neko-breathe': {
          '0%, 100%': { boxShadow: '0 0 8px rgb(244 114 182 / 15%)' },
          '50%': { boxShadow: '0 0 16px rgb(244 114 182 / 30%)' },
        },
      },
      animation: {
        'fade-in': 'fade-in var(--duration-normal) var(--ease-out) forwards',
        'slide-up': 'slide-up var(--duration-normal) var(--ease-out) forwards',
        'scale-in': 'scale-in var(--duration-fast) var(--ease-spring) forwards',
        'neko-press': 'neko-press 0.4s ease-out',
        'neko-ear-wiggle': 'neko-ear-wiggle 0.6s ease-in-out',
        'neko-breathe': 'neko-breathe 3s ease-in-out infinite',
      }
    },
  },
  plugins: [
    plugin(({ addComponents }) => {
      const surfaceShell = {
        background: 'var(--surface-shell-bg)',
        '-webkit-backdrop-filter': 'var(--surface-shell-blur)',
        'backdrop-filter': 'var(--surface-shell-blur)',
        border: '1px solid var(--surface-shell-border)',
        'box-shadow': 'var(--surface-shell-shadow), var(--liquid-glass-highlight)',
      }

      const surfaceWorkspace = {
        background: 'var(--surface-workspace-bg)',
        '-webkit-backdrop-filter': 'var(--surface-workspace-blur)',
        'backdrop-filter': 'var(--surface-workspace-blur)',
        border: '1px solid var(--surface-workspace-border)',
        'box-shadow': 'var(--surface-workspace-shadow)',
      }

      const surfaceCard = {
        background: 'var(--surface-card-bg)',
        '-webkit-backdrop-filter': 'var(--surface-card-blur)',
        'backdrop-filter': 'var(--surface-card-blur)',
        border: '1px solid var(--surface-card-border)',
        'box-shadow': 'var(--surface-card-shadow), var(--glass-inner-glow)',
      }

      const surfaceModal = {
        background: 'var(--surface-modal-bg)',
        '-webkit-backdrop-filter': 'var(--surface-modal-blur)',
        'backdrop-filter': 'var(--surface-modal-blur)',
        border: '1px solid var(--surface-modal-border)',
        'box-shadow': 'var(--surface-modal-shadow), var(--glass-inner-glow)',
      }

      const surfaceStatus = {
        background: 'var(--surface-status-bg)',
        '-webkit-backdrop-filter': 'var(--surface-status-blur)',
        'backdrop-filter': 'var(--surface-status-blur)',
        border: '1px solid var(--surface-status-border)',
        'box-shadow': 'var(--surface-status-shadow)',
      }

      addComponents({
        '.surface-shell': surfaceShell,
        '.liquid-glass': surfaceShell,

        '.surface-workspace': surfaceWorkspace,
        '.glass-effect': surfaceWorkspace,
        '.glass-surface': surfaceWorkspace,

        '.surface-card': surfaceCard,
        '.glass-effect-strong': surfaceCard,
        '.glass-elevated': surfaceCard,

        '.surface-modal': surfaceModal,
        '.glass-modal': surfaceModal,

        '.surface-status': surfaceStatus,
      })
    }),
  ],
} satisfies Config
