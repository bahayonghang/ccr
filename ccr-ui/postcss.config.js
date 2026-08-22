// Tailwind v4：经 @tailwindcss/postcss 插件接入；autoprefixer 由 v4 内置的
// lightningcss 前缀能力取代，不再单独挂载。
export default {
  plugins: {
    '@tailwindcss/postcss': {},
  },
}