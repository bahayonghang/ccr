# CCR UI - Development Environment Cleanup Script
# Terminates processes listening on specified TCP ports (supports custom ports)
# NOTE: Excludes VS Code related processes to avoid killing shared IDE services

param(
    [string]$BackendPort = "48081",
    [string]$VitePort = "15173"
)

# ========== UTF-8 Encoding Setup (Fix Chinese character display) ==========
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::InputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
chcp 65001 | Out-Null

# 支持环境变量覆盖
if ($env:BACKEND_PORT) { $BackendPort = $env:BACKEND_PORT }
if ($env:VITE_PORT) { $VitePort = $env:VITE_PORT }

# 读取前端实际端口（用于处理 Vite 自动换端口的情况）
$scriptRoot = $PSScriptRoot
if (-not $scriptRoot) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
}
if (-not $scriptRoot) {
    $scriptRoot = (Get-Location).Path
}
$rootDir = $scriptRoot
if ((Split-Path -Leaf $scriptRoot) -ieq "scripts") {
    $rootDir = Split-Path -Parent $scriptRoot
}
$frontendPortFile = Join-Path $rootDir "logs/frontend.port"
$backendPidFile = Join-Path $rootDir ".backend.pid"
$frontendPidFile = Join-Path $rootDir ".frontend.pid"
$actualVitePort = $null
if (Test-Path $frontendPortFile) {
    $portFromFile = Get-Content $frontendPortFile -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($portFromFile -match '^\d+$') {
        $actualVitePort = $portFromFile
    }
}

# VS Code 相关进程名（不应被杀死）
$excludedProcessNames = @('code', 'Code', 'code-server', 'node', 'electron')

$ports = @([int]$BackendPort, [int]$VitePort)
if ($actualVitePort -and ($actualVitePort -ne $VitePort)) {
    $ports += [int]$actualVitePort
}
$ports = $ports | Sort-Object -Unique
Write-Output ("... Terminating old processes on ports " + ($ports -join ", ") + " ...")

if (Test-Path $backendPidFile) {
    $backendPid = Get-Content $backendPidFile -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($backendPid -match '^\d+$') {
        $backendPid = [int]$backendPid
        $backendProc = Get-Process -Id $backendPid -ErrorAction SilentlyContinue
        if ($backendProc) {
            $backendListening = Get-NetTCPConnection -OwningProcess $backendPid -State Listen -ErrorAction SilentlyContinue |
                Where-Object { $_.LocalPort -eq [int]$BackendPort } | Select-Object -First 1
            if ($backendListening) {
                Write-Output ("  - Stopping backend process (PID: " + $backendPid + ") from PID file ...")
                Stop-Process -Id $backendPid -Force -ErrorAction SilentlyContinue
            }
        }
    }
    Remove-Item -Path $backendPidFile -Force -ErrorAction SilentlyContinue
}

if (Test-Path $frontendPidFile) {
    $frontendPid = Get-Content $frontendPidFile -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($frontendPid -match '^\d+$') {
        $frontendPid = [int]$frontendPid
        $frontendProc = Get-Process -Id $frontendPid -ErrorAction SilentlyContinue
        if ($frontendProc) {
            $frontendPortToCheck = $VitePort
            if ($actualVitePort) {
                $frontendPortToCheck = $actualVitePort
            }
            $frontendListening = Get-NetTCPConnection -OwningProcess $frontendPid -State Listen -ErrorAction SilentlyContinue |
                Where-Object { $_.LocalPort -eq [int]$frontendPortToCheck } | Select-Object -First 1
            if ($frontendListening) {
                Write-Output ("  - Stopping frontend process (PID: " + $frontendPid + ") from PID file ...")
                Stop-Process -Id $frontendPid -Force -ErrorAction SilentlyContinue
            }
        }
    }
    Remove-Item -Path $frontendPidFile -Force -ErrorAction SilentlyContinue
}

foreach ($port in $ports) {
    try {
        $conn = Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction Stop 2>$null
        if ($conn) {
            $procId = $conn.OwningProcess
            $proc = Get-Process -Id $procId -ErrorAction SilentlyContinue
            
            if ($proc) {
                # 检查是否为 VS Code 相关进程
                $isExcluded = $false
                foreach ($excludedName in $excludedProcessNames) {
                    if ($proc.ProcessName -match $excludedName) {
                        $isExcluded = $true
                        break
                    }
                }
                
                if ($isExcluded) {
                    Write-Output ("  - Skipping port " + $port + " (VS Code related process: " + $proc.ProcessName + ", PID: " + $procId + ")")
                } else {
                    Write-Output ("  - Terminating process on port " + $port + " (" + $proc.ProcessName + ", PID: " + $procId + ") ...")
                    Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue
                }
            } else {
                Write-Output ("  - Process on port " + $port + " (PID: " + $procId + ") no longer exists.")
            }
        }
    } catch {
        # Port is not in use or other error - this is fine
        Write-Output ("  - Port " + $port + " is not in use. No action needed.")
    }
}

# 清理 PID 文件（如果存在）
if (Test-Path "$rootDir/.backend.pid") {
    Remove-Item "$rootDir/.backend.pid" -Force -ErrorAction SilentlyContinue
}
if (Test-Path "$rootDir/.frontend.pid") {
    Remove-Item "$rootDir/.frontend.pid" -Force -ErrorAction SilentlyContinue
}
