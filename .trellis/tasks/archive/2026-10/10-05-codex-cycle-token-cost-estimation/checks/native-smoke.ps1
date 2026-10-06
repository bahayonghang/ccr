param(
    [Parameter(Mandatory = $true)][string]$Binary,
    [int]$Width = 140,
    [int]$Height = 40
)

$ErrorActionPreference = 'Stop'
$nativeBinary = (Resolve-Path -LiteralPath $Binary).Path
$nativeFixture = Join-Path ([IO.Path]::GetTempPath()) ('ccr-codex-native-' + [guid]::NewGuid().ToString('N'))
$nativeRoot = Join-Path $nativeFixture 'ccr'
$nativeCodex = Join-Path $nativeFixture 'codex'
$nativeAccountDir = Join-Path $nativeRoot 'platforms/codex'
$nativeSessionDir = Join-Path $nativeCodex 'sessions'
$nativeClaude = Join-Path $nativeFixture 'claude'
$nativeGrok = Join-Path $nativeFixture 'grok'
foreach ($nativeDirectory in @($nativeRoot, $nativeAccountDir, $nativeSessionDir, $nativeClaude, $nativeGrok)) {
    New-Item -ItemType Directory -Path $nativeDirectory -Force | Out-Null
}

$nativeNow = [DateTimeOffset]::UtcNow
$nativeStarted = $nativeNow.AddHours(-2).ToString('o')
$nativeSaved = $nativeNow.AddDays(-1).ToString('o')
$nativeRegistry = @"
version = "1.0"
current_auth = "synthetic"

[[usage_ledger]]
account_name = "synthetic"
account_id = "synthetic-account"
started_at = "$nativeStarted"

[accounts.synthetic]
account_id = "synthetic-account"
auth_method = "chatgpt"
plan_type = "plus"
saved_at = "$nativeSaved"
"@
Set-Content -LiteralPath (Join-Path $nativeAccountDir 'auth_registry.toml') -Value $nativeRegistry -Encoding utf8NoBOM
Set-Content -LiteralPath (Join-Path $nativeCodex 'config.toml') -Value 'model = "gpt-6.1-sol"', 'model_provider = "openai"', 'cli_auth_credentials_store = "file"' -Encoding utf8NoBOM
$nativeLines = [Collections.Generic.List[string]]::new()
$nativeLines.Add((@{type = 'session_meta'; timestamp = $nativeStarted; payload = @{id = 'synthetic-session'; model = 'gpt-6.1-sol'; model_provider = 'openai'; account_id = 'synthetic-account'}} | ConvertTo-Json -Compress -Depth 8))
for ($nativeIndex = 0; $nativeIndex -lt 100; $nativeIndex++) {
    $nativeLines.Add((@{
        type = 'response.completed'
        timestamp = $nativeNow.AddHours(-1).AddSeconds($nativeIndex).ToString('o')
        response = @{
            id = ('synthetic-response-' + $nativeIndex)
            model = 'gpt-6.1-sol'
            speed = 'standard'
            usage = @{
                input_tokens = 100000
                cached_input_tokens = 90000
                cache_write_input_tokens = 5000
                output_tokens = 2000
                reasoning_output_tokens = 1000
                total_tokens = 102000
            }
        }
    } | ConvertTo-Json -Compress -Depth 8))
}
Set-Content -LiteralPath (Join-Path $nativeSessionDir 'synthetic-rollout.jsonl') -Value $nativeLines -Encoding utf8NoBOM

# Every path override is process-local. No auth.json exists in the fixture.
# Quota preview must fail on the missing local auth file before an HTTP request.
$nativeEnvironment = @{
    TERM = 'xterm-256color'
    USERPROFILE = $nativeFixture
    CCR_ROOT = $nativeRoot
    CCR_DATA_DIR = $nativeRoot
    CCR_CONFIG_PATH = (Join-Path $nativeRoot 'config.toml')
    CCR_LOCK_DIR = (Join-Path $nativeFixture 'locks')
    CCR_CODEX_DIR = $nativeCodex
    CLAUDE_CONFIG_DIR = $nativeClaude
    CLAUDE_JSON_PATH = (Join-Path $nativeClaude '.claude.json')
    CCR_SETTINGS_PATH = (Join-Path $nativeClaude 'settings.json')
    CCR_BACKUP_DIR = (Join-Path $nativeClaude 'backups')
    GROK_HOME = $nativeGrok
}
$nativePrevious = @{}
foreach ($nativeKey in $nativeEnvironment.Keys) {
    $nativePrevious[$nativeKey] = [Environment]::GetEnvironmentVariable($nativeKey, 'Process')
    [Environment]::SetEnvironmentVariable($nativeKey, $nativeEnvironment[$nativeKey], 'Process')
}
try {
    try { [Console]::SetWindowSize($Width, $Height) } catch { Write-Host ('Requested size unavailable: ' + $_.Exception.GetType().Name) }
    Write-Host ('Synthetic native fixture: ' + $nativeFixture)
    Write-Host ('Native console size: ' + [Console]::WindowWidth + 'x' + [Console]::WindowHeight)
    Write-Host ('Native console redirected: input=' + [Console]::IsInputRedirected + '; output=' + [Console]::IsOutputRedirected)
    Write-Host 'Expected rolling total: 10.2M Token / 100 usage records / API USD 5.15; capacity N/A without samples.'
    & $nativeBinary codex
    $nativeExitCode = $LASTEXITCODE
    Write-Host ('Native process exit: ' + $nativeExitCode)
    exit $nativeExitCode
} finally {
    foreach ($nativeKey in $nativePrevious.Keys) {
        [Environment]::SetEnvironmentVariable($nativeKey, $nativePrevious[$nativeKey], 'Process')
    }
}
