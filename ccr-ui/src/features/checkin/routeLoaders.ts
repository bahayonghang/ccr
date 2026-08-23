export function checkinRouteLoader() {
  return import('./CheckinView').then((mod) => ({ Component: mod.CheckinView }))
}

export function checkinAccountDashboardRouteLoader() {
  return import('./CheckinAccountDashboardView').then((mod) => ({
    Component: mod.CheckinAccountDashboardView,
  }))
}
