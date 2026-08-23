export function loadSyncView() {
  return import('./SyncView').then((mod) => ({ Component: mod.SyncView }))
}

export function loadWslManagementView() {
  return import('./WslManagementView').then((mod) => ({ Component: mod.WslManagementView }))
}

export function loadSshManagementView() {
  return import('./SshManagementView').then((mod) => ({ Component: mod.SshManagementView }))
}

export const syncRouteLoaders = {
  sync: loadSyncView,
  'wsl-management': loadWslManagementView,
  wsl: loadWslManagementView,
  'ssh-management': loadSshManagementView,
  ssh: loadSshManagementView,
} as const
