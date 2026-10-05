// providers-catalog.json 前端防漂移测试
//
// 与 Rust 侧 builtin_providers.rs 的测试解析同一份 catalog JSON：
// - schemaVersion 校验（故意改坏时显式报错）
// - 条目结构校验
// - platforms 块无敏感字段（对齐 Rust test_platforms_blocks_contain_no_sensitive_fields）
// - BuiltinProvider 手工镜像接口与 catalog checkin 块的字段对应关系（抓双端漂移）
import { describe, expect, it } from 'vitest'
import rawCatalog from '../../../crates/ccr-checkin/data/providers-catalog.json'
import {
  CHECKIN_CATALOG_PROVIDER_TEMPLATES,
  PROVIDERS_CATALOG,
  PROVIDERS_CATALOG_SCHEMA_VERSION,
  parseProvidersCatalog,
} from '@/configs/providersCatalog'
import {
  filterAvailableBuiltinProviders,
  resolveBuiltinProvider,
} from '@/features/checkin/lib/builtinProviderLookup'
import type {
  BuiltinProvider,
  CdkProviderConfig,
  CheckinProvider,
  OAuthProviderConfig,
} from '@/types/checkin'

const cloneCatalog = () =>
  JSON.parse(JSON.stringify(rawCatalog)) as { schemaVersion: number; providers: unknown[] }

const camelToSnake = (key: string) => key.replace(/[A-Z]/g, (ch) => `_${ch.toLowerCase()}`)

describe('providers catalog schema validation', () => {
  it('parses the bundled catalog with schemaVersion 1', () => {
    expect(PROVIDERS_CATALOG.schemaVersion).toBe(PROVIDERS_CATALOG_SCHEMA_VERSION)
    expect(PROVIDERS_CATALOG.providers.length).toBeGreaterThan(0)

    const ids = PROVIDERS_CATALOG.providers.map((entry) => entry.id)
    expect(new Set(ids).size).toBe(ids.length)
    for (const entry of PROVIDERS_CATALOG.providers) {
      expect(entry.id.startsWith('builtin-'), `id ${entry.id} must keep builtin- prefix`).toBe(true)
    }
  })

  it('rejects a tampered schemaVersion with an explicit error', () => {
    const tampered = cloneCatalog()
    tampered.schemaVersion = 999

    expect(() => parseProvidersCatalog(tampered)).toThrowError(/schemaVersion/)
    expect(() => parseProvidersCatalog(tampered)).toThrowError(/999/)
  })

  it('rejects structurally invalid catalogs with explicit errors', () => {
    expect(() => parseProvidersCatalog(null)).toThrowError(/根节点/)
    expect(() => parseProvidersCatalog([])).toThrowError(/根节点/)
    expect(() => parseProvidersCatalog({ schemaVersion: 1, providers: {} })).toThrowError(/数组/)
    expect(() =>
      parseProvidersCatalog({ schemaVersion: 1, providers: [{ id: 'builtin-x' }] })
    ).toThrowError(/providers\[0\]\.name/)

    const missingCheckinField = cloneCatalog()
    const firstEntry = missingCheckinField.providers[0] as { checkin: Record<string, unknown> }
    delete firstEntry.checkin.baseUrl
    expect(() => parseProvidersCatalog(missingCheckinField)).toThrowError(/checkin\.baseUrl/)
  })
})

