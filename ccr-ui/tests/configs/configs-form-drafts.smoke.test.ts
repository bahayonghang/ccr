import { beforeEach, describe, expect, it } from 'vitest'
import { emptyConfigForm, isConfigFormDraft } from '@/features/configs/lib/configForm'
import { NEW_CONFIG_DRAFT_ID } from '@/features/configs/types'
import { useConfigsViewStore } from '@/features/configs/stores'

describe('configs form drafts (AC11)', () => {
  beforeEach(() => {
    useConfigsViewStore.setState(useConfigsViewStore.getInitialState())
  })

  it('keeps add-config draft after leave and restore', () => {
    const draft = { ...emptyConfigForm(), name: 'relay', auth_token: 'sk-test' }
    useConfigsViewStore.getState().setFormDraft(NEW_CONFIG_DRAFT_ID, draft)
    expect(useConfigsViewStore.getState().formDrafts[NEW_CONFIG_DRAFT_ID]).toEqual(draft)
    expect(isConfigFormDraft(useConfigsViewStore.getState().formDrafts[NEW_CONFIG_DRAFT_ID])).toBe(true)
  })

  it('keeps edit-config draft keyed by config id', () => {
    const draft = { ...emptyConfigForm(), description: 'prod', base_url: 'https://api.example.com' }
    useConfigsViewStore.getState().setFormDraft('work', draft)
    expect(useConfigsViewStore.getState().formDrafts.work).toEqual(draft)
    useConfigsViewStore.getState().clearFormDraft('work')
    expect(useConfigsViewStore.getState().formDrafts.work).toBeUndefined()
  })
})
