# Windows-only acceptance probe for the browser Vite development server.

[CmdletBinding()]
param(
    [string]$RootDir,
    [string]$HostName = '127.0.0.1',
    [ValidateRange(1, 65535)]
    [int]$Port = 15174,
    [ValidateRange(10, 600)]
    [int]$SoakSeconds = 60,
    [ValidateRange(1, 60)]
    [int]$IdleCpuSampleSeconds = 10,
    [ValidateRange(1, 60000)]
    [int]$MaxHandleGrowth = 500,
    [ValidateRange(1, 4096)]
    [int]$MaxPrivateGrowthMB = 256,
    [ValidateRange(0.1, 100)]
    [double]$MaxMachineCpuPercent = 5
)

$ErrorActionPreference = 'Stop'

if ($env:OS -ne 'Windows_NT') {
    throw 'verify-vite-resources.ps1 is supported only on Windows.'
}
if ($IdleCpuSampleSeconds -gt $SoakSeconds) {
    throw 'IdleCpuSampleSeconds cannot exceed SoakSeconds.'
}

if (-not $RootDir) {
    $RootDir = Split-Path -Parent $PSScriptRoot
}
$RootDir = [IO.Path]::GetFullPath($RootDir)
$targetRoot = [IO.Path]::GetFullPath((Join-Path $RootDir 'src-tauri/target'))
$probeDir = Join-Path $targetRoot ('.ccr-vite-resource-probe-{0}' -f [guid]::NewGuid().ToString('N'))
$stdoutPath = Join-Path ([IO.Path]::GetTempPath()) ('ccr-vite-resource-{0}.stdout.log' -f $PID)
$stderrPath = Join-Path ([IO.Path]::GetTempPath()) ('ccr-vite-resource-{0}.stderr.log' -f $PID)
$wrapper = $null
$viteProcessId = $null
$result = [ordered]@{
    port = $Port
    probeDirectory = $probeDir
    thresholds = [ordered]@{
        maxHandleGrowth = $MaxHandleGrowth
        maxPrivateGrowthMB = $MaxPrivateGrowthMB
        maxMachineCpuPercent = $MaxMachineCpuPercent
    }
}
$failure = $null

function Get-ListenerProcessId {
    $listener = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if ($listener) { return [int]$listener.OwningProcess }
    return $null
}

function Get-ViteSample {
    param([int]$ProcessId)

    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    return [ordered]@{
        timestamp = [DateTimeOffset]::UtcNow.ToString('O')
        cpuSeconds = [math]::Round($process.CPU, 3)
        workingSetMB = [math]::Round($process.WorkingSet64 / 1MB, 1)
        privateMB = [math]::Round($process.PrivateMemorySize64 / 1MB, 1)
        handles = $process.HandleCount
    }
}

function Wait-ForCondition {
    param(
        [scriptblock]$Condition,
        [int]$TimeoutSeconds,
        [string]$Label
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    while ([DateTime]::UtcNow -lt $deadline) {
        if (& $Condition) { return }
        Start-Sleep -Milliseconds 200
    }
    throw "Timed out waiting for $Label."
}

try {
    if (Get-ListenerProcessId) {
        throw "Port $Port is already in use."
    }

    $node = (Get-Command node -ErrorAction Stop).Source
    $wrapper = Start-Process -FilePath $node `
        -ArgumentList @('scripts/dev-web-warm-start.mjs', '--host', $HostName, '--strictPort', '--port', "$Port") `
        -WorkingDirectory $RootDir `
        -WindowStyle Hidden `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath `
        -PassThru

    Wait-ForCondition -TimeoutSeconds 120 -Label 'the bounded Vite health signal' -Condition {
        if ($wrapper.HasExited) {
            $recentError = if (Test-Path $stderrPath) { Get-Content -Raw $stderrPath } else { '' }
            throw "Warm-start wrapper exited early with code $($wrapper.ExitCode): $recentError"
        }
        (Test-Path $stderrPath) -and ((Get-Content -Raw $stderrPath) -match '\[dev:web\] ready\s+')
    }

    $viteProcessId = Get-ListenerProcessId
    if (-not $viteProcessId) {
        throw "No listening Vite process was found on port $Port after readiness."
    }

    $baseline = Get-ViteSample -ProcessId $viteProcessId
    New-Item -ItemType Directory -Path $probeDir | Out-Null
    for ($index = 0; $index -lt 20; $index++) {
        Set-Content -LiteralPath (Join-Path $probeDir ('probe-{0:D2}.txt' -f $index)) `
            -Value ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()) `
            -Encoding ASCII
        Start-Sleep -Milliseconds 100
    }

    $preCpuDelay = $SoakSeconds - $IdleCpuSampleSeconds
    if ($preCpuDelay -gt 0) { Start-Sleep -Seconds $preCpuDelay }

    $cpuStart = Get-ViteSample -ProcessId $viteProcessId
    Start-Sleep -Seconds $IdleCpuSampleSeconds
    $final = Get-ViteSample -ProcessId $viteProcessId
    $logicalProcessors = [Environment]::ProcessorCount
    $cpuPercent = (($final.cpuSeconds - $cpuStart.cpuSeconds) / $IdleCpuSampleSeconds / $logicalProcessors) * 100
    $handleGrowth = $final.handles - $baseline.handles
    $privateGrowthMB = $final.privateMB - $baseline.privateMB

    $result.wrapperPid = $wrapper.Id
    $result.vitePid = $viteProcessId
    $result.baseline = $baseline
    $result.final = $final
    $result.delta = [ordered]@{
        handles = $handleGrowth
        privateMB = [math]::Round($privateGrowthMB, 1)
        idleMachineCpuPercent = [math]::Round($cpuPercent, 2)
    }
    $result.resourcePassed = (
        $handleGrowth -le $MaxHandleGrowth -and
        $privateGrowthMB -le $MaxPrivateGrowthMB -and
        $cpuPercent -le $MaxMachineCpuPercent
    )
} catch {
    $failure = $_
    $result.error = $_.Exception.Message
} finally {
    if (Test-Path -LiteralPath $probeDir) {
        Remove-Item -LiteralPath $probeDir -Recurse -Force
    }

    if ($wrapper -and -not $wrapper.HasExited) {
        & taskkill.exe /PID $wrapper.Id /T /F 2>$null | Out-Null
    }

    $deadline = [DateTime]::UtcNow.AddSeconds(5)
    while ([DateTime]::UtcNow -lt $deadline -and (Get-ListenerProcessId)) {
        Start-Sleep -Milliseconds 200
    }
    $result.listenerStopped = -not [bool](Get-ListenerProcessId)
    $result.parentStopped = -not $wrapper -or $wrapper.HasExited

    Remove-Item -LiteralPath $stdoutPath, $stderrPath -Force -ErrorAction SilentlyContinue
}

$result.passed = (
    -not $failure -and
    $result.resourcePassed -eq $true -and
    $result.listenerStopped -eq $true -and
    $result.parentStopped -eq $true
)
$result | ConvertTo-Json -Depth 6

if (-not $result.passed) {
    if ($failure) { throw $failure }
    throw 'Vite resource or lifecycle thresholds failed.'
}
