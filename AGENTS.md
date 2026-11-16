# AGENTS.md

## Overview

Aplikasi desktop Windows yang memantau kesehatan hardware dan menerjemahkan metrik
teknis menjadi status yang dipahami orang awam. Dua permukaan: ikon system tray
(jawaban cepat) dan jendela detail (penjelasan + saran tindakan).

Proyek ini **gelar karya**, jadi metode dan bukti pengujian sama pentingnya dengan
fitur. Desain lengkap ada di `DESIGN.md` — baca itu sebelum mengubah arsitektur.

## Stack

| Lapisan | Teknologi |
|---|---|
| Shell desktop | Tauri v2 |
| Backend | Rust (edition 2024) |
| Frontend | React 19 + TypeScript + Vite 8 |
| Styling | Tailwind CSS v4 (plugin Vite, CSS-first) |
| Target | Windows 10/11 saja |

## Perintah

```bash
npm install          # pasang dependensi frontend
npm run tauri dev    # jalankan aplikasi (dev, hot reload)
npm run tauri build  # build installer Windows
npm run typecheck    # tsc --noEmit
npm run lint:rust    # cargo clippy -D warnings
npm run lint         # typecheck + clippy
npm run build        # typecheck + vite build (dipakai Tauri saat build)
cargo test --manifest-path src-tauri/Cargo.toml   # unit test Rust
cargo run --example probe --manifest-path src-tauri/Cargo.toml  # probe sensor
cargo run --example snapshot --manifest-path src-tauri/Cargo.toml  # snapshot JSON
npm run dev          # server Vite saja (dipakai build debug saat dijalankan manual)
cargo run --manifest-path src-tauri/Cargo.toml -- --simulate-missing-temp
                     # mode demo "sensor hilang"; butuh `npm run dev` karena build debug memuat devUrl
```

Flag `--simulate-missing-temp` juga berlaku pada instance yang sudah berjalan
(menjalankan exe dengan flag saat aplikasi ada di tray akan memaksa mode simulasi
dan memunculkan jendelanya).

## Struktur

```
src/                  # React + TS
  App.tsx
  components/         # HealthBadge, MetricCard, StatusExplanation, Settings
  lib/                # ipc.ts, types.ts, i18n.ts
src-tauri/src/
  main.rs             # bootstrap
  lib.rs              # Tauri builder, setup, invoke_handler
  runtime.rs          # tray, jendela, command IPC, wiring poller
  monitor/            # poller.rs, model.rs
  sensors/            # trait SensorProvider + implementasi provider
  health.rs           # ambang batas + perhitungan status
  settings.rs         # settings JSON
docs/                 # bukti pengujian (probe, black-box, a11y, soak) + instrumen
                      # uji responden (responden.md, kartu-angka.html) + alat PowerShell
```

## Konvensi penting

1. **Rust adalah sumber kebenaran.** Polling dan perhitungan status di Rust;
   frontend hanya merender hasil. Jangan polling dari `setInterval` di React —
   WebView2 men-throttle timer saat jendela tersembunyi.
2. **Status hanya 3 level + `unknown`.** Jangan menambah level tanpa mengubah
   `DESIGN.md`.
3. **Ambang batas berasal dari `DESIGN.md` §7.** Jangan mengarang angka baru;
   setiap perubahan harus punya dasar teknis yang bisa dikutip.
4. **Status berubah setelah 3 sampel bertahan** (±6 detik), bukan seketika.
5. **Setiap provider sensor boleh gagal.** Kembalikan `Option`, tandai
   "tidak tersedia". Tidak boleh ada angka palsu (0 °C) dan tidak boleh panic.
6. **Tanpa admin, tanpa jaringan, tanpa database.** Jangan menambah dependensi
   yang melanggar tiga batasan itu.
7. **Bilingual ID/EN lewat kunci i18n.** Rust mengirim `reasonKey`, frontend
   menerjemahkan. Jangan menulis teks UI langsung di Rust.
8. **Warna tidak pernah sendirian.** Status selalu disertai teks. Hijau/kuning/
   merah khusus status kesehatan; biru untuk tombol, tautan, dan fokus.
