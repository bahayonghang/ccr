export interface OpenCodeProviderFormValues {
  id: string
  name: string
  npm: string
  apiKey: string
  baseURL: string
  enabled: boolean
  modelsJson: string
  extraOptionsJson: string
  rootExtraJson: string
}

export const emptyProviderForm = (): OpenCodeProviderFormValues => ({
  id: '',
  name: '',
  npm: '',
  apiKey: '',
  baseURL: '',
  enabled: true,
  modelsJson: '{}',
  extraOptionsJson: '{}',
  rootExtraJson: '{}',
})
