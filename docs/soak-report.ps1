# Rekap CSV hasil docs\soak.ps1 menjadi ringkasan + tabel Markdown.
#
# Pemakaian:
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\soak-report.ps1 -Csv "$env:TEMP\soak.csv"
param(
    [string]$Csv = "$env:TEMP\soak.csv"
)

$rows = Import-Csv -Path $Csv -Encoding UTF8
$measured = $rows | Where-Object { [int]$_.menit -gt 0 }
$cpu = $measured | Where-Object { $_.cpuPersen } | ForEach-Object { [double]$_.cpuPersen }
$sorted = @($cpu | Sort-Object)
$median = $null
if ($sorted.Count -gt 0) {
    $mid = [int][math]::Floor($sorted.Count / 2)
    if ($sorted.Count % 2 -eq 1) { $median = $sorted[$mid] }
    else { $median = ($sorted[$mid - 1] + $sorted[$mid]) / 2 }
}

$ramMax = ($measured | ForEach-Object { [double]$_.ramMB } | Measure-Object -Maximum).Maximum
$privMax = ($measured | ForEach-Object { [double]$_.privateMB } | Measure-Object -Maximum).Maximum
$hMin = ($measured | ForEach-Object { [int]$_.handleCount } | Measure-Object -Minimum).Minimum
$hMax = ($measured | ForEach-Object { [int]$_.handleCount } | Measure-Object -Maximum).Maximum
$tMin = ($measured | ForEach-Object { [int]$_.threadCount } | Measure-Object -Minimum).Minimum
$tMax = ($measured | ForEach-Object { [int]$_.threadCount } | Measure-Object -Maximum).Maximum

Write-Output "TOTAL_SAMPEL=$($rows.Count) DURASI_MENIT=$($measured[-1].menit)"
Write-Output ("CPU_MEDIAN={0}% CPU_MAKS={1}% CPU_MIN={2}%" -f [math]::Round($median, 2), ($cpu | Measure-Object -Maximum).Maximum, ($cpu | Measure-Object -Minimum).Minimum)
Write-Output ("RAM_AWAL={0}MB RAM_AKHIR={1}MB RAM_MAKS={2}MB" -f $measured[0].ramMB, $measured[-1].ramMB, $ramMax)
Write-Output ("PRIVATE_AWAL={0}MB PRIVATE_AKHIR={1}MB PRIVATE_MAKS={2}MB" -f $measured[0].privateMB, $measured[-1].privateMB, $privMax)
Write-Output ("HANDLE_MIN={0} HANDLE_MAKS={1} THREAD_MIN={2} THREAD_MAKS={3}" -f $hMin, $hMax, $tMin, $tMax)
Write-Output "PID_UNIK=$((($rows | ForEach-Object { $_.pid } | Sort-Object -Unique) -join ','))"
Write-Output ""
Write-Output "| Menit | CPU | RAM (MB) | Private (MB) | Handle | Thread |"
Write-Output "|---:|---:|---:|---:|---:|---:|"
foreach ($row in $rows) {
    $cpuText = "-"
    if ($row.cpuPersen) { $cpuText = "$($row.cpuPersen)%" }
    Write-Output ("| {0} | {1} | {2} | {3} | {4} | {5} |" -f $row.menit, $cpuText, $row.ramMB, $row.privateMB, $row.handleCount, $row.threadCount)
}
