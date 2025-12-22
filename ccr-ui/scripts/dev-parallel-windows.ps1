# CCR UI - Windows Development Server Parallel Launcher
# Uses PowerShell background jobs to run backend/frontend in current window
# Avoids opening new popup windows

param(
    [string]$RootDir = $PSScriptRoot,
    [string]$BackendPort = "38081",
    [string]$VitePort = "5173"
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

Write-Host "[CCR] Starting development environment (parallel mode)..." -ForegroundColor Cyan
Write-Host ""

# ========== Start Backend (Background Job) ==========
Write-Host "[Backend] Starting server (background job)..." -ForegroundColor Yellow

$backendJob = Start-Job -ScriptBlock {
    param($workDir, $port)
    Set-Location "$workDir/backend"

    # Run cargo and log output (传递端口参数)
    $logPath = "$workDir/logs/backend-console.log"
    cargo run -- --port $port 2>&1 | Tee-Object -FilePath $logPath -Append
} -ArgumentList $RootDir, $BackendPort

Write-Host "[Backend] Started in background (Job ID: $($backendJob.Id))" -ForegroundColor Green
Write-Host "          Log file: logs/backend-console.log" -ForegroundColor Gray
Write-Host ""

# ========== Wait for Backend Ready ==========
# 动态超时：已编译则 30 秒，未编译则 120 秒
if (Test-Path "$RootDir/backend/target/debug") {
    $maxWait = 30
} else {
    $maxWait = 120
}
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

    # Suppress PowerShell treating stderr as error (bun/vite outputs to stderr)
    $ErrorActionPreference = "Continue"

    # Frontend runs in foreground with live output, also writes to log (传递端口参数)
    # Use cmd /c to prevent PowerShell from treating stderr as terminating error
    # The '|| exit 0' ensures that if the process is killed (e.g. Ctrl+C), it doesn't return failure to Just
    cmd /c "bun run dev -- --port $VitePort 2>&1 || exit 0" | Tee-Object -FilePath "$RootDir/logs/frontend.log" -Append
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

# Explicitly exit 0 to ensure Just doesn't report a failure when the user stops the dev server
exit 0
