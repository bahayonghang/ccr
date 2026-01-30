export const tokens = {
  colors: {
    // Base Backgrounds
    bg: {
      base: 'var(--color-bg-base)',         // Main app background
      elevated: 'var(--color-bg-elevated)', // Cards, Modals
      surface: 'var(--color-bg-surface)',   // Inputs, secondary areas
      overlay: 'var(--color-bg-overlay)',   // Hover states, dropdowns
    },
    // Text Hierarchy
    text: {
      primary: 'var(--color-text-primary)',
      secondary: 'var(--color-text-secondary)',
      muted: 'var(--color-text-muted)',
      ghost: 'var(--color-text-ghost)',
      disabled: 'var(--color-text-disabled)',
      inverted: 'var(--color-text-inverted)', // Text on accent backgrounds
    },
    // Accent Colors (Brand & Semantic)
    accent: {
      primary: 'var(--color-accent-primary)',     // Main brand color (e.g., Electric Blue)
      secondary: 'var(--color-accent-secondary)', // Secondary brand (e.g., Neon Purple)
      success: 'var(--color-success)',
      warning: 'var(--color-warning)',
      danger: 'var(--color-danger)',
      info: 'var(--color-info)',
    },
    // Border Colors
    border: {
      subtle: 'var(--color-border-subtle)',
      default: 'var(--color-border-default)',
      strong: 'var(--color-border-strong)',
      interactive: 'var(--color-border-interactive)', // Focus rings, active states
    },
    // Static / Raw Colors (for charts or specific overrides)
    static: {
      white: '#ffffff',
      black: '#000000',
      transparent: 'transparent',
    }
  },

  // Depth & Elevation
  shadows: {
    none: 'none',
    sm: 'var(--shadow-sm)',
    md: 'var(--shadow-md)',
    lg: 'var(--shadow-lg)',
    xl: 'var(--shadow-xl)',
    glow: {
      primary: 'var(--shadow-glow-primary)',
      success: 'var(--shadow-glow-success)',
      danger: 'var(--shadow-glow-danger)',
    },
  },

  // Spacing Scale (Fluid)
  spacing: {
    0: '0px',
    px: '1px',
    1: 'var(--space-1)', // 4px
    2: 'var(--space-2)', // 8px
    3: 'var(--space-3)', // 12px
    4: 'var(--space-4)', // 16px
    5: 'var(--space-5)', // 20px
    6: 'var(--space-6)', // 24px
    8: 'var(--space-8)', // 32px
    10: 'var(--space-10)', // 40px
    12: 'var(--space-12)', // 48px
    16: 'var(--space-16)', // 64px
    20: 'var(--space-20)', // 80px
  },

  // Border Radius
  borderRadius: {
    none: '0px',
    sm: 'var(--radius-sm)',
    md: 'var(--radius-md)',
    lg: 'var(--radius-lg)',
    full: '9999px',
  },

  // Animation & Motion
  transition: {
    fast: 'var(--duration-fast)',
    normal: 'var(--duration-normal)',
    slow: 'var(--duration-slow)',
  },
  ease: {
    default: 'var(--ease-default)',
    in: 'var(--ease-in)',
    out: 'var(--ease-out)',
    spring: 'var(--ease-spring)',
  },

  // Typography
  fontFamily: {
    sans: 'var(--font-sans)',
    mono: 'var(--font-mono)',
  },
  fontSize: {
    xs: 'var(--text-xs)',
    sm: 'var(--text-sm)',
    base: 'var(--text-base)',
    lg: 'var(--text-lg)',
    xl: 'var(--text-xl)',
    '2xl': 'var(--text-2xl)',
    '3xl': 'var(--text-3xl)',
  },
} as const;

export type DesignTokens = typeof tokens;
