/** 读取 Tauri 页面 nonce，供运行时 stylesheet 使用。 */
export function readPageCspNonce(): string | undefined {
  if (typeof document === 'undefined') return undefined
  return (
    document.querySelector<HTMLStyleElement>('style[nonce]')?.nonce
    || document.querySelector<HTMLScriptElement>('script[nonce]')?.nonce
  )
}

/** 在 React 挂载前为 Radix 的 react-style-singleton 设置 get-nonce 兼容入口。 */
export function initializeRuntimeStyleNonce(): void {
  const nonce = readPageCspNonce()
  if (nonce) Object.assign(globalThis, { __webpack_nonce__: nonce })
}
