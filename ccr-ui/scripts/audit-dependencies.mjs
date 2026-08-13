#!/usr/bin/env bun

import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const scriptPath = fileURLToPath(import.meta.url)
const uiRoot = path.resolve(path.dirname(scriptPath), '..')

export function validateAllowlist(policy, now = new Date()) {
  const failures = []
  const exceptions = Array.isArray(policy?.exceptions) ? policy.exceptions : []
  const maxActiveExceptions = policy?.maxActiveExceptions

  if (!Number.isInteger(maxActiveExceptions) || maxActiveExceptions < 0) {
    failures.push('maxActiveExceptions must be a non-negative integer')
  } else if (exceptions.length > maxActiveExceptions) {
    failures.push(`active exceptions ${exceptions.length} exceed limit ${maxActiveExceptions}`)
  }

  const seen = new Set()
  for (const exception of exceptions) {
    for (const field of ['id', 'package', 'owner', 'rationale', 'expires']) {
      if (typeof exception?.[field] !== 'string' || exception[field].trim() === '') {
        failures.push(`${exception?.id ?? '(unknown)'}: missing ${field}`)
      }
    }

    if (seen.has(exception?.id)) failures.push(`${exception.id}: duplicate exception`)
    seen.add(exception?.id)

    const expiry = Date.parse(`${exception?.expires}T23:59:59Z`)
    if (!Number.isFinite(expiry)) {
      failures.push(`${exception?.id ?? '(unknown)'}: invalid expiry`)
    } else if (expiry < now.getTime()) {
      failures.push(`${exception.id}: exception expired on ${exception.expires}`)
    }

    if (!Array.isArray(exception?.patchedVersions) || exception.patchedVersions.length === 0) {
      failures.push(`${exception?.id ?? '(unknown)'}: patchedVersions must not be empty`)
    }
  }

  return failures
}

export function collectAdvisories(report) {
  return Object.entries(report ?? {}).flatMap(([packageName, advisories]) =>
    (Array.isArray(advisories) ? advisories : []).map((advisory) => ({
      ...advisory,
      id: path.basename(new URL(advisory.url).pathname),
      package: packageName,
    })),
  )
}

export function validateAuditReport(report, policy) {
  const failures = []
  const advisories = collectAdvisories(report)
  const exceptions = new Map(policy.exceptions.map((exception) => [exception.id, exception]))

  for (const advisory of advisories) {
    const exception = exceptions.get(advisory.id)
    if (!exception) {
      failures.push(`${advisory.package}: unapproved ${advisory.id} (${advisory.severity})`)
    } else if (exception.package !== advisory.package) {
      failures.push(`${advisory.id}: package ${advisory.package} does not match ${exception.package}`)
    }
  }

  for (const exception of policy.exceptions) {
    if (!advisories.some((advisory) => advisory.id === exception.id)) {
      failures.push(`${exception.id}: stale exception is no longer reported`)
    }
  }

  return failures
}

function loadJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(uiRoot, relativePath), 'utf8'))
}

function validatePatches(manifest, policy) {
  const failures = []
  if (!Array.isArray(policy?.exceptions) || policy.exceptions.length === 0) {
    if (manifest.patchedDependencies && Object.keys(manifest.patchedDependencies).length > 0) {
      failures.push('patchedDependencies must be empty when no advisory exceptions are active')
    }
    return failures
  }

  for (const exception of policy.exceptions) {
    for (const version of exception.patchedVersions ?? []) {
      const packageKey = `${exception.package}@${version}`
      const patchPath = manifest.patchedDependencies?.[packageKey]
      if (typeof patchPath !== 'string') {
        failures.push(`${packageKey}: missing patchedDependencies entry`)
      }
    }
  }

  return failures
}

function runAudit() {
  const result = spawnSync(process.execPath, ['audit', '--json', '--audit-level=high'], {
    cwd: uiRoot,
    encoding: 'utf8',
  })
  if (result.error) throw result.error
  if (![0, 1].includes(result.status)) {
    throw new Error(result.stderr.trim() || `bun audit exited with ${result.status}`)
  }
  return JSON.parse(result.stdout || '{}')
}

function main() {
  const manifest = loadJson('package.json')
  const policy = loadJson('scripts/frontend-audit-allowlist.json')
  const report = runAudit()
  const failures = [
    ...validateAllowlist(policy),
    ...validatePatches(manifest, policy),
    ...validateAuditReport(report, policy),
  ]

  if (failures.length > 0) {
    process.stderr.write(`frontend dependency audit failed:\n${failures.map((failure) => `- ${failure}`).join('\n')}\n`)
    process.exit(1)
  }

  const advisories = collectAdvisories(report)
  process.stdout.write(
    `frontend dependency audit passed: ${advisories.length} reported advisories, ${policy.exceptions.length}/${policy.maxActiveExceptions} active exceptions\n`,
  )
}

if (path.resolve(process.argv[1] ?? '') === scriptPath) main()