describe('providers catalog platforms blocks', () => {
  // 对齐 Rust 侧扫描：platforms 块只允许出现非敏感字段
  const FORBIDDEN_KEY_FRAGMENTS = ['token', 'secret', 'password', 'cookie', 'credential']

  const assertKeysSafe = (value: unknown, path: string) => {
    if (Array.isArray(value)) {
      value.forEach((child, index) => assertKeysSafe(child, `${path}[${index}]`))
      return
    }
    if (!value || typeof value !== 'object') return

    for (const [key, child] of Object.entries(value as Record<string, unknown>)) {
      const lowered = key.toLowerCase()
      for (const forbidden of FORBIDDEN_KEY_FRAGMENTS) {
        expect(lowered.includes(forbidden), `sensitive key '${key}' found at ${path}`).toBe(false)
      }
      // apiKeyUrl（取 key 的文档地址）是模板契约允许的字段，其余含 key 的字段一律拒绝
      if (lowered.includes('key')) {
        expect(key, `unexpected key-like field '${key}' at ${path}`).toBe('apiKeyUrl')
      }
      assertKeysSafe(child, `${path}.${key}`)
    }
  }

  it('contains no sensitive fields in any platforms block', () => {
    for (const entry of PROVIDERS_CATALOG.providers) {
      if (entry.platforms) {
        assertKeysSafe(entry.platforms, entry.id)
      }
    }
  })

  it('gives every standard check-in site claude/codex overrides matching the check-in baseUrl', () => {
    const standard = PROVIDERS_CATALOG.providers.filter(
      (entry) => entry.checkinCategory === 'standard'
    )
    expect(standard.length).toBeGreaterThan(0)

    for (const entry of standard) {
      expect(entry.checkin, `standard provider ${entry.id} missing checkin block`).toBeTruthy()
      expect(entry.platforms, `standard provider ${entry.id} missing platforms`).toBeTruthy()
      for (const platform of ['claude', 'codex'] as const) {
        const override = entry.platforms?.[platform]
        expect(override?.baseUrl, `${entry.id} missing ${platform}.baseUrl`).toBe(
          entry.checkin?.baseUrl
        )
      }
    }
  })

  it('projects exactly the platforms-bearing entries into provider templates', () => {
    const expectedIds = PROVIDERS_CATALOG.providers
      .filter((entry) => entry.platforms)
      .map((entry) => entry.id)
      .sort()
    const actualIds = CHECKIN_CATALOG_PROVIDER_TEMPLATES.map((template) => template.id).sort()

    expect(actualIds).toEqual(expectedIds)
    // 投影结果不得携带 checkin 块的 OAuth client id 等签到专属数据，更不得携带密钥字段
    const serialized = JSON.stringify(CHECKIN_CATALOG_PROVIDER_TEMPLATES)
    expect(serialized).not.toMatch(
      /client_?id|apiKey":|api_key|auth_?token|secret|password|cookie/i
    )
  })
})

describe('BuiltinProvider mirror consistency with the catalog', () => {
  // 编译期穷举 TS 镜像接口的 wire 字段（satisfies 保证与 keyof 完全一致，缺/多字段都会编译报错）
  const BUILTIN_PROVIDER_WIRE_KEYS = {
    id: true,
    name: true,
    description: true,
    domain: true,
    base_url: true,
    checkin_path: true,
    balance_path: true,
    user_info_path: true,
    auth_header: true,
    auth_prefix: true,
    supports_checkin: true,
    requires_waf_bypass: true,
    requires_cf_clearance: true,
    checkin_bugged: true,
    icon: true,
    category: true,
    cdk_config: true,
    oauth_config: true,
  } satisfies Record<keyof BuiltinProvider, true>

  const CDK_CONFIG_WIRE_KEYS = {
    cdk_type: true,
    cdk_source_url: true,
    topup_path: true,
    requires_cdk_cookies: true,
    requires_access_token: true,
  } satisfies Record<keyof CdkProviderConfig, true>

  const OAUTH_CONFIG_WIRE_KEYS = {
    github_client_id: true,
    linuxdo_client_id: true,
    oauth_state_path: true,
  } satisfies Record<keyof OAuthProviderConfig, true>

  // catalog checkin 块字段 → list_builtin_providers wire 字段的映射；
  // null 表示后端专属字段（不出现在 BuiltinProvider wire 上）
  const CHECKIN_KEY_TO_WIRE: Record<string, string | null> = {
    baseUrl: 'base_url',
    checkinPath: 'checkin_path',
    balancePath: 'balance_path',
    userInfoPath: 'user_info_path',
    authHeader: 'auth_header',
    authPrefix: 'auth_prefix',
    supportsCheckin: 'supports_checkin',
    requiresWafBypass: 'requires_waf_bypass',
    requiresCfClearance: 'requires_cf_clearance',
    checkinBugged: 'checkin_bugged',
    // WAF cookie 名单是后端 WAF policy 数据，不进入 BuiltinProvider wire
    wafCookieNames: null,
    cdk: 'cdk_config',
    oauth: 'oauth_config',
  }

  // BuiltinProvider 上来自条目通用元数据的 wire 字段（catalog 顶层 → Rust 投影）
  const METADATA_WIRE_KEYS = ['id', 'name', 'description', 'domain', 'icon', 'category']

  const collectKeys = (objects: Array<Record<string, unknown> | undefined>) => {
    const keys = new Set<string>()
    for (const obj of objects) {
      if (obj) Object.keys(obj).forEach((key) => keys.add(key))
    }
    return keys
  }

  it('keeps the BuiltinProvider interface in sync with the catalog checkin block', () => {
    const checkinBlocks = PROVIDERS_CATALOG.providers.map(
      (entry) => entry.checkin as Record<string, unknown> | undefined
    )
    const observedKeys = collectKeys(checkinBlocks)
    expect(observedKeys.size).toBeGreaterThan(0)

    const expectedWireKeys = new Set<string>(METADATA_WIRE_KEYS)
    for (const key of observedKeys) {
      // catalog checkin 块出现未知字段 → Rust/TS 双端镜像需要同步评审
      expect(
        key in CHECKIN_KEY_TO_WIRE,
        `catalog checkin field '${key}' is not mapped; update BuiltinProvider mirrors on both ends`
      ).toBe(true)
      const wireKey = CHECKIN_KEY_TO_WIRE[key]
      if (wireKey) expectedWireKeys.add(wireKey)
    }

    expect([...expectedWireKeys].sort()).toEqual(Object.keys(BUILTIN_PROVIDER_WIRE_KEYS).sort())
  })

  it('keeps the CdkProviderConfig and OAuthProviderConfig mirrors in sync', () => {
    const cdkBlocks = PROVIDERS_CATALOG.providers.map(
      (entry) => entry.checkin?.cdk as Record<string, unknown> | undefined
    )
    const cdkKeys = [...collectKeys(cdkBlocks)].map(camelToSnake).sort()
    expect(cdkKeys).toEqual(Object.keys(CDK_CONFIG_WIRE_KEYS).sort())

    const oauthBlocks = PROVIDERS_CATALOG.providers.map(
      (entry) => entry.checkin?.oauth as Record<string, unknown> | undefined
    )
    const oauthKeys = [...collectKeys(oauthBlocks)].map(camelToSnake).sort()
    expect(oauthKeys).toEqual(Object.keys(OAUTH_CONFIG_WIRE_KEYS).sort())
  })
})

