# Uji ketahanan (soak test)

Mengukur pemakaian sumber daya aplikasi saat hanya ikon tray yang aktif — kondisi
pemakaian sehari-hari: jendela detail ditutup, poller tetap berjalan tiap 2 detik.
Tujuannya membuktikan dua NFR (`DESIGN.md` §4): CPU idle **< 2 %** dan RAM
**< 150 MB**, sekaligus memastikan tidak ada kebocoran memori/handle.

## 1. Metode

- Sampler: `docs/soak.ps1` (PowerShell, satu sampel per menit). Skrip ini hanya
  membaca `Get-Process hwmonitor`, jadi tidak menambah beban ke aplikasi.
- CPU dihitung dari selisih `TotalProcessorTime` **proses**, dibagi `60 s × 16
  core logis` — sama seperti angka "CPU" di Task Manager (persen dari seluruh
  mesin, bukan persen dari satu core).
- RAM yang dicatat: working set (`WorkingSet64`) dan private bytes
  (`PrivateMemorySize64`). NFR 150 MB dibandingkan dengan working set, angka yang
  sama dengan yang dilihat pengguna di Task Manager.
- Rekap: `docs/soak-report.ps1` (median, maksimum, tabel Markdown).
- Jendela detail ditutup lewat `WM_CLOSE` (tersembunyi ke tray) sebelum
  pengambilan sampel; `IsWindowVisible=false` dipastikan lewat Win32.

Mesin uji: Windows 11 Pro 10.0.26200, Intel Core i5-13400F (10 core / 16 thread),
RAM 15,9 GB, layar 2560×1440 @ 150 %.

## 2. Seri 1 + 2 — build release, satu sesi (PID 24620)

Aplikasi dijalankan 15:26:17 lalu disembunyikan ke tray. Sampler pertama berhenti
sendiri di menit 11 (proses PowerShell-nya mati, bukan aplikasinya); sampler
kedua melanjutkan sesi yang sama dari menit ke-12 sampai ke-32. Karena itu ada
dua tabel, tetapi tetap **satu proses** dengan PID sama.

### Seri 1 (menit 0–11, uptime 0–11 menit)

Sampel 12; CPU median 0,02 %, maks 0,02 %, min 0,01 %; RAM 52,4–52,9 MB;
private 29,1–29,3 MB; handle 441–450; thread 23–29.

| Menit | CPU | RAM (MB) | Private (MB) | Handle | Thread |
|---:|---:|---:|---:|---:|---:|
| 0 | — | 52.5 | 29.2 | 446 | 28 |
| 1 | 0.02% | 52.5 | 29.1 | 444 | 24 |
| 2 | 0.02% | 52.5 | 29.1 | 444 | 24 |
| 3 | 0.01% | 52.4 | 29.1 | 442 | 23 |
| 4 | 0.01% | 52.5 | 29.1 | 443 | 25 |
| 5 | 0.02% | 52.8 | 29.2 | 446 | 28 |
| 6 | 0.02% | 52.9 | 29.3 | 450 | 29 |
| 7 | 0.02% | 52.8 | 29.2 | 450 | 26 |
| 8 | 0.01% | 52.8 | 29.2 | 443 | 25 |
| 9 | 0.02% | 52.8 | 29.2 | 443 | 25 |
| 10 | 0.01% | 52.8 | 29.2 | 441 | 25 |
| 11 | 0.01% | 52.8 | 29.2 | 444 | 25 |

### Seri 2 (menit 0–20 tabel = uptime 12–32 menit)

Sampel 21; CPU median 0,02 %, maks 0,02 %, min 0,01 %; RAM 52,8–53,0 MB;
private 29,1–29,2 MB; handle 441–444; thread 23–26.

| Menit | CPU | RAM (MB) | Private (MB) | Handle | Thread |
|---:|---:|---:|---:|---:|---:|
| 0 | — | 52.8 | 29.2 | 444 | 25 |
| 1 | 0.02% | 52.9 | 29.2 | 443 | 26 |
| 2 | 0.02% | 52.9 | 29.2 | 443 | 26 |
| 3 | 0.02% | 52.8 | 29.1 | 441 | 23 |
| 4 | 0.02% | 52.9 | 29.1 | 444 | 24 |
| 5 | 0.02% | 52.9 | 29.1 | 444 | 24 |
| 6 | 0.02% | 52.9 | 29.1 | 442 | 23 |
| 7 | 0.02% | 53.0 | 29.2 | 443 | 25 |
| 8 | 0.02% | 52.9 | 29.1 | 441 | 23 |
| 9 | 0.02% | 52.9 | 29.1 | 444 | 24 |
| 10 | 0.02% | 52.9 | 29.1 | 444 | 24 |
| 11 | 0.01% | 52.9 | 29.1 | 442 | 23 |
| 12 | 0.01% | 53.0 | 29.2 | 443 | 25 |
| 13 | 0.01% | 53.0 | 29.2 | 443 | 25 |
| 14 | 0.01% | 53.0 | 29.1 | 444 | 24 |
| 15 | 0.01% | 52.9 | 29.1 | 444 | 24 |
| 16 | 0.02% | 52.9 | 29.1 | 442 | 23 |
| 17 | 0.01% | 53.0 | 29.2 | 443 | 25 |
| 18 | 0.01% | 52.9 | 29.1 | 441 | 23 |
| 19 | 0.02% | 53.0 | 29.1 | 444 | 24 |
| 20 | 0.01% | 52.9 | 29.1 | 444 | 24 |

