/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: '#8b5cf6',
          dark: '#6d28d9',
        },
        secondary: {
          DEFAULT: '#a855f7',
          dark: '#7e22ce',
        },
      },
    },
  },
  plugins: [],
}

