export function checkinRouteLoader() {
  return import('./CheckinView').then((mod) => ({ Component: mod.CheckinView }))
}

export function checkinAccountDashboardRouteLoader() {
  return import('./CheckinAccountDashboardView').then((mod) => ({
    Component: mod.CheckinAccountDashboardView,
  }))
}

/** checkin 域懒加载表。key 对齐 routeCatalog id。 */
export const checkinRouteLoaders = {
  checkin: checkinRouteLoader,
  'checkin-account-dashboard': checkinAccountDashboardRouteLoader,
} as const
