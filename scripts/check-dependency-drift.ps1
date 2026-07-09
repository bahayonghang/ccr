param(
    [switch]$Verbose
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

$RootDir = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
$RootCargoPath = Join-Path $RootDir 'Cargo.toml'
$TauriCargoPath = Join-Path $RootDir 'ccr-ui\src-tauri\Cargo.toml'

$AllowedDrift = @{
    'chrono' = 'Tauri manifest keeps a broad 0.4 constraint; lockfile resolves to a workspace-compatible 0.4.x release.'
    'dirs' = 'Tauri manifest keeps a broad 6 constraint; lockfile resolves to the workspace-compatible 6.0.0 release.'
    'reqwest' = 'Tauri manifest pins a lower 0.13 patch line until desktop HTTP behavior is rechecked; tracked as dependency-governance follow-up.'
    'serde' = 'Tauri manifest keeps a broad 1.0 constraint; lockfile resolves to a workspace-compatible 1.0.x release.'
    'serde_json' = 'Tauri manifest keeps a broad 1.0 constraint; lockfile resolves to a workspace-compatible 1.0.x release.'
    'thiserror' = 'Tauri manifest keeps a broad 2 constraint; lockfile resolves to a compatible version selected by the workspace graph.'
    'tokio' = 'Tauri manifest pins a lower 1.x floor for desktop runtime compatibility; tracked as dependency-governance follow-up.'
    'toml' = 'Tauri slash-command parsing still uses the older toml API surface; keep explicit until migration is tested.'
    'tracing' = 'Tauri manifest keeps a broad 0.1 constraint; lockfile resolves to a workspace-compatible 0.1.x release.'
    'uuid' = 'Tauri manifest pins a lower 1.x floor without serde feature; keep explicit until feature parity is evaluated.'
    'walkdir' = 'Tauri manifest keeps a broad 2 constraint; lockfile resolves to the workspace-compatible 2.5.0 release.'
}

function Fail([string]$Message) {
    Write-Error "❌ $Message"
    exit 1
}

function Get-TomlSection([string]$Content, [string]$SectionName) {
    $escaped = [regex]::Escape($SectionName)
    $match = [regex]::Match($Content, "(?ms)^\[$escaped\]\s*(.*?)(?=^\[|\z)")
    if (-not $match.Success) {
        return ''
    }
    return $match.Groups[1].Value
}

function Get-DependencyVersions([string]$SectionContent) {
    $versions = @{}
    foreach ($rawLine in ($SectionContent -split "`n")) {
        $line = ($rawLine -replace '#.*$', '').Trim()
        if ([string]::IsNullOrWhiteSpace($line) -or -not $line.Contains('=')) {
            continue
        }
        $parts = $line -split '=', 2
        $name = $parts[0].Trim()
        $value = $parts[1].Trim()
        $version = $null
        if ($value -match '^"([^"]+)"$') {
            $version = $matches[1]
        } elseif ($value -match 'version\s*=\s*"([^"]+)"') {
            $version = $matches[1]
        }
        if (-not [string]::IsNullOrWhiteSpace($version)) {
            $versions[$name] = $version
        }
    }
    return $versions
}

foreach ($path in @($RootCargoPath, $TauriCargoPath)) {
    if (-not (Test-Path $path)) {
        Fail "文件不存在: $path"
    }
}

$rootCargo = Get-Content $RootCargoPath -Raw
$tauriCargo = Get-Content $TauriCargoPath -Raw
$workspaceDeps = Get-DependencyVersions (Get-TomlSection $rootCargo 'workspace.dependencies')
$tauriDeps = Get-DependencyVersions (Get-TomlSection $tauriCargo 'dependencies')

if ($workspaceDeps.Count -eq 0) {
    Fail '根 Cargo.toml 缺少可解析的 [workspace.dependencies]'
}
if ($tauriDeps.Count -eq 0) {
    Fail 'ccr-ui/src-tauri/Cargo.toml 缺少可解析的 [dependencies]'
}

$failures = New-Object System.Collections.Generic.List[string]
$drifts = New-Object System.Collections.Generic.List[string]
$checked = 0
foreach ($name in ($tauriDeps.Keys | Sort-Object)) {
    if (-not $workspaceDeps.ContainsKey($name)) {
        continue
    }
    $checked++
    $workspaceVersion = [string]$workspaceDeps[$name]
    $tauriVersion = [string]$tauriDeps[$name]
    if ($workspaceVersion -eq $tauriVersion) {
        continue
    }
    $entry = "$name root=$workspaceVersion tauri=$tauriVersion"
    if ($AllowedDrift.ContainsKey($name)) {
        $drifts.Add("$entry reason=$($AllowedDrift[$name])") | Out-Null
    } else {
        $failures.Add($entry) | Out-Null
    }
}

foreach ($name in ($AllowedDrift.Keys | Sort-Object)) {
    if (-not $workspaceDeps.ContainsKey($name) -or -not $tauriDeps.ContainsKey($name)) {
        $failures.Add("allowlist entry '$name' no longer maps to a repeated dependency") | Out-Null
        continue
    }
    if ([string]$workspaceDeps[$name] -eq [string]$tauriDeps[$name]) {
        $failures.Add("allowlist entry '$name' is stale because versions now match") | Out-Null
    }
}

if ($failures.Count -gt 0) {
    Write-Host '❌ Root/Tauri dependency drift check failed:' -ForegroundColor Red
    foreach ($failure in $failures) {
        Write-Host "  - $failure" -ForegroundColor Red
    }
    Write-Host 'Add an explicit reason to the allowlist only after reviewing the drift.' -ForegroundColor Yellow
    exit 1
}

if ($Verbose) {
    Write-Host "🔎 Repeated dependencies checked: $checked"
    if ($drifts.Count -gt 0) {
        Write-Host '📌 Explicitly allowed root/Tauri dependency drifts:'
        foreach ($drift in $drifts) {
            Write-Host "  - $drift"
        }
    } else {
        Write-Host '📌 No root/Tauri dependency drifts found.'
    }
}

Write-Host '✅ root/Tauri dependency drift 检查通过'
