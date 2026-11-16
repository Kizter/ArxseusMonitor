# Pengukur akurasi: nilai aplikasi vs acuan Windows (perfmon + Get-Volume).
#
# Metode (docs/akurasi.md):
# - Tiap sesi memasang satu jendela 2 detik: `snapshot.exe` mencetak satu baris
#   per tick (interval 2 s), sementara Get-Counter mengambil sampel per 2 s pada
#   jendela waktu yang sama. Pasangan diambil per indeks dari dua sesi, sampel
#   pertama dibuang karena masih warm-up provider.
# - Acuan CPU: \Processor(_Total)\% Processor Time - basis yang sama dengan
#   sysinfo. \Processor Information(_Total)\% Processor Utility dicatat sebagai
#   pembanding "angka Task Manager" (Win11 memakai Utility untuk angka CPU).
# - Acuan RAM: (total - \Memory\Available MBytes) / total, sama dengan cara
#   Task Manager menghitung "In use".
# - Acuan disk: persen kosong volume C: dari Get-Volume.
#
# Pemakaian (dari akar repo):
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\akurasi.ps1 -LoadThreads 0
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\akurasi.ps1 -LoadThreads 8
#
# Tanpa admin. `load.ps1` memakai runspace pool; bila mesin membatasi CPU
# (hypervisor / skema daya), beban nyata lebih rendah dari jumlah thread.

param(
    [int]$LoadThreads = 0,
    [string]$SnapshotExe = "src-tauri\target\debug\examples\snapshot.exe",
    [string]$OutFile = ""
)

$ErrorActionPreference = 'Stop'

if (-not (Test-Path $SnapshotExe)) {
    Write-Error "Snapshot tidak ditemukan: $SnapshotExe (build dulu: cargo build --example snapshot --manifest-path src-tauri/Cargo.toml)"
    exit 1
}

$totalMB = [double](Get-CimInstance Win32_OperatingSystem).TotalVisibleMemorySize / 1KB

function Get-DiskFreePercent {
    $v = Get-Volume -DriveLetter C
    [math]::Round(($v.SizeRemaining / $v.Size) * 100, 2)
}

function Get-AppSession {
    $lines = & $SnapshotExe
    $rows = @()
    foreach ($line in $lines) {
        if ($line -match '^#(\d+).*cpu\s+([\d.]+)%\s+\w+\s\|\s+ram\s+([\d.]+)%\s+\w+\s\|\s+disk\s+([\d.]+)%') {
            $rows += [pscustomobject]@{
                Index   = [int]$matches[1]
                Cpu     = [double]$matches[2]
                Ram     = [double]$matches[3]
                Disk    = [double]$matches[4]
            }
        }
    }
    , $rows
}

$loadProc = $null
if ($LoadThreads -gt 0) {
    $loadProc = Start-Process powershell -ArgumentList @(
        '-NoProfile', '-ExecutionPolicy', 'Bypass',
        '-File', (Join-Path $PSScriptRoot 'load.ps1'),
        '-Threads', "$LoadThreads", '-Seconds', '240'
    ) -PassThru -WindowStyle Hidden
    Write-Output ("Beban: {0} thread (pid {1}), menunggu 6 detik sampai stabil." -f $LoadThreads, $loadProc.Id)
    Start-Sleep -Seconds 6
} else {
    Write-Output "Beban: idle (tidak ada beban tambahan)."
}

$diskFree = Get-DiskFreePercent
Write-Output ("Acuan disk: {0}% kosong pada C:." -f $diskFree)

