import { createBrowserRouter, redirect, type LazyRouteFunction, type RouteObject } from 'react-router'
import { checkinRouteLoaders } from '@/features/checkin/routeLoaders'
import { claudeRouteLoaders } from '@/features/claude/routeLoaders'
import { codexRouteLoaders } from '@/features/codex/routeLoaders'
import { commandsRouteLoaders } from '@/features/commands/routeLoaders'
import { configsRouteLoaders } from '@/features/configs/routeLoaders'
import { geminiRouteLoaders } from '@/features/gemini/routeLoaders'
import { grokRouteLoaders } from '@/features/grok/routeLoaders'
import { mcpRouteLoaders } from '@/features/mcp/routeLoaders'
import { monitoringRouteLoaders } from '@/features/monitoring/routeLoaders'
import { opencodeRouteLoaders } from '@/features/opencode/routeLoaders'
import { syncRouteLoaders } from '@/features/sync/routeLoaders'
import { trayRouteLoaders } from '@/features/tray/routeLoaders'
import { usageRouteLoaders } from '@/features/usage/routeLoaders'
import { App } from './App'
import { localeWarmupLoader } from './localeLoader'
import { MainLayout } from './MainLayout'
import { layoutChildCatalog, trayCatalog, type CatalogEntry } from './routeCatalog'

type LazyLoader = LazyRouteFunction<RouteObject>

const loadPlaceholder = () =>
  import('./placeholders/RoutePlaceholder').then((mod) => ({ Component: mod.RoutePlaceholder }))

// 后写覆盖短别名：settings→configs，slash-commands→commands，agents→gemini。
const lazyById: Record<string, LazyLoader> = {
  ...claudeRouteLoaders,
  ...codexRouteLoaders,
  ...grokRouteLoaders,
  ...geminiRouteLoaders,
  ...opencodeRouteLoaders,
  ...usageRouteLoaders,
  ...configsRouteLoaders,
  ...commandsRouteLoaders,
  ...syncRouteLoaders,
  ...monitoringRouteLoaders,
  ...mcpRouteLoaders,
  ...trayRouteLoaders,
  ...checkinRouteLoaders,
}

const resolveLazy = (id: string | undefined): LazyLoader =>
  (id ? lazyById[id] : undefined) ?? loadPlaceholder

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
    lazy: resolveLazy(entry.id),
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
        lazy: resolveLazy(trayCatalog.id),
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
