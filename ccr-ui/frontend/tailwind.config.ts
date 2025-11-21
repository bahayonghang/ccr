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
          bg: '#fdfbf7', // Rice paper
          red: '#ff4d4f', // Zhu Hong
          jade: '#10b981', // Fei Cui
          blue: '#1677b3', // Dian Lan
          gold: '#f59e0b', // Teng Huang
          ink: '#1e293b', // Mo Hei
          purple: '#7c3aed', // Zi
        }
      }
    },
  },
  plugins: [],
} satisfies Config