describe('builtin provider lookup (builtin_id first, name fallback)', () => {
  const builtinProviders = [
    { id: 'builtin-anyrouter', name: 'AnyRouter' },
    { id: 'builtin-runawaytime', name: 'RunAnytime' },
  ] as BuiltinProvider[]

  const makeProvider = (overrides: Partial<CheckinProvider>): CheckinProvider =>
    ({
      id: 'p-1',
      name: 'AnyRouter',
      base_url: 'https://anyrouter.top',
      checkin_path: '/api/user/sign_in',
      balance_path: '/api/user/self',
      user_info_path: '/api/user/self',
      auth_header: 'Authorization',
      auth_prefix: 'Bearer',
      enabled: true,
      created_at: '2026-01-01T00:00:00Z',
      ...overrides,
    }) as CheckinProvider

  it('resolves by builtin_id even after the provider is renamed', () => {
    const renamed = makeProvider({
      name: '我的中转站',
      base_url: 'https://example.com',
      builtin_id: 'builtin-runawaytime',
    })
    expect(resolveBuiltinProvider(builtinProviders, renamed)?.id).toBe('builtin-runawaytime')
  })

  it('falls back to name matching for legacy rows without builtin_id', () => {
    const legacy = makeProvider({ name: 'AnyRouter' })
    expect(resolveBuiltinProvider(builtinProviders, legacy)?.id).toBe('builtin-anyrouter')

    const unknown = makeProvider({ name: 'My Custom' })
    expect(resolveBuiltinProvider(builtinProviders, unknown)).toBeUndefined()
  })

  it('filters added builtins by builtin_id first and legacy name as fallback', () => {
    // 改名后的行：name 对不上，但 builtin_id 仍能把该内置站标记为已添加
    const renamed = makeProvider({ name: '我的中转站', builtin_id: 'builtin-anyrouter' })
    expect(filterAvailableBuiltinProviders(builtinProviders, [renamed]).map((bp) => bp.id)).toEqual(
      ['builtin-runawaytime']
    )

    // 旧数据行：无 builtin_id，按 name 回退判定
    const legacy = makeProvider({ name: 'RunAnytime' })
    expect(filterAvailableBuiltinProviders(builtinProviders, [legacy]).map((bp) => bp.id)).toEqual([
      'builtin-anyrouter',
    ])

    expect(filterAvailableBuiltinProviders(builtinProviders, []).map((bp) => bp.id)).toEqual([
      'builtin-anyrouter',
      'builtin-runawaytime',
    ])
  })
})
