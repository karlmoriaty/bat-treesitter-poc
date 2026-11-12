param(
    [Parameter(Mandatory=$true)][string]$Exe,
    [Parameter(Mandatory=$true)][string]$File,
    [string]$Extra = ""
)

$out = Join-Path $env:TEMP "peakout.txt"
$argList = @($File)
if ($Extra -ne "") { $argList += $Extra }

$p = Start-Process -FilePath $Exe -ArgumentList $argList -PassThru -NoNewWindow `
    -RedirectStandardOutput $out -RedirectStandardError (Join-Path $env:TEMP "peakerr.txt")

$peak = 0
$sw = [System.Diagnostics.Stopwatch]::StartNew()
while (-not $p.HasExited) {
    try {
        $p.Refresh()
        if ($p.PeakWorkingSet64 -gt $peak) { $peak = $p.PeakWorkingSet64 }
    } catch { }
    Start-Sleep -Milliseconds 40
    if ($sw.Elapsed.TotalSeconds -gt 600) { $p.Kill(); break }
}
$sw.Stop()

"{0} [{1}] peak={2:N1}MB elapsed={3:N0}ms" -f (Split-Path $Exe -Leaf), $Extra, ($peak/1MB), $sw.Elapsed.TotalMilliseconds