export type OAuthStep = 0 | 1 | 2 | 3
export type OAuthType = 'github' | 'linuxdo'

export interface OAuthWizardState {
  step: OAuthStep
  selectedProviderId: string
  selectedOAuthType: OAuthType
  loading: boolean
  oauthError: string
  authorizeUrl: string
  extractionGuide: string[]
  copied: boolean
  parseError: string
  creatingAccount: boolean
  createSuccess: boolean
  createError: string
}

export const initialOAuthWizardState = (): OAuthWizardState => ({
  step: 0,
  selectedProviderId: '',
  selectedOAuthType: 'linuxdo',
  loading: false,
  oauthError: '',
  authorizeUrl: '',
  extractionGuide: [],
  copied: false,
  parseError: '',
  creatingAccount: false,
  createSuccess: false,
  createError: '',
})

export type OAuthWizardAction =
  | { type: 'RESET' }
  | { type: 'SELECT_PROVIDER'; id: string; oauthType: OAuthType }
  | { type: 'SELECT_OAUTH_TYPE'; oauthType: OAuthType }
  | { type: 'FETCH_URL_START' }
  | { type: 'FETCH_URL_SUCCESS'; url: string; guide: string[] }
  | { type: 'FETCH_URL_ERROR'; message: string }
  | { type: 'BACK' }
  | { type: 'GOTO_CREDENTIALS' }
  | { type: 'PARSE_ERROR'; message: string }
  | { type: 'CLEAR_PARSE_ERROR' }
  | { type: 'GOTO_CONFIRM' }
  | { type: 'CREATE_START' }
  | { type: 'CREATE_SUCCESS' }
  | { type: 'CREATE_ERROR'; message: string }
  | { type: 'COPIED' }
  | { type: 'CLEAR_COPIED' }

const applyNav = (state: OAuthWizardState, action: OAuthWizardAction): OAuthWizardState => {
  if (action.type === 'RESET') return initialOAuthWizardState()
  if (action.type === 'SELECT_PROVIDER') {
    return { ...state, selectedProviderId: action.id, selectedOAuthType: action.oauthType }
  }
  if (action.type === 'SELECT_OAUTH_TYPE') {
    return { ...state, selectedOAuthType: action.oauthType }
  }
  if (action.type === 'BACK') {
    if (state.createSuccess || state.step === 0) return state
    return { ...state, step: (state.step - 1) as OAuthStep, parseError: '', createError: '' }
  }
  if (action.type === 'GOTO_CREDENTIALS') {
    if (!state.authorizeUrl) return state
    return { ...state, step: 2, parseError: '' }
  }
  if (action.type === 'GOTO_CONFIRM') {
    return { ...state, step: 3, parseError: '', createError: '' }
  }
  return state
}

const applyFetch = (state: OAuthWizardState, action: OAuthWizardAction): OAuthWizardState => {
  if (action.type === 'FETCH_URL_START') {
    return {
      ...state,
      step: 1,
      loading: true,
      oauthError: '',
      authorizeUrl: '',
      extractionGuide: [],
    }
  }
  if (action.type === 'FETCH_URL_SUCCESS') {
    if (state.step !== 1) return state
    return {
      ...state,
      loading: false,
      authorizeUrl: action.url,
      extractionGuide: action.guide,
      oauthError: '',
    }
  }
  if (action.type === 'FETCH_URL_ERROR') {
    if (state.step !== 1) return state
    return { ...state, loading: false, oauthError: action.message, authorizeUrl: '' }
  }
  if (action.type === 'COPIED') return { ...state, copied: true }
  if (action.type === 'CLEAR_COPIED') return { ...state, copied: false }
  return state
}

const applyCreate = (state: OAuthWizardState, action: OAuthWizardAction): OAuthWizardState => {
  if (action.type === 'PARSE_ERROR') return { ...state, parseError: action.message }
  if (action.type === 'CLEAR_PARSE_ERROR') return { ...state, parseError: '' }
  if (action.type === 'CREATE_START') return { ...state, creatingAccount: true, createError: '' }
  if (action.type === 'CREATE_SUCCESS') {
    return { ...state, creatingAccount: false, createSuccess: true }
  }
  if (action.type === 'CREATE_ERROR') {
    return { ...state, creatingAccount: false, createError: action.message }
  }
  return state
}

export function oauthWizardReducer(
  state: OAuthWizardState,
  action: OAuthWizardAction,
): OAuthWizardState {
  const fromNav = applyNav(state, action)
  if (fromNav !== state) return fromNav
  const fromFetch = applyFetch(state, action)
  if (fromFetch !== state) return fromFetch
  return applyCreate(state, action)
}
