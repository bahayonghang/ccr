import type { Config } from 'tailwindcss'

export default {
  darkMode: 'class',
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
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
