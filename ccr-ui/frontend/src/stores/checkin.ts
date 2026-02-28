import { defineStore } from 'pinia'
import { api } from '@/api/core'
import { listCheckinAccounts, listCheckinRecords } from '@/api/client'
import type { AccountInfo, CheckinRecordInfo, CheckinRecordsQuery } from '@/types/checkin'

const CACHE_TTL = 30 * 1000 // 30s

interface DashboardStats {
  consecutive_days: number
  total_days: number
  longest_consecutive: number
  monthly_rate: number
  weekly_rate: number
  total_accounts: number
  checked_in_today: number
  not_checked_in_today: number
}

interface CheckinState {
  stats: DashboardStats | null
  statsLastFetchedAt: number
  statsLoading: boolean
  statsError: string | null

  accounts: AccountInfo[]
  accountsLastFetchedAt: number
  accountsLoading: boolean
  accountsError: string | null

  records: CheckinRecordInfo[]
  recordsTotal: number
  recordsLastFetchedAt: number
  recordsLoading: boolean
  recordsError: string | null
}

export const useCheckinStore = defineStore('checkin', {
  state: (): CheckinState => ({
    stats: null,
    statsLastFetchedAt: 0,
    statsLoading: false,
    statsError: null,

    accounts: [],
    accountsLastFetchedAt: 0,
    accountsLoading: false,
    accountsError: null,

    records: [],
    recordsTotal: 0,
    recordsLastFetchedAt: 0,
    recordsLoading: false,
    recordsError: null,
  }),

  getters: {
    isStatsCacheValid: (state) =>
      Date.now() - state.statsLastFetchedAt < CACHE_TTL && state.stats !== null,
    isAccountsCacheValid: (state) =>
      Date.now() - state.accountsLastFetchedAt < CACHE_TTL && state.accounts.length > 0,
    isRecordsCacheValid: (state) =>
      Date.now() - state.recordsLastFetchedAt < CACHE_TTL && state.records.length > 0,
  },

  actions: {
    async fetchStats(force = false) {
      if (!force && this.isStatsCacheValid) return this.stats
      this.statsLoading = true
      this.statsError = null
      try {
        const response = await api.get<DashboardStats>('/checkin/dashboard/stats')
        this.stats = response.data
        this.statsLastFetchedAt = Date.now()
        return this.stats
      } catch (err: unknown) {
        this.statsError = err instanceof Error ? err.message : '加载统计失败'
        throw err
      } finally {
        this.statsLoading = false
      }
    },

    async fetchAccounts(force = false) {
      if (!force && this.isAccountsCacheValid) return this.accounts
      this.accountsLoading = true
      this.accountsError = null
      try {
        const response = await listCheckinAccounts()
        this.accounts = response.accounts
        this.accountsLastFetchedAt = Date.now()
        return this.accounts
      } catch (err: unknown) {
        this.accountsError = err instanceof Error ? err.message : '加载账号失败'
        throw err
      } finally {
        this.accountsLoading = false
      }
    },

    async fetchRecords(params?: CheckinRecordsQuery, force = false) {
      if (!force && this.isRecordsCacheValid) return this.records
      this.recordsLoading = true
      this.recordsError = null
      try {
        const query = params || { page: 1, page_size: 20 }
        const response = await listCheckinRecords(query)
        this.records = response.records
        this.recordsTotal = response.total
        this.recordsLastFetchedAt = Date.now()
        return this.records
      } catch (err: unknown) {
        this.recordsError = err instanceof Error ? err.message : '加载记录失败'
        throw err
      } finally {
        this.recordsLoading = false
      }
    },

    invalidate() {
      this.statsLastFetchedAt = 0
      this.accountsLastFetchedAt = 0
      this.recordsLastFetchedAt = 0
    },
  },
})
