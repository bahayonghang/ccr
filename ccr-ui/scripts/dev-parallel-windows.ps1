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

# ========== ANSI Escape Sequence Handling ==========

# Remove ANSI escape sequences from text
function Remove-AnsiEscapeSequences {
    param([string]$Text)
    # Match all ANSI escape sequences: ESC[...m, ESC[...H, ESC]...BEL, etc.
    $ansiPattern = '\x1b\[[0-9;]*[a-zA-Z~]|\x1b\][^\x07]*\x07'
    return $Text -replace $ansiPattern, ''
}

# Write cleaned log with timestamp
function Write-CleanLog {
    param(
        [Parameter(ValueFromPipeline)]
        [string]$Line,
        [string]$LogPath
    )
    process {
        if (-not [string]::IsNullOrEmpty($Line)) {
            $cleanLine = Remove-AnsiEscapeSequences -Text $Line
            $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
            $logEntry = "[$timestamp] $cleanLine"
            
            # Write to log file (cleaned format with timestamp)
            Add-Content -Path $LogPath -Value $logEntry -Encoding UTF8
            
            # Output to console (original format with colors)
            Write-Host $Line
        }
    }
}

# ========== Log Directory Setup ==========

# Create logs directory structure
$logsDir = Join-Path $RootDir "logs"
$backendLogsDir = Join-Path $logsDir "backend"
$frontendLogsDir = Join-Path $logsDir "frontend"

if (-not (Test-Path $backendLogsDir)) {
    New-Item -ItemType Directory -Path $backendLogsDir -Force | Out-Null
}
if (-not (Test-Path $frontendLogsDir)) {
    New-Item -ItemType Directory -Path $frontendLogsDir -Force | Out-Null
}

# Get today's date for log file names
$logDate = Get-Date -Format "yyyy-MM-dd"
$frontendLogPath = Join-Path $frontendLogsDir "$logDate.log"
$backendConsoleLogPath = Join-Path $backendLogsDir "console-$logDate.log"

Write-Host "[CCR] Starting development environment (parallel mode)..." -ForegroundColor Cyan
Write-Host ""

# ========== Pre-compile Backend (避免健康检查超时) ==========
Write-Host "[Backend] Pre-compiling..." -ForegroundColor Yellow

$backendDir = Join-Path $RootDir "backend"
Push-Location $backendDir
try {
    # 先编译，确保二进制文件存在，避免启动时编译超时
    # 使用 & 执行 cargo，让输出直接显示
    & cargo build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Backend compilation failed (exit code: $LASTEXITCODE)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Write-Host "[Backend] Compilation successful" -ForegroundColor Green
} finally {
    Pop-Location
}

# ========== Start Backend (Background Job) ==========
Write-Host "[Backend] Starting server (background job)..." -ForegroundColor Yellow

$backendJob = Start-Job -ScriptBlock {
    param($workDir, $port, $logPath)
    Set-Location "$workDir/backend"

    # Run cargo (已预编译，直接运行) 传递端口参数
    cargo run -- --port $port 2>&1 | Tee-Object -FilePath $logPath -Append
} -ArgumentList $RootDir, $BackendPort, $backendConsoleLogPath

Write-Host "[Backend] Started in background (Job ID: $($backendJob.Id))" -ForegroundColor Green
Write-Host "          Log file: $backendConsoleLogPath" -ForegroundColor Gray
Write-Host ""

# ========== Wait for Backend Ready ==========
# 编译已在启动前完成，只需等待服务器启动，固定 30 秒超时
$maxWait = 30
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
        Write-Host "        Check $backendConsoleLogPath for details" -ForegroundColor Red
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
Write-Host "           Log file: $frontendLogPath" -ForegroundColor Gray
Write-Host ""
Write-Host "[TIP] Press Ctrl+C to stop both backend and frontend servers" -ForegroundColor Cyan
Write-Host "======================================================================" -ForegroundColor DarkGray
Write-Host ""

try {
    Set-Location "$RootDir/frontend"

    # Suppress PowerShell treating stderr as error (bun/vite outputs to stderr)
    $ErrorActionPreference = "Continue"

    # Frontend runs in foreground with live output using Write-CleanLog
    # --host 0.0.0.0 allows access from LAN IP addresses
    cmd /c "bun run dev -- --host 0.0.0.0 --port $VitePort 2>&1 || exit 0" | ForEach-Object {
        Write-CleanLog -Line $_ -LogPath $frontendLogPath
    }
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
