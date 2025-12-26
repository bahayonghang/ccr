# CCR UI - Windows Fast Development Server (Release Mode)
# Uses pre-compiled Release binary for faster startup

param(
    [string]$RootDir = $PSScriptRoot,
    [string]$BackendPort = "48081",
    [string]$VitePort = "15173"
)

$ErrorActionPreference = "Stop"

# 支持环境变量覆盖
if ($env:BACKEND_PORT) { $BackendPort = $env:BACKEND_PORT }
if ($env:VITE_PORT) { $VitePort = $env:VITE_PORT }

# Navigate to ccr-ui root directory
if ($RootDir -eq $PSScriptRoot) {
    $RootDir = Split-Path -Parent $PSScriptRoot
}
Set-Location $RootDir

# Create logs directory
if (-not (Test-Path logs)) {
    New-Item -ItemType Directory -Path logs -Force | Out-Null
}

Write-Host "[CCR] Fast development mode (Release build)..." -ForegroundColor Cyan
Write-Host ""

# ========== Check/Build Release Binary ==========
$backendBinary = "$RootDir/backend/target/release/ccr-ui-backend.exe"
if (-not (Test-Path $backendBinary)) {
    Write-Host "[Backend] Release binary not found, building..." -ForegroundColor Yellow
    Set-Location "$RootDir/backend"
    cargo build --release
    if (-not $?) {
        Write-Host "[ERROR] Failed to build Release binary" -ForegroundColor Red
        exit 1
    }
    Write-Host "[Backend] Release build completed" -ForegroundColor Green
    Set-Location $RootDir
}

# ========== Start Backend (Background Job) ==========
Write-Host "[Backend] Starting server (Release mode, background job)..." -ForegroundColor Yellow

$backendJob = Start-Job -ScriptBlock {
    param($workDir, $binary, $port)
    Set-Location $workDir

    # Run pre-compiled binary directly (传递端口参数)
    $logPath = "$workDir/logs/backend-console.log"
    & $binary --port $port 2>&1 | Tee-Object -FilePath $logPath -Append
} -ArgumentList $RootDir, $backendBinary, $BackendPort

Write-Host "[Backend] Started in background (Job ID: $($backendJob.Id))" -ForegroundColor Green
Write-Host "          Log file: logs/backend-console.log" -ForegroundColor Gray
Write-Host ""

# ========== Wait for Backend Ready ==========
# Release 模式启动很快，只需 15 秒超时
$maxWait = 15
Write-Host "[Backend] Waiting for health check (http://127.0.0.1:$BackendPort/health)..." -ForegroundColor Cyan

$backendReady = $false
for ($i = 0; $i -lt $maxWait; $i++) {
    # Check if backend job is still running
    $jobState = (Get-Job -Id $backendJob.Id).State
    if ($jobState -eq "Failed" -or $jobState -eq "Stopped") {
        Write-Host ""
        Write-Host "[ERROR] Backend process exited unexpectedly. Check logs:" -ForegroundColor Red
        Receive-Job -Job $backendJob -ErrorAction SilentlyContinue | Write-Host
        Remove-Job -Job $backendJob -Force
        exit 1
    }

    # Health check
    try {
        $response = Invoke-WebRequest -UseBasicParsing -TimeoutSec 2 -Uri "http://127.0.0.1:$BackendPort/health" -ErrorAction SilentlyContinue
        if ($response.StatusCode -eq 200) {
            Write-Host "[Backend] Ready!" -ForegroundColor Green
            $backendReady = $true
            break
        }
    } catch {
        # Silent failure, continue waiting
    }

    Start-Sleep -Seconds 1

    # Timeout detection
    if ($i -eq ($maxWait - 1)) {
        Write-Host ""
        Write-Host "[ERROR] Backend health check timeout (${maxWait}s)" -ForegroundColor Red
        Write-Host "        Check logs/backend-console.log for details" -ForegroundColor Red
        Write-Host ""
        Write-Host "Recent backend log output:" -ForegroundColor Yellow
        Receive-Job -Job $backendJob -Keep | Select-Object -Last 20 | Write-Host
        Stop-Job -Job $backendJob
        Remove-Job -Job $backendJob -Force
        exit 1
    }
}

Write-Host ""

# ========== Start Frontend (Foreground) ==========
Write-Host "[Frontend] Starting server (foreground, live logs visible)..." -ForegroundColor Yellow
Write-Host "           Log file: logs/frontend.log" -ForegroundColor Gray
Write-Host ""
Write-Host "[TIP] Press Ctrl+C to stop both backend and frontend servers" -ForegroundColor Cyan
Write-Host "======================================================================" -ForegroundColor DarkGray
Write-Host ""

try {
    Set-Location "$RootDir/frontend"

    # Suppress PowerShell treating stderr as error
    $ErrorActionPreference = "Continue"

    # Frontend runs in foreground with live output (传递端口参数)
    # --host 0.0.0.0 allows access from LAN IP addresses
    cmd /c "bun run dev -- --host 0.0.0.0 --port $VitePort 2>&1 || exit 0" | Tee-Object -FilePath "$RootDir/logs/frontend.log" -Append
} finally {
    # Cleanup: Stop backend job
    Write-Host ""
    Write-Host "[Cleanup] Stopping backend server..." -ForegroundColor Yellow

    if (Get-Job -Id $backendJob.Id -ErrorAction SilentlyContinue) {
        Stop-Job -Job $backendJob -ErrorAction SilentlyContinue
        Remove-Job -Job $backendJob -Force -ErrorAction SilentlyContinue
        Write-Host "[Cleanup] Backend stopped" -ForegroundColor Green
    }

    Write-Host "[CCR] Development environment closed" -ForegroundColor Cyan
}

exit 0
