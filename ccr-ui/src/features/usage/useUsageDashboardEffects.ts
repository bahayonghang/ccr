import { useEffect, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import type { UsageFeatureCapability } from '@/types/usage'

const AUTO_REFRESH_MS = 30_000

export function useUsageSnapshotRefresh(refresh: () => Promise<void>) {
  const snapshotUnlisten = useRef<(() => void) | null>(null)
  useEffect(() => {
    if (!isTauriRuntime()) return
    let disposed = false
    void listen('usage:snapshot-updated', () => {
      if (!disposed) void refresh()
    }).then((unlisten) => {
      if (disposed) unlisten()
      else snapshotUnlisten.current = unlisten
    })
    return () => {
      disposed = true
      snapshotUnlisten.current?.()
    }
  }, [refresh])
}

export function useUsageBootstrapImport(input: {
  unsupported: boolean
  hasUsageData: boolean
  isLoading: boolean
  isFetched: boolean
  syncCapability: UsageFeatureCapability | null
  startImportJob: (opts: { reason: 'bootstrap'; recentDays: number }) => Promise<unknown>
}) {
  const attempted = useRef(false)
  const { unsupported, hasUsageData, isLoading, isFetched, syncCapability, startImportJob } = input
  useEffect(() => {
    if (!isTauriRuntime() || unsupported || hasUsageData || attempted.current) return
    if (isLoading || !isFetched) return
    if (syncCapability && !syncCapability.supported) return
    attempted.current = true
    void startImportJob({ reason: 'bootstrap', recentDays: 30 })
  }, [unsupported, hasUsageData, isLoading, isFetched, syncCapability, startImportJob])
}

export function useUsageAutoRefresh(refetch: () => Promise<unknown>) {
  useEffect(() => {
    if (!isTauriRuntime()) return undefined
    const id = window.setInterval(() => {
      if (document.hidden) return
      void refetch()
    }, AUTO_REFRESH_MS)
    return () => window.clearInterval(id)
  }, [refetch])
}
