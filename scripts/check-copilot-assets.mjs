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

for (const file of expectedFiles) {
  const fullPath = path.join(repoRoot, file)
  if (!existsSync(fullPath)) {
    errors.push(`Missing expected file: ${file}`)
  }
}

const sharedSkillsPath = path.join(repoRoot, '.claude', 'skills')
if (!existsSync(sharedSkillsPath)) {
  errors.push('Missing canonical shared skills directory: .claude/skills')
}

for (const spec of frontmatterSpecs) {
  const frontmatter = parseFrontmatter(spec.file)
  for (const key of spec.required) {
    if (!(key in frontmatter) || String(frontmatter[key]).trim() === '') {
      errors.push(`Missing frontmatter key "${key}" in ${spec.file}`)
    }
  }
}

const trackedFiles = execFileSync('git', ['ls-files', '-z'], {
  cwd: repoRoot,
  encoding: 'utf8'
})
  .split('\0')
  .filter(Boolean)

for (const relativePath of trackedFiles) {
  if (relativePath === 'scripts/check-copilot-assets.mjs') {
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
  if (content === null || !content.startsWith('---\n')) {
    return {}
  }

  const endIndex = content.indexOf('\n---\n', 4)
  if (endIndex === -1) {
    return {}
  }

  const frontmatter = content.slice(4, endIndex)
  const parsed = {}

  for (const line of frontmatter.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) {
      continue
    }

    const separatorIndex = trimmed.indexOf(':')
    if (separatorIndex === -1) {
      continue
    }

    const key = trimmed.slice(0, separatorIndex).trim()
    const value = trimmed
      .slice(separatorIndex + 1)
      .trim()
      .replace(/^['"]|['"]$/g, '')

    parsed[key] = value
  }

  return parsed
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
