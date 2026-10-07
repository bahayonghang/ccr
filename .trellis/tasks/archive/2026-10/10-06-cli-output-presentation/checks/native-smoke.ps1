param(
    [Parameter(Mandatory = $true)][string]$Binary,
    [Parameter(Mandatory = $true)][string]$FixtureReport,
    [ValidateSet(40, 80, 120)][int]$Width = 80,
    [ValidateSet('dark', 'light')][string]$Background = 'dark',
    [ValidateSet('normal', 'no-color', 'dumb')][string]$Mode = 'normal',
    [string]$Round = 'final',
    [switch]$SkipDoctor
)

$ErrorActionPreference = 'Stop'
$nativeBinary = (Resolve-Path -LiteralPath $Binary).Path
$nativeFixture = Get-Content -LiteralPath $FixtureReport -Raw | ConvertFrom-Json
$nativeOverrides = @{}
foreach ($property in $nativeFixture.environment_overrides.PSObject.Properties) {
    $nativeOverrides[$property.Name] = [string]$property.Value
}
$nativeOverrides['TERM'] = if ($Mode -eq 'dumb') { 'dumb' } else { 'xterm-256color' }
$nativeOverrides['NO_COLOR'] = if ($Mode -eq 'no-color') { '1' } else { $null }
$nativeOverrides['CLICOLOR'] = '1'
$nativeOverrides['CLICOLOR_FORCE'] = $null
$nativeOverrides['COLUMNS'] = [string]$Width
foreach ($name in @('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'ANTHROPIC_AUTH_TOKEN', 'ANTHROPIC_API_KEY', 'ANTHROPIC_BASE_URL', 'OPENAI_API_KEY', 'XAI_API_KEY', 'FORCE_COLOR', 'CLAUDE_CODE_OAUTH_TOKEN', 'CLAUDE_CODE_USE_BEDROCK', 'CLAUDE_CODE_USE_VERTEX', 'CLAUDE_CODE_USE_FOUNDRY')) {
    $nativeOverrides[$name] = $null
}
$nativePrevious = @{}
$nativePreviousForeground = [Console]::ForegroundColor
$nativePreviousBackground = [Console]::BackgroundColor
Add-Type -Path (Join-Path $PSScriptRoot 'ConsoleEvidence.cs')
$nativeSnapshotPrefix = Join-Path $PSScriptRoot ('native-' + $Round + '-' + $Width + '-' + $Background + '-' + $Mode)
$nativeExitCodes = @()
$nativeCases = @()
$nativeCopiedCommands = @()
$nativeAuthPath = Join-Path $nativeOverrides['CCR_CODEX_DIR'] 'auth.json'
$nativeOriginalAuth = [IO.File]::ReadAllBytes($nativeAuthPath)
$nativePreviousLocation = Get-Location
try {
    Set-Location -LiteralPath $nativeFixture.fixture
    foreach ($name in $nativeOverrides.Keys) {
        $nativePrevious[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
        if ($null -eq $nativeOverrides[$name]) { Remove-Item -LiteralPath ('Env:' + $name) -ErrorAction SilentlyContinue }
        else { Set-Item -LiteralPath ('Env:' + $name) -Value $nativeOverrides[$name] }
    }
    [ConsoleEvidence]::ConfigureTheme($Background -eq 'light')
    [Console]::SetWindowSize($Width, 40)
    [Console]::Clear()
    Write-Host ('NATIVE_BEGIN width=' + [Console]::WindowWidth + '; height=' + [Console]::WindowHeight + '; background=' + $Background + '; mode=' + $Mode + '; stdin_redirected=' + [Console]::IsInputRedirected + '; stdout_redirected=' + [Console]::IsOutputRedirected)
    Write-Host 'CASE save-long-multiline'
    $nativeCases += 'save-long-multiline'
    $nativeDescription = ('long-field-' + ('abcdefghijklmnopqrstuvwxyz0123456789' * 3) + "`n多行描述保持完整")
    & $nativeBinary codex auth save --description $nativeDescription --force -- teacher
    $nativeExitCodes += $LASTEXITCODE
    [ConsoleEvidence]::Snapshot($nativeSnapshotPrefix + '-save-screen.json')
    Write-Host ('EXIT=' + $LASTEXITCODE)
    [Console]::Clear()
    Write-Host 'CASE duplicate-error'
    $nativeCases += 'duplicate-error'
    & $nativeBinary codex auth save teacher
    $nativeExitCodes += $LASTEXITCODE
    [ConsoleEvidence]::Snapshot($nativeSnapshotPrefix + '-duplicate-screen.json')
    Write-Host ('EXIT=' + $LASTEXITCODE)
    $nativeSourceSnapshot = $nativeSnapshotPrefix + '-duplicate-screen.json'
    $nativeScreen = Get-Content -LiteralPath $nativeSourceSnapshot -Raw | ConvertFrom-Json
    $nativeScreenText = ($nativeScreen.Rows | ForEach-Object { $_.Text }) -join ''
    $nativeCopy = [regex]::Match($nativeScreenText, 'ccr codex auth save --force -- teacher').Value
    if (-not $nativeCopy) { throw 'Ordinary overwrite suggestion missing from native screen' }
    [Console]::Clear()
    Write-Host 'CASE copy-normal'
    Write-Host ('COPIED=' + $nativeCopy)
    $nativeCases += 'copy-normal'
    $nativeCopyArguments = $nativeCopy.Split(' ', [StringSplitOptions]::RemoveEmptyEntries)[1..6]
    & $nativeBinary @nativeCopyArguments
    $nativeExitCodes += $LASTEXITCODE
    [ConsoleEvidence]::Snapshot($nativeSnapshotPrefix + '-copy-normal-screen.json')
    $nativeCopiedCommands += @{ text = $nativeCopy; exit_code = $LASTEXITCODE; source_snapshot = $nativeSourceSnapshot; result_snapshot = $nativeSnapshotPrefix + '-copy-normal-screen.json' }
    Write-Host ('EXIT=' + $LASTEXITCODE)
    [Console]::Clear()
    Write-Host 'CASE current-runtime'
    $nativeCases += 'current-runtime'
    & $nativeBinary codex auth current
    $nativeExitCodes += $LASTEXITCODE
    [ConsoleEvidence]::Snapshot($nativeSnapshotPrefix + '-current-screen.json')
    Write-Host ('EXIT=' + $LASTEXITCODE)
    if (-not $SkipDoctor) {
        [Console]::Clear()
        Write-Host 'CASE doctor-statuses'
        $nativeCases += 'doctor-statuses'
        & $nativeBinary doctor --platform codex
        $nativeExitCodes += $LASTEXITCODE
        [ConsoleEvidence]::Snapshot($nativeSnapshotPrefix + '-doctor-screen.json')
        Write-Host ('EXIT=' + $LASTEXITCODE)
    }
    $nativeAuth = [Text.Encoding]::UTF8.GetString($nativeOriginalAuth) | ConvertFrom-Json
    $nativeNoEmailPayload = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes('{"sub":"synthetic"}')).TrimEnd('=').Replace('+', '-').Replace('/', '_')
    $nativeAuth.tokens.id_token = 'eyJhbGciOiJub25lIn0.' + $nativeNoEmailPayload + '.signature'
    [IO.File]::WriteAllText($nativeAuthPath, ($nativeAuth | ConvertTo-Json -Depth 8), [Text.UTF8Encoding]::new($false))
    [Console]::Clear()
    Write-Host 'CASE missing-fields'
    $nativeCases += 'missing-fields'
    & $nativeBinary codex auth save --force -- native-missing
    $nativeExitCodes += $LASTEXITCODE
    [ConsoleEvidence]::Snapshot($nativeSnapshotPrefix + '-missing-fields-screen.json')
    Write-Host ('EXIT=' + $LASTEXITCODE)
    [Console]::Clear()
    Write-Host 'CASE leading-seed'
    $nativeCases += 'leading-seed'
    & $nativeBinary codex auth save --force -- -teacher
    $nativeExitCodes += $LASTEXITCODE
    [ConsoleEvidence]::Snapshot($nativeSnapshotPrefix + '-leading-seed-screen.json')
    Write-Host ('EXIT=' + $LASTEXITCODE)
    [Console]::Clear()
    Write-Host 'CASE duplicate-leading'
    $nativeCases += 'duplicate-leading'
    & $nativeBinary codex auth save -- -teacher
    $nativeExitCodes += $LASTEXITCODE
    [ConsoleEvidence]::Snapshot($nativeSnapshotPrefix + '-duplicate-leading-screen.json')
    Write-Host ('EXIT=' + $LASTEXITCODE)
    $nativeSourceSnapshot = $nativeSnapshotPrefix + '-duplicate-leading-screen.json'
    $nativeScreen = Get-Content -LiteralPath $nativeSourceSnapshot -Raw | ConvertFrom-Json
    $nativeScreenText = ($nativeScreen.Rows | ForEach-Object { $_.Text }) -join ''
    $nativeCopy = [regex]::Match($nativeScreenText, 'ccr codex auth save --force -- -teacher').Value
    if (-not $nativeCopy) { throw 'Leading-hyphen overwrite suggestion missing from native screen' }
    [Console]::Clear()
    Write-Host 'CASE copy-leading'
    Write-Host ('COPIED=' + $nativeCopy)
    $nativeCases += 'copy-leading'
    $nativeCopyArguments = $nativeCopy.Split(' ', [StringSplitOptions]::RemoveEmptyEntries)[1..6]
    & $nativeBinary @nativeCopyArguments
    $nativeExitCodes += $LASTEXITCODE
    [ConsoleEvidence]::Snapshot($nativeSnapshotPrefix + '-copy-leading-screen.json')
    $nativeCopiedCommands += @{ text = $nativeCopy; exit_code = $LASTEXITCODE; source_snapshot = $nativeSourceSnapshot; result_snapshot = $nativeSnapshotPrefix + '-copy-leading-screen.json' }
    Write-Host ('EXIT=' + $LASTEXITCODE)
    Write-Host 'NATIVE_END'
    $nativeReceipt = [ordered]@{
        binary = $nativeBinary
        sha256 = (Get-FileHash -LiteralPath $nativeBinary -Algorithm SHA256).Hash.ToLowerInvariant()
        width = [Console]::WindowWidth
        height = [Console]::WindowHeight
        background = $Background
        mode = $Mode
        stdin_redirected = [Console]::IsInputRedirected
        stdout_redirected = [Console]::IsOutputRedirected
        stderr_redirected = [Console]::IsErrorRedirected
        cases = $nativeCases
        copied_commands = $nativeCopiedCommands
        doctor = if ($SkipDoctor) { 'NOT_RUN_WINDOWS_ISOLATION' } else { 'RUN' }
        exit_codes = $nativeExitCodes
        description = $nativeDescription
        snapshot_prefix = $nativeSnapshotPrefix
    }
    [IO.File]::WriteAllText($nativeSnapshotPrefix + '-receipt.json', ($nativeReceipt | ConvertTo-Json -Depth 4), [Text.UTF8Encoding]::new($false))
} finally {
    Set-Location -LiteralPath $nativePreviousLocation.Path
    [IO.File]::WriteAllBytes($nativeAuthPath, $nativeOriginalAuth)
    [ConsoleEvidence]::RestoreTheme()
    [Console]::ForegroundColor = $nativePreviousForeground
    [Console]::BackgroundColor = $nativePreviousBackground
    foreach ($name in $nativePrevious.Keys) {
        if ($null -eq $nativePrevious[$name]) { Remove-Item -LiteralPath ('Env:' + $name) -ErrorAction SilentlyContinue }
        else { Set-Item -LiteralPath ('Env:' + $name) -Value $nativePrevious[$name] }
    }
}
exit 0
