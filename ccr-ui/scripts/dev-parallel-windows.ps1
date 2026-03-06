# CCR UI - Windows Development Server Parallel Launcher
# Uses PowerShell background jobs to run Rust/Vite processes in current window
# Avoids opening new popup windows

param(
    [string]$RootDir = $PSScriptRoot,
    [string]$BackendPort = "48081",
    [string]$VitePort = "15173"
)

# ========== UTF-8 Encoding Setup (Fix Chinese character display) ==========
# Set console output encoding to UTF-8 to properly display Chinese and emoji
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
[Console]::InputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8
# Also set code page to UTF-8 (65001)
chcp 65001 | Out-Null

$ErrorActionPreference = "Stop"

# ========== Ctrl+C Handler Setup ==========
# Track if we're in cleanup mode to prevent double cleanup
$script:IsCleaningUp = $false
$script:BackendJobId = $null
$script:BackendPid = $null
$script:FrontendPid = $null
$script:BackendPidFile = $null
$script:FrontendPidFile = $null

# Register Ctrl+C handler to ensure clean exit
$null = Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action {
    if (-not $script:IsCleaningUp -and $script:BackendJobId) {
        $script:IsCleaningUp = $true
        Stop-Job -Id $script:BackendJobId -ErrorAction SilentlyContinue
        Remove-Job -Id $script:BackendJobId -Force -ErrorAction SilentlyContinue
    }
}

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
            if ($script:FrontendPortFile -and ($cleanLine -match 'Local:\s+http://localhost:(\d+)/')) {
                $detectedPort = $Matches[1]
                $portChanged = $detectedPort -ne $script:FrontendPort
                if ($portChanged) {
                    $script:FrontendPort = $detectedPort
                    Set-Content -Path $script:FrontendPortFile -Value $script:FrontendPort -Encoding ASCII -ErrorAction SilentlyContinue
                }
                if ($detectedPort -ne $script:FrontendPortAnnounced) {
                    $script:FrontendPortAnnounced = $detectedPort
                    Write-Host ("📍 前端: http://localhost:{0} (Vue 3 + Vite)" -f $detectedPort) -ForegroundColor Cyan
                }
                if ($script:FrontendPidFile -and ($portChanged -or -not $script:FrontendPid)) {
                    try {
                        $frontendConn = Get-NetTCPConnection -LocalPort $detectedPort -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1
                        if ($frontendConn) {
                            $script:FrontendPid = $frontendConn.OwningProcess
                            Set-Content -Path $script:FrontendPidFile -Value $script:FrontendPid -Encoding ASCII -ErrorAction SilentlyContinue
                        }
                    } catch {
                        # Ignore errors during PID capture
                    }
                }
            }
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
$frontendPortFile = Join-Path $logsDir "frontend.port"
$backendPidFile = Join-Path $RootDir ".backend.pid"
$frontendPidFile = Join-Path $RootDir ".frontend.pid"
$script:FrontendPortFile = $frontendPortFile
$script:FrontendPort = $VitePort
$script:FrontendPortAnnounced = $null
$script:BackendPidFile = $backendPidFile
$script:FrontendPidFile = $frontendPidFile
Set-Content -Path $frontendPortFile -Value $VitePort -Encoding ASCII -ErrorAction SilentlyContinue

Write-Host "[CCR] Starting development environment (parallel mode)..." -ForegroundColor Cyan
Write-Host ""

# ========== Pre-compile Backend (避免健康检查超时) ==========
Write-Host "[Backend] Pre-compiling..." -ForegroundColor Yellow

# Native target now lives in src-tauri under ccr-ui
$backendDir = Join-Path $RootDir "src-tauri"
$backendBinary = Join-Path $backendDir "target/debug/ccr-desktop.exe"
Push-Location $backendDir
try {
    # 在 src-tauri 中编译，确保二进制文件输出到当前结构位置
    # 使用 & 执行 cargo，让输出直接显示
    & cargo build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Backend compilation failed (exit code: $LASTEXITCODE)" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    if (-not (Test-Path $backendBinary)) {
        Write-Host "[ERROR] Backend binary not found: $backendBinary" -ForegroundColor Red
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
    param($workDir, $binary, $port, $logPath)
    # Set working directory to backend folder for correct relative paths
    Set-Location $workDir

    # Run pre-compiled backend binary (传递端口参数)
    & "$binary" --port $port 2>&1 | Tee-Object -FilePath $logPath -Append
} -ArgumentList $backendDir, $backendBinary, $BackendPort, $backendConsoleLogPath

# Save job ID for Ctrl+C handler
$script:BackendJobId = $backendJob.Id

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
            try {
                $backendConn = Get-NetTCPConnection -LocalPort $BackendPort -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1
                if ($backendConn) {
                    $script:BackendPid = $backendConn.OwningProcess
                    if ($script:BackendPidFile) {
                        Set-Content -Path $script:BackendPidFile -Value $script:BackendPid -Encoding ASCII -ErrorAction SilentlyContinue
                    }
                }
            } catch {
                # Ignore errors during PID capture
            }
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

# ========== Vite Dep Pre-bundling（noDiscovery 模式下快速完成）==========
# 在 Vite 服务器启动前预打包 npm 依赖，填充 .vite/deps 缓存
# 服务器启动时直接命中缓存，跳过优化阶段
Write-Host "[Frontend] Pre-bundling Vite dependencies..." -ForegroundColor Yellow
Push-Location $RootDir
try {
    $prebundleResult = & cmd /c "bun run prebundle" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[Frontend] Pre-bundling complete" -ForegroundColor Green
    } else {
        Write-Host "[WARN] Vite pre-bundling had warnings (non-fatal), continuing..." -ForegroundColor Yellow
    }
} catch {
    Write-Host "[WARN] Vite pre-bundling skipped: $_" -ForegroundColor Yellow
} finally {
    Pop-Location
}
Write-Host ""