9. **Aksesibilitas:** kontras AA, fokus keyboard terlihat, teks ≥ 14 px.
10. **YAGNI.** Fitur di luar `DESIGN.md` §3 (non-goals) tidak dikerjakan tanpa
    persetujuan.

## Status saat ini

Fase 0 selesai: scaffold + toolchain validasi.
Fase 1 selesai (13 Nov): 4 provider sensor + probe CLI. Hasil di
`docs/sensor-probe.md`.
Fase 2 selesai (13 Nov): model `Snapshot`, health engine (3 level + `unknown`,
latch 3 sampel), poller thread + circuit breaker, settings JSON. 30 unit test.
Fase 3 selesai (13 Nov): tray (ikon runtime 32 px, tooltip 3 baris, menu
Buka/Pengaturan/Keluar), jendela detail React (banner, 5 kartu metrik, panel
"Kenapa?", intro first-run, pengaturan ID/EN), command IPC
`get_snapshot`/`get_settings`/`save_settings`/`configure_tray`/`set_tray_tooltip`,
close = sembunyi ke tray. 32 unit test lulus, clippy bersih, `npm run build` OK.

Bukti smoke test Fase 3 (13 Nov, mesin uji 2560×1440 @150%):
- jendela 360×520 tampil di pojok kanan bawah (margin kanan 18 px, bawah 84 px
  fisik = 12/56 px × 1.5);
- tooltip tray 3 baris ID: `Aman — … / CPU … · RAM … / Suhu kartu grafis …`;
  piksel tengah ikon = `#22C55E` (hijau level safe);
- menu tray terbaca "Buka / Pengaturan / Keluar" (MSAA), item "Buka" memunculkan
  jendela, klik kiri tray menyembunyikan jendela;
- tutup jendela (WM_CLOSE) → `IsWindowVisible=false`, proses tetap hidup;
- UI lewat DevTools: 5 kartu + angka masuk akal (CPU 5,7%, RAM 69,4%,
  GPU 45 °C), ganti bahasa EN→ID mengubah seluruh teks + tooltip, interval 5 s
  membuat `intervalMs=5000` (poller restart), intro muncul saat `introSeen=false`
  dan tersimpan `true` setelah "Mengerti";
- settings tersimpan di `%APPDATA%\hwmonitor\settings.json` dan bertahan setelah
  reload.

Redesign UI GitHub (14 Nov): seluruh permukaan memakai token Primer gelap
(`#0d1117`/`#161b22`/`#30363d`/`#e6edf3`), metrik jadi baris padat dalam satu
kotak bergaris, pil status berlatar tipis, angka monospace, tanpa bayangan.

Bukti verifikasi redesign (14 Nov, DevTools/CDP, viewport 322×465 CSS px):
- kanvas `#0d1117`, teks `#e6edf3`, border `#30363d`, radius 6 px, `outline` biru;
- kontras terukur: banner 6,5:1; saran 5,4:1; label 16:1; detail 6,2:1; pil Aman
  6,5:1, Waspada 5,9:1, Menunggu data 5,0:1 — semua ≥ 4,5:1;
- ukuran teks terkecil 14 px, tidak ada overflow horizontal
  (`scrollWidth = clientWidth = 322`), konten menggulir di dalam panel
  (`scrollHeight 1093 / clientHeight 351`), footer tetap di bawah;
- fokus keyboard: Tab mencapai tombol dan baris, `:focus-visible` benar,
  outline biru 2 px terlihat; Enter dan Space pada baris metrik mengganti pilihan
  (`aria-pressed` + panel "Kenapa?" ikut berubah); select formulir memakai border
  biru + ring 2 px `#58a6ff`;
- alur detail → pengaturan → detail, ganti bahasa EN↔ID (judul jadi "Settings"
  lalu "Pengaturan"), settings tetap `id`/2 s/`auto`/`introSeen=false`;
- `npm run lint` bersih, `cargo test` 32 lulus, `npm run build` OK.

