# Performance Benchmark Script (PowerShell)
# Test cache optimization performance

Write-Host "CCR UI Backend Cache Performance Benchmark" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# Check if backend is running
try {
    $null = Invoke-WebRequest -Uri "http://127.0.0.1:38081/api/version" -UseBasicParsing -ErrorAction Stop
    Write-Host "[OK] Backend is running" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Backend not running. Start with: cargo run --release" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Test endpoint
$endpoint = "http://127.0.0.1:38081/api/claude/agents"

# Warmup request
Write-Host "[Warmup] Warming up cache..." -ForegroundColor Yellow
$null = Invoke-WebRequest -Uri $endpoint -UseBasicParsing -ErrorAction SilentlyContinue
Write-Host "[OK] Warmup complete" -ForegroundColor Green
Write-Host ""

# Performance test function
function Benchmark {
    param(
        [string]$Name,
        [int]$Count
    )

    Write-Host "[Test] $Name ($Count requests)" -ForegroundColor Cyan

    $start = Get-Date
    for ($i = 1; $i -le $Count; $i++) {
        $null = Invoke-WebRequest -Uri $endpoint -UseBasicParsing -ErrorAction SilentlyContinue
    }
    $end = Get-Date

    $duration = ($end - $start).TotalMilliseconds
    $avg = [math]::Round($duration / $Count, 2)
    $throughput = [math]::Round($Count * 1000 / $duration, 2)

    Write-Host "  Total: ${duration}ms" -ForegroundColor White
    Write-Host "  Average: ${avg}ms/req" -ForegroundColor White
    Write-Host "  Throughput: ${throughput} req/s" -ForegroundColor White
    Write-Host ""
}

# Run benchmarks
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "[Benchmark] Cache Hit Performance Test" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

Benchmark -Name "10 requests" -Count 10
Benchmark -Name "50 requests" -Count 50
Benchmark -Name "100 requests" -Count 100

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "[OK] Benchmark Complete!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "[Expected Results]" -ForegroundColor Yellow
Write-Host "  - Cache hit: less than 5ms/req" -ForegroundColor White
Write-Host "  - Throughput: more than 200 req/s" -ForegroundColor White
Write-Host "  - Performance gain: 50-100x vs no cache" -ForegroundColor White
