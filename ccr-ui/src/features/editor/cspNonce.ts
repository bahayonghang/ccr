/** 读取页面 CSP nonce，供 CodeMirror 运行时 stylesheet 注入。 */
export function readPageCspNonce(): string | undefined {
  if (typeof document === 'undefined') return undefined
  return (
    document.querySelector<HTMLStyleElement>('style[nonce]')?.nonce
    || document.querySelector<HTMLScriptElement>('script[nonce]')?.nonce
  )
}
