import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { claudeObserver as api } from '@/api/tauri'
import type {
  BreakdownRow,
  CacheStatsDto,
  DailyPoint,
  HeatmapCell,
  InsightDto,
  SessionRow,
  SubscriptionDto,
  TopToolRow,
} from '@/types/claudeObserver'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { logger } from '@/utils/logger'

/*
 * ========================================================================
 * Pinia store: claudeObserver
 * ========================================================================
 * 目标：
 *   缓存 Claude Observer 面板所需的全部数据切片，统一 loading / error 形态。
 * 数据源：
 *   1) Tauri command claude_observer_* 族（见 src/api/tauri.ts）。
 *   2) Tauri event `claude_observer:updated` —— P4 才会真正发，
 *      此处只做幂等订阅，缺事件不影响功能。
 * 操作要点：
 *   1) fetchAll() 首屏拉取 insight / daily / breakdown / cache / behavior。
 *   2) updateSubscription() 写订阅，写完刷新 insight。
 *   3) 通过 disposeEventListener() 给调用方机会卸载监听。
 */

type Slice<T> = {
  loading: boolean
  error: string | null
  data: T | null
}

const emptySlice = <T>(): Slice<T> => ({
  loading: false,
  error: null,
  data: null,
})

const toMessage = (err: unknown): string => {
  if (err instanceof Error) return err.message
  if (typeof err === 'string') return err
  try {
    return JSON.stringify(err)
  } catch {
    return String(err)
  }
}

export const useClaudeObserverStore = defineStore('claudeObserver', () => {
  /*
   * ========================================================================
   * 步骤1：定义状态切片
   * ========================================================================
   * 数据：
   * 1) insight —— 首屏 Hero 三卡 + 订阅 banner 聚合体
   * 2) cacheStats / daily / costByProject / costByModel —— Cost / Token tab
   * 3) heatmap / topTools / topSessions —— Behavior tab
   */
  const insight = ref<Slice<InsightDto>>(emptySlice())
  const subscription = ref<Slice<SubscriptionDto>>(emptySlice())
  const cacheStats = ref<Slice<CacheStatsDto>>(emptySlice())
  const daily = ref<Slice<DailyPoint[]>>(emptySlice())
  const costByProject = ref<Slice<BreakdownRow[]>>(emptySlice())
  const costByModel = ref<Slice<BreakdownRow[]>>(emptySlice())
  const heatmap = ref<Slice<HeatmapCell[]>>(emptySlice())
  const topTools = ref<Slice<TopToolRow[]>>(emptySlice())
  const topSessions = ref<Slice<SessionRow[]>>(emptySlice())

  /*
   * ========================================================================
   * 步骤2：通用包装器
   * ========================================================================
   * 目标：把 loading / error / data 状态切换收敛到一个函数。
   */
  const runWith = async <T>(slot: { value: Slice<T> }, loader: () => Promise<T>) => {
    if (!isTauriRuntime()) {
      // 1.1 非 Tauri 运行时直接置空，避免 invoke() 抛错被记成 error
      slot.value = { loading: false, error: null, data: null }
      return
    }
    slot.value = { loading: true, error: null, data: slot.value.data }
    try {
      const data = await loader()
      slot.value = { loading: false, error: null, data }
    } catch (err) {
      slot.value = { loading: false, error: toMessage(err), data: slot.value.data }
      logger.warn('[claudeObserver] load failed', err)
    }
  }

  /*
   * ========================================================================
   * 步骤3：动作
   * ========================================================================
   * 1) fetchInsight() —— 仅刷 Hero
   * 2) fetchAll() —— 一次性把面板用到的数据全部拉一遍
   * 3) updateSubscription() —— 写订阅，写完刷 Hero
   */
  const fetchInsight = async () => {
    await runWith(insight, () => api.getInsight('today'))
    // 3.1 同步 subscription 切片，方便对话框直接读
    if (insight.value.data) {
      subscription.value = {
        loading: false,
        error: null,
        data: insight.value.data.subscription,
      }
    }
  }

  const fetchCacheStats = async () => {
    await runWith(cacheStats, () => api.cacheStats())
  }

  const fetchDaily = async (days = 30) => {
    await runWith(daily, () => api.dailyTrend(days))
  }

  const fetchCostByProject = async (days = 30, limit = 10) => {
    await runWith(costByProject, () => api.costBreakdown('project', days, limit))
  }

  const fetchCostByModel = async (days = 30, limit = 10) => {
    await runWith(costByModel, () => api.costBreakdown('model', days, limit))
  }

  const fetchHeatmap = async (days = 30) => {
    await runWith(heatmap, () => api.toolHeatmap(days))
  }

  const fetchTopTools = async (days = 30, limit = 10) => {
    await runWith(topTools, () => api.topTools(days, limit))
  }

  const fetchTopSessions = async (limit = 10) => {
    await runWith(topSessions, () => api.topSessions(limit, 'cost'))
  }

  const fetchAll = async () => {
    logger.info('[claudeObserver] fetchAll 开始')
    // 3.2 并发拉取，互不阻塞
    await Promise.all([
      fetchInsight(),
      fetchCacheStats(),
      fetchDaily(30),
      fetchCostByProject(30, 10),
      fetchCostByModel(30, 10),
      fetchHeatmap(30),
      fetchTopTools(30, 10),
      fetchTopSessions(10),
    ])
    logger.info('[claudeObserver] fetchAll 完成')
  }

  const updateSubscription = async (payload: {
    mode: string
    plan: string
    monthly_usd: number
  }) => {
    if (!isTauriRuntime()) return
    subscription.value = { loading: true, error: null, data: subscription.value.data }
    try {
      const next = await api.subscriptionSet(payload.mode, payload.plan, payload.monthly_usd)
      subscription.value = { loading: false, error: null, data: next }
      // 3.3 同步刷新 insight，让 banner / ROI 一起更新
      await fetchInsight()
    } catch (err) {
      subscription.value = {
        loading: false,
        error: toMessage(err),
        data: subscription.value.data,
      }
      logger.warn('[claudeObserver] updateSubscription failed', err)
      throw err
    }
  }

  /*
   * ========================================================================
   * 步骤4：事件订阅
   * ========================================================================
   * 订阅 `claude_observer:updated`，收到事件就重新 fetchAll。
   * P4 尚未发该事件时，listen 也不会报错。
   */
  let unlistenFn: UnlistenFn | null = null
  let listenerStarted = false

  const startEventListener = async () => {
    if (!isTauriRuntime() || listenerStarted) return
    listenerStarted = true
    try {
      unlistenFn = await listen('claude_observer:updated', () => {
        logger.info('[claudeObserver] received claude_observer:updated, refetching')
        void fetchAll()
      })
    } catch (err) {
      // 4.1 监听失败不致命，仅记录
      listenerStarted = false
      logger.warn('[claudeObserver] failed to subscribe claude_observer:updated', err)
    }
  }

  const disposeEventListener = async () => {
    if (unlistenFn) {
      try {
        unlistenFn()
      } catch (err) {
        logger.warn('[claudeObserver] failed to unlisten claude_observer:updated', err)
      }
    }
    unlistenFn = null
    listenerStarted = false
  }

  return {
    insight,
    subscription,
    cacheStats,
    daily,
    costByProject,
    costByModel,
    heatmap,
    topTools,
    topSessions,
    fetchInsight,
    fetchCacheStats,
    fetchDaily,
    fetchCostByProject,
    fetchCostByModel,
    fetchHeatmap,
    fetchTopTools,
    fetchTopSessions,
    fetchAll,
    updateSubscription,
    startEventListener,
    disposeEventListener,
  }
})
