import type {
  AccountInfo,
  BuiltinProvider,
  CheckinDisplayResponse,
  CheckinFlowPhase,
  CheckinLogEntry,
  CheckinProvider,
  CheckinRecordInfo,
  TodayCheckinStats,
} from '@/types/checkin'
import type { CheckinTabId } from './checkinFormat'
import type { CheckinDataBox } from './checkinData'
import type { CheckinJobBox } from './checkinJob'
import type { WafRecoveryBox } from './checkinWafRecovery'

export interface CheckinRuntimeBox extends CheckinDataBox, CheckinJobBox, WafRecoveryBox {
  activeTab: CheckinTabId
  showCheckinConfirm: boolean
  showOAuthWizard: boolean
  pendingEditAccountId: string | null
  balanceRefreshing: boolean
}

export const createCheckinRuntimeBox = (): CheckinRuntimeBox => ({
  loading: false,
  error: null,
  recordsLoadError: null,
  providers: [] as CheckinProvider[],
  accounts: [] as AccountInfo[],
  records: [] as CheckinRecordInfo[],
  todayStats: null as TodayCheckinStats | null,
  builtinProviders: [] as BuiltinProvider[],
  checkinLoading: false,
  checkinResult: null as CheckinDisplayResponse | null,
  checkinResultRef: null,
  showProgressModal: false,
  checkinFlowPhase: 'finished' as CheckinFlowPhase,
  checkinProgress: { total: 0, completed: 0, currentAccountName: '' },
  checkinLogs: [] as CheckinLogEntry[],
  wafRecoveryRunning: false,
  wafRecoveryProviderName: null,
  wafRecoveryMessage: null,
  activeCheckinJobId: null,
  activeTab: 'accounts',
  showCheckinConfirm: false,
  showOAuthWizard: false,
  pendingEditAccountId: null,
  balanceRefreshing: false,
})
