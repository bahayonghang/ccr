import { test, expect } from 'bun:test'
import { flattenCodexSettings, buildCodexSettingsPayload } from './settings-codex-map-baseline'
test('A14 changing only model preserves notification event array', () => {
  const values = flattenCodexSettings({ model: 'old', tui: { notifications: ['agent-turn-complete'] } })
  values.model = 'new'
  expect(buildCodexSettingsPayload(values).tui?.notifications).toEqual(['agent-turn-complete'])
})
