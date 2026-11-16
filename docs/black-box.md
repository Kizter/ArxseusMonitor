# Uji black-box (Fase 4)

Bukti pengujian fungsional aplikasi lewat antarmuka nyata: jendela, tray, dan
berkas settings. Semua angka di bawah adalah hasil pengukuran pada mesin uji,
bukan perkiraan.

## Lingkungan uji

| Item | Nilai |
|---|---|
| OS | Windows 11 (build 26200) |
| Layar | 2560 × 1440, DPI 144 (skala 150%) |
| Build uji | `cargo build` debug + Vite dev server (`http://localhost:1420`), lalu diulang pada `npm run tauri build` (release, aset ter-bundle) |
| WebView2 | mode debug jarak jauh (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`) |
| Alat ukur | CDP `Runtime.evaluate` + `invoke`, UIA/MSAA, `GetWindowRect`/`IsWindowVisible`, log stdout |

Catatan metode:

- Jendela dibaca lewat Win32 pada proses `hwmonitor.exe` dengan kelas
  `Tauri Window`. `Process.MainWindowHandle` **tidak** dipakai karena bisa
  menunjuk ke jendela bantu (`Tao Thread Event Target`) sehingga hasilnya
  menyesatkan.
- Menu tray dibuka dengan `Shift+F10` pada ikon tray (flyout "Show Hidden
  Icons"), lalu item dipilih lewat UI Automation atau klik tetikus nyata.
- Penutupan jendela diuji dengan `WM_CLOSE` — jalur yang sama dengan tombol X.

## Hasil

### B1. Startup dan tampilan awal

| Yang diperiksa | Hasil |
|---|---|
| Judul jendela | `HWMonitor` |
| Banner | `Waspada` + alasan + saran (contoh: "CPU cukup sibuk…") |
| Baris metrik | 5: Beban CPU, Memori (RAM), Ruang penyimpanan, Suhu CPU, Suhu kartu grafis |
| Gulir horizontal | tidak ada (`scrollWidth == clientWidth`) |
| Kartu perkenalan | tampil saat `introSeen=false` |
| Footer | "Baru saja diperbarui" + "Sumber suhu: …" |

Bukti (nilai saat uji, mesin sibuk karena build berjalan):

```
Beban CPU: 37%, Waspada
Memori (RAM): 82,8%, Waspada
Ruang penyimpanan: 16,8%, Aman
Suhu CPU: —, Menunggu data
Suhu kartu grafis: 66 °C, Aman
```

Suhu CPU kosong adalah perilaku yang benar di mesin uji: LHM tidak berjalan dan
ACPI menolak tanpa admin (`0x80041003`). Tidak ada angka 0 °C palsu.

### B2. Tutup jendela (tombol X / `WM_CLOSE`)

| Yang diperiksa | Hasil |
|---|---|
| `IsWindowVisible` setelah WM_CLOSE | `false` |
| Proses setelah WM_CLOSE | tetap hidup, 1 proses, PID sama |
| Poller | tetap berjalan (snapshot berikutnya tetap datang lewat `get_snapshot`) |

```
VISIBLE_AWAL=True  VISIBLE_SETELAH_CLOSE=False
PROSES_MASIH_ADA=True  JUMLAH_PROSES=1  PID=7332
```

### B3. Posisi dan ukuran jendela

| Yang diperiksa | Hasil | Target (`DESIGN.md` §8) |
|---|---|---|
| Ukuran luar | 562 × 836 px fisik | 360 × 520 CSS × 1,5 + bingkai |
| Margin kanan | 18 px | 12 CSS × 1,5 = 18 px ✔ |
| Margin bawah | 84 px | 56 CSS × 1,5 = 84 px ✔ |
| Posisi | (1980, 520) | pojok kanan bawah |

### B4. Tooltip tray

Tiga baris, dipisah baris baru (`\n`), bahasa mengikuti setelan:

```
HWMonitor Aman — Semua bagian yang terpantau dalam batas aman.
CPU 6,6% · RAM 69,6%
Suhu CPU tidak terdeteksi
```

Baris 3 menjadi `Suhu CPU tidak terdeteksi` saat sensor suhu tidak ada — bukan
angka palsu.

### B5. Menu tray

| Item | Aksi | Hasil |
|---|---|---|
| Buka | tampilkan jendela | dari kondisi tersembunyi → `IsWindowVisible=true`, PID sama |
| Pengaturan | tampilkan jendela + buka setelan | jendela tampak, judul header `Pengaturan`, 3 `select`, tombol `Kembali` |
| Keluar | keluar aplikasi | proses hilang (diperiksa 3 detik setelah klik), log `[app] keluar dari menu tray` |

Catatan: log di atas hanya terlihat pada build debug. Build release memakai
`windows_subsystem = "windows"` sehingga tidak punya konsol; bukti keluarnya
proses di release adalah hilangnya `hwmonitor.exe` dari daftar proses.

Urutan item terbaca MSAA: `Buka / Pengaturan / Keluar`.

### B6. Klik kiri ikon tray

| Kondisi awal | Setelah klik | Hasil |
|---|---|---|
| jendela tersembunyi | tampil | `VISIBLE_SETELAH_KLIK_1=True` |
| jendela tampil | tersembunyi | `VISIBLE_SETELAH_KLIK_2=False` |

### B7. Ganti bahasa (ID → EN → ID)

| Yang diperiksa | Hasil |
|---|---|
| Judul setelan | `Settings` |
| Tombol kembali | `Back` |
| Label formulir | `Language`, `Refresh rate`, `CPU temperature source`, `Start without a window` |
| `document.documentElement.lang` | `en` |
| Kembali ke ID | header `HWMonitor`, banner + tooltip Indonesia lagi, `get_settings().language = "id"` |

### B8. Interval 5 detik benar-benar mengubah irama poller

| Yang diperiksa | Hasil |
|---|---|
| `snapshot.intervalMs` | `5000` |
| Jarak antar `snapshot.at` | 5026 ms (target 5000 ms) |

### B9. Sumber suhu dimatikan (setelan `off`)

| Yang diperiksa | Hasil |
|---|---|
| `metrics.cpuTemp` | `value: null`, `level: "unknown"`, `reasonKey: "temp.cpu.unavailable"` |
| Baris suhu CPU | `Suhu CPU: —, Menunggu data` |
| Angka 0 °C di layar | tidak ada (`/(^|[^\d])0\s*°C/` tidak cocok) |

### B10. Sensor hilang total (`--simulate-missing-temp`)

Mode demo sidang: semua sumber suhu dimatikan tanpa mengubah berkas settings.

| Yang diperiksa | Hasil |
|---|---|
| Log startup | `[app] mode simulasi: semua sumber suhu dimatikan` |
| `settings.temperatureSource` | tetap `auto` (berkas tidak diubah) |
| `metrics.cpuTemp` / `gpuTemp` | `null` / `null`, `level: "unknown"` |
| Dua baris suhu | `Suhu CPU: —, Menunggu data`, `Suhu kartu grafis: —, Menunggu data` |
| Metrik lain | tetap terbaca (CPU 7,8%, RAM 69,3%, disk 17,5%) |
| `overall` | `safe` — status keseluruhan tidak ikut "unknown" hanya karena suhu |
| Footer | `Sumber suhu: tidak tersedia` |
| Angka 0 °C | tidak ada |

Konsekuensi yang dicatat: karena NVML dimatikan seluruhnya, nama GPU (mis. "NVIDIA …")
juga tidak tampil — baris GPU hanya menyebut suhu tidak tersedia. Ini disengaja
(D25) demi satu jalur kode yang sederhana saat demo.

Mode ini juga berlaku saat aplikasi sudah berjalan: menjalankan
`hwmonitor.exe --simulate-missing-temp` ketika instance pertama hidup akan
memaksa instance tersebut masuk mode simulasi (log: `[app] instance kedua meminta
mode simulasi: semua sumber suhu dimatikan`) dan memunculkan jendelanya, lalu
proses kedua keluar. Tanpa jalur ini flag akan diam-diam diabaikan pada skenario
demo yang paling mungkin: aplikasi sudah lama di tray, presenter baru menjalankan
shortcut.

### B11. Berkas settings rusak

Berkas ditulis `{ ini bukan json` (UTF-8 tanpa BOM), lalu aplikasi dijalankan.

| Yang diperiksa | Hasil |
|---|---|
| Log | `[settings] …\settings.json tidak bisa dibaca sebagai JSON (key must be a string at line 1 column 3); memakai default` |
| `get_settings` | `id` / 2 s / `auto` / `startMinimized=false` / `introSeen=false` |
| Aplikasi | tetap jalan, snapshot hidup (`overall: safe`), intro tampil |
| Berkas setelah uji | dikembalikan ke isi semula |

### B12. Satu instance

Sudah diuji pada Fase 3 dan diulang pada build yang sama: instance kedua keluar
`exit 0` < 1 s, jumlah proses tetap 1 dengan PID sama, jendela instance pertama
dimunculkan. Log: `[app] instance kedua terdeteksi: tampilkan jendela yang ada`.

## Verifikasi ulang pada build release (15 Nov)

Seluruh skenario di atas diulang pada `target\release\hwmonitor.exe` hasil
`npm run tauri build` (aset frontend ter-bundle, tanpa dev server).

| Yang diperiksa | Hasil |
|---|---|
| Ukuran binary | 4,55 MB |
| Installer MSI | 2,18 MiB |
| Installer NSIS (`.exe`) | 1,49 MiB |
| Waktu sampai jendela tampak | 852 ms setelah proses dijalankan |
| Waktu sampai WebView2 siap menerima perintah | 842 ms |
| Tampilan awal | 5 baris metrik, tanpa overflow; `Suhu CPU: —, Menunggu data`; GPU 46 °C dari NVML |
| Intro | tampil saat `introSeen=false`, hilang setelah "Mengerti", tersimpan `true` |
| ID → EN → ID | `Settings` / `Back` / label EN / `lang=en`, lalu kembali Indonesia |
| Interval 5 s | `intervalMs=5000`, jarak sampel 5012 ms |
| Sumber suhu `off` | `cpuTemp=null` (`Suhu CPU: —, Menunggu data`), GPU tetap 47 °C dari NVML — sesuai niat setelan yang hanya mematikan suhu CPU |
| Simulasi `--simulate-missing-temp` | `cpuTemp` dan `gpuTemp` dua-duanya `null`, dua baris "Menunggu data", `settings` tetap `auto`, `overall=safe` |
| Posisi jendela | (1980, 520), margin kanan 18 px, bawah 84 px — identik dengan build debug |
| WM_CLOSE | jendela tersembunyi, proses hidup, 1 PID |

Angka startup diukur dengan `Stopwatch` dari `Start-Process` sampai (a) target
CDP tersedia dan (b) jendela kelas `Tauri Window` terlihat — bukan dari dalam
aplikasi, jadi sudah termasuk waktu muat WebView2.

## Verifikasi ulang setelah perapian aksesibilitas (15 Nov, build final)

Build yang sama dengan yang akan diserahkan (`npm run tauri build` setelah
perubahan `aria-current`, landmark `main`, kontras kontrol formulir, dan penjaga
fokus). Skenario yang berubah selectornya diuji ulang utuh; sisanya hanya
dipastikan tidak ikut berubah.

| Yang diperiksa | Hasil |
|---|---|
| Ukuran binary / MSI / NSIS | 4,55 MB / 2,18 MiB / 1,49 MiB (tidak berubah) |
| Startup (cache hangat) | jendela terlihat 636 ms; CDP siap 586 ms (sebelumnya 852/842 ms pada cache dingin) |
| Baris metrik | 5 baris `button[aria-controls="why-panel"]`, tanpa `aria-pressed`, satu `aria-current="true"` pada baris terburuk |
| Nama aksesibel baris | `Beban CPU: 12,3%, 16 thread, Aman` (detail ikut terbaca) |
| Panel "Kenapa?" | region bernama (`aria-labelledby="why-heading"` → `Kenapa?`) |
| Teks terkecil / overflow | 14 px / tanpa overflow horizontal |
| Gulir PageDown asli (tombol dikirim ke jendela) | `scrollTop 0 → 396` dari maks 397, fokus tetap di baris |
| Footer | menempel di dasar jendela, y 413–465 dari 465 px |
| Intro | tampil saat `introSeen=false`, "Mengerti" → tersimpan `true`, fokus pindah ke `#status-banner` |
| Buka dari tray, fokus kosong | fokus jatuh ke `<h1>`; bila baris masih terfokus, fokus tidak dirampas |
| ID → EN → ID | `Settings` / `Back` / label EN / `lang=en`, kembali ke Indonesia tanpa sisa teks EN |
| Interval 5 s | `intervalMs=5000`, jarak sampel 5014 ms |
| Sumber suhu `off` | `cpuTemp=null` (`Suhu CPU: —, Menunggu data`), GPU tetap 47 °C dari NVML — setelan ini memang hanya mematikan suhu CPU |
| `--simulate-missing-temp` lewat instance kedua | instance kedua keluar `exit 0` dalam 14 ms; instance pertama: `cpuTemp` dan `gpuTemp` `null`, dua baris "Menunggu data", `settings` tetap `auto`, `overall=safe`, tanpa angka 0 °C |
| Satu instance | 1 PID sesudah instance kedua, jendela instance pertama dimunculkan |

Perbandingan kontras dan detail struktur ARIA ada di `docs/a11y.md` §2; pemakaian
sumber daya saat hanya tray aktif ada di `docs/soak-test.md`.

## Kesimpulan

Semua skenario black-box pada `DESIGN.md` §10 butir 1 lulus pada mesin uji:
buka/tutup, tray (tooltip, menu, klik kiri), ganti bahasa, settings tersimpan
dan rusak, serta sensor hilang (dua jalur: setelan `off` dan flag simulasi).
Tidak ditemukan angka palsu, panic, atau proses yang tertinggal.

Catatan alat uji (bukan bug aplikasi): saat flyout tray dibuka berulang kali
dengan `Shift+F10` tanpa fokus yang benar, Windows kadang menyisakan jendela
menu `#32768` lama; klik pada menu itu tidak diteruskan ke aplikasi. Setelah
flyout ditutup dan dibuka ulang, item "Keluar" bekerja normal (diuji dua kali).