### Baca angka

- Total CPU proses setelah 32 menit: **5,422 detik** (`cpuDetik`), yaitu 0,28 %
  dari satu core atau **0,018 % dari seluruh mesin** — di bawah ambang 2 %
  dengan selisih dua orde besaran.
- Working set bergerak 52,4 → 53,0 MB dan **tidak naik lagi** selama 20 menit
  terakhir; private bytes justru turun tipis (29,3 → 29,1 MB). Pola ini khas
  alokasi awal yang stabil, bukan kebocoran.
- Handle 441–450 dan thread 23–29 berosilasi mengikuti satu putaran polling
  (buka sensor → tutup), tanpa tren menaik. Tidak ada penumpukan handle.

## 3. Seri 3 — build final (PID 26136)

Diukur pada binary yang benar-benar dikirim (`npm run tauri build` setelah perbaikan
aksesibilitas), 16:04:25–16:24:26. Jendela disembunyikan ke tray sebelum sampel
pertama diambil.

Sampel 21; CPU median 0,02 %, maks 0,02 %, min 0,01 %; RAM 34,0–52,8 MB;
private 28,6–29,1 MB; handle 441–450; thread 23–29; total CPU proses 3,750 detik
untuk 20 menit.

| Menit | CPU | RAM (MB) | Private (MB) | Handle | Thread |
|---:|---:|---:|---:|---:|---:|
| 0 | — | 52.2 | 28.8 | 444 | 28 |
| 1 | 0.01% | 52.2 | 28.6 | 444 | 24 |
| 2 | 0.02% | 52.2 | 28.6 | 442 | 23 |
| 3 | 0.02% | 52.2 | 28.6 | 442 | 23 |
| 4 | 0.02% | 52.3 | 29.0 | 443 | 25 |
| 5 | 0.02% | 52.6 | 29.1 | 446 | 28 |
| 6 | 0.01% | 52.7 | 29.1 | 450 | 29 |
| 7 | 0.02% | 52.6 | 29.1 | 450 | 26 |
| 8 | 0.02% | 52.6 | 29.0 | 443 | 25 |
| 9 | 0.02% | 52.6 | 29.0 | 443 | 25 |
| 10 | 0.02% | 52.8 | 29.1 | 446 | 28 |
| 11 | 0.01% | 52.8 | 29.1 | 446 | 26 |
| 12 | 0.02% | 34.0 | 29.0 | 442 | 23 |
| 13 | 0.02% | 34.0 | 29.0 | 443 | 25 |
| 14 | 0.02% | 34.0 | 29.0 | 441 | 23 |
| 15 | 0.02% | 34.0 | 29.0 | 444 | 24 |
| 16 | 0.02% | 34.0 | 29.0 | 444 | 24 |
| 17 | 0.01% | 34.0 | 29.0 | 444 | 25 |
| 18 | 0.02% | 34.1 | 29.1 | 443 | 26 |
| 19 | 0.02% | 34.1 | 29.0 | 443 | 25 |
| 20 | 0.02% | 34.0 | 29.0 | 441 | 23 |

Pada menit 12 working set turun dari 52,8 MB ke 34,0 MB dan tetap di sana.
Penurunan itu berasal dari memory trimming Windows pada proses yang lama tidak
menyentuh UI (halaman yang sudah tidak dipakai dilepas), bukan dari aplikasi
melepas buffer: private bytes tidak ikut turun (29,0 MB). Angka ini justru
menguntungkan NFR.

## 4. Perbandingan dengan NFR

| NFR (`DESIGN.md` §4) | Ambang | Terukur | Status |
|---|---|---|---|
| CPU saat idle | < 2 % | 0,02 % median (maks 0,02 %) dari seluruh mesin; 0,28 % dari satu core | lulus |
| RAM (working set) | < 150 MB | 34,0–53,0 MB | lulus |
| Private bytes | (tidak ada NFR) | 28,6–29,3 MB | informatif |
| Kebocoran handle/thread | tidak naik terus | 441–450 handle, 23–29 thread, berosilasi mengikuti satu putaran polling | lulus |
| Stabilitas proses | tetap satu instance | satu PID per sesi selama pengamatan (32 menit dan 20 menit), tanpa restart | lulus |

## 5. Batas pengujian

- Durasi 32 menit (build pra-perapian) + 20 menit (build final). Kebocoran lambat
  yang baru muncul setelah berjam-jam tidak tertangkap; karena itu uji ini
  dilengkapi pemeriksaan tren (handle/thread/memori tidak monoton naik), bukan
  hanya ambang absolut.
- Sampel diambil per menit, sehingga lonjakan singkat (mis. sesaat setelah
  jendela dibuka) tidak terlihat. Beban saat jendela terbuka diukur terpisah
  sebagai waktu startup di `docs/black-box.md`.
- Aplikasi diuji dalam keadaan tersembunyi di tray dengan sumber suhu `auto`
  (NVML aktif, CPU temp tidak tersedia karena butuh admin). Kondisi beban
  berat (CPU 100 %, disk penuh) tidak termasuk cakupan.
