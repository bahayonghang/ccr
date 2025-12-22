# CCR UI - Troubleshooting Script for Windows
# Diagnoses common development server issues

Write-Host "================================================" -ForegroundColor Cyan
Write-Host "      CCR UI - Troubleshooting Utility         " -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($isAdmin) {
    Write-Host "[+] Running with Administrator privileges" -ForegroundColor Green
} else {
    Write-Host "[!] NOT running as Administrator" -ForegroundColor Yellow
    Write-Host "    Some checks require elevated permissions" -ForegroundColor Yellow
}
Write-Host ""

# 1. Check port availability
Write-Host "[1] Checking required ports..." -ForegroundColor Cyan
$ports = @(38081, 5173, 5174, 3000)
foreach ($port in $ports) {
    $listener = Get-NetTCPConnection -State Listen -LocalPort $port -ErrorAction SilentlyContinue
    if ($listener) {
        Write-Host "    [OCCUPIED] Port $port is in use by PID: $($listener.OwningProcess)" -ForegroundColor Red
        $process = Get-Process -Id $listener.OwningProcess -ErrorAction SilentlyContinue
        if ($process) {
            Write-Host "               Process: $($process.ProcessName)" -ForegroundColor Yellow
        }
    } else {
        Write-Host "    [FREE] Port $port is available" -ForegroundColor Green
    }
}
Write-Host ""

# 2. Check for crashed processes
Write-Host "[2] Checking for crashed backend/frontend processes..." -ForegroundColor Cyan
$crashedProcs = @()
$crashedProcs += Get-Process -Name "ccr-ui-backend" -ErrorAction SilentlyContinue
$crashedProcs += Get-Process -Name "cargo" -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -like "*ccr-ui*" }
$crashedProcs += Get-Process -Name "node" -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -like "*vite*" }
$crashedProcs += Get-Process -Name "bun" -ErrorAction SilentlyContinue

if ($crashedProcs.Count -gt 0) {
    Write-Host "    Found $($crashedProcs.Count) potentially crashed process(es):" -ForegroundColor Yellow
   foreach ($proc in $crashedProcs) {
        Write-Host "      - PID $($proc.Id): $($proc.ProcessName)" -ForegroundColor Yellow
    }
    Write-Host "    Recommendation: Kill these processes with 'just dev-clean'" -ForegroundColor Cyan
} else {
    Write-Host "    No crashed processes found" -ForegroundColor Green
}
Write-Host ""

# 3. Check firewall rules
Write-Host "[3] Checking Windows Firewall rules..." -ForegroundColor Cyan
if ($isAdmin) {
    $rules = Get-NetFirewallRule -DisplayName "*ccr*" -ErrorAction SilentlyContinue
    if ($rules) {
        Write-Host "    Found CCR firewall rules:" -ForegroundColor Green
        foreach ($rule in $rules) {
            Write-Host "      - $($rule.DisplayName): $($rule.Enabled)" -ForegroundColor Gray
        }
    } else {
        Write-Host "    No specific CCR firewall rules found (using defaults)" -ForegroundColor Yellow
    }
} else {
    Write-Host "    [SKIPPED] Requires Administrator privileges" -ForegroundColor Yellow
}
Write-Host ""

# 4. Check CCR binary
Write-Host "[4] Checking CCR binary..." -ForegroundColor Cyan
$ccrPath = (Get-Command ccr -ErrorAction SilentlyContinue).Path
if ($ccrPath) {
    Write-Host "    [FOUND] CCR binary at: $ccrPath" -ForegroundColor Green
    try {
        $version = & ccr --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "    Version: $version" -ForegroundColor Green
        } else {
            Write-Host "    [ERROR] CCR command failed with exit code: $LASTEXITCODE" -ForegroundColor Red
            Write-Host "    This may cause backend initialization to fail!" -ForegroundColor Red
        }
    } catch {
        Write-Host "    [ERROR] Failed to execute CCR: $_" -ForegroundColor Red
    }
} else {
    Write-Host "    [NOT FOUND] CCR binary not in PATH" -ForegroundColor Yellow
    Write-Host "    Backend will have limited functionality" -ForegroundColor Yellow
}
Write-Host ""

# 5. Check Rust/Cargo
Write-Host "[5] Checking Rust toolchain..." -ForegroundColor Cyan
$cargoPath = (Get-Command cargo -ErrorAction SilentlyContinue).Path
if ($cargoPath) {
    $cargoVersion = & cargo --version
    Write-Host "    [FOUND] $cargoVersion" -ForegroundColor Green
} else {
    Write-Host "    [ERROR] Cargo not found in PATH!" -ForegroundColor Red
}
Write-Host ""

# 6. Check Bun
Write-Host "[6] Checking Bun runtime..." -ForegroundColor Cyan
$bunPath = (Get-Command bun -ErrorAction SilentlyContinue).Path
if ($bunPath) {
    $bunVersion = & bun --version
    Write-Host "    [FOUND] Bun v$bunVersion" -ForegroundColor Green
}  else {
    Write-Host "    [ERROR] Bun not found in PATH!" -ForegroundColor Red
}
Write-Host ""

# 7. Check logs
Write-Host "[7] Checking recent logs..." -ForegroundColor Cyan
if (Test-Path "logs/backend-console.log") {
    $backendLog = Get-Content "logs/backend-console.log" -Tail 5 -ErrorAction SilentlyContinue
    Write-Host "    Last 5 lines of backend log:" -ForegroundColor Gray
    $backendLog | ForEach-Object { Write-Host "      $_" -ForegroundColor DarkGray }
} else {
    Write-Host "    No backend log found" -ForegroundColor Yellow
}
Write-Host ""

# Summary
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "              Troubleshooting Complete          " -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Recommended actions:" -ForegroundColor Cyan
Write-Host "  1. If ports are occupied: run 'just dev-clean'" -ForegroundColor White
Write-Host "  2. If firewall is blocking: allow PowerShell/Cargo in firewall" -ForegroundColor White
Write-Host "  3. If CCR command fails: check CCR installation" -ForegroundColor White
Write-Host "  4. For detailed backend logs: cat logs/backend-console.log" -ForegroundColor White
Write-Host ""
