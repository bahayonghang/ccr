import { defineStore } from 'pinia'
import { getErrorMessage } from '@/utils/errorHandler'
import { ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import {
  ensureSessionIndexV2,
  getUsageCapabilitiesV2,
  getHomeUsageOverviewV2,
  getSessionIndexJobStatusV2,
  getUsageImportJobStatusV2,
  startUsageImportJobV2,
} from '@/api'
import { logger } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import type {
  HomeUsageOverviewResponse,
  UsageCapabilityReport,
  SessionIndexJobSnapshot,
  StartSessionIndexJobResponse,
  StartUsageImportJobResponse,
  UsageImportJobSnapshot,
  UsageSnapshotUpdatedPayload,
} from '@/types/usage'

const OVERVIEW_CACHE_TTL_MS = 30_000
const HOME_WARMUP_RETRY_COOLDOWN_MS = 20_000

type LoadOptions = {
  force?: boolean
  background?: boolean
}

export const useHomeUsageOverviewStore = defineStore('homeUsageOverview', () => {
  const overview = ref<HomeUsageOverviewResponse | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const activeDays = ref(30)
  const usageWarmupRunning = ref(false)
  const sessionWarmupRunning = ref(false)
  const lastSessionWarmupIndexed = ref(0)
  const currentUsageJob = ref<UsageImportJobSnapshot | null>(null)
  const currentSessionJob = ref<SessionIndexJobSnapshot | null>(null)
  const usageCapabilities = ref<UsageCapabilityReport | null>(null)

  const overviewCache = new Map<number, { data: HomeUsageOverviewResponse; ts: number }>()

  let activeUsageJobId: string | null = null
  let activeSessionJobId: string | null = null
  let usageJobUnlisteners: UnlistenFn[] = []
  let sessionJobUnlisteners: UnlistenFn[] = []
  let usageSnapshotUnlistener: UnlistenFn | null = null
  let usageWarmupLastAttemptAt = 0
  let sessionWarmupLastAttemptAt = 0
  let retryProbeTimer: ReturnType<typeof setTimeout> | null = null

  const shouldRetryWarmup = (lastAttemptAt: number) =>
    Date.now() - lastAttemptAt >= HOME_WARMUP_RETRY_COOLDOWN_MS

  const clearRetryProbe = () => {
    if (retryProbeTimer !== null) {
      clearTimeout(retryProbeTimer)
      retryProbeTimer = null
    }
  }

  const clearUsageJobListeners = async () => {
    const unlisteners = usageJobUnlisteners
    usageJobUnlisteners = []
    await Promise.all(unlisteners.map((unlisten) => unlisten()))
  }

  const clearSessionJobListeners = async () => {
    const unlisteners = sessionJobUnlisteners
    sessionJobUnlisteners = []
    await Promise.all(unlisteners.map((unlisten) => unlisten()))
  }

  const invalidate = (days?: number) => {
    if (days == null) {
      overviewCache.clear()
      return
    }
    overviewCache.delete(days)
  }

  const ensureUsageSnapshotListener = async () => {
    if (!isTauriRuntime() || usageSnapshotUnlistener) return

    usageSnapshotUnlistener = await listen<UsageSnapshotUpdatedPayload>(
      'usage:snapshot-updated',
      () => {
        invalidate()
        if (!overview.value) return
        void loadOverview(activeDays.value, { force: true, background: true }).catch((loadError) => {
          logger.error('[home-usage-overview] snapshot refresh failed', loadError)
        })
      },
    )
  }

  const refreshActiveOverview = async () => {
    invalidate()
    await loadOverview(activeDays.value, { force: true, background: true })
  }

  const scheduleRetryProbe = () => {
    if (!isTauriRuntime() || retryProbeTimer !== null) return

    retryProbeTimer = setTimeout(() => {
      retryProbeTimer = null
      invalidate()
      void loadOverview(activeDays.value, { force: true, background: true }).catch((loadError) => {
        logger.error('[home-usage-overview] retry probe failed', loadError)
      })
    }, HOME_WARMUP_RETRY_COOLDOWN_MS)
  }

  const handleUsageJobSnapshot = async (snapshot: UsageImportJobSnapshot) => {
    if (snapshot.job_id !== activeUsageJobId) return

    currentUsageJob.value = snapshot
    usageWarmupRunning.value = snapshot.status !== 'finished' && snapshot.status !== 'failed' && snapshot.status !== 'cancelled'

    if (snapshot.status === 'recent_ready' || snapshot.status === 'finished') {
      await refreshActiveOverview()
    }

    if (snapshot.status === 'finished' || snapshot.status === 'failed' || snapshot.status === 'cancelled') {
      usageWarmupRunning.value = false
      activeUsageJobId = null
      await clearUsageJobListeners()
      if (snapshot.status === 'failed') {
        scheduleRetryProbe()
      }
    }
  }

  const handleSessionJobSnapshot = async (snapshot: SessionIndexJobSnapshot) => {
    if (snapshot.job_id !== activeSessionJobId) return

    currentSessionJob.value = snapshot
    sessionWarmupRunning.value = snapshot.status !== 'finished' && snapshot.status !== 'failed'
    lastSessionWarmupIndexed.value = snapshot.sessions_added + snapshot.sessions_updated

    if (snapshot.status === 'finished') {
      await refreshActiveOverview()
    }

    if (snapshot.status === 'finished' || snapshot.status === 'failed') {
      sessionWarmupRunning.value = false
      activeSessionJobId = null
      await clearSessionJobListeners()
      if (snapshot.status === 'failed') {
        scheduleRetryProbe()
      }
    }
  }

  const trackUsageImportJob = async (jobId: string) => {
    if (!isTauriRuntime()) return
    if (activeUsageJobId === jobId && usageJobUnlisteners.length > 0) return

    await clearUsageJobListeners()
    activeUsageJobId = jobId
    usageWarmupRunning.value = true

    usageJobUnlisteners = await Promise.all([
      listen<UsageImportJobSnapshot>('usage:job-progress', (event) => {
        void handleUsageJobSnapshot(event.payload)
      }),
      listen<UsageImportJobSnapshot>('usage:job-recent-ready', (event) => {
        void handleUsageJobSnapshot(event.payload)
      }),
      listen<UsageImportJobSnapshot>('usage:job-finished', (event) => {
        void handleUsageJobSnapshot(event.payload)
      }),
      listen<UsageImportJobSnapshot>('usage:job-failed', (event) => {
        void handleUsageJobSnapshot(event.payload)
      }),
    ])

    const latest = await getUsageImportJobStatusV2<UsageImportJobSnapshot>(jobId)
    await handleUsageJobSnapshot(latest)
  }

  const trackSessionIndexJob = async (jobId: string) => {
    if (!isTauriRuntime()) return
    if (activeSessionJobId === jobId && sessionJobUnlisteners.length > 0) return

    await clearSessionJobListeners()
    activeSessionJobId = jobId
    sessionWarmupRunning.value = true

    sessionJobUnlisteners = await Promise.all([
      listen<SessionIndexJobSnapshot>('usage:session-index-progress', (event) => {
        void handleSessionJobSnapshot(event.payload)
      }),
      listen<SessionIndexJobSnapshot>('usage:session-index-finished', (event) => {
        void handleSessionJobSnapshot(event.payload)
      }),
      listen<SessionIndexJobSnapshot>('usage:session-index-failed', (event) => {
        void handleSessionJobSnapshot(event.payload)
      }),
    ])

    const latest = await getSessionIndexJobStatusV2<SessionIndexJobSnapshot>(jobId)
    await handleSessionJobSnapshot(latest)
  }

  const maybeWarmOverview = async (data: HomeUsageOverviewResponse, days: number) => {
    if (!isTauriRuntime()) return

    if (usageCapabilities.value) {
      const overviewCap = usageCapabilities.value.features.home_overview
      const syncCap = usageCapabilities.value.features.sync_json_events
      if (overviewCap && !overviewCap.supported) return
      if (syncCap && !syncCap.supported && data.bootstrap.needs_usage_import) return
    }

    if (data.bootstrap.is_warm) {
      clearRetryProbe()
      return
    }

    if (data.bootstrap.usage_job_id) {
      await trackUsageImportJob(data.bootstrap.usage_job_id)
    } else if (
      data.bootstrap.needs_usage_import &&
      !usageWarmupRunning.value &&
      shouldRetryWarmup(usageWarmupLastAttemptAt)
    ) {
      usageWarmupLastAttemptAt = Date.now()
      try {
        const response = await startUsageImportJobV2<StartUsageImportJobResponse>(
          undefined,
          days,
          undefined,
        )
        await trackUsageImportJob(response.job_id)
      } catch (warmupError) {
        logger.error('[home-usage-overview] failed to start usage warmup', warmupError)
        scheduleRetryProbe()
      }
    }

    if (data.bootstrap.session_job_id) {
      await trackSessionIndexJob(data.bootstrap.session_job_id)
    } else if (
      data.bootstrap.needs_session_index &&
      !sessionWarmupRunning.value &&
      shouldRetryWarmup(sessionWarmupLastAttemptAt)
    ) {
      sessionWarmupLastAttemptAt = Date.now()
      try {
        const response = await ensureSessionIndexV2<StartSessionIndexJobResponse>()
        await trackSessionIndexJob(response.job_id)
      } catch (warmupError) {
        logger.error('[home-usage-overview] failed to start session warmup', warmupError)
        scheduleRetryProbe()
      }
    }

    if (
      (data.bootstrap.needs_usage_import && !usageWarmupRunning.value)
      || (data.bootstrap.needs_session_index && !sessionWarmupRunning.value)
    ) {
      scheduleRetryProbe()
    } else {
      clearRetryProbe()
    }
  }

  async function loadOverview(days: number, options: LoadOptions = {}) {
    await ensureUsageSnapshotListener()
    activeDays.value = days
    const force = options.force ?? false
    const background = options.background ?? false
    const hadData = overview.value !== null
    const cached = overviewCache.get(days)

    if (!force && cached && Date.now() - cached.ts < OVERVIEW_CACHE_TTL_MS) {
      overview.value = cached.data
      error.value = null
      loading.value = false
      await maybeWarmOverview(cached.data, days)
      return cached.data
    }

    if (!background || !hadData) {
      loading.value = true
    }
    if (!background) {
      error.value = null
    }

    try {
      if (isTauriRuntime()) {
        usageCapabilities.value = await getUsageCapabilitiesV2<UsageCapabilityReport>()
      }
      const data = await getHomeUsageOverviewV2<HomeUsageOverviewResponse>(days)
      overview.value = data
      error.value = null
      overviewCache.set(days, { data, ts: Date.now() })
      await maybeWarmOverview(data, days)
      return data
    } catch (loadError) {
      if (!background || !hadData) {
        error.value = getErrorMessage(loadError)
      }
      throw loadError
    } finally {
      if (!background || !hadData) {
        loading.value = false
      }
    }
  }

  async function teardown() {
    activeUsageJobId = null
    activeSessionJobId = null
    usageWarmupRunning.value = false
    sessionWarmupRunning.value = false
    clearRetryProbe()
    await clearUsageJobListeners()
    await clearSessionJobListeners()
    if (usageSnapshotUnlistener) {
      await usageSnapshotUnlistener()
      usageSnapshotUnlistener = null
    }
  }

  return {
    overview,
    loading,
    error,
    activeDays,
    usageWarmupRunning,
    sessionWarmupRunning,
    lastSessionWarmupIndexed,
    currentUsageJob,
    currentSessionJob,
    usageCapabilities,
    loadOverview,
    invalidate,
    teardown,
  }
})
