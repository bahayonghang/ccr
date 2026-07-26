param([switch]$Verbose)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$arguments = @((Join-Path $PSScriptRoot 'check_dependency_drift.py'))
if ($Verbose) { $arguments += '--verbose' }

$python = Get-Command python -ErrorAction SilentlyContinue
if ($python) {
    & $python.Source @arguments
} else {
    & py -3 @arguments
}
exit $LASTEXITCODE
