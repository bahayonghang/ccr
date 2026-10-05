import type { AccountInfo, CdkExtraConfig } from '@/types/checkin'
import type { AccountFormValues } from '../components/AccountFormFields'
import { extractCookiesFieldValue } from './accountCookies'

export const emptyAccountForm = (providerId: string): AccountFormValues => ({
  provider_id: providerId,
  name: '',
  session: '',
  api_user: '',
  enabled: true,
  fuli_cookies: '',
  b4u_cdk_cookies: '',
  x666_access_token: '',
})

const fromUnknownRecord = (parsed: object): Record<string, string> => {
  const result: Record<string, string> = {}
  for (const [key, value] of Object.entries(parsed as Record<string, unknown>)) {
    if (typeof value === 'string') result[key] = value
  }
  return result
}

const readStringMap = (raw: string): Record<string, string> | null => {
  try {
    const parsed: unknown = JSON.parse(raw)
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return null
    return fromUnknownRecord(parsed)
  } catch {
    return null
  }
}

const nestedMap = (value: unknown): Record<string, string> | undefined => {
  if (!value || typeof value !== 'object') return undefined
  return readStringMap(JSON.stringify(value)) ?? undefined
}

export const parseCdkExtra = (raw?: string): CdkExtraConfig => {
  if (!raw) return {}
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return {}
  }
  if (!parsed || typeof parsed !== 'object') return {}
  const rec = parsed as Record<string, unknown>
  return {
    fuli_cookies: nestedMap(rec.fuli_cookies),
    b4u_cdk_cookies: nestedMap(rec.b4u_cdk_cookies),
    x666_access_token: typeof rec.x666_access_token === 'string' ? rec.x666_access_token : undefined,
  }
}

export const extraFromForm = (
  values: AccountFormValues,
): CdkExtraConfig | 'invalid-fuli' | 'invalid-b4u' => {
  const extraConfig: CdkExtraConfig = {}
  if (values.fuli_cookies) {
    const parsed = readStringMap(values.fuli_cookies)
    if (!parsed) return 'invalid-fuli'
    extraConfig.fuli_cookies = parsed
  }
  if (values.b4u_cdk_cookies) {
    const parsed = readStringMap(values.b4u_cdk_cookies)
    if (!parsed) return 'invalid-b4u'
    extraConfig.b4u_cdk_cookies = parsed
  }
  if (values.x666_access_token) extraConfig.x666_access_token = values.x666_access_token
  return extraConfig
}

export const valuesFromAccount = (input: {
  account: AccountInfo
  extra: CdkExtraConfig
  session: string
  apiUser: string
}): AccountFormValues => ({
  provider_id: input.account.provider_id,
  name: input.account.name,
  session: input.session,
  api_user: input.apiUser,
  enabled: input.account.enabled,
  fuli_cookies: input.extra.fuli_cookies ? JSON.stringify(input.extra.fuli_cookies) : '',
  b4u_cdk_cookies: input.extra.b4u_cdk_cookies ? JSON.stringify(input.extra.b4u_cdk_cookies) : '',
  x666_access_token: input.extra.x666_access_token || '',
})

export const valuesFromCookies = (input: {
  account: AccountInfo
  extra: CdkExtraConfig
  cookiesJson: string
  apiUser?: string | null
}): AccountFormValues =>
  valuesFromAccount({
    account: input.account,
    extra: input.extra,
    session: extractCookiesFieldValue(input.cookiesJson),
    apiUser:
      typeof input.apiUser === 'string' && input.apiUser.trim()
        ? input.apiUser
        : input.account.api_user || '',
  })
