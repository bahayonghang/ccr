import { createBrowserRouter, redirect, type RouteObject } from 'react-router'
import { App } from './App'
import { localeWarmupLoader } from './localeLoader'
import { MainLayout } from './MainLayout'
import { layoutChildCatalog, trayCatalog, type CatalogEntry } from './routeCatalog'
const loadPlaceholder = () =>
  import('./placeholders/RoutePlaceholder').then((mod) => ({ Component: mod.RoutePlaceholder }))

const toChildRoute = (entry: CatalogEntry): RouteObject => {
  if (entry.redirect) {
    const target = entry.redirect
    return {
      path: entry.path,
      id: entry.id,
      handle: entry.handle,
      loader: () => redirect(target),
    }
  }
  return {
    path: entry.path,
    id: entry.id,
    handle: entry.handle,
    lazy: loadPlaceholder,
    loader: localeWarmupLoader,
  }
}

function RouteHydrateFallback() {
  return (
    <div className="flex min-h-[200px] items-center justify-center">
      <div className="loading-spinner h-8 w-8 border-accent-primary/30 border-t-accent-primary" />
    </div>
  )
}

function TrayErrorBoundary() {
  return (
    <div role="alert" className="p-6 text-text-primary">
      托盘面板渲染失败
    </div>
  )
}

export const appRoutes: RouteObject[] = [
  {
    Component: App,
    HydrateFallback: RouteHydrateFallback,
    children: [
      {
        path: 'tray/codex',
        id: trayCatalog.id,
        handle: trayCatalog.handle,
        lazy: loadPlaceholder,
        ErrorBoundary: TrayErrorBoundary,
      },
      {
        path: '/',
        Component: MainLayout,
        HydrateFallback: RouteHydrateFallback,
        children: layoutChildCatalog.map(toChildRoute),
      },
    ],
  },
]

export const router = createBrowserRouter(appRoutes)
