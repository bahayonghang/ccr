export function loadMonitoringView() {
  return import('./MonitoringView').then((mod) => ({ Component: mod.MonitoringView }))
}

export const monitoringRouteLoaders = {
  monitoring: loadMonitoringView,
} as const
