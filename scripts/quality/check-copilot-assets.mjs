#!/usr/bin/env node

import { execFileSync } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import path from 'node:path'

const repoRoot = process.cwd()

const expectedFiles = [
  '.github/copilot-instructions.md',
  '.github/instructions/rust.instructions.md',
  '.github/instructions/ui.instructions.md',
  '.github/instructions/docs.instructions.md',
  '.github/prompts/rust-change.prompt.md',
  '.github/prompts/ui-change.prompt.md',
  '.github/prompts/docs-change.prompt.md',
  '.github/agents/researcher.agent.md',
  '.github/agents/implementer.agent.md',
  '.github/agents/reviewer.agent.md',
  'docs/guide/github-copilot-workspace.md',
  'docs/en/guide/github-copilot-workspace.md'
]

const frontmatterSpecs = [
  {
    file: '.github/instructions/rust.instructions.md',
    required: ['applyTo', 'description']
  },
  {
    file: '.github/instructions/ui.instructions.md',
    required: ['applyTo', 'description']
  },
  {
    file: '.github/instructions/docs.instructions.md',
    required: ['applyTo', 'description']
  },
  {
    file: '.github/prompts/rust-change.prompt.md',
    required: ['description', 'agent']
  },
  {
    file: '.github/prompts/ui-change.prompt.md',
    required: ['description', 'agent']
  },
  {
    file: '.github/prompts/docs-change.prompt.md',
    required: ['description', 'agent']
  },
  {
    file: '.github/agents/researcher.agent.md',
    required: ['name', 'description']
  },
  {
    file: '.github/agents/implementer.agent.md',
    required: ['name', 'description']
  },
  {
    file: '.github/agents/reviewer.agent.md',
    required: ['name', 'description']
  }
]

const forbiddenPatterns = [
  'GitHub Copilot CLI',
  'Codex (Copilot)',
  'Codex (GitHub Copilot)',
  '~/.codex/skills/',
  '~/.codex/prompts/'
]

const textExtensions = new Set([
  '.md',
  '.mjs',
  '.js',
  '.ts',
  '.tsx',
  '.json',
  '.toml',
  '.yml',
  '.yaml',
  '.rs',
  '.vue',
  '.txt',
  '.css',
  '.scss',
  '.sh',
  '.ps1'
])

const basenameAllowlist = new Set([
  'justfile',
  'AGENTS.md',
  'CLAUDE.md',
  'GEMINI.md'
])

const errors = []
const trackedFiles = execFileSync('git', ['ls-files', '-z'], {
  cwd: repoRoot,
  encoding: 'utf8'
})
  .split('\0')
  .filter(Boolean)

for (const file of expectedFiles) {
  const fullPath = path.join(repoRoot, file)
  if (!existsSync(fullPath)) {
    errors.push(`Missing expected file: ${file}`)
  }
}

const trackedSkills = trackedFiles.filter((file) =>
  file.startsWith('.github/skills/') && file.endsWith('/SKILL.md')
)
if (trackedSkills.length === 0) {
  errors.push('Missing tracked shared skills: .github/skills/*/SKILL.md')
}
for (const file of ['AGENTS.md', ...trackedSkills]) {
  if (!trackedFiles.includes(file) || !existsSync(path.join(repoRoot, file))) {
    errors.push(`Missing tracked shared rule: ${file}`)
  }
}

for (const spec of frontmatterSpecs) {
  const frontmatter = parseFrontmatter(spec.file)
  if (frontmatter === null) continue
  for (const key of spec.required) {
    if (!Object.hasOwn(frontmatter, key) || frontmatter[key].trim() === '') {
      errors.push(`Missing frontmatter key "${key}" in ${spec.file}`)
    }
  }
}

for (const relativePath of trackedFiles) {
  if (relativePath === 'scripts/quality/check-copilot-assets.mjs') {
    continue
  }

  if (!shouldScan(relativePath)) {
    continue
  }

  const fullPath = path.join(repoRoot, relativePath)
  const content = readUtf8(fullPath)
  if (content === null) {
    continue
  }

  for (const pattern of forbiddenPatterns) {
    if (content.includes(pattern)) {
      errors.push(`Forbidden text "${pattern}" found in ${relativePath}`)
    }
  }
}

if (errors.length > 0) {
  console.error('GitHub Copilot workspace asset check failed:')
  for (const error of errors) {
    console.error(`- ${error}`)
  }
  process.exit(1)
}

console.log(`Validated ${expectedFiles.length} GitHub Copilot workspace assets.`)
console.log(`Scanned ${trackedFiles.length} tracked files for terminology drift.`)

function parseFrontmatter(relativePath) {
  const fullPath = path.join(repoRoot, relativePath)
  const content = readUtf8(fullPath)
  if (content === null) {
    errors.push(`Cannot read frontmatter in ${relativePath}`)
    return null
  }
  const lines = content.replace(/^\uFEFF/, '').replace(/\r\n/g, '\n').split('\n')
  if (lines[0] !== '---') {
    errors.push(`Missing opening frontmatter delimiter in ${relativePath}`)
    return null
  }

  const endIndex = lines.indexOf('---', 1)
  if (endIndex === -1) {
    errors.push(`Unclosed frontmatter in ${relativePath}`)
    return null
  }

  const parsed = Object.create(null)

  for (const [index, line] of lines.slice(1, endIndex).entries()) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) {
      continue
    }

    const field = trimmed.match(/^([A-Za-z][\w-]*)\s*:\s*(.*)$/)
    if (!field || Object.hasOwn(parsed, field[1])) {
      errors.push(`Malformed frontmatter field in ${relativePath}:${index + 2}`)
      return null
    }

    const value = parseScalar(field[2])
    if (value === null) {
      errors.push(`Malformed frontmatter value in ${relativePath}:${index + 2}`)
      return null
    }
    parsed[field[1]] = value
  }

  return parsed
}

function parseScalar(rawValue) {
  if (rawValue.startsWith("'")) {
    const quoted = rawValue.match(/^'((?:[^']|'')*)'(?:\s+#.*)?$/)
    return quoted ? quoted[1].replace(/''/g, "'") : null
  }
  if (rawValue.startsWith('"')) {
    const quoted = rawValue.match(/^("(?:[^"\\]|\\.)*")(?:\s+#.*)?$/)
    if (!quoted) return null
    try {
      return JSON.parse(quoted[1])
    } catch {
      return null
    }
  }
  const value = rawValue.replace(/(?:^|\s+)#.*$/, '').trim()
  if (/^[\[\]{|>]/.test(value) || /:\s/.test(value)) return null
  return /^(?:null|Null|NULL|~)$/.test(value) ? '' : value
}

function shouldScan(relativePath) {
  const extension = path.extname(relativePath).toLowerCase()
  if (textExtensions.has(extension)) {
    return true
  }

  return basenameAllowlist.has(path.basename(relativePath))
}

function readUtf8(filePath) {
  try {
    return readFileSync(filePath, 'utf8')
  } catch {
    return null
  }
}
