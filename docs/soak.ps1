# Sampler soak test: catat CPU/RAM/handle/thread proses aplikasi sekali per menit.
#
# Pemakaian (dari akar repo):
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\soak.ps1 -Minutes 30 -OutFile "$env:TEMP\soak.csv"
#
# Nama proses mengikuti nama binary (Cargo): `arxseus-monitor`. Hasil soak lama
# memakai `hwmonitor`; lewati -ProcessName hwmonitor untuk membaca arsip itu.
#
# CPU dihitung dari selisih TotalProcessorTime proses (bukan CPU mesin), lalu
# dibagi jumlah core logis supaya angkanya sebanding dengan Task Manager.
param(
    [int]$Minutes = 30,
    [string]$OutFile = "$env:TEMP\soak.csv",
    [string]$ProcessName = "arxseus-monitor"
)

$cores = [Environment]::ProcessorCount
$rows = @()
$previous = $null

for ($minute = 0; $minute -le $Minutes; $minute++) {
    $p = Get-Process $ProcessName -ErrorAction SilentlyContinue
    if (-not $p) { Write-Output "PROSES_HILANG_PADA_MENIT=$minute"; break }

    $cpuTotal = $p.TotalProcessorTime.TotalSeconds
    $line = [pscustomobject]@{
        menit       = $minute
        pid         = $p.Id
        cpuPersen   = $null
        ramMB       = [math]::Round($p.WorkingSet64 / 1MB, 1)
        privateMB   = [math]::Round($p.PrivateMemorySize64 / 1MB, 1)
        handleCount = $p.HandleCount
        threadCount = $p.Threads.Count
        cpuDetik    = [math]::Round($cpuTotal, 3)
    }

    if ($previous) {
        $elapsed = ($p.TotalProcessorTime - $previous.cpu).TotalSeconds
        $wall = ($minute - $previous.menit) * 60
        $line.cpuPersen = [math]::Round(100 * $elapsed / ($wall * $cores), 2)
    }

    $previous = @{ cpu = $p.TotalProcessorTime; menit = $minute }
    $rows += $line
    $rows | Export-Csv -Path $OutFile -NoTypeInformation -Encoding UTF8
    Write-Output ("menit={0} cpu={1}% ram={2}MB private={3}MB handles={4} threads={5}" -f `
            $minute, $line.cpuPersen, $line.ramMB, $line.privateMB, $line.handleCount, $line.threadCount)

    if ($minute -lt $Minutes) { Start-Sleep -Seconds 60 }
}

$measured = $rows | Where-Object { $_.menit -gt 0 }
if ($measured) {
    $cpuValues = $measured | Where-Object { $_.cpuPersen -ne $null } | ForEach-Object { $_.cpuPersen }
    Write-Output ("RINGKASAN menit={0} cpu_median={1}% cpu_maks={2}% ram_awal={3}MB ram_akhir={4}MB private_maks={5}MB" -f `
            $measured[-1].menit, ($cpuValues | Sort-Object)[[int]($cpuValues.Count / 2)], ($cpuValues | Measure-Object -Maximum).Maximum, `
            $measured[0].ramMB, $measured[-1].ramMB, ($measured | Measure-Object -Property privateMB -Maximum).Maximum)
}