# ========== Start Frontend (Foreground) ==========
Write-Host "[Frontend] Starting server (foreground, live logs visible)..." -ForegroundColor Yellow
Write-Host "           Log file: $frontendLogPath" -ForegroundColor Gray
Write-Host ""
Write-Host "[TIP] Press Ctrl+C to stop both backend and frontend servers" -ForegroundColor Cyan
Write-Host "======================================================================" -ForegroundColor DarkGray
Write-Host ""

# Use try/catch/finally to handle Ctrl+C gracefully
$exitCode = 0
try {
    Set-Location $RootDir

    # Suppress PowerShell treating stderr as error (bun/vite outputs to stderr)
    $ErrorActionPreference = "Continue"

    # Frontend runs in foreground with live output
    # --host 0.0.0.0 allows access from LAN IP addresses
    # Run vite directly without any pipe to avoid stdin closure issues

    Write-Host ("📍 前端: http://localhost:{0} (Vue 3 + Vite)" -f $VitePort) -ForegroundColor Cyan
    Set-Content -Path $frontendPortFile -Value $VitePort -Encoding ASCII -ErrorAction SilentlyContinue

    # Run vite using cmd /c to keep stdin connected
    # This prevents Vite from exiting immediately due to stdin closure
    $env:FORCE_COLOR = "1"

    # Use cmd /c to wrap the command - this keeps stdin open properly
    # Also use bun which is the project's package manager
    $viteProcess = Start-Process -FilePath "cmd" -ArgumentList "/c", "bun run dev -- --host 0.0.0.0 --port $VitePort" -NoNewWindow -PassThru -WorkingDirectory $RootDir
    $script:FrontendPid = $viteProcess.Id
    if ($script:FrontendPidFile) {
        Set-Content -Path $script:FrontendPidFile -Value $script:FrontendPid -Encoding ASCII -ErrorAction SilentlyContinue
    }

    # Wait for the vite process (blocks until Ctrl+C or process exit)
    $viteProcess.WaitForExit()
} catch {
    # Ctrl+C or other interruption - this is expected behavior
    Write-Host ""
    Write-Host "[Info] Received interrupt signal..." -ForegroundColor Yellow
} finally {
    # Mark cleanup in progress
    $script:IsCleaningUp = $true

    # Stop frontend process if still running (find by port)
    try {
        $frontendPid = $script:FrontendPid
        if (-not $frontendPid) {
            $frontendPort = $script:FrontendPort
            if (-not $frontendPort) {
                $frontendPort = $VitePort
            }
            $frontendConn = Get-NetTCPConnection -LocalPort $frontendPort -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($frontendConn) {
                $frontendPid = $frontendConn.OwningProcess
            }
        }
        if ($frontendPid) {
            $frontendProc = Get-Process -Id $frontendPid -ErrorAction SilentlyContinue
            if ($frontendProc -and $frontendProc.ProcessName -notmatch 'code|Code|electron') {
                Write-Host "[Cleanup] Stopping frontend server (PID: $frontendPid)..." -ForegroundColor Yellow
                Stop-Process -Id $frontendPid -Force -ErrorAction SilentlyContinue
                Write-Host "[Cleanup] Frontend stopped" -ForegroundColor Green
            }
        }
    } catch {
        # Ignore errors during cleanup
    }

    # Cleanup: Stop backend job
    Write-Host ""
    Write-Host "[Cleanup] Stopping backend server..." -ForegroundColor Yellow

    $backendPid = $script:BackendPid
    if (-not $backendPid) {
        try {
            $backendConn = Get-NetTCPConnection -LocalPort $BackendPort -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($backendConn) {
                $backendPid = $backendConn.OwningProcess
            }
        } catch {
            # Ignore errors during PID capture
        }
    }

    if ($backendPid) {
        Write-Host "[Cleanup] Stopping backend process (PID: $backendPid)..." -ForegroundColor Yellow
        Stop-Process -Id $backendPid -Force -ErrorAction SilentlyContinue
    }

    if (Get-Job -Id $backendJob.Id -ErrorAction SilentlyContinue) {
        Stop-Job -Job $backendJob -ErrorAction SilentlyContinue | Out-Null
        Wait-Job -Job $backendJob -Timeout 2 | Out-Null
        Remove-Job -Job $backendJob -Force -ErrorAction SilentlyContinue
    }

    # Fallback: ensure backend is not still listening
    try {
        $backendConn = Get-NetTCPConnection -LocalPort $BackendPort -State Listen -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($backendConn) {
            $fallbackPid = $backendConn.OwningProcess
            if ($fallbackPid -and $fallbackPid -ne $backendPid) {
                Write-Host "[Cleanup] Stopping backend process (PID: $fallbackPid)..." -ForegroundColor Yellow
                Stop-Process -Id $fallbackPid -Force -ErrorAction SilentlyContinue
            }
        }
    } catch {
        # Ignore errors during cleanup
    }

    if ($script:BackendPidFile) {
        Remove-Item -Path $script:BackendPidFile -Force -ErrorAction SilentlyContinue
    }
    if ($script:FrontendPidFile) {
        Remove-Item -Path $script:FrontendPidFile -Force -ErrorAction SilentlyContinue
    }

    Write-Host "[Cleanup] Backend stopped" -ForegroundColor Green

    # Unregister event handler
    Unregister-Event -SourceIdentifier PowerShell.Exiting -ErrorAction SilentlyContinue

    Write-Host "[CCR] Development environment closed" -ForegroundColor Cyan
}

# Explicitly exit 0 to ensure Just doesn't report a failure when the user stops the dev server
[Environment]::Exit(0)
