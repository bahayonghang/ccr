declare module 'dompurify' {
  export interface DOMPurifyLike {
    sanitize(dirty: string, cfg?: Record<string, unknown>): string
  }

  const DOMPurify: DOMPurifyLike
  export default DOMPurify
}
