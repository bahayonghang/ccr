export const loadDashboardView = () =>
  import('./dashboard/DashboardView').then((mod) => ({ Component: mod.DashboardView }))

export const loadUsageDashboardView = () =>
  import('./UsageDashboardView').then((mod) => ({ Component: mod.UsageDashboardView }))

export const loadBudgetView = () =>
  import('./budget/BudgetView').then((mod) => ({ Component: mod.BudgetView }))

export const loadPricingView = () =>
  import('./pricing/PricingView').then((mod) => ({ Component: mod.PricingView }))

/** usage 域懒加载表。key 对齐 routeCatalog id。 */
export const usageRouteLoaders = {
  dashboard: loadDashboardView,
  usage: loadUsageDashboardView,
  budget: loadBudgetView,
  pricing: loadPricingView,
} as const
