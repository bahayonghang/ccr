'use strict'
const fs = require('node:fs')
const path = require('node:path')
const { createRequire, Module } = require('node:module')
const { spawnSync } = require('node:child_process')
const root = path.resolve(__dirname, '../../../..')
const uiRequire = createRequire(path.join(root, 'ccr-ui/package.json'))
const braces = uiRequire('braces')
const parsePath = uiRequire.resolve('braces/lib/parse')
const cases = []
for (const method of ['parse', 'compile', 'expand']) {
  for (const depth of [100, 101, 2000]) cases.push({ name: `brace-${method}-${depth}`, method, depth, kind: 'brace' })
  cases.push({ name: `paren-${method}-4999`, method, depth: 4999, kind: 'paren' })
}
for (const method of ['compile', 'expand']) cases.push({ name: `ast-paren-${method}-4999`, method, depth: 4999, kind: 'ast-paren' })
const normal = ['a/{b,c}/d', 'file-{1..3}.txt', '{a,{b,c}}', '\\{a,b\\}', 'a/(b|c)/d', '${a,b}', '{01..03}', '{a,b}{c,d}']
if (process.argv[2] === '--worker') {
  const entry = cases.find(row => row.name === process.argv[3])
  const input = entry.kind.includes('paren') ? '('.repeat(entry.depth) + 'x' + ')'.repeat(entry.depth) : '{a,'.repeat(entry.depth) + 'x' + '}'.repeat(entry.depth)
  const t = performance.now()
  try {
    const argument = entry.kind === 'ast-paren' ? braces.parse(input) : input
    const result = braces[entry.method](argument)
    process.stdout.write(JSON.stringify({ ...entry, inputLength: input.length, status: 'success', elapsedMs: performance.now() - t, resultLength: typeof result === 'string' ? result.length : Array.isArray(result) ? result.length : null }))
  } catch (error) {
    process.stdout.write(JSON.stringify({ ...entry, inputLength: input.length, status: 'error', elapsedMs: performance.now() - t, errorName: error.name, errorMessage: error.message }))
  }
} else {
  const parseSource = fs.readFileSync(parsePath, 'utf8')
  const originalSource = parseSource.replace('const MAX_BRACE_DEPTH = 100;\n\n', '').replace(/      if \(depth > MAX_BRACE_DEPTH\) \{\r?\n        throw new SyntaxError\(`Input brace depth \(\$\{depth\}\) exceeds max depth \(\$\{MAX_BRACE_DEPTH\}\)`\);\r?\n      \}\r?\n/, '')
  if (originalSource === parseSource || originalSource.includes('MAX_BRACE_DEPTH')) throw new Error('Unpatched in-memory comparison reconstruction failed')
  const originalParseModule = new Module(parsePath, module)
  originalParseModule.filename = parsePath
  originalParseModule.paths = Module._nodeModulePaths(path.dirname(parsePath))
  originalParseModule._compile(originalSource, parsePath)
  const originalParse = originalParseModule.exports
  const compatibility = normal.map(input => {
    const beforeCompile = braces.compile(originalParse(input))
    const beforeExpand = braces.expand(originalParse(input))
    const afterCompile = braces.compile(input)
    const afterExpand = braces.expand(input)
    return { input, beforeCompile, afterCompile, compileEqual: beforeCompile === afterCompile, beforeExpand, afterExpand, expandEqual: JSON.stringify(beforeExpand) === JSON.stringify(afterExpand) }
  })
  const runtimes = ['node', 'bun']
  const results = runtimes.flatMap(runtime => cases.map(entry => {
    const t = performance.now()
    const child = spawnSync(runtime, [__filename, '--worker', entry.name], { encoding: 'utf8', timeout: 5000, cwd: root, windowsHide: true })
    let payload = null
    try { payload = JSON.parse(child.stdout) } catch {}
    return { runtime, ...entry, wallMs: performance.now() - t, processExit: child.status, signal: child.signal, processError: child.error?.message ?? null, payload, stderr: child.stderr }
  }))
  const receipt = { startedUtc: new Date().toISOString(), command: 'node research/resume-2026-10-04-ui-braces-probe.cjs', node: process.version, installedBraces: uiRequire('braces/package.json').version, installedParsePath: parsePath, patchPresent: parseSource.includes('const MAX_BRACE_DEPTH = 100'), timeoutMsPerCase: 5000, compatibility, results }
  fs.writeFileSync(path.join(__dirname, 'resume-2026-10-04-ui-braces-probe-v2-receipt.json'), JSON.stringify(receipt, null, 2) + '\n')
  for (const result of results) process.stdout.write(JSON.stringify({ runtime: result.runtime, name: result.name, ...result.payload, processExit: result.processExit, processError: result.processError }) + '\n')
  process.stdout.write(`Compatibility ${compatibility.filter(row => row.compileEqual && row.expandEqual).length}/${compatibility.length}\n`)
}

