import {
  getOpenCodeConfig,
  getOpenCodeTuiSettings,
  updateOpenCodeConfig,
  updateOpenCodeTuiSettings,
} from '@/api'
import { asBool, asList, asString, boolField, selectField, splitList, textField } from '@/configs/settings-helpers'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { probeLocalEnvironment } from '@/configs/probeLocal'
import type { SettingsConfig, SettingsFieldOption } from '@/configs/settings-types'
import { SettingsValidationError } from '@/configs/settings-types'
import { dirtySettingsPatch, settingsRecord } from '@/configs/settings-patch'

const SHARE_OPTIONS: SettingsFieldOption[] = [
  { value: 'manual', labelKey: 'manual' },
  { value: 'auto', labelKey: 'auto' },
  { value: 'disabled', labelKey: 'disabled' },
]

const parseRecord = (raw: string): Record<string, unknown> => {
  const parsed: unknown = JSON.parse(raw || '{}')
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return {}
  return parsed as Record<string, unknown>
}

export const opencodeSettingsConfig: SettingsConfig = {
  cacheKey: 'settings-opencode',
  homePath: '/opencode',
  module: 'opencode',
  i18nPrefix: 'opencode.settings',
  titleKey: 'opencode.settings.title',
  subtitleKey: 'opencode.settings.subtitle',
  features: { dualFile: true, dirtyPatch: true, localOnly: true },
  probe: probeLocalEnvironment,
  notify: surfaceNotify,
  tabs: [
    { id: 'runtime', labelKey: 'opencode.settings.tabs.runtime' },
    { id: 'tui', labelKey: 'opencode.settings.tabs.tui' },
  ],
  fields: [
    textField('model', 'runtime', 'opencode.settings.fields.model'),
    textField('smallModel', 'runtime', 'opencode.settings.fields.smallModel'),
    textField('defaultAgent', 'runtime', 'opencode.settings.fields.defaultAgent'),
    selectField({ id: 'share', tab: 'runtime', labelKey: 'opencode.settings.fields.share', options: SHARE_OPTIONS }),
    boolField('snapshot', 'runtime', 'opencode.settings.fields.snapshot'),
    selectField({ id: 'autoupdate', tab: 'runtime', labelKey: 'opencode.settings.fields.autoupdate', options: [
      { value: 'true', labelKey: 'true' }, { value: 'false', labelKey: 'false' }, { value: 'notify', labelKey: 'notify' },
    ] }),
    { id: 'serverPort', tab: 'runtime', kind: 'number', labelKey: 'opencode.settings.fields.serverPort' },
    textField('serverHostname', 'runtime', 'opencode.settings.fields.serverHostname'),
    boolField('serverMdns', 'runtime', 'opencode.settings.fields.serverMdns'),
    { id: 'toolsJson', tab: 'runtime', kind: 'textarea', labelKey: 'opencode.settings.fields.toolsJson' },
    { id: 'permissionJson', tab: 'runtime', kind: 'textarea', labelKey: 'opencode.settings.fields.permissionJson' },
    { id: 'instructionsText', tab: 'runtime', kind: 'textarea', labelKey: 'opencode.settings.fields.instructions' },
    textField('theme', 'tui', 'opencode.settings.fields.theme'),
    boolField('mouse', 'tui', 'opencode.settings.fields.mouse'),
    { id: 'keybindsJson', tab: 'tui', kind: 'textarea', labelKey: 'opencode.settings.fields.keybindsJson' },
  ],
  load: async () => {
    const [runtime, tui] = await Promise.all([getOpenCodeConfig(), getOpenCodeTuiSettings()])
    return { source: { runtime, tui }, values: {
      model: asString(runtime.model),
      smallModel: asString(runtime.small_model),
      defaultAgent: asString(runtime.default_agent),
      share: asString(runtime.share) || 'manual',
      snapshot: asBool(runtime.snapshot),
      autoupdate: asString(runtime.autoupdate),
      serverPort: runtime.server?.port ?? '',
      serverHostname: asString(runtime.server?.hostname),
      serverMdns: asBool(runtime.server?.mdns),
      toolsJson: JSON.stringify(runtime.tools ?? {}, null, 2),
      permissionJson: JSON.stringify(runtime.permission ?? {}, null, 2),
      instructionsText: asList(runtime.instructions),
      theme: asString(tui.theme),
      mouse: asBool(tui.mouse),
      keybindsJson: JSON.stringify(tui.keybinds ?? {}, null, 2),
    } }
  },
  save: async ({ values, dirtyKeys, snapshot }) => {
    const source = settingsRecord(snapshot.source)
    const runtime = dirtySettingsPatch(RUNTIME_PATHS, {
      model: asString(values.model) || undefined,
      small_model: asString(values.smallModel) || undefined,
      default_agent: asString(values.defaultAgent) || undefined,
      share: (asString(values.share) || 'manual') as 'manual' | 'auto' | 'disabled',
      snapshot: asBool(values.snapshot),
      autoupdate: values.autoupdate === 'true' ? true : values.autoupdate === 'false' ? false : values.autoupdate,
      server: {
        port: values.serverPort === '' ? undefined : Number(values.serverPort),
        hostname: asString(values.serverHostname) || undefined,
        mdns: asBool(values.serverMdns),
      },
      tools: parseRecord(asString(values.toolsJson)),
      permission: parseRecord(asString(values.permissionJson)),
      instructions: splitList(values.instructionsText),
    }, { dirtyKeys, source: source.runtime })
    const tui = dirtySettingsPatch(TUI_PATHS, {
      theme: asString(values.theme) || undefined,
      mouse: asBool(values.mouse),
      keybinds: parseRecord(asString(values.keybindsJson)),
    }, { dirtyKeys, source: source.tui })
    if ([runtime, tui].some((patch) => Object.values(patch).some((value) => value === undefined))) {
      throw new SettingsValidationError('settingsRaw.opencodeClearUnsupported')
    }
    if (Object.keys(runtime).length) await updateOpenCodeConfig(runtime)
    if (Object.keys(tui).length) await updateOpenCodeTuiSettings(tui)
    return { status: 'saved' }
  },
}

const RUNTIME_PATHS = {
  model: 'model', smallModel: 'small_model', defaultAgent: 'default_agent', share: 'share',
  snapshot: 'snapshot', autoupdate: 'autoupdate', serverPort: 'server.port',
  serverHostname: 'server.hostname', serverMdns: 'server.mdns', toolsJson: 'tools',
  permissionJson: 'permission', instructionsText: 'instructions',
}
const TUI_PATHS = { theme: 'theme', mouse: 'mouse', keybindsJson: 'keybinds' }
