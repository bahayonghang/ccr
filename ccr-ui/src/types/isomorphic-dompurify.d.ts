declare module 'isomorphic-dompurify' {
  import type { Config } from 'dompurify'

  export interface DOMPurifyLike {
    sanitize(dirty: string, cfg?: Config): string
  }

  const DOMPurify: DOMPurifyLike
  export default DOMPurify
}
