# Pengisi RAM terkendali untuk uji pemahaman (docs/responden.md).
# Mengalokasikan RAM bertahap sampai persentase pemakaian menyentuh target,
# menahannya beberapa detik, lalu melepas semuanya. Tanpa admin, tanpa unduhan.
#
# Contoh (waspada, target 82%):
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\ramfill.ps1 -TargetPercent 82 -HoldSeconds 20
# Contoh (bahaya, target 94%):
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\ramfill.ps1 -TargetPercent 94 -HoldSeconds 20
#
# Pengaman: berhenti bila RAM tersedia < 600 MB, bila pemakaian mencapai 96%,
# atau bila total alokasi mencapai batas (default 6 GB). Ctrl+C melepas semua.
#
# Catatan metode:
# - Pemakaian dihitung seperti sysinfo di aplikasi: (total - tersedia) / total,
#   dengan "tersedia" = counter \Memory\Available MBytes.
# - Halaman harus benar-benar ditulis. `[byte[]]::new()` saja menghasilkan
#   halaman demand-zero yang tidak menambah pemakaian fisik (terukur: 9 GB
#   teralokasi, pemakaian tidak bergerak). Karena itu tiap blok di-`Clear`.
# - Konfirmasi akhir tetap pada banner aplikasi; ambang resmi di DESIGN.md 7.

param(
  [int]$TargetPercent = 82,
  [int]$HoldSeconds = 20,
  [int]$ChunkMB = 512,
  [int]$MaxTotalMB = 0
)

if ($TargetPercent -lt 50 -or $TargetPercent -gt 96) {
  Write-Error "TargetPercent harus 50-96."
  exit 1
}
if ($HoldSeconds -lt 1 -or $HoldSeconds -gt 300) {
  Write-Error "HoldSeconds harus 1-300."
  exit 1
}
if ($ChunkMB -lt 64 -or $ChunkMB -gt 1024) {
  Write-Error "ChunkMB harus 64-1024 (batas array PowerShell 5.1)."
  exit 1
}

function Get-Usage {
  $totalMB = [double](Get-CimInstance Win32_OperatingSystem).TotalVisibleMemorySize / 1KB
  $availMB = $null
  try {
    $availMB = (Get-Counter '\Memory\Available MBytes' -SampleInterval 1 -MaxSamples 1).CounterSamples[0].CookedValue
  } catch {
    $availMB = [double](Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory / 1KB
  }
  [pscustomobject]@{
    UsedPercent = [math]::Round((1 - ($availMB / $totalMB)) * 100, 1)
    AvailMB = [math]::Round($availMB)
    TotalMB = [math]::Round($totalMB)
  }
}

$blocks = New-Object System.Collections.ArrayList
$allocatedMB = 0
$state = Get-Usage
# Batas alokasi mengikuti ukuran RAM: 60% dari total, kecuali ditentukan sendiri.
# Ini menjaga target 93% tetap bisa dicapai di mesin 32 GB tanpa membuka jalan
# menghabiskan seluruh RAM.
if ($MaxTotalMB -le 0) {
  $MaxTotalMB = [int]($state.TotalMB * 0.6)
}
Write-Output ("Awal: {0}% terpakai, tersedia {1} MB (batas alokasi {2} MB)." -f $state.UsedPercent, $state.AvailMB, $MaxTotalMB)

while ($state.UsedPercent -lt $TargetPercent) {
  # Blok terakhir menyesuaikan sisa kebutuhan supaya tidak melompati target
  # (blok tetap 512 MB pada target 94% meleset ke 96,3%).
  $needMB = [int][math]::Ceiling($state.AvailMB - ($state.TotalMB * (1 - $TargetPercent / 100)))
  if ($needMB -lt 32) {
    Write-Output "Berhenti: sisa kebutuhan di bawah 32 MB."
    break
  }
  $stepMB = [math]::Min($ChunkMB, [math]::Max(32, $needMB))
  if ($state.AvailMB - $stepMB -lt 600) {
    Write-Output ("Berhenti: langkah {0} MB akan menyisakan kurang dari 600 MB." -f $stepMB)
    break
  }
  if ($allocatedMB + $stepMB -gt $MaxTotalMB) {
    Write-Output ("Berhenti: batas alokasi {0} MB tercapai." -f $MaxTotalMB)
    break
  }
  $block = [byte[]]::new($stepMB * 1MB)
  [Array]::Clear($block, 0, $block.Length)
  [void]$blocks.Add($block)
  $allocatedMB += $stepMB
  $state = Get-Usage
  Write-Output ("  alokasi {0} MB -> {1}% terpakai, tersedia {2} MB." -f $allocatedMB, $state.UsedPercent, $state.AvailMB)
  if ($state.UsedPercent -ge 96) {
    Write-Output "Berhenti: pemakaian menyentuh 96%."
    break
  }
}

Write-Output ("Tahan {0} detik pada {1}% terpakai. Sekarang lakukan tangkapan layar." -f $HoldSeconds, $state.UsedPercent)
for ($t = $HoldSeconds; $t -gt 0; $t -= 5) {
  Start-Sleep -Seconds ([math]::Min(5, $t))
  Write-Output ("  {0} detik tersisa - {1}% terpakai." -f [math]::Max(0, $t - 5), (Get-Usage).UsedPercent)
}

$blocks.Clear()
$blocks = $null
[System.GC]::Collect()
Start-Sleep -Milliseconds 900
$state = Get-Usage
Write-Output ("Selesai: {0}% terpakai, tersedia {1} MB." -f $state.UsedPercent, $state.AvailMB)
