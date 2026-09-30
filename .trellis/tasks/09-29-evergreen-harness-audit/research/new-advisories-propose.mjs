import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'

const out = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(out, '../../../..')
const require = createRequire(path.join(root, 'ccr-ui/package.json'))
const semver = require('semver')
const read = (name) => JSON.parse(fs.readFileSync(path.join(out, `new-advisories-${name}.json`), 'utf8'))
const advisories = ['GHSA-qhr7-859c-m2p7', 'GHSA-6j4f-fj2g-mc7p', 'GHSA-q2hr-2g5m-vwhr', 'GHSA-hrr3-gc8f-f4qj'].map(read)
const registry = Object.fromEntries(['brace-expansion', 'fast-uri'].map((name) => [name, read(`${name}-registry`)]))
const packageName = (identifier) => identifier.slice(0, identifier.lastIndexOf('@'))
const version = (identifier) => identifier.slice(identifier.lastIndexOf('@') + 1)
const applicable = (name) => advisories.flatMap((advisory) => advisory.vulnerabilities
  .filter((item) => item.package.ecosystem === 'npm' && item.package.name === name)
  .map((item) => ({ advisory: advisory.ghsa_id, range: item.vulnerable_version_range.replaceAll(',', ' '), first_patched_version: item.first_patched_version })))
const bunEntries = (file) => Object.fromEntries(fs.readFileSync(path.join(root, file), 'utf8').split(String.fromCharCode(10)).flatMap((raw) => {
  const line = raw.trimEnd()
  const quote = String.fromCharCode(34)
  const index = line.indexOf(quote + ': ')
  if (!line.startsWith('    ' + quote) || index < 0 || line[index + 3] !== '[') return []
  return [[line.slice(5, index), JSON.parse(line.slice(index + 3).replace(/,$/, ''))]]
}))

function resolveBun(entries, parent, name) {
  let current = parent
  while (current) {
    if (Object.hasOwn(entries, `${current}/${name}`)) return `${current}/${name}`
    const separator = current.lastIndexOf('/')
    current = separator >= 0 ? current.slice(0, separator) : ''
  }
  return Object.hasOwn(entries, name) ? name : null
}

const changes = []
function add(file, node, name, old, parents, lockIntegrity, dev) {
  const ranges = applicable(name)
  const affected = ranges.filter(({ range }) => semver.satisfies(old, range))
  if (!affected.length) return
  assert(parents.length > 0, `${file}/${node}: missing parent range`)
  const candidates = Object.keys(registry[name].versions).filter((candidate) =>
    semver.valid(candidate) && !semver.prerelease(candidate) && semver.major(candidate) === semver.major(old) &&
    semver.gt(candidate, old) && parents.every(({ range }) => semver.satisfies(candidate, range)) &&
    ranges.every(({ range }) => !semver.satisfies(candidate, range)))
  candidates.sort(semver.compare)
  assert(candidates.length > 0, `${name}@${old}: no compatible patched release`)
  const selected = candidates[0]
  const oldMetadata = registry[name].versions[old]
  const newMetadata = registry[name].versions[selected]
  assert.equal(lockIntegrity, oldMetadata.dist.integrity)
  assert.deepEqual(newMetadata.dependencies ?? {}, oldMetadata.dependencies ?? {})
  assert.deepEqual(newMetadata.engines ?? {}, oldMetadata.engines ?? {})
  changes.push({ file, node, package: name, old_version: old, new_version: selected, parents, dev,
    affected_advisories: affected, all_new_advisory_ranges_checked: ranges,
    old_integrity: lockIntegrity, new_integrity: newMetadata.dist.integrity, tarball: newMetadata.dist.tarball,
    old_published: registry[name].time[old], new_published: registry[name].time[selected],
    dependencies_unchanged: true, engines_unchanged: true,
    old_metadata: oldMetadata, new_metadata: newMetadata })
}

const surfaces = []
for (const file of ['ccr-ui/bun.lock', 'docs/bun.lock']) {
  const entries = bunEntries(file)
  const nodes = Object.entries(entries).filter(([, entry]) => Object.hasOwn(registry, packageName(entry[0])))
  surfaces.push({ file, entries: Object.keys(entries).length, target_nodes: nodes.map(([key]) => key) })
  for (const [key, entry] of nodes) {
    const name = packageName(entry[0])
    const parents = Object.entries(entries).flatMap(([parent, value]) => {
      const range = value[2].dependencies?.[name]
      return range && resolveBun(entries, parent, name) === key ? [{ node: parent, package: value[0], range }] : []
    })
    add(file, key, name, version(entry[0]), parents, entry.at(-1), 'transitive; root dev path verified separately')
  }
}

const file = 'ccr-vscode/package-lock.json'
const packages = JSON.parse(fs.readFileSync(path.join(root, file), 'utf8')).packages
const nodes = Object.entries(packages).filter(([key]) => Object.keys(registry).some((name) => key.endsWith(`/node_modules/${name}`) || key === `node_modules/${name}`))
surfaces.push({ file, entries: Object.keys(packages).length, target_nodes: nodes.map(([key]) => key) })
for (const [key, entry] of nodes) {
  const name = key.slice(key.lastIndexOf('node_modules/') + 'node_modules/'.length)
  const parents = Object.entries(packages).flatMap(([parent, value]) => {
    const range = value.dependencies?.[name]
    if (!range) return []
    let current = parent
    while (true) {
      const candidate = `${current ? `${current}/` : ''}node_modules/${name}`
      if (Object.hasOwn(packages, candidate)) return candidate === key ? [{ node: parent, package: `${parent}@${value.version}`, range }] : []
      if (!current) return []
      const separator = current.lastIndexOf('/')
      current = separator >= 0 ? current.slice(0, separator) : ''
    }
  })
  add(file, key, name, entry.version, parents, entry.integrity, entry.dev === true)
}

const report = { generated_utc: new Date().toISOString(), criteria: 'Lowest stable higher release in the existing major that satisfies all resolved parents and excludes every matching official advisory range. Product files remain unchanged.', surfaces, changes }
fs.writeFileSync(path.join(out, 'new-advisories-version-proposal.json'), JSON.stringify(report, null, 2) + String.fromCharCode(10))
console.log(JSON.stringify({ surfaces, changes: changes.map(({ old_metadata, new_metadata, ...item }) => item) }, null, 2))
