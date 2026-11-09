import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { createRequire } from 'node:module'

const research = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(research, '../../../..')
const require = createRequire(path.join(root, 'ccr-ui/package.json'))
const semver = require('semver')
const evidence = path.join(research, 'evidence')
const read = (name) => JSON.parse(fs.readFileSync(path.join(evidence, name), 'utf8'))
const before = read('lock-before.json')
const changes = read('dependency-changes.json').changes
const official = read('official-advisories.json')
const lines = fs.readFileSync(path.join(root, 'ccr-ui/bun.lock'), 'utf8').split(/\r?\n/)
const after = Object.fromEntries(lines.flatMap((line) => {
  const match = /^    "([^"]+)": (\[.*\]),?$/.exec(line)
  return match ? [[match[1], JSON.parse(match[2])]] : []
}))
assert.deepEqual(Object.keys(after), Object.keys(before))
const changed = Object.keys(after).filter((name) => JSON.stringify(before[name]) !== JSON.stringify(after[name]))
assert.deepEqual(changed.sort(), changes.map(({ name }) => name).sort())
let advisoryRanges = 0
for (const change of changes) {
  const entry = after[change.name]
  assert.equal(entry[0], `${change.name}@${change.new_version}`)
  assert.equal(entry.at(-1), change.verified_tarball_integrity)
  assert.deepEqual(entry.slice(1, -1), before[change.name].slice(1, -1))
  for (const parent of change.parents) {
    assert(semver.satisfies(change.new_version, parent.range), `${change.name}: ${parent.range}`)
  }
  for (const item of official.filter(({ package: name }) => name === change.name)) {
    assert.equal(item.status, 'fetched')
    const vulnerabilities = item.advisory.vulnerabilities.filter(({ package: pkg }) => pkg.name === change.name)
    assert(vulnerabilities.length > 0)
    for (const vulnerability of vulnerabilities) {
      const range = vulnerability.vulnerable_version_range.replaceAll(',', ' ')
      assert(!semver.satisfies(change.new_version, range), `${change.name}: ${item.advisory.ghsa_id}`)
      advisoryRanges += 1
    }
  }
}
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'ccr-ui/package.json'), 'utf8'))
assert.equal(manifest.overrides, undefined)
assert.equal(manifest.patchedDependencies, undefined)
const policy = JSON.parse(fs.readFileSync(path.join(root, 'ccr-ui/scripts/frontend-audit-allowlist.json'), 'utf8'))
assert.deepEqual(policy, { maxActiveExceptions: 0, exceptions: [] })
const result = { package_count_before: Object.keys(before).length, package_count_after: Object.keys(after).length,
  changed_packages: changed, parent_ranges_satisfied: true, dependency_metadata_unchanged: true,
  current_official_advisories: official.length, advisory_ranges_verified: advisoryRanges,
  selected_versions_outside_all_reported_ranges: true, policy_unchanged: true }
fs.writeFileSync(path.join(evidence, 'lock-diff-summary.json'), `${JSON.stringify(result, null, 2)}\n`)
console.log(JSON.stringify(result, null, 2))
