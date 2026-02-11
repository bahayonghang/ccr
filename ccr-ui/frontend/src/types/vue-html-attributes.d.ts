import 'vue'

declare module 'vue' {
  interface HTMLAttributes {
    [key: `data-${string}`]: unknown
  }
}

export {}
