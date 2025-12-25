# CCR UI - Development Environment Cleanup Script
# Terminates processes listening on specified TCP ports (supports custom ports)
# NOTE: Excludes VS Code related processes to avoid killing shared IDE services

param(
    [string]$BackendPort = "38081",
    [string]$VitePort = "5173"
)

# 支持环境变量覆盖
if ($env:BACKEND_PORT) { $BackendPort = $env:BACKEND_PORT }
if ($env:VITE_PORT) { $VitePort = $env:VITE_PORT }

# VS Code 相关进程名（不应被杀死）
$excludedProcessNames = @('code', 'Code', 'code-server', 'node', 'electron')

Write-Output "... Terminating old processes on ports $BackendPort, $VitePort ..."

$ports = @([int]$BackendPort, [int]$VitePort)

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
$scriptDir = Split-Path -Parent $PSScriptRoot
if (Test-Path "$scriptDir/.backend.pid") {
    Remove-Item "$scriptDir/.backend.pid" -Force -ErrorAction SilentlyContinue
}
if (Test-Path "$scriptDir/.frontend.pid") {
    Remove-Item "$scriptDir/.frontend.pid" -Force -ErrorAction SilentlyContinue
}
