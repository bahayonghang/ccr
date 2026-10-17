/**
 * Read a platform profile catalog from profiles.db.
 *
 * The database is the live catalog. Legacy profiles.toml is only the
 * pre-migration file. SQLite bytes are never parsed as TOML.
 */

import { execFile } from "child_process"
import * as fs from "fs"
import * as os from "os"
import * as path from "path"
import * as TOML from "smol-toml"
import { getProfilesDbPath, getProfilesPath } from "./ccrPaths"

interface SqlStatement {
  all: () => unknown[]
  get: () => unknown
}

interface SqlDatabase {
  prepare: (sql: string) => SqlStatement
  close: () => void
}

interface ProfileRow {
  name: string
  description: string | null
  base_url: string | null
  auth_token: string | null
  model: string | null
  small_fast_model: string | null
  provider: string | null
  provider_type: string | null
  account: string | null
  tags_json: string | null
  usage_count: number | null
  enabled: number | null
  extra_toml: string
}

const PROFILE_SQL = `
SELECT name, description, base_url, auth_token, model, small_fast_model,
       provider, provider_type, account, tags_json, usage_count, enabled, extra_toml
FROM profiles
ORDER BY sort_order ASC, name ASC
`

export async function readProfileDocument(
  platformName: string,
): Promise<Record<string, unknown> | null> {
  const dbPath = getProfilesDbPath(platformName)
  if (fs.existsSync(dbPath)) {
    const fromDatabase = await readSqliteDocument(dbPath)
    if (fromDatabase) {
      return fromDatabase
    }
    const exported = await exportCanonicalToml(platformName)
    if (exported) {
      return TOML.parse(exported) as Record<string, unknown>
    }
    console.error(`Failed to read profile catalog for ${platformName}: ${dbPath}`)
    return null
  }

  const tomlPath = getProfilesPath(platformName)
  try {
    const content = await fs.promises.readFile(tomlPath, "utf-8")
    return TOML.parse(content) as Record<string, unknown>
  } catch (error) {
    if (isNotFoundError(error)) {
      return null
    }
    throw error
  }
}

export function exportCanonicalToml(platformName: string): Promise<string | null> {
  return new Promise((resolve) => {
    execFile(
      "ccr",
      ["export", "--platform", platformName, "--stdout"],
      { timeout: 30_000 },
      (error, stdout) => {
        if (error || !stdout.trim()) {
          resolve(null)
          return
        }
        resolve(stdout)
      },
    )
  })
}

export function catalogEditPath(platformName: string): string {
  return path.join(
    os.tmpdir(),
    `ccr-${platformName}-catalog-${process.pid}-${Date.now()}.toml`,
  )
}

async function readSqliteDocument(dbPath: string): Promise<Record<string, unknown> | null> {
  const database = await openReadOnly(dbPath)
  if (!database) {
    return null
  }
  try {
    return documentFromDatabase(database)
  } catch (error) {
    console.error("Failed to query profile catalog", error)
    return null
  } finally {
    database.close()
  }
}

async function openReadOnly(dbPath: string): Promise<SqlDatabase | null> {
  try {
    const loader = new Function(
      "specifier",
      "return import(specifier)",
    ) as (specifier: string) => Promise<{
      DatabaseSync: new (filename: string, options: { readOnly: boolean }) => SqlDatabase
    }>
    const sqlite = await loader("node:sqlite")
    return new sqlite.DatabaseSync(dbPath, { readOnly: true })
  } catch {
    return null
  }
}

function documentFromDatabase(database: SqlDatabase): Record<string, unknown> {
  const header = database.prepare("SELECT current_config FROM document WHERE id = 1").get() as
    | { current_config: string | null }
    | undefined
  const document: Record<string, unknown> = {
    current_config: header?.current_config ?? "",
  }
  const rows = database.prepare(PROFILE_SQL).all() as ProfileRow[]
  for (const row of rows) {
    const profile: Record<string, unknown> = {}
    if (row.extra_toml) {
      const extra = TOML.parse(row.extra_toml) as Record<string, unknown>
      Object.assign(profile, extra)
    }
    assignString(profile, "description", row.description)
    assignString(profile, "base_url", row.base_url)
    assignString(profile, "auth_token", row.auth_token)
    assignString(profile, "model", row.model)
    assignString(profile, "small_fast_model", row.small_fast_model)
    assignString(profile, "provider", row.provider)
    assignString(profile, "provider_type", row.provider_type)
    assignString(profile, "account", row.account)
    if (row.tags_json) {
      try {
        profile.tags = JSON.parse(row.tags_json) as unknown
      } catch {
        // A damaged tags column should not hide the rest of the catalog.
      }
    }
    if (row.usage_count !== null) {
      profile.usage_count = row.usage_count
    }
    if (row.enabled !== null) {
      profile.enabled = row.enabled !== 0
    }
    document[row.name] = profile
  }
  return document
}

function assignString(profile: Record<string, unknown>, key: string, value: string | null): void {
  if (value) {
    profile[key] = value
  }
}

function isNotFoundError(error: unknown): boolean {
  if (!error || typeof error !== "object" || !("code" in error)) {
    return false
  }
  return (error as { code?: string }).code === "ENOENT"
}
