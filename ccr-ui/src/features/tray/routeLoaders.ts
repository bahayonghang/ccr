export function loadCodexTrayPanel() {
  return import('./CodexTrayPanelView').then((mod) => ({ Component: mod.CodexTrayPanelView }))
}

export const trayRouteLoaders = {
  'codex-tray-panel': loadCodexTrayPanel,
  'tray/codex': loadCodexTrayPanel,
} as const
