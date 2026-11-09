param(
  [Parameter(Mandatory = $true)][string]$Exe,
  [Parameter(Mandatory = $true)][string]$OutDir,
  # 形如 "overview-light-en-2000x1080=2000x1080"，等号右侧为物理像素窗口尺寸
  [Parameter(Mandatory = $true)][string[]]$Shots,
  [int]$SettleMs = 6000
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class W {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr a, int x, int y, int cx, int cy, uint f);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint f);
  [DllImport("user32.dll")] public static extern bool IsZoomed(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  [DllImport("user32.dll")] public static extern bool ScreenToClient(IntPtr h, ref POINT p);
}
'@
[void][W]::SetProcessDPIAware()
New-Item -ItemType Directory -Force $OutDir | Out-Null

$proc = Start-Process -FilePath $Exe -PassThru
$deadline = (Get-Date).AddSeconds(30)
while ($proc.MainWindowHandle -eq 0 -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 300; $proc.Refresh() }
if ($proc.MainWindowHandle -eq 0) { throw 'main window did not appear' }
$h = $proc.MainWindowHandle
Start-Sleep -Milliseconds 1500
$wasZoomed = [W]::IsZoomed($h)
$orig = New-Object W+RECT
[void][W]::GetWindowRect($h, [ref]$orig)
"original: zoomed=$wasZoomed rect=$($orig.L),$($orig.T),$($orig.R),$($orig.B)"

function Save-Window([IntPtr]$hwnd, [string]$path) {
  $r = New-Object W+RECT
  [void][W]::GetWindowRect($hwnd, [ref]$r)
  $w = $r.R - $r.L; $hh = $r.B - $r.T
  $bmp = New-Object System.Drawing.Bitmap $w, $hh
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  $ok = [W]::PrintWindow($hwnd, $dc, 2)
  $g.ReleaseHdc($dc)
  # PrintWindow 对 GPU 合成窗口可能返回全黑：抽样判断，必要时改用屏幕拷贝
  $dark = 0
  foreach ($i in 1..40) { $p = $bmp.GetPixel([int]($w * $i / 41), [int]($hh / 2)); if ($p.R + $p.G + $p.B -lt 30) { $dark++ } }
  if (-not $ok -or $dark -gt 35) {
    [void][W]::SetForegroundWindow($hwnd); Start-Sleep -Milliseconds 400
    $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
    $mode = 'screen'
  } else { $mode = 'printwindow' }
  $g.Dispose(); $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
  "$path ($w x $hh, $mode)"
}

function Post-Click([IntPtr]$hwnd, [int]$wx, [int]$wy) {
  # 坐标是截图里的窗口相对物理像素：换算成客户区坐标后投递，不移动用户的真实鼠标
  $r = New-Object W+RECT; [void][W]::GetWindowRect($hwnd, [ref]$r)
  $pt = New-Object W+POINT; $pt.X = $r.L + $wx; $pt.Y = $r.T + $wy
  [void][W]::ScreenToClient($hwnd, [ref]$pt)
  $lp = [IntPtr]((($pt.Y -band 0xFFFF) -shl 16) -bor ($pt.X -band 0xFFFF))
  [void][W]::PostMessage($hwnd, 0x0200, [IntPtr]::Zero, $lp)  # WM_MOUSEMOVE
  Start-Sleep -Milliseconds 120
  [void][W]::PostMessage($hwnd, 0x0201, [IntPtr]1, $lp)        # WM_LBUTTONDOWN, MK_LBUTTON
  Start-Sleep -Milliseconds 80
  [void][W]::PostMessage($hwnd, 0x0202, [IntPtr]::Zero, $lp)   # WM_LBUTTONUP
  "click window($wx,$wy) -> client($($pt.X),$($pt.Y))"
}

function Post-Scroll([IntPtr]$hwnd, [int]$notches) {
  # 直接向窗口投递 WM_MOUSEWHEEL（lParam 为屏幕坐标），不移动用户的真实鼠标；负数向上
  $r = New-Object W+RECT; [void][W]::GetWindowRect($hwnd, [ref]$r)
  $x = $r.L + [int](($r.R - $r.L) * 0.55); $y = $r.T + [int](($r.B - $r.T) * 0.6)
  $lp = [IntPtr](($y -shl 16) -bor ($x -band 0xFFFF))
  $delta = if ($notches -lt 0) { 120 } else { -120 }
  $wp = [IntPtr](($delta -band 0xFFFF) -shl 16)
  foreach ($i in 1..[Math]::Abs($notches)) { [void][W]::PostMessage($hwnd, 0x020A, $wp, $lp); Start-Sleep -Milliseconds 60 }
  Start-Sleep -Milliseconds 800
}

foreach ($shot in $Shots) {
  # 形如 name=WxH[@动作...]；动作按顺序执行，然后截图：
  #   @scroll / @scroll:N（N 格，负数向上）  @click:X,Y（窗口相对物理像素）  @wait:MS
  #   @keep：不重设窗口尺寸（沿用上一张的尺寸与页面状态）
  $name, $spec = $shot -split '=', 2
  $parts = $spec -split '@'
  $dims = $parts[0]; $actions = @($parts | Select-Object -Skip 1)
  if ($actions -notcontains 'keep') {
    $cw, $ch = ($dims -split 'x') | ForEach-Object { [int]$_ }
    [void][W]::ShowWindow($h, 9)  # SW_RESTORE
    [void][W]::SetWindowPos($h, [IntPtr]::Zero, 0, 0, $cw, $ch, 0x0004)  # SWP_NOZORDER
    Start-Sleep -Milliseconds $SettleMs
  }
  foreach ($action in $actions) {
    if ($action -eq 'keep') { continue }
    $verb, $arg = $action -split ':', 2
    switch ($verb) {
      'scroll' { Post-Scroll $h ($(if ($arg) { [int]$arg } else { 12 })) }
      'click' { $cx, $cy = ($arg -split ',') | ForEach-Object { [int]$_ }; Post-Click $h $cx $cy; Start-Sleep -Milliseconds 1500 }
      'wait' { Start-Sleep -Milliseconds ([int]$arg) }
      default { throw "unknown action $action" }
    }
  }
  Save-Window $h (Join-Path $OutDir "$name.png")
}

# 恢复原始位置与最大化状态，等偏好写入器落盘后结束进程（退出确认框会挡住正常关闭）
[void][W]::ShowWindow($h, 9)
[void][W]::SetWindowPos($h, [IntPtr]::Zero, $orig.L, $orig.T, $orig.R - $orig.L, $orig.B - $orig.T, 0x0004)
if ($wasZoomed) { [void][W]::ShowWindow($h, 3) }
Start-Sleep -Milliseconds 3000
$after = New-Object W+RECT
[void][W]::GetWindowRect($h, [ref]$after)
"restored: zoomed=$([W]::IsZoomed($h)) rect=$($after.L),$($after.T),$($after.R),$($after.B)"
Stop-Process -Id $proc.Id
'stopped'