Temuan penting: **suhu CPU tidak tersedia tanpa admin** di mesin uji
(`0x80041003`), suhu GPU tersedia via NVML, `sysinfo` warm 0,6 ms. Strategi suhu
dikunci best-effort. Jalur LHM belum terverifikasi.
Baterai dikeluarkan dari MVP (D19), settings pakai JSON polos (D20), penempatan
jendela manual (D22), label tray dari React (D23), single instance lewat
`tauri-plugin-single-instance` (D24).

Single instance selesai (14 Nov): plugin didaftarkan sebagai plugin pertama di
`lib.rs`, callback memanggil `runtime::show_window`. Bukti uji pada
`target\debug\hwmonitor.exe`:
- instance pertama jalan (jendela tampil) → WM_CLOSE menyembunyikan jendela,
  proses tetap 1;
- instance kedua dijalankan → proses kedua keluar dengan exit 0 dalam < 1 s,
  jumlah proses tetap 1 dengan PID sama, jendela instance pertama muncul lagi,
  dan log mencatat `[app] instance kedua terdeteksi: tampilkan jendela yang ada`;
- dengan `startMinimized: true` instance pertama mulai tanpa jendela, lalu
  instance kedua memunculkannya.

Belum dikerjakan: nama produk final (diputuskan di fase terakhir, D di
`DESIGN.md` §13), verifikasi jalur LHM di mesin demo.

Fase 4 — pengujian fungsional/black-box + perapian UI (15 Nov):

- `docs/black-box.md`: 12 skenario (B1–B12) dijalankan pada build debug dan
  diulang pada build release — buka/tutup, tray (tooltip, menu, klik kiri),
  ganti bahasa, settings tersimpan/rusak, sumber suhu `off`, `--simulate-missing-temp`,
  single instance.
- Release build: binary 4,55 MB; MSI 2,18 MiB; NSIS 1,49 MiB; startup 852 ms
  sampai jendela tampak (842 ms sampai CDP siap) — NFR < 15 MB dan < 2 s terpenuhi.
- `--simulate-missing-temp` (D25) kini juga berlaku lewat instance kedua:
  argumen instance kedua dibaca di `runtime::on_second_instance`, flag disimpan
  sebagai `AtomicBool`, poller di-restart, jendela dimunculkan. Tanpa jalur ini
  flag diam-diam diabaikan pada skenario demo yang paling mungkin (aplikasi sudah
  lama di tray).
- Audit aksesibilitas statis + perbaikan: fokus tidak hilang saat intro ditutup,
  `aria-current`/`aria-controls` pada baris metrik, detail angka masuk nama
  aksesibel, landmark `main`, heading sr-only "Ringkasan metrik", panel "Kenapa?"
  jadi region bernama, `aria-describedby` untuk catatan formulir, `lang` pada opsi
  bahasa, kontras hover 5,85:1, border kontrol 4,12:1, `role="alert"` yang
  diumumkan ulang. Ringkasan 31 temuan dan batas pengujian ada di `docs/a11y.md`.
- Test Rust: `monitor_wiring_matches_the_simulation_flag` menguji filter lewat
  `Monitor::new`, `tick_without_temperature_sources_produces_a_snapshot` bebas
  WMI, versi provider nyata ditandai `#[ignore]` — 34 test lulus, 1 diabaikan.

Uji ketahanan (15 Nov, `docs/soak-test.md`, alat `docs/soak.ps1` +
`docs/soak-report.ps1`): dua sesi, 52 menit total, jendela tersembunyi di tray.

- CPU idle 0,02 % median dari seluruh mesin (NFR < 2 %), yaitu 0,28 % dari satu
  core; total 3,75 detik CPU untuk 20 menit.
- Working set 34,0–53,0 MB (NFR < 150 MB); private bytes 28,6–29,3 MB. Pada sesi
  kedua working set turun ke 34 MB karena memory trimming Windows — private
  bytes tidak turun, jadi bukan aplikasi melepas buffer dan bukan kebocoran.
- Handle 441–450, thread 23–29, berosilasi mengikuti satu putaran polling; tidak
  ada tren menaik. Satu PID per sesi tanpa restart.

