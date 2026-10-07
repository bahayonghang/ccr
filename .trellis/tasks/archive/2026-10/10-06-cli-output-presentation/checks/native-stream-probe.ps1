param(
    [Parameter(Mandatory = $true)][string]$ProbeBinary,
    [string]$Output = (Join-Path $PSScriptRoot 'native-stream-files-retest.json')
)
$ErrorActionPreference = 'Stop'
$nativeProbe = (Resolve-Path -LiteralPath $ProbeBinary).Path
$nativeTemp = Join-Path ([IO.Path]::GetTempPath()) ('ccr-stream-evidence-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $nativeTemp | Out-Null
$nativeRecords = @()
    [Console]::SetWindowSize(80, 30)
    foreach ($nativeMode in @('normal', 'force', 'dumb-force', 'no-color')) {
        foreach ($nativeRedirection in @('stdout', 'stderr', 'both')) {
            $nativeOut = Join-Path $nativeTemp ($nativeMode + '-' + $nativeRedirection + '.stdout')
            $nativeErr = Join-Path $nativeTemp ($nativeMode + '-' + $nativeRedirection + '.stderr')
            Write-Host ('CASE_BEGIN mode=' + $nativeMode + '; redirected=' + $nativeRedirection + '; console_stdout_redirected=' + [Console]::IsOutputRedirected)
            $nativeStart = [Diagnostics.ProcessStartInfo]::new($nativeProbe)
            $nativeStart.UseShellExecute = $false
            $nativeStart.RedirectStandardOutput = $nativeRedirection -ne 'stderr'
            $nativeStart.RedirectStandardError = $nativeRedirection -ne 'stdout'
            foreach ($argument in @('--exact', 'output_probe', '--ignored', '--nocapture')) { $nativeStart.ArgumentList.Add($argument) }
            foreach ($name in @('NO_COLOR', 'CLICOLOR_FORCE')) { [void]$nativeStart.Environment.Remove($name) }
            $nativeStart.Environment['CCR_OUTPUT_PROBE'] = 'messages'
            $nativeStart.Environment['CLICOLOR'] = '1'
            $nativeStart.Environment['TERM'] = if ($nativeMode -eq 'dumb-force') { 'dumb' } else { 'xterm-256color' }
            if ($nativeMode -eq 'force' -or $nativeMode -eq 'dumb-force') { $nativeStart.Environment['CLICOLOR_FORCE'] = '1' }
            if ($nativeMode -eq 'force' -or $nativeMode -eq 'no-color') { $nativeStart.Environment['NO_COLOR'] = '1' }
            $nativeChild = [Diagnostics.Process]::Start($nativeStart)
            if ($nativeStart.RedirectStandardOutput) { $nativeOutTask = $nativeChild.StandardOutput.ReadToEndAsync() }
            if ($nativeStart.RedirectStandardError) { $nativeErrTask = $nativeChild.StandardError.ReadToEndAsync() }
            $nativeChild.WaitForExit()
            $nativeCode = $nativeChild.ExitCode
            if ($nativeStart.RedirectStandardOutput) { [IO.File]::WriteAllText($nativeOut, $nativeOutTask.GetAwaiter().GetResult(), [Text.UTF8Encoding]::new($false)) }
            if ($nativeStart.RedirectStandardError) { [IO.File]::WriteAllText($nativeErr, $nativeErrTask.GetAwaiter().GetResult(), [Text.UTF8Encoding]::new($false)) }
            $nativeRecord = @{mode=$nativeMode; redirected=$nativeRedirection; exit_code=$nativeCode}
            if (Test-Path -LiteralPath $nativeOut) { $nativeRecord.stdout = [IO.File]::ReadAllText($nativeOut) }
            if (Test-Path -LiteralPath $nativeErr) { $nativeRecord.stderr = [IO.File]::ReadAllText($nativeErr) }
            $nativeRecords += $nativeRecord
            Write-Host ('CASE_END exit=' + $nativeCode)
        }
    }
    $nativeOutput = $Output
    [IO.File]::WriteAllText($nativeOutput, ($nativeRecords | ConvertTo-Json -Depth 6), [Text.UTF8Encoding]::new($false))
    Write-Host ('RECEIPT=' + $nativeOutput)
