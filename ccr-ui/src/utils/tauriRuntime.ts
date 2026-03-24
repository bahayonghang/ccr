const hasOwn = (key: '__TAURI__' | '__TAURI_INTERNALS__'): boolean => {
  return typeof window !== 'undefined' && key in window
}

export const isTauriRuntime = (): boolean => {
  return hasOwn('__TAURI__') || hasOwn('__TAURI_INTERNALS__')
}

