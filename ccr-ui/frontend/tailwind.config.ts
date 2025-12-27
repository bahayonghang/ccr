import type { Config } from 'tailwindcss'

export default {
  darkMode: 'class',
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      // 设计令牌 - 间距系统
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
      // 字体系统
      fontSize: {
        'xs': 'var(--font-size-xs)',
        'sm': 'var(--font-size-sm)',
        'base': 'var(--font-size-base)',
        'lg': 'var(--font-size-lg)',
        'xl': 'var(--font-size-xl)',
        '2xl': 'var(--font-size-2xl)',
        '3xl': 'var(--font-size-3xl)',
        '4xl': 'var(--font-size-4xl)',
        '5xl': 'var(--font-size-5xl)',
        '6xl': 'var(--font-size-6xl)',
      },
      fontFamily: {
        'display': ['Inter', 'SF Pro Display', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif'],
        'body': ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif'],
      },
      fontWeight: {
        'light': 'var(--font-weight-light)',
        'normal': 'var(--font-weight-normal)',
        'medium': 'var(--font-weight-medium)',
        'semibold': 'var(--font-weight-semibold)',
        'bold': 'var(--font-weight-bold)',
        'extrabold': 'var(--font-weight-extrabold)',
        'black': 'var(--font-weight-black)',
      },
      lineHeight: {
        'tight': 'var(--line-height-tight)',
        'normal': 'var(--line-height-normal)',
        'relaxed': 'var(--line-height-relaxed)',
        'loose': 'var(--line-height-loose)',
      },
      borderRadius: {
        'xs': 'var(--radius-sm)',
        'sm': 'var(--radius-md)',
        'md': 'var(--radius-lg)',
        'lg': 'var(--radius-xl)',
        'xl': 'var(--radius-2xl)',
        '2xl': 'var(--radius-3xl)',
        'full': 'var(--radius-full)',
      },
      colors: {
        bg: {
          primary: 'var(--bg-primary)',
          secondary: 'var(--bg-secondary)',
          tertiary: 'var(--bg-tertiary)',
          card: 'var(--bg-card)',
          hover: 'var(--bg-hover)',
        },
        border: {
          color: 'var(--border-color)',
          hover: 'var(--border-hover)',
        },
        text: {
          primary: 'var(--text-primary)',
          secondary: 'var(--text-secondary)',
          muted: 'var(--text-muted)',
        },
        accent: {
          primary: 'rgb(var(--accent-primary-rgb) / <alpha-value>)',
          secondary: 'rgb(var(--accent-secondary-rgb) / <alpha-value>)',
          tertiary: 'rgb(var(--accent-tertiary-rgb) / <alpha-value>)',
          success: 'rgb(var(--accent-success-rgb) / <alpha-value>)',
          warning: 'rgb(var(--accent-warning-rgb) / <alpha-value>)',
          danger: 'rgb(var(--accent-danger-rgb) / <alpha-value>)',
          info: 'rgb(var(--accent-info-rgb) / <alpha-value>)',
        },
        platform: {
          codex: 'var(--platform-codex)',
          gemini: 'var(--platform-gemini)',
          claude: 'var(--platform-claude)',
          qwen: 'var(--platform-qwen)',
          iflow: 'var(--platform-iflow)',
        },
        state: {
          empty: 'var(--state-empty)',
          error: 'var(--state-error)',
          loading: 'var(--state-loading)',
        },
        guofeng: {
          bg: {
            DEFAULT: '#fdfbf7', // Rice Paper (Xuan Zhi)
            secondary: '#f5f2e9', // Aged Paper
            tertiary: '#e8e4d9', // Silk
            card: '#ffffff', // White Jade
          },
          ink: {
            DEFAULT: '#1e293b', // Mo Hei (Ink Black)
            light: '#475569', // Light Ink
            lighter: '#94a3b8', // Watery Ink
          },
          red: {
            DEFAULT: '#ff4d4f', // Zhu Hong (Vermilion)
            dim: '#d63031',
          },
          jade: {
            DEFAULT: '#10b981', // Fei Cui (Jade Green)
            dim: '#059669',
          },
          blue: {
            DEFAULT: '#1677b3', // Dian Lan (Indigo)
            dim: '#0e5a8a',
          },
          gold: {
            DEFAULT: '#f59e0b', // Teng Huang (Rattan Yellow)
            dim: '#d97706',
          },
          border: '#e2e8f0',
        }
      }
    },
  },
  plugins: [],
} satisfies Config
