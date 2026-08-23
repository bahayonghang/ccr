export type AddMethod = 'oauth' | 'token' | 'api' | 'local'

export interface AddAccountFormValues {
  preferredAccountName: string
  importContent: string
  importSwitchAfter: boolean
  apiKey: string
  apiBaseUrl: string
  providerName: string
  saveProvider: boolean
  switchAfterAdd: boolean
}

export const ADD_ACCOUNT_DEFAULTS: AddAccountFormValues = {
  preferredAccountName: '',
  importContent: '',
  importSwitchAfter: true,
  apiKey: '',
  apiBaseUrl: '',
  providerName: '',
  saveProvider: false,
  switchAfterAdd: true,
}
