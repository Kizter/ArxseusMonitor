# Pembangkit beban CPU untuk uji pemahaman (docs/responden.md).
# Memakai runspace pool: satu proses PowerShell, N thread sibuk, lalu berhenti
# sendiri. Tanpa unduhan, tanpa admin, tanpa proses yang tertinggal.
#
# Contoh:
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\load.ps1 -Threads 12 -Seconds 20
#
# Kenapa runspace, bukan Start-Job: Start-Job membuat satu proses per job dan
# butuh ~0,5 detik per proses, sehingga beban baru penuh setelah separuh durasi
# habis (terukur: 16 job hanya menghasilkan ~37% dari 16 thread).
#
# Ambang status aplikasi: < 70% aman, 70-90% waspada, > 90% bahaya.
# Status berubah setelah 3 sampel bertahan (~6 detik), jadi beri >= 20 detik.

param(
  [int]$Threads = 12,
  [int]$Seconds = 20
)

if ($Threads -lt 1 -or $Threads -gt 64) {
  Write-Error "Threads harus 1-64."
  exit 1
}
if ($Seconds -lt 3 -or $Seconds -gt 600) {
  Write-Error "Seconds harus 3-600."
  exit 1
}

$work = {
  param($until)
  $x = 0
  while ((Get-Date) -lt $until) {
    for ($i = 0; $i -lt 50000; $i++) { $x = ($x + $i) % 7 }
  }
}

$pool = [runspacefactory]::CreateRunspacePool(1, $Threads)
$pool.Open()

# Deadline dihitung setelah pool siap, bukan sebelum thread dibuat.
$until = (Get-Date).AddSeconds($Seconds)
$runs = @()
for ($n = 0; $n -lt $Threads; $n++) {
  $ps = [powershell]::Create()
  $ps.RunspacePool = $pool
  [void]$ps.AddScript($work).AddArgument($until)
  $runs += [pscustomobject]@{ Shell = $ps; Handle = $ps.BeginInvoke() }
}

Write-Output ("Beban: {0} thread, {1} detik. Ctrl+C untuk berhenti lebih awal." -f $Threads, $Seconds)

foreach ($run in $runs) { $run.Shell.EndInvoke($run.Handle) }
foreach ($run in $runs) { $run.Shell.Dispose() }
$pool.Close()
$pool.Dispose()
Write-Output "Beban selesai."
