# CCR UI - Windows browser web dev launcher
# Browser preview only; do not launch the Tauri desktop shell.

param(
    [string]$RootDir = $PSScriptRoot,
    [string]$BackendPort = "48081",
    [string]$VitePort = "15173"
)

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::InputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
chcp 65001 | Out-Null

$ErrorActionPreference = 'Stop'
$script:ExitCode = 0

if ($env:BACKEND_PORT) { $BackendPort = $env:BACKEND_PORT }
if ($env:VITE_PORT) { $VitePort = $env:VITE_PORT }

if ($RootDir -eq $PSScriptRoot) {
    $RootDir = Split-Path -Parent $PSScriptRoot
}
Set-Location $RootDir

function Invoke-CleanDevEnvironment {
    Write-Output '[WebDev] Cleaning browser web dev environment...'
    & powershell -ExecutionPolicy Bypass -File (Join-Path $RootDir 'scripts/clean_dev.ps1') -BackendPort $BackendPort -VitePort $VitePort -StopTauriDesktop
    if ($env:CCR_DEV_RESET_VITE_CACHE -eq '1' -and (Test-Path 'node_modules/.vite')) {
        Write-Output '[WebDev] CCR_DEV_RESET_VITE_CACHE=1; removing the Vite dependency cache...'
        Remove-Item -Recurse -Force 'node_modules/.vite'
    }
    Write-Output '[WebDev] Cleanup complete'
    Write-Output ''
}

function Get-PortOwners {
    param([int]$Port)

    $owners = @()
    $seen = @{}

    try {
        $connections = @(Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue |
            Where-Object { $_.State -in @('Listen', 'Bound') })
        foreach ($connection in $connections) {
            $portProcessId = [int]$connection.OwningProcess
            if ($seen.ContainsKey($portProcessId)) {
                continue
            }
            $seen[$portProcessId] = $true
            try {
                $proc = Get-CimInstance Win32_Process -Filter "ProcessId=$portProcessId" -ErrorAction Stop
                $owners += [pscustomobject]@{
                    ProcessId = $portProcessId
                    Name = $proc.Name
                    ExecutablePath = $proc.ExecutablePath
                    CommandLine = $proc.CommandLine
                }
            } catch {
                $owners += [pscustomobject]@{
                    ProcessId = $portProcessId
                    Name = $null
                    ExecutablePath = $null
                    CommandLine = $null
                }
            }
        }
    } catch {
        # Ignore and fall back to netstat below.
    }

    $pattern = "^\s*TCP\s+\S+:$Port\s+\S+\s+LISTENING\s+(\d+)\s*$"
    $matches = netstat -ano -p tcp | Select-String -Pattern $pattern

    foreach ($match in $matches) {
        if ($match.Matches.Count -eq 0) {
            continue
        }

        $portProcessId = [int]$match.Matches[0].Groups[1].Value
        if ($seen.ContainsKey($portProcessId)) {
            continue
        }
        $seen[$portProcessId] = $true
        try {
            $proc = Get-CimInstance Win32_Process -Filter "ProcessId=$portProcessId" -ErrorAction Stop
            $owners += [pscustomobject]@{
                ProcessId = $portProcessId
                Name = $proc.Name
                ExecutablePath = $proc.ExecutablePath
                CommandLine = $proc.CommandLine
            }
        } catch {
            $owners += [pscustomobject]@{
                ProcessId = $portProcessId
                Name = $null
                ExecutablePath = $null
                CommandLine = $null
            }
        }
    }

    return @($owners | Sort-Object ProcessId -Unique)
}

Invoke-CleanDevEnvironment

$portReady = $false
for ($i = 0; $i -lt 5; $i++) {
    $owners = Get-PortOwners -Port ([int]$VitePort)
    if ($owners.Count -eq 0) {
        $portReady = $true
        break
    }
    Start-Sleep -Seconds 1
}

if (-not $portReady) {
    Write-Host ("[ERROR] Frontend port {0} is still in use; cannot start browser preview." -f $VitePort) -ForegroundColor Red
    Write-Host 'Port owner details:' -ForegroundColor Yellow
    foreach ($owner in $owners) {
        Write-Host ("  - PID: {0}, Name: {1}" -f $owner.ProcessId, $owner.Name)
        if ($owner.CommandLine) {
            Write-Host ("    Cmd: {0}" -f $owner.CommandLine)
        } elseif ($owner.ExecutablePath) {
            Write-Host ("    Exe: {0}" -f $owner.ExecutablePath)
        }
    }
    exit 1
}

Write-Output '[WebDev] Starting browser web preview...'
Write-Output ("[WebDev] Frontend: http://localhost:{0}" -f $VitePort)
Write-Output ("[WebDev] Bind address: 0.0.0.0:{0}" -f $VitePort)
Write-Output '[WebDev] Note: this does not launch the Tauri desktop shell.'
Write-Output '[WebDev] Note: pages relying on Tauri invoke() may be limited in pure browser runtime.'
Write-Output ''

New-Item -ItemType Directory -Force -Path (Join-Path $RootDir 'logs') | Out-Null
Set-Content -Path (Join-Path $RootDir 'logs/frontend.port') -Value $VitePort -Encoding ASCII

try {
    $env:BACKEND_PORT = "$BackendPort"
    & bun run dev:web -- --host 0.0.0.0 --strictPort --port "$VitePort"
    if ($LASTEXITCODE -eq 255) {
        $script:ExitCode = 0
    } elseif ($LASTEXITCODE) {
        $script:ExitCode = $LASTEXITCODE
    }
} catch {
    Write-Host ''
    Write-Host '[Info] Web dev server stopped' -ForegroundColor Yellow
    $script:ExitCode = 0
} finally {
    [Environment]::Exit($script:ExitCode)
}