$pairs = @()
foreach ($session in 1..2) {
    $job = Start-Job -ScriptBlock {
        $samples = Get-Counter -Counter @(
            '\Processor(_Total)\% Processor Time',
            '\Processor Information(_Total)\% Processor Utility',
            '\Memory\Available MBytes'
        ) -SampleInterval 2 -MaxSamples 4 -ErrorAction Stop
        # CounterSamples dikembalikan berurutan sesuai daftar counter, per sampel:
        # 0 = Processor Time, 1 = Processor Utility, 2 = Available MBytes.
        # Jangan mengelompokkan per Timestamp - milidetiknya bisa beda antar
        # counter sehingga satu sampel terpecah jadi dua grup.
        $c = @($samples.CounterSamples)
        $rows = @()
        for ($i = 0; $i + 2 -lt $c.Count; $i += 3) {
            $rows += [pscustomobject]@{
                CpuTime = [double]$c[$i].CookedValue
                CpuUtil = [double]$c[$i + 1].CookedValue
                AvailMB = [double]$c[$i + 2].CookedValue
            }
        }
        , $rows
    }
    Start-Sleep -Milliseconds 400
    $app = Get-AppSession
    $ref = Receive-Job -Job $job -Wait -AutoRemoveJob -ErrorAction Continue
    if (-not $ref) {
        Write-Error "Sampel acuan gagal diambil (Get-Counter). Jalankan manual untuk melihat pesan errornya."
        exit 1
    }
    $count = [math]::Min($app.Count, $ref.Count)
    for ($i = 0; $i -lt $count; $i++) {
        if ($app[$i].Index -eq 1) { continue }
        $ramRef = [math]::Round((1 - ($ref[$i].AvailMB / $totalMB)) * 100, 2)
        $pairs += [pscustomobject]@{
            Session    = $session
            Sample     = $app[$i].Index
            CpuApp     = $app[$i].Cpu
            CpuTimeRef = [math]::Round($ref[$i].CpuTime, 2)
            CpuUtilRef = [math]::Round($ref[$i].CpuUtil, 2)
            RamApp     = $app[$i].Ram
            RamRef     = $ramRef
            DiskApp    = $app[$i].Disk
            DiskRef    = $diskFree
        }
    }
}

if ($LoadThreads -gt 0 -and $loadProc) {
    Stop-Process -Id $loadProc.Id -Force -ErrorAction SilentlyContinue
}

$pairs = $pairs | Select-Object -First 5

function Get-Median($values) {
    $sorted = $values | Sort-Object
    $n = $sorted.Count
    if ($n -eq 0) { return 0 }
    if ($n % 2 -eq 1) { return $sorted[[int](($n - 1) / 2)] }
    return ($sorted[$n / 2 - 1] + $sorted[$n / 2]) / 2
}

function Get-DiffPercent($app, $ref) {
    if ($ref -eq 0) { return [double]::NaN }
    [math]::Round(([math]::Abs($app - $ref) / [math]::Abs($ref)) * 100, 1)
}

$rows = foreach ($p in $pairs) {
    [pscustomobject]@{
        'Sampel'         = "sesi$($p.Session)#$($p.Sample)"
        'CPU app'        = $p.CpuApp
        'CPU perfmon'    = $p.CpuTimeRef
        'selisih poin'   = [math]::Round([math]::Abs($p.CpuApp - $p.CpuTimeRef), 2)
        'selisih %'      = Get-DiffPercent $p.CpuApp $p.CpuTimeRef
        'CPU Utility'    = $p.CpuUtilRef
        'RAM app'        = $p.RamApp
        'RAM acuan'      = $p.RamRef
        'selisih poin '  = [math]::Round([math]::Abs($p.RamApp - $p.RamRef), 2)
        'Disk app'       = $p.DiskApp
        'Disk acuan'     = $p.DiskRef
        'selisih poin  ' = [math]::Round([math]::Abs($p.DiskApp - $p.DiskRef), 2)
    }
}

$rows | Format-Table -AutoSize | Out-String -Width 220 | Write-Output

$cpuDiff = @($pairs | ForEach-Object { Get-DiffPercent $_.CpuApp $_.CpuTimeRef })
$ramDiff = @($pairs | ForEach-Object { Get-DiffPercent $_.RamApp $_.RamRef })
$diskDiff = @($pairs | ForEach-Object { Get-DiffPercent $_.DiskApp $_.DiskRef })

$cpuMeanApp = [math]::Round(($pairs | Measure-Object -Property CpuApp -Average).Average, 2)
$cpuMeanRef = [math]::Round(($pairs | Measure-Object -Property CpuTimeRef -Average).Average, 2)
$utilMean = [math]::Round(($pairs | Measure-Object -Property CpuUtilRef -Average).Average, 2)

Write-Output ("CPU  median selisih = {0}%  (maks {1}%)  | rata-rata app {2}% vs perfmon {3}% vs Utility {4}%" -f `
    (Get-Median $cpuDiff), ($cpuDiff | Measure-Object -Maximum).Maximum, $cpuMeanApp, $cpuMeanRef, $utilMean)
Write-Output ("RAM  median selisih = {0}%  (maks {1}%)" -f (Get-Median $ramDiff), ($ramDiff | Measure-Object -Maximum).Maximum)
Write-Output ("Disk median selisih = {0}%  (maks {1}%)" -f (Get-Median $diskDiff), ($diskDiff | Measure-Object -Maximum).Maximum)

if ($OutFile) {
    $rows | Export-Csv -Path $OutFile -NoTypeInformation -Encoding UTF8
    Write-Output ("CSV: {0}" -f $OutFile)
}
