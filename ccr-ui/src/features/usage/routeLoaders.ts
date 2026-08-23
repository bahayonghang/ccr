export const loadDashboardView = () =>
  import('./dashboard/DashboardView').then((mod) => ({ Component: mod.DashboardView }))

export const loadUsageDashboardView = () =>
  import('./UsageDashboardView').then((mod) => ({ Component: mod.UsageDashboardView }))

export const loadBudgetView = () =>
  import('./budget/BudgetView').then((mod) => ({ Component: mod.BudgetView }))

export const loadPricingView = () =>
  import('./pricing/PricingView').then((mod) => ({ Component: mod.PricingView }))