Bukti aksesibilitas terukur (build final, CDP): 5 baris metrik tanpa
`aria-pressed`, satu `aria-current`; nama aksesibel memuat detail angka; ring
fokus 2 px `#58a6ff` (baris metrik digambar ke dalam karena induk
`overflow:hidden`); PageDown asli menggulir `0 → 396` dari maks 397 px; fokus ke
`#status-banner` setelah "Mengerti"; muncul dari tray memfokuskan `<h1>` bila
fokus kosong dan tidak merampas fokus bila baris masih terfokus; startup 636 ms
(cache hangat).

Temuan baru saat pengukuran (F31): `document.visibilityState` tetap `"visible"`
walau jendela disembunyikan lewat `WM_CLOSE`, sedangkan `document.hasFocus()`
berubah `true → false`. Penjaga "jangan pindahkan fokus saat jendela tidak
dipakai" karenanya memakai `hasFocus()`; versi `visibilityState` sebelumnya tidak
pernah menahan apa pun.

Persiapan uji responden (16 Nov): instrumen lengkap di `docs/responden.md`
(recruitment + etika, 7 tugas dengan kriteria bantuan, 8 stimulus pemahaman,
SUS 10 item + cara hitung, lembar akurasi 5 sampel, pilot 2 orang, template
rekap, batas pengujian). Alat stimulus masuk repo: `docs/ramfill.ps1`,
`docs/load.ps1`, `docs/kartu-angka.html`.

Kalibrasi stimulus pada mesin uji (hasil snapshot nyata, bukan perkiraan):

- idle: `overall=safe`, CPU 4-10%, RAM 56,9-57,7%, disk 16,7% kosong, GPU 45-47 °C,
  suhu CPU `unknown`;
- `ramfill -TargetPercent 82` → aplikasi `ram 81,9% warn`, `overall=warn`;
- `ramfill -TargetPercent 93` → `ram 93,1% danger`, `overall=danger`;
- `--simulate-missing-temp` pada instance aplikasi → dua suhu `unknown`, `overall=safe`;
  `cargo run --example snapshot` **tidak** meneruskan flag itu (contoh memanggil
  `Monitor::new(&settings, false)`), jadi jangan dipakai sebagai bukti S4;
- 16 thread sibuk hanya mencapai 56-73% CPU total (per-core 57-72%), jadi beban
  CPU **tidak** dipakai untuk memicu Waspada/Bahaya; pemicunya RAM.

Temuan alat: `[byte[]]::new()` menghasilkan halaman demand-zero sehingga 9 GB
teralokasi tanpa mengubah pemakaian fisik; `ramfill.ps1` menulis tiap blok
(`[Array]::Clear`) dan memakai `\Memory\Available MBytes` agar sama dengan
perhitungan `sysinfo`. Latch 3 sampel terlihat saat RAM naik cepat (77,8 → 92,9%
tetap `safe`), karena itu prosedur menuntut konfirmasi banner sebelum tangkapan.
Disk C: hanya 1,7 poin di atas ambang Waspada (16,7% vs 15%), harus dicek ulang
sebelum sesi.

Finalisasi dan publikasi (16 Nov):

- **Nama produk final: ArxseusMonitor** (D26). Yang berubah: `productName`,
  jendela (`title`, tooltip tray, `appName` ID/EN), `identifier`
  `com.arxseus.monitor`, folder settings `%APPDATA%\ArxseusMonitor`, crate
  `arxseus-monitor` + lib `arxseus_monitor_lib`, paket npm `arxseus-monitor`,
  `-ProcessName` default di `docs/soak.ps1`. Settings lama di
  `%APPDATA%\hwmonitor` sengaja tidak dimigrasikan. Binary hasil build tetap
  `arxseus-monitor.exe` (nama bin Cargo); installer memakai `productName`.
- **Uji responden dibatalkan** (D27): instrumen di `docs/responden.md` disimpan
  sebagai rancangan metode, bukan bukti. Kriteria DESIGN §5 nomor 3-4 tidak punya
  data — tulis apa adanya di laporan.
