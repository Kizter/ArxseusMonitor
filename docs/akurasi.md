# Akurasi angka: aplikasi vs acuan Windows

Menguji success criteria DESIGN.md §5 nomor 2: **CPU, RAM, disk selisih < 5%**.
Diukur 16 Nov pada build rilis `arxseus-monitor.exe` (mesin uji i5-13400F,
15,86 GB, Windows 11 build 26200).

Alat: `docs/akurasi.ps1`. Jalankan ulang dengan:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File docs\akurasi.ps1 -LoadThreads 0
powershell -NoProfile -ExecutionPolicy Bypass -File docs\akurasi.ps1 -LoadThreads 8
```

## Metode

| Aspek | Cara |
|---|---|
| Nilai aplikasi | `examples/snapshot.exe` — memakai `Monitor` yang sama dengan aplikasi, satu tick per 2 detik |
| Acuan CPU | `\Processor(_Total)\% Processor Time` (basis yang sama dengan `sysinfo`: tanpa bobot frekuensi) |
| Pembanding CPU | `\Processor Information(_Total)\% Processor Utility` (basis angka CPU Task Manager Windows 11) |
| Acuan RAM | `(total - \Memory\Available MBytes) / total`, sama dengan "In use" |
| Acuan disk | persen kosong volume C: dari `Get-Volume` |
| Jumlah sampel | 2 sesi x 4 tick, sampel #1 dibuang (warm-up) -> **5 pasangan** |
| Jendela | Kedua sisi 2 detik, mulai bersamaan; pergeseran <= 1 detik |

Dua kondisi diukur terpisah karena angka CPU hanya bermakna dibandingkan pada
beban yang stabil: **idle** (tanpa beban tambahan) dan **beban tetap**
(`docs/load.ps1 -Threads 8`, mesin ini membatasi 8 thread ke ~51%).

## Hasil: idle

| Sampel | CPU app | CPU perfmon | selisih poin | selisih % | CPU Utility | RAM app | RAM acuan | Disk app | Disk acuan |
|---|---|---|---|---|---|---|---|---|---|
| sesi1#2 | 7,0 | 13,31 | 6,31 | 47,4 | 12,18 | 61,4 | 61,23 | 16,9 | 16,91 |
| sesi1#3 | 7,5 | 4,70 | 2,80 | 59,6 | 10,01 | 61,4 | 61,14 | 16,9 | 16,91 |
| sesi1#4 | 4,7 | 4,55 | 0,15 | 3,3 | 10,50 | 61,1 | 60,98 | 16,9 | 16,91 |
| sesi2#2 | 7,1 | 7,72 | 0,62 | 8,0 | 9,95 | 61,2 | 61,18 | 16,9 | 16,91 |
| sesi2#3 | 6,2 | 8,62 | 2,42 | 28,1 | 14,13 | 61,2 | 61,20 | 16,9 | 16,91 |

Median: **CPU 2,80 poin (28,1%)**, RAM **0,2%**, disk **0,1%**.

## Hasil: beban tetap 8 thread

| Sampel | CPU app | CPU perfmon | selisih poin | selisih % | CPU Utility | RAM app | RAM acuan | Disk app | Disk acuan |
|---|---|---|---|---|---|---|---|---|---|
| sesi1#2 | 51,0 | 50,98 | 0,02 | 0,0 | 81,29 | 60,9 | 60,91 | 16,9 | 16,91 |
| sesi1#3 | 51,2 | 50,34 | 0,86 | 1,7 | 80,69 | 60,9 | 60,78 | 16,9 | 16,91 |
| sesi1#4 | 51,9 | 51,76 | 0,14 | 0,3 | 83,07 | 60,8 | 60,68 | 16,9 | 16,91 |
| sesi2#2 | 52,4 | 52,36 | 0,04 | 0,1 | 84,57 | 61,2 | 60,49 | 16,9 | 16,91 |
| sesi2#3 | 54,8 | 51,48 | 3,32 | 6,4 | 84,21 | 60,5 | 60,49 | 16,9 | 16,91 |

Median: **CPU 0,14 poin (0,3%)**, RAM **0,2%** (maks 1,2%), disk **0,1%**.

## Kesimpulan

| Metrik | Median selisih | Target | Status |
|---|---|---|---|
| CPU (beban tetap, vs `% Processor Time`) | 0,3% (0,14 poin) | < 5% | lulus |
| RAM | 0,2% (maks 1,2%) | < 5% | lulus |
| Disk C: kosong | 0,1% | < 5% | lulus |

Pada idle keduanya bergerak di 5-13% dan jendela 2 detik tidak persis sama, jadi
selisih **relatif** membesar (median 28,1%) walau selisih absolutnya kecil
(median 2,80 poin). Karena itu angka CPU hanya dipakai sebagai bukti pada beban
tetap. Ini sesuai catatan metode di `docs/responden.md` §10: bandingkan pada
kondisi relatif stabil.

## Temuan penting: CPU aplikasi bukan angka Task Manager

Pada beban yang sama, `% Processor Utility` terukur **82,8%** sementara
`% Processor Time` dan aplikasi ada di **~52%**. `Utility` memberi bobot
frekuensi (turbo), `Time` tidak. Aplikasi memakai basis `Time` karena itu yang
dihitung `sysinfo` dari `NtQuerySystemInformation`.

Konsekuensinya, dibandingkan Task Manager Windows 11 (yang menampilkan
`Utility`), angka CPU aplikasi akan **tampak lebih rendah saat CPU di-turbo**.
Ini perbedaan definisi metrik, bukan galat pembacaan: terhadap basis yang sama
selisihnya 0,3%. Ambang DESIGN.md §7 (70/90%) juga berlaku pada basis `Time`.

## Batas pengujian

1. Acuan diambil dari counter Windows, **bukan** GUI Task Manager (GUI tidak
   bisa diotomasi); angka GUI perlu dibaca manual saat demo.
2. Jendela sampel kedua sisi bergeser <= 1 detik; pada beban yang berubah cepat
   selisih per sampel bisa > 5% (terlihat pada sesi2#3: 6,4%).
3. Suhu tidak diuji: suhu CPU butuh admin (`0x80041003`) dan suhu GPU tidak punya
   acuan independen di mesin ini (NVML satu-satunya sumber).
4. Hanya volume C: yang diukur.
5. RAM dibandingkan sebagai persentase, bukan byte, karena acuan membulatkan.
