import assert from 'node:assert/strict'
import test from 'node:test'
import {
  auditDocBannedPatterns,
  auditHiddenCommandPageContent,
  extractTopLevelCommands,
  extractUiDefaults
} from './audit-docs.mjs'

test('extractTopLevelCommands parses single-line, multi-line, bare hide, and comment braces safely', () => {
  const sample = `
pub enum Commands {
    /// JSON payload example: { ok, platform }
    #[command(name = "help")]
    Help {
        #[arg(hide = true)]
        path: Vec<String>,
    },
    /// Unbalanced doc brace { should never break braceDepth
    #[command(alias = "ls", hide = true)] // trailing comment with }
    List,
    #[command(
        hide = true,
        name = "legacy-export" // comment inside multi-line attr
    )]
    Export {
        output: Option<String>,
    },
    #[command(name = "temp-token")]
    #[command(hide)]
    TempToken,
    #[command(name = "hideout", hide = false)]
    Hideout,
    Validate {
        json: bool,
    },
}
`

  assert.deepEqual(extractTopLevelCommands(sample), [
    { name: 'help', hidden: false },
    { name: 'list', hidden: true },
    { name: 'legacy-export', hidden: true },
    { name: 'temp-token', hidden: true },
    { name: 'hideout', hidden: false },
    { name: 'validate', hidden: false }
  ])
})

test('extractTopLevelCommands classifies real CLI definitions into 25 public and 11 hidden commands', () => {
  const commands = extractTopLevelCommands()
  const publicNames = commands.filter(cmd => !cmd.hidden).map(cmd => cmd.name)
  const hiddenNames = commands.filter(cmd => cmd.hidden).map(cmd => cmd.name).sort()

  assert.equal(publicNames.length, 25)
  assert.equal(hiddenNames.length, 11)
  assert.deepEqual(hiddenNames, [
    'add',
    'clear',
    'delete',
    'disable',
    'enable',
    'export',
    'import',
    'init',
    'list',
    'optimize',
    'switch'
  ])
})

test('extractUiDefaults extracts default frontend and backend ports from definitions.rs', () => {
  assert.deepEqual(extractUiDefaults(), {
    uiPort: '15173',
    backendPort: '38081'
  })
})

test('auditHiddenCommandPageContent requires deprecation or retirement markers case-insensitively', () => {
  assert.deepEqual(
    auditHiddenCommandPageContent(
      'list',
      '> [!WARNING]\n> 该命令在 7.x 中已弃用并已隐藏。',
      '> [!WARNING]\n> This command is deprecated and hidden in 7.x.'
    ),
    []
  )

  const missingBoth = auditHiddenCommandPageContent(
    'list',
    '# list\n列出所有配置。',
    '# list\nList all profiles.'
  )
  assert.equal(missingBoth.length, 2)
  assert.match(missingBoth[0], /reference\/commands\/list\.md/)
  assert.match(missingBoth[1], /en\/reference\/commands\/list\.md/)
})

test('auditDocBannedPatterns blocks retired/hidden commands, plaintext secret flags, and dead routes with scoped exemptions', () => {
  const activeIssues = auditDocBannedPatterns(
    'guide/quick-start.md',
    [
      'ccr switch work',
      'ccr platform switch codex',
      'ccr -y delete old',
      'ccr claude profile create relay --auth-token sk-ant-123',
      'ccr claude profile set-field work model --value sonnet'
    ].join('\n')
  )
  assert.equal(activeIssues.length, 5)
  assert.equal(auditDocBannedPatterns('guide/quick-start.md', 'ccr list').length, 1)
  assert.equal(auditDocBannedPatterns('guide/quick-start.md', 'ccr ls').length, 1)
  assert.equal(
    auditDocBannedPatterns('guide/quick-start.md', 'ccr grok profile create relay --api-key=xai-123').length,
    1
  )

  // --auth-token-stdin and --api-key-stdin must NOT trigger plaintext secret flag checks
  assert.deepEqual(
    auditDocBannedPatterns(
      'guide/quick-start.md',
      [
        'printf "%s" "$TOKEN" | ccr claude profile create relay --auth-token-stdin',
        'printf "%s" "$KEY" | ccr grok profile create relay --api-key-stdin'
      ].join('\n')
    ),
    []
  )

  // Deprecated command pages may mention hidden/retired commands for migration context,
  // but must still reject nonexistent commands and dead routes
  assert.deepEqual(
    auditDocBannedPatterns('reference/commands/switch.md', 'Legacy `ccr switch work` is retired.'),
    []
  )
  assert.equal(
    auditDocBannedPatterns('reference/commands/switch.md', 'See /reference/api or ccr restore.').length,
    2
  )

  // Historical-mention pages may reference ccr web / ccr migrate / hidden commands,
  // but must STILL fail on nonexistent commands and dead routes
  assert.deepEqual(
    auditDocBannedPatterns(
      'guide/entrypoints.md',
      'Historical `ccr web`, `ccr migrate`, and `ccr platform switch codex` moved.'
    ),
    []
  )
  const historicalAlwaysBanned = auditDocBannedPatterns(
    'guide/entrypoints.md',
    [
      'ccr platform migrate',
      'ccr restore',
      'ccr backups',
      'See /reference/api and /reference/commands/web'
    ].join('\n')
  )
  assert.equal(historicalAlwaysBanned.length, 5)
})
