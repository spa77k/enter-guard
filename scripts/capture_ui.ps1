$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WindowCapture {
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window, out Rect rect);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr window, IntPtr dc, uint flags);
}
'@
New-Item -ItemType Directory artifacts -Force | Out-Null
$p = Start-Process rust/target/x86_64-pc-windows-msvc/release/enter-guard.exe -PassThru
try {
    for ($i = 0; $i -lt 40; $i++) {
        Start-Sleep -Milliseconds 250
        $p.Refresh()
        if ($p.HasExited) { throw 'Application exited before screenshot' }
        if ($p.MainWindowHandle -ne [IntPtr]::Zero) { break }
    }
    $window = $p.MainWindowHandle
    if ($window -eq [IntPtr]::Zero) { throw 'No application window' }
    Start-Sleep -Milliseconds 500
    $rect = New-Object WindowCapture+Rect
    if (-not [WindowCapture]::GetWindowRect($window, [ref]$rect)) { throw 'GetWindowRect failed' }
    $bitmap = New-Object Drawing.Bitmap ($rect.Right - $rect.Left), ($rect.Bottom - $rect.Top)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    $dc = $graphics.GetHdc()
    try {
        if (-not [WindowCapture]::PrintWindow($window, $dc, 2)) { throw 'PrintWindow failed' }
    } finally { $graphics.ReleaseHdc($dc) }
    try { $bitmap.Save((Join-Path (Resolve-Path artifacts) 'EnterGuard-paused.png')) }
    finally { $graphics.Dispose(); $bitmap.Dispose() }
} finally {
    if (-not $p.HasExited) {
        $p.CloseMainWindow() | Out-Null
        if (-not $p.WaitForExit(5000)) { $p.Kill(); throw 'Application failed to close after screenshot' }
    }
}