- Build rilis baru (16 Nov): `arxseus-monitor.exe` 4,55 MB,
  `ArxseusMonitor_0.1.0_x64_en-US.msi` 2,18 MiB,
  `ArxseusMonitor_0.1.0_x64-setup.exe` 1,49 MiB; startup 268-336 ms (3 kali,
  cache hangat) sampai jendela `class="Tauri Window"`, `title="ArxseusMonitor"`
  terlihat. Gate: `npm run lint` bersih, `cargo test` 34 lulus + 1 diabaikan.
- `README.md` ditulis untuk repo publik; tangkapan jendela ada di
  `docs/img/aplikasi.png` (PrintWindow, 562x836 px fisik @150%).
- Berkas bukti historis (`docs/black-box.md`, `a11y.md`, `soak-test.md`,
  `sensor-probe.md`) **tetap memakai nama lama** `hwmonitor`/`HWMonitor` karena
  itu catatan hasil pengukuran yang sudah terjadi. Jangan "merapikan" nama di
  dalamnya tanpa menjalankan ulang pengukurannya.
- Repo: `https://github.com/Kizter/ArxseusMonitor` (branch `main`).

Akurasi angka dan temuan jaringan (16 Nov, build rilis `12:05:38`):

- Alat: `docs/akurasi.ps1`, bukti `docs/akurasi.md`. Metode: `examples/snapshot.exe`
  (satu tick per 2 s) dipasangkan dengan `Get-Counter` 2 s; 2 sesi x 4 tick,
  sampel #1 dibuang → 5 pasangan. Acuan CPU `\Processor(_Total)\% Processor Time`,
  RAM `(total - Available MBytes)/total`, disk persen kosong C: dari `Get-Volume`.
  **Jangan** `Group-Object Timestamp` pada `CounterSamples` multi-counter —
  milidetik tiap counter bisa beda dan satu sampel terpecah dua grup; pakai urutan
  indeks (0=Processor Time, 1=Utility, 2=Available MBytes).
- Hasil beban tetap 8 thread: CPU median selisih 0,14 poin (0,3%), RAM 0,2%
  (maks 1,2%), disk 0,1% → lulus target < 5%. Idle: RAM/disk tetap 0,2%/0,1%,
  tetapi CPU median 28,1% relatif (2,80 poin absolut) karena kedua nilai kecil
  (5-13%) dan jendela 2 s tidak identik. Bukti akurasi CPU hanya sah pada beban
  tetap.
- Temuan: pada beban yang sama `% Processor Utility` = 82,8% sementara
  `% Processor Time` dan aplikasi ≈ 52%. Task Manager Win11 menampilkan Utility
  (berbobot turbo), `sysinfo` memakai Time. Jadi angka CPU aplikasi akan tampak
  lebih rendah dari Task Manager saat turbo — perbedaan definisi, bukan galat.
  Angka GUI Task Manager belum dibaca manual (tetap di `DESIGN.md` §13).
- `docs/jaringan.md`: proses aplikasi 0 koneksi, tetapi runtime WebView2
  (`msedgewebview2.exe`) membuka 2 koneksi TLS `Established` ke Microsoft
  (`40.99.x`, `52.98.x`:443, sertifikat `CN=outlook.com`) yang bertahan ≥ 21 s.
- Percobaan mematikannya **gagal**: 8 flag `--disable-*`/`--no-pings` terpasang
  (diverifikasi ada di command line proses browser) tetapi jumlah koneksi tetap 2;
  `--host-resolver-rules` dibuang WebView2. `additionalBrowserArgs` dikembalikan
  kosong dan build ulang — jangan menambah flag tanpa mengukur ulang jumlah
  koneksi. Konfigurasi bersih tetap membuka 2 koneksi.
- Klaim NFR §4 diubah jadi dua bagian: backend Rust + frontend 0 request;
  runtime WebView2 2 koneksi TLS ke Microsoft. Demo "benar-benar offline" dilakukan
  dengan adaptor jaringan dimatikan.
