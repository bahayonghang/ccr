import type { Config } from 'tailwindcss'

export default {
  darkMode: ['class', '[data-theme="dark"]'],
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      // ========== 设计令牌 - 间距系统 ==========
      spacing: {
        'xs': 'var(--space-xs)',
        'sm': 'var(--space-sm)',
        'md': 'var(--space-md)',
        'lg': 'var(--space-lg)',
        'xl': 'var(--space-xl)',
        '2xl': 'var(--space-2xl)',
        '3xl': 'var(--space-3xl)',
        '4xl': 'var(--space-4xl)',
        '5xl': 'var(--space-5xl)',
      },

      // ========== 字体系统 ==========
      fontSize: {
        'xs': 'var(--text-xs)',
        'sm': 'var(--text-sm)',
        'base': 'var(--text-base)',
        'lg': 'var(--text-lg)',
        'xl': 'var(--text-xl)',
        '2xl': 'var(--text-2xl)',
        '3xl': 'var(--text-3xl)',
        '4xl': 'var(--text-4xl)',
        '5xl': 'var(--text-5xl)',
        '6xl': 'var(--text-6xl)',
      },
      fontFamily: {
        'sans': ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif'],
        'mono': ['JetBrains Mono', 'SF Mono', 'Fira Code', 'Cascadia Code', 'Consolas', 'monospace'],
        'display': ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'sans-serif'],
        'body': ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'sans-serif'],
      },
      fontWeight: {
        'light': '300',
        'normal': '400',
        'medium': '500',
        'semibold': '600',
        'bold': '700',
        'extrabold': '800',
        'black': '900',
      },
      lineHeight: {
        'tight': '1.25',
        'snug': '1.375',
        'normal': '1.5',
        'relaxed': '1.625',
        'loose': '2',
      },

      // ========== 圆角系统 ==========
      borderRadius: {
        'xs': 'var(--radius-sm)',
        'sm': 'var(--radius-md)',
        'md': 'var(--radius-lg)',
        'lg': 'var(--radius-xl)',
        'xl': 'var(--radius-2xl)',
        '2xl': 'var(--radius-3xl)',
        'full': 'var(--radius-full)',
      },

      // ========== 颜色系统 ==========
      colors: {
        // 背景色
        bg: {
          primary: 'var(--color-bg-base)',
          secondary: 'var(--color-bg-elevated)',
          tertiary: 'var(--color-bg-surface)',
          card: 'var(--color-bg-elevated)',
          hover: 'var(--color-bg-overlay)',
          // 向后兼容
          base: 'var(--color-bg-base)',
          elevated: 'var(--color-bg-elevated)',
          surface: 'var(--color-bg-surface)',
          overlay: 'var(--color-bg-overlay)',
        },
        // 边框色
        border: {
          color: 'var(--color-border-default)',
          hover: 'var(--color-border-strong)',
          subtle: 'var(--color-border-subtle)',
          default: 'var(--color-border-default)',
          strong: 'var(--color-border-strong)',
          accent: 'var(--color-border-accent)',
        },
        // 文字色
        text: {
          primary: 'var(--color-text-primary)',
          secondary: 'var(--color-text-secondary)',
          muted: 'var(--color-text-muted)',
          disabled: 'var(--color-text-disabled)',
        },
        // 强调色 (支持透明度)
        accent: {
          primary: 'rgb(var(--color-accent-primary-rgb) / <alpha-value>)',
          secondary: 'rgb(var(--color-accent-secondary-rgb) / <alpha-value>)',
          tertiary: 'rgb(var(--color-info-rgb) / <alpha-value>)',
          success: 'rgb(var(--color-success-rgb) / <alpha-value>)',
          warning: 'rgb(var(--color-warning-rgb) / <alpha-value>)',
          danger: 'rgb(var(--color-danger-rgb) / <alpha-value>)',
          info: 'rgb(var(--color-info-rgb) / <alpha-value>)',
        },
        // 平台专属色
        platform: {
          claude: 'var(--color-platform-claude)',
          codex: 'var(--color-platform-codex)',
          gemini: 'var(--color-platform-gemini)',
          qwen: 'var(--color-platform-qwen)',
          iflow: 'var(--color-platform-iflow)',
        },
        // 状态色
        state: {
          empty: 'var(--color-text-muted)',
          error: 'var(--color-danger)',
          loading: 'var(--color-accent-primary)',
        },
        // Neo 颜色（直接值）
        neo: {
          cyan: '#06B6D4',
          purple: '#8B5CF6',
          success: '#10B981',
          warning: '#F59E0B',
          danger: '#EF4444',
          info: '#3B82F6',
        },
        // 向后兼容: guofeng 映射到 neo
        guofeng: {
          bg: {
            DEFAULT: 'var(--color-bg-base)',
            secondary: 'var(--color-bg-elevated)',
            tertiary: 'var(--color-bg-surface)',
            card: 'var(--color-bg-elevated)',
          },
          ink: {
            DEFAULT: 'var(--color-text-primary)',
            light: 'var(--color-text-secondary)',
            lighter: 'var(--color-text-muted)',
          },
          jade: {
            DEFAULT: 'var(--color-accent-primary)',
            dim: '#0891B2',
          },
          indigo: 'var(--color-accent-secondary)',
          red: {
            DEFAULT: 'var(--color-danger)',
            dim: '#DC2626',
          },
          gold: {
            DEFAULT: 'var(--color-warning)',
            dim: '#D97706',
          },
          blue: {
            DEFAULT: 'var(--color-info)',
            dim: '#2563EB',
          },
          border: 'var(--color-border-default)',
          info: 'var(--color-info)',
          text: {
            primary: 'var(--color-text-primary)',
            secondary: 'var(--color-text-secondary)',
            muted: 'var(--color-text-muted)',
          },
        },
      },

      // ========== 阴影系统 ==========
      boxShadow: {
        'glow-primary': 'var(--glow-primary)',
        'glow-secondary': 'var(--glow-secondary)',
        'glow-success': 'var(--glow-success)',
        'glow-warning': 'var(--glow-warning)',
        'glow-danger': 'var(--glow-danger)',
        'glow-info': 'var(--glow-info)',
      },

      // ========== 动画 ==========
      animation: {
        'fade-in': 'fade-in var(--duration-normal) var(--ease-out) forwards',
        'slide-up': 'slide-up var(--duration-slow) var(--ease-out) forwards',
        'slide-down': 'slide-down var(--duration-slow) var(--ease-out) forwards',
        'scale-in': 'scale-in var(--duration-normal) var(--ease-out) forwards',
        'pulse-subtle': 'pulse-subtle 2s var(--ease-in-out) infinite',
        'pulse-glow': 'pulse-glow 2s var(--ease-in-out) infinite',
        'sidebar-item-enter': 'sidebar-item-enter var(--duration-slow) var(--ease-out) forwards',
        'card-enter': 'card-enter var(--duration-slow) var(--ease-out) forwards',
        'nav-hover': 'nav-hover 0.4s var(--ease-out)',
      },

      // ========== 过渡 ==========
      transitionDuration: {
        'instant': 'var(--duration-instant)',
        'fast': 'var(--duration-fast)',
        'normal': 'var(--duration-normal)',
        'slow': 'var(--duration-slow)',
        'slower': 'var(--duration-slower)',
      },

      transitionTimingFunction: {
        'out': 'var(--ease-out)',
        'in-out': 'var(--ease-in-out)',
        'spring': 'var(--ease-spring)',
        'out-back': 'var(--ease-out-back)',
      },

      // ========== Z-Index ==========
      zIndex: {
        'behind': '-1',
        'dropdown': '10',
        'sticky': '20',
        'fixed': '30',
        'modal-backdrop': '40',
        'modal': '50',
        'popover': '60',
        'tooltip': '70',
        'toast': '80',
      },
    },
  },
  plugins: [],
} satisfies Config
