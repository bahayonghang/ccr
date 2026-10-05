import assert from 'node:assert/strict'
import { execFileSync, spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { after, test } from 'node:test'
import { fileURLToPath } from 'node:url'

const checker = fileURLToPath(new URL('./check-copilot-assets.mjs', import.meta.url))
const ownedRoot = mkdtempSync(path.join(tmpdir(), 'ccr-copilot-assets-'))
const instruction = '.github/instructions/rust.instructions.md'
const sharedSkill = '.github/skills/ccr-gate-recovery/SKILL.md'
const scopedFiles = [
  ...['rust', 'ui', 'docs'].map((name) => ({
    file: `.github/instructions/${name}.instructions.md`,
    fields: 'applyTo: "**/*.rs"\ndescription: "fixture description"'
  })),
  ...['rust', 'ui', 'docs'].map((name) => ({
    file: `.github/prompts/${name}-change.prompt.md`,
    fields: 'description: "fixture description"\nagent: implementer'
  })),
  ...['researcher', 'implementer', 'reviewer'].map((name) => ({
    file: `.github/agents/${name}.agent.md`,
    fields: `name: ${name}\ndescription: "fixture description"`
  }))
]

function writeFile(root, relative, content) {
  const fullPath = path.join(root, relative)
  mkdirSync(path.dirname(fullPath), { recursive: true })
  writeFileSync(fullPath, content, 'utf8')
}

function git(root, args) {
  return execFileSync('git', ['-c', 'core.autocrlf=false', ...args], {
    cwd: root,
    encoding: 'utf8',
    windowsHide: true
  })
}

function fixture({ newline = '\n', bom = '' } = {}) {
  const root = mkdtempSync(path.join(ownedRoot, 'case-'))
  const files = new Map([
    ['AGENTS.md', '# Shared project rules\n'],
    [sharedSkill, '# Shared gate workflow\n'],
    ['.github/copilot-instructions.md', '# Read AGENTS.md and .github/skills/\n'],
    ['docs/guide/github-copilot-workspace.md', '# Workspace guide\n'],
    ['docs/en/guide/github-copilot-workspace.md', '# Workspace guide\n'],
    ...scopedFiles.map(({ file, fields }) => [file, `---\n${fields}\n---\n\n# Fixture\n`])
  ])
  for (const [file, content] of files) {
    writeFile(root, file, bom + content.replaceAll('\n', newline))
  }
  git(root, ['init', '--quiet'])
  git(root, ['add', '--force', '--', ...files.keys()])
  return root
}

function runCheck(root) {
  return spawnSync(process.execPath, [checker], {
    cwd: root,
    encoding: 'utf8',
    timeout: 30000,
    windowsHide: true
  })
}

function assertRejected(root, expected) {
  const result = runCheck(root)
  assert.equal(result.status, 1, result.stderr)
  assert.match(result.stderr, expected)
  assert.doesNotMatch(result.stdout, /Validated/)
}

after(() => rmSync(ownedRoot, { recursive: true, force: true }))

for (const newline of ['\n', '\r\n']) {
  for (const bom of ['', '\uFEFF']) {
    test(`accepts ${newline === '\n' ? 'LF' : 'CRLF'} with ${bom ? 'BOM' : 'no BOM'} without changing bytes`, () => {
      const root = fixture({ newline, bom })
      const before = readFileSync(path.join(root, instruction))
      const result = runCheck(root)
      assert.equal(result.status, 0, result.stderr)
      assert.match(result.stdout, /Validated 12 GitHub Copilot workspace assets/)
      assert.deepEqual(readFileSync(path.join(root, instruction)), before)
      assert.equal(existsSync(path.join(root, '.claude')), false)
    })
  }
}

const invalidHeaders = [
  ['missing field', '---\napplyTo: "**/*.rs"\n---\n', /Missing frontmatter key "description"/],
  ['empty plain value', '---\napplyTo: "**/*.rs"\ndescription:\n---\n', /Missing frontmatter key "description"/],
  ['empty double-quoted value', '---\napplyTo: "**/*.rs"\ndescription: ""\n---\n', /Missing frontmatter key "description"/],
  ['empty single-quoted value', "---\napplyTo: '**/*.rs'\ndescription: ''\n---\n", /Missing frontmatter key "description"/],
  ['whitespace-only value', '---\napplyTo: "**/*.rs"\ndescription: "  "\n---\n', /Missing frontmatter key "description"/],
  ['comment-only value', '---\napplyTo: "**/*.rs"\ndescription: # no value\n---\n', /Missing frontmatter key "description"/],
  ['null value', '---\napplyTo: "**/*.rs"\ndescription: null\n---\n', /Missing frontmatter key "description"/],
  ['missing opening delimiter', 'applyTo: "**/*.rs"\ndescription: valid\n---\n', /Missing opening frontmatter delimiter/],
  ['false opening delimiter', '---invalid\napplyTo: "**/*.rs"\ndescription: valid\n---\n', /Missing opening frontmatter delimiter/],
  ['unclosed header', '---\napplyTo: "**/*.rs"\ndescription: valid\n', /Unclosed frontmatter/],
  ['false closing delimiter', '---\napplyTo: "**/*.rs"\ndescription: valid\n---invalid\n', /Unclosed frontmatter/],
  ['missing field separator', '---\napplyTo: "**/*.rs"\ndescription valid\n---\n', /Malformed frontmatter field/],
  ['duplicate field', '---\napplyTo: "**/*.rs"\ndescription: valid\ndescription: second\n---\n', /Malformed frontmatter field/],
  ['unclosed quoted value', '---\napplyTo: "**/*.rs"\ndescription: "invalid\n---\n', /Malformed frontmatter value/],
  ['mismatched quoted value', "---\napplyTo: '**/*.rs'\ndescription: 'invalid\"\n---\n", /Malformed frontmatter value/],
  ['invalid double-quote escape', '---\napplyTo: "**/*.rs"\ndescription: "bad\\q"\n---\n', /Malformed frontmatter value/],
  ['structured required value', '---\napplyTo: "**/*.rs"\ndescription: []\n---\n', /Malformed frontmatter value/]
]

for (const [name, header, expected] of invalidHeaders) {
  test(`rejects ${name}`, () => {
    const root = fixture()
    writeFile(root, instruction, header)
    assertRejected(root, expected)
  })
}

for (const [file, key] of [
  [instruction, 'applyTo'],
  ['.github/prompts/rust-change.prompt.md', 'agent'],
  ['.github/agents/researcher.agent.md', 'name']
]) {
  test(`requires ${key} in its asset type`, () => {
    const root = fixture()
    const content = readFileSync(path.join(root, file), 'utf8')
    writeFile(root, file, content.split('\n').filter((line) => !line.startsWith(`${key}:`)).join('\n'))
    assertRejected(root, new RegExp(`Missing frontmatter key "${key}"`))
  })
}

test('accepts quoted comments and an exact closing delimiter at EOF', () => {
  const root = fixture()
  writeFile(root, instruction, "---\n# comment\napplyTo: '**/*.rs' # glob\ndescription: 'Use ''quoted'' text # inside' # outside\n---")
  const result = runCheck(root)
  assert.equal(result.status, 0, result.stderr)
})

test('requires shared rules to be tracked even when the file exists', () => {
  const root = fixture()
  git(root, ['rm', '--cached', '--quiet', '--', 'AGENTS.md'])
  assertRejected(root, /Missing tracked shared rule: AGENTS.md/)
})

test('local skills do not replace tracked shared skills', () => {
  const root = fixture()
  git(root, ['rm', '--cached', '--quiet', '--', sharedSkill])
  mkdirSync(path.join(root, '.claude', 'skills'), { recursive: true })
  assertRejected(root, /Missing tracked shared skills/)
})

test('rejects a missing tracked shared skill', () => {
  const root = fixture()
  rmSync(path.join(root, sharedSkill))
  assertRejected(root, /Missing tracked shared rule/)
})

test('rejects missing expected assets', () => {
  const root = fixture()
  rmSync(path.join(root, '.github/copilot-instructions.md'))
  assertRejected(root, /Missing expected file/)
})

const forbiddenTerms = [
  ['GitHub Copilot', 'CLI'].join(' '),
  ['Codex', '(Copilot)'].join(' '),
  ['Codex', '(GitHub Copilot)'].join(' '),
  ['~/.codex', 'skills/'].join('/'),
  ['~/.codex', 'prompts/'].join('/')
]
for (const [index, term] of forbiddenTerms.entries()) {
  test(`rejects tracked terminology drift case ${index + 1}`, () => {
    const root = fixture()
    writeFile(root, 'TERMINOLOGY.md', term)
    git(root, ['add', '--', 'TERMINOLOGY.md'])
    assertRejected(root, /Forbidden text/)
  })
}
