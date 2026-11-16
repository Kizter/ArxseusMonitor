# ArxseusMonitor

Monitor kesehatan hardware untuk pengguna awam (Windows 10/11). Aplikasi membaca
metrik hardware, menerjemahkannya menjadi tiga status yang bisa langsung
dipahami — **Aman**, **Waspada**, **Bahaya** — plus status **Menunggu data** bila
sebuah sensor tidak terbaca, lalu menampilkan saran tindakan singkat.

Dua permukaan:

- **Ikon system tray** — jawaban cepat: warna status, tooltip berisi status satu
  baris dan angka utama.
- **Jendela detail** — lima metrik (CPU, RAM, ruang penyimpanan, suhu CPU, suhu
  kartu grafis), panel "Kenapa?" yang menjelaskan alasan status dengan bahasa
  sehari-hari, dan pengaturan bahasa/interval/sumber suhu.

![Jendela aplikasi](.)

## Prinsip

- **Tanpa admin, tanpa jaringan, tanpa database.** Tidak ada telemetri, tidak ada
  unduhan, tidak ada layanan latar belakang. Kode aplikasi tidak membuka koneksi
  apa pun; runtime WebView2 bawaan Microsoft tetap membuka dua koneksi TLS saat
  start — terukur di `docs/jaringan.md`.
- **Tidak ada angka palsu.** Sensor yang gagal ditandai "tidak tersedia", bukan
  diisi 0 °C, dan aplikasi tidak panic.
- **Status tidak berkedip.** Status baru berubah setelah 3 sampel bertahan
  (±6 detik pada interval 2 detik).
- **Bilingual ID/EN.** Teks status dikirim Rust sebagai kunci, frontend yang
  menerjemahkan — satu sumber istilah.
- **Warna tidak pernah sendirian.** Setiap status selalu disertai teks dan ikon.

## Stack

| Lapisan | Teknologi |
|---|---|
| Shell desktop | Tauri v2 |
| Backend | Rust (edition 2024) |
| Frontend | React 19 + TypeScript + Vite 8 |
| Styling | Tailwind CSS v4 (plugin Vite, CSS-first) |

Sumber kebenaran ada di Rust: polling dan perhitungan status dilakukan di thread
sendiri, frontend hanya merender snapshot terakhir. Alasannya, WebView2 men-throttle
timer JavaScript saat jendela tersembunyi di tray.

## Menjalankan

```bash
npm install
npm run tauri dev            # jalankan aplikasi (hot reload)

npm run tauri build          # build installer Windows (MSI + NSIS)
npm run build                # typecheck + vite build
npm run lint                 # typecheck + cargo clippy -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Mode demo untuk kondisi "sensor suhu tidak tersedia":

```bash
cargo run --manifest-path src-tauri/Cargo.toml -- --simulate-missing-temp
```

Flag yang sama juga berlaku pada instance yang sudah berjalan: menjalankan exe
dengan flag itu akan memaksa mode simulasi pada instance di tray dan memunculkan
jendelanya.

Alat bantu di `docs/`:

```bash
cargo run --example probe --manifest-path src-tauri/Cargo.toml     # uji provider sensor
cargo run --example snapshot --manifest-path src-tauri/Cargo.toml  # snapshot JSON
powershell -NoProfile -ExecutionPolicy Bypass -File docs\soak.ps1 -Minutes 30
powershell -NoProfile -ExecutionPolicy Bypass -File docs\akurasi.ps1 -LoadThreads 8
```

## Ambang batas status

| Metrik | Aman | Waspada | Bahaya |
|---|---|---|---|
| Beban CPU | < 70% | 70–90% | > 90% |
| Pemakaian RAM | < 80% | 80–92% | > 92% |
| Ruang disk kosong (C:) | > 15% | 8–15% | < 8% |
| Suhu CPU | < 80 °C | 80–90 °C | > 90 °C |
| Suhu GPU | < 80 °C | 80–90 °C | > 90 °C |

Status keseluruhan = level terburuk dari metrik yang terbaca. Metrik yang tidak
terbaca tidak dianggap aman maupun berbahaya.

## Hasil pengujian

| Berkas | Isi |
|---|---|
| `docs/black-box.md` | 12 skenario fungsional pada build debug dan release |
| `docs/soak-test.md` | Dua sesi 52 menit: CPU idle 0,02% mesin, working set 34–53 MB, tanpa kebocoran handle |
| `docs/a11y.md` | Audit aksesibilitas: kontras AA, fokus keyboard, nama aksesibel, landmark |
| `docs/sensor-probe.md` | Hasil probe tiap provider sensor di mesin uji |
| `docs/akurasi.md` | Akurasi CPU/RAM/disk vs counter Windows: median selisih 0,3% / 0,2% / 0,1% |
| `docs/jaringan.md` | Pemakaian jaringan terukur: aplikasi 0 koneksi, runtime WebView2 2 koneksi TLS |
| `docs/responden.md` | Rancangan instrumen uji responden (tidak dijalankan, lihat D27 di `DESIGN.md`) |

Angka build rilis: binary 4,55 MB, MSI 2,18 MiB, NSIS 1,49 MiB, startup
268–336 ms sampai jendela tampil, CPU idle 0,02% (target < 2%), working set
34–53 MB (target < 150 MB).

## Batasan yang diketahui

- **Windows saja.** Ditulis untuk Win32 (tray, WMI, NVML) — tidak ada target
  macOS/Linux.
- **Suhu CPU tidak selalu tersedia.** Tanpa driver atau LibreHardwareMonitor yang
  berjalan sebagai admin, Windows menolak membaca `MSAcpi_ThermalZoneTemperature`
  (`0x80041003`). Aplikasi menandainya "Menunggu data", bukan menebak.
- Suhu GPU bergantung pada NVML; kartu non-NVIDIA tidak terbaca.
- Baterai belum didukung (mesin uji desktop).
- **Angka CPU memakai basis `% Processor Time`**, sama dengan `sysinfo`. Task
  Manager Windows 11 menampilkan `% Processor Utility` yang memberi bobot turbo,
  jadi saat CPU di-turbo angka di aplikasi ini tampak lebih rendah. Terhadap basis
  yang sama selisihnya 0,3% (`docs/akurasi.md`).
- **Runtime WebView2 membuka dua koneksi TLS ke Microsoft** saat start (di luar
  kendali konfigurasi Tauri, diukur di `docs/jaringan.md`). Kode aplikasi sendiri
  tidak melakukan request jaringan.
- Uji pemahaman ke pengguna awam belum dijalankan, sehingga klaim "lebih mudah
  dipahami daripada angka mentah" belum punya data responden (D27).

## Dokumen

- `DESIGN.md` — desain lengkap: tujuan, non-goals, arsitektur, ambang batas,
  rencana pengujian, dan seluruh keputusan (D1–D29).
- `AGENTS.md` — konvensi kerja dan riwayat fase pengembangan.
