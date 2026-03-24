[CmdletBinding()]
param(
    [string]$ExePath = (Join-Path $PSScriptRoot '..\src-tauri\target\release\ccr-desktop.exe'),
    [int]$WaitSeconds = 5,
    [int]$MinWidth = 800,
    [int]$MinHeight = 600
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$resolvedExe = Resolve-Path $ExePath -ErrorAction Stop

Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;

public static class ReleaseWindowProbe {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int count);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder text, int count);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
}
'@

$process = $null

try {
    Write-Host "Starting release application and waiting for the main window..." -ForegroundColor Cyan
    $process = Start-Process -FilePath $resolvedExe -PassThru
    Start-Sleep -Seconds $WaitSeconds

    $targetPid = [uint32]$process.Id
    $windows = New-Object System.Collections.Generic.List[object]

    $callback = [ReleaseWindowProbe+EnumWindowsProc]{
        param($hWnd, $lParam)

        $windowPid = [uint32]0
        [ReleaseWindowProbe]::GetWindowThreadProcessId($hWnd, [ref]$windowPid) | Out-Null
        if ($windowPid -ne $targetPid) {
            return $true
        }

        $titleBuilder = New-Object System.Text.StringBuilder 512
        $classBuilder = New-Object System.Text.StringBuilder 256
        [ReleaseWindowProbe]::GetWindowText($hWnd, $titleBuilder, $titleBuilder.Capacity) | Out-Null
        [ReleaseWindowProbe]::GetClassName($hWnd, $classBuilder, $classBuilder.Capacity) | Out-Null

        $rect = New-Object ReleaseWindowProbe+RECT
        [ReleaseWindowProbe]::GetWindowRect($hWnd, [ref]$rect) | Out-Null

        $windows.Add([pscustomobject]@{
            Handle  = [int64]$hWnd
            Visible = [ReleaseWindowProbe]::IsWindowVisible($hWnd)
            Class   = $classBuilder.ToString()
            Title   = $titleBuilder.ToString()
            Width   = $rect.Right - $rect.Left
            Height  = $rect.Bottom - $rect.Top
            Left    = $rect.Left
            Top     = $rect.Top
        }) | Out-Null

        return $true
    }

    [ReleaseWindowProbe]::EnumWindows($callback, [IntPtr]::Zero) | Out-Null

    if ($windows.Count -eq 0) {
        throw "No top-level windows were created by the release process."
    }

    $mainWindow = $windows | Where-Object {
        $_.Class -eq 'Tauri Window' -and $_.Visible -and $_.Width -ge $MinWidth -and $_.Height -ge $MinHeight
    } | Select-Object -First 1

    if (-not $mainWindow) {
        Write-Host "Visible main Tauri window was not found." -ForegroundColor Red
        $windows | Sort-Object Width -Descending | Format-Table -AutoSize | Out-String | Write-Host
        throw "The release main window is hidden or smaller than ${MinWidth}x${MinHeight}."
    }

    Write-Host ("Visible main window detected at {0}x{1}." -f $mainWindow.Width, $mainWindow.Height) -ForegroundColor Green
}
finally {
    if ($process -and (Get-Process -Id $process.Id -ErrorAction SilentlyContinue)) {
        Stop-Process -Id $process.Id -Force
    }
}
