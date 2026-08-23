import { useCallback, useEffect, useState } from 'react'
import { getUsageCapabilitiesV2, getUsageSummaryV2 } from '@/api'
import type { UsageCapabilityReport, UsageFeatureCapability, UsageSummary } from '@/types/usage'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import type { TranslateFunction } from '@/utils/tf'

const toErrorMessage = (error: unknown) => {
  if (error instanceof Error) return error.message
  return String(error)
}

const overviewCapability = (report: UsageCapabilityReport | null | undefined) => {
  return report?.features.overview ?? null
}

const capabilityDetail = (capability: UsageFeatureCapability, t: TranslateFunction) => {
  if (capability.detail) return capability.detail
  if (capability.reason) return t('monitoring.usageUnsupportedReason', { reason: capability.reason })
  return t('monitoring.usageUnavailableDescription')
}

export function useMonitoringUsage(t: TranslateFunction) {
  const [usageSummary, setUsageSummary] = useState<UsageSummary | null>(null)
  const [usageLoading, setUsageLoading] = useState(false)
  const [usageStatus, setUsageStatus] = useState<'idle' | 'ready' | 'unavailable'>('idle')
  const [usageUnavailableDetail, setUsageUnavailableDetail] = useState('')
  const [usageUpdatedAt, setUsageUpdatedAt] = useState<string | null>(null)

  const loadUsageSummary = useCallback(async () => {
    setUsageLoading(true)
    setUsageUnavailableDetail('')
    const stamp = new Date().toISOString()

    if (!isTauriRuntime()) {
      setUsageSummary(null)
      setUsageStatus('unavailable')
      setUsageUnavailableDetail(t('monitoring.usageUnavailableDescription'))
      setUsageUpdatedAt(stamp)
      setUsageLoading(false)
      return
    }

    let capabilityError: unknown = null
    try {
      const capabilities = await getUsageCapabilitiesV2()
      const capability = overviewCapability(capabilities)
      if (capability && !capability.supported) {
        setUsageSummary(null)
        setUsageStatus('unavailable')
        setUsageUnavailableDetail(capabilityDetail(capability, t))
        setUsageUpdatedAt(stamp)
        setUsageLoading(false)
        return
      }
    } catch (error) {
      capabilityError = error
    }

    try {
      const summary = await getUsageSummaryV2()
      if (!summary || typeof summary.total_requests !== 'number') {
        throw new Error('Invalid usage summary payload')
      }
      setUsageSummary(summary)
      setUsageStatus('ready')
      setUsageUnavailableDetail('')
      setUsageUpdatedAt(stamp)
    } catch (error) {
      setUsageSummary(null)
      setUsageStatus('unavailable')
      setUsageUnavailableDetail(
        capabilityError ? `${toErrorMessage(capabilityError)} · ${toErrorMessage(error)}` : toErrorMessage(error),
      )
      setUsageUpdatedAt(stamp)
    } finally {
      setUsageLoading(false)
    }
  }, [t])

  useEffect(() => {
    void loadUsageSummary()
  }, [loadUsageSummary])

  return {
    usageSummary,
    usageLoading,
    usageStatus,
    usageUnavailableDetail,
    usageUpdatedAt,
    loadUsageSummary,
  }
}
