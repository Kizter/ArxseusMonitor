# HWMONITOR — Desain Aplikasi Monitor Kesehatan Hardware

> Dokumen desain untuk gelar karya. Divalidasi melalui sesi brainstorming (Understanding Lock: 13 November 2026).
> Status: **tervalidasi** — siap diturunkan menjadi rencana implementasi.

---

## 1. Ringkasan

Aplikasi desktop Windows yang memantau kesehatan hardware dan **menerjemahkan metrik teknis menjadi status yang dipahami orang awam**. Dua permukaan: ikon system tray (jawaban cepat) dan jendela detail (penjelasan + tindakan).

- **Stack:** Tauri v2, Rust, React + TypeScript + Vite + Tailwind CSS
- **Platform:** Windows 10/11, satu mesin, satu pengguna, offline penuh
- **Deadline aplikasi:** 4 Desember 2026 (laporan & sidang menyusul)
- **Metode:** Prototype / Iterative

### Rumusan masalah

Pengguna awam melihat angka hardware (78 °C, RAM 93%) tetapi tidak tahu apakah angka itu berbahaya atau tidak. Aplikasi monitoring yang ada menampilkan data mentah untuk pengguna teknis, bukan keputusan untuk pengguna awam.

### Klaim yang harus dibuktikan

Status terjemahan (aman / waspada / bahaya + saran tindakan) membuat pengguna awam **lebih cepat dan lebih tepat** menilai kondisi komputernya dibandingkan tampilan angka mentah.

---

## 2. Target user & responden

| Peran | Siapa |
|---|---|
| Pengguna | Pengguna komputer non-teknis |
| Responden uji | ±8 mahasiswa non-IT + ±5 keluarga non-teknis (±13 orang) |
| Bahasa pengujian | Indonesia (EN tersedia di aplikasi, tidak diuji) |

---

## 3. Batasan & non-goals

**Batasan:** Windows 10/11 · tanpa admin · tanpa driver kernel · tanpa jaringan · tanpa database · satu pengguna · 22 hari kalender.

**Non-goals (eksplisit):** fan control · overclocking · benchmark · overlay game · histori jangka panjang/database · cloud/akun/login · auto-update · mobile/Linux/macOS · driver kernel sendiri · autostart Windows · tema terang · lebih dari 2 bahasa.

---

## 4. Non-functional requirements

| Aspek | Target |
|---|---|
| CPU aplikasi (idle) | < 2% pada mesin 4-core |
| RAM aplikasi | < 150 MB |
| Ukuran installer | < 15 MB |
| Waktu buka | < 2 detik sampai tray siap |
| Interval update | 2 detik (sysinfo), 5 detik (suhu/WMI); dapat diubah 1/2/5 detik |
| Hak akses | Tanpa elevasi admin |
| Jaringan | 0 request dari kode aplikasi (backend Rust + frontend). Runtime WebView2 tetap membuka 2 koneksi TLS ke Microsoft saat start — di luar kendali konfigurasi, terukur di `docs/jaringan.md` |
| Penyimpanan | 1 file settings JSON (`tauri-plugin-store`) |
| Reliabilitas | Demo 30 menit tanpa crash; sensor hilang tidak menjatuhkan aplikasi |
| Instance | Single instance (D24) |

---

## 5. Success criteria (terukur)

| # | Kriteria | Target |
|---|---|---|
| 1 | Fungsional | CPU, RAM, disk, suhu (best-effort), status kesehatan tampil; tray + jendela detail; 30 menit tanpa crash |
| 2 | Akurasi | CPU/RAM/disk selisih < 5% vs Task Manager/HWiNFO, dicatat per-metrik |
| 3 | Usability | SUS ≥ 68 |
| 4 | Pemahaman (klaim inti) | ≥ 80% responden menjawab benar "apakah PC ini sehat?" dalam < 10 detik, dibandingkan tampilan angka mentah |
| 5 | Transparansi | Sensor tidak tersedia → pesan eksplisit, bukan 0 °C |
| 6 | Akademik | Tabel black-box, rekap SUS per responden, bukti screenshot |

Catatan (D27): kriteria 3 (SUS) dan 4 (pemahaman) **tidak diukur** karena uji
responden dilewati. Yang dilaporkan: black-box (`docs/black-box.md`), soak
(`docs/soak-test.md`), aksesibilitas (`docs/a11y.md`), dan akurasi vs Task
Manager. Instrumen uji pemahaman tetap tersedia di `docs/responden.md` bila
nanti diperlukan.

Catatan kriteria 2 (16 Nov, `docs/akurasi.md`): terhadap counter Windows yang
jadi basis perhitungan, median selisih **CPU 0,3%** (beban tetap), **RAM 0,2%**,
**disk 0,1%** — lulus target < 5%. Dua hal yang harus ditulis di laporan:

1. Acuan diotomasi adalah counter Windows, bukan GUI Task Manager. Pada beban
   yang sama, `% Processor Utility` (basis angka CPU Task Manager) terukur 82,8%
   saat `% Processor Time` dan aplikasi ~52%. Aplikasi memakai basis `Time` yang
   sama dengan `sysinfo`; jadi pada CPU yang ter-turbo, angka aplikasi tampak
   lebih rendah daripada Task Manager karena perbedaan definisi, bukan galat.
2. Pada idle selisih **relatif** membesar (median 28,1%) karena kedua nilai
   kecil (5-13%) dan jendela 2 detik tidak identik; selisih absolutnya median
   2,80 poin. Bukti akurasi CPU karena itu diambil pada beban tetap.

---

## 6. Arsitektur

Satu arah data, empat lapisan:

```
[ Sensor Providers ] → [ Aggregator ] → [ Health Engine ] → [ Tray + UI ]
```

- **Rust sebagai sumber kebenaran.** Background task polling → agregasi → hitung status → `emit` satu payload bertipe ke frontend. Tray dikelola Rust, tetap akurat walau jendela ditutup.
- **Frontend hanya merender.** Tidak ada polling dari sisi webview (menghindari throttle WebView2 saat hidden).
- **Provider boleh gagal sendiri-sendiri.** `PartialMetrics` dengan semua field `Option`; yang gagal ditandai "tidak tersedia", bukan error.

### Provider sensor

| Provider | Sumber | Data | Catatan |
|---|---|---|---|
| `SysinfoProvider` | crate `sysinfo` | CPU load, RAM, disk, network, uptime | wajib, selalu tersedia |
| `ThermalWmiProvider` | WMI `MSAcpi_ThermalZoneTemperature` | suhu sistem | best-effort, tanpa admin |
| `NvmlProvider` | crate `nvml-wrapper` | suhu & VRAM GPU | hanya mesin NVIDIA |
| `LhmWmiProvider` | WMI `root\LibreHardwareMonitor` | suhu core, fan, voltase | hanya jika LHM berjalan |

Suhu core CPU tanpa driver **tidak dapat diandalkan** di Windows. Karena itu strategi suhu bersifat hibrida: aplikasi tetap berfungsi penuh tanpa LHM, dan diperkaya bila LHM tersedia.

### Struktur folder

```
ArxseusMonitor/
├─ DESIGN.md
├─ AGENTS.md
├─ package.json
├─ src/                        # React + TS + Vite + Tailwind
│  ├─ App.tsx
│  ├─ components/              # HealthBadge, MetricCard, StatusExplanation, Settings
│  └─ lib/                     # ipc.ts, types.ts, i18n.ts
└─ src-tauri/
   ├─ Cargo.toml
   ├─ tauri.conf.json
   ├─ capabilities/default.json
   └─ src/
      ├─ main.rs               # bootstrap
      ├─ lib.rs                # builder, plugin single instance, tray, commands
      ├─ monitor/
      │  ├─ poller.rs
      │  └─ model.rs
      ├─ sensors/
      │  ├─ mod.rs             # trait SensorProvider
      │  ├─ sysinfo_provider.rs
      │  ├─ thermal_wmi.rs
      │  ├─ nvml.rs
      │  └─ lhm_wmi.rs
      ├─ health.rs             # ambang batas + status
      └─ settings.rs
```

---

## 7. Model data & Health Engine

```ts
type Level = "safe" | "warn" | "danger" | "unknown";
type Metric = { value: number | null; unit: string; level: Level; reasonKey: string };
type Snapshot = {
  at: number;
  overall: Level;
  metrics: Record<"cpu" | "ram" | "disk" | "cpuTemp" | "gpuTemp", Metric>;
  device: {
    cpuBrand: string | null; cpuCores: number | null; gpuName: string | null;
    ramUsedBytes: number | null; ramTotalBytes: number | null; uptimeSecs: number | null;
  };
  disks: { name: string; mount: string; totalBytes: number; freeBytes: number; freePct: number }[];
  cpuTempSource: "lhm" | "acpi" | "nvml" | null;
  intervalMs: number;
};
```

Baterai dikeluarkan dari MVP (D19). Kunci `reasonKey` yang mungkin dikirim:

| Metrik | Aman | Waspada | Bahaya | Tidak tersedia |
|---|---|---|---|---|
| cpu | `cpu.ok` | `cpu.high` | `cpu.critical` | `cpu.unavailable` |
| ram | `ram.ok` | `ram.high` | `ram.critical` | `ram.unavailable` |
| disk | `disk.ok` | `disk.low` | `disk.critical` | `disk.unavailable` |
| cpuTemp | `temp.cpu.ok` | `temp.cpu.warm` | `temp.cpu.critical` | `temp.cpu.unavailable` |
| gpuTemp | `temp.gpu.ok` | `temp.gpu.warm` | `temp.gpu.critical` | `temp.gpu.unavailable` |

Disk mewakili partisi dengan ruang kosong **paling sedikit**; `disks` membawa
rincian per partisi untuk kartu detail.

### Ambang batas

| Metrik | Aman | Waspada | Bahaya | Dasar |
|---|---|---|---|---|
| Beban CPU | < 70% | 70–90% | > 90% | praktik umum monitoring |
| RAM | < 80% | 80–92% | > 92% | sebelum paging berat |
| Ruang disk | > 15% kosong | 8–15% | < 8% | Windows bermasalah < 10% |
| Suhu CPU | < 75 °C | 75–90 °C | > 90 °C | mendekati Tjmax CPU modern |
| Suhu GPU | < 80 °C | 80–90 °C | > 90 °C | batas aman GPU konsumen |
| ~~Baterai~~ | — | — | — | Dikeluarkan dari MVP (D19) |

### Aturan

1. **Sustained, bukan sesaat.** Status berubah setelah kondisi bertahan 3 sampel (±6 detik). Mencegah ikon tray berkedip.
2. **Overall = level terburuk.** Metrik `unknown` tidak dihitung, tetapi tetap ditampilkan dengan tanda `?`.
3. **Alasan sebagai kunci i18n.** Rust mengirim `reasonKey` (`"ram.high"`, `"temp.cpu.critical"`); frontend menerjemahkan ke kalimat + saran tindakan.
4. **Tanpa angka palsu.** Sensor tidak tersedia → "Tidak terdeteksi di perangkat ini".

---

## 8. UI/UX

### Sistem visual (adaptasi GitHub Primer, mode gelap)

- Kanvas `#0d1117`, permukaan `#161b22`, border rambut `#30363d`, teks `#e6edf3`, teks sekunder `#8b949e`.
- Aksen interaktif biru `#58a6ff`; tombol utama biru `#1f6feb` (hover `#1a5fd0` — lebih gelap, bukan lebih terang, agar teks putih tetap ≥ 4,5:1).
- Batas kontrol formulir (`<select>`) memakai `#6e7681` (4,12:1 di atas kanvas). Border rambut `#30363d` hanya 1,57:1, di bawah ambang 3:1 untuk komponen antarmuka (WCAG 2.2 SC 1.4.11).
- Hijau `#3fb950`, kuning `#d29922`, merah `#f85149` **hanya** untuk status kesehatan, tidak untuk tombol atau tautan.
- Tipografi system-ui; 14 px untuk isi, 16 px untuk judul panel; monospace untuk angka dan nilai teknis (`C:\`, sumber suhu).
- Radius 6 px, border 1 px, tanpa bayangan dekoratif; jarak kelipatan 4 px.
- Metrik disusun sebagai baris padat dalam satu kotak bergaris; baris terpilih ditandai garis biru 3 px di tepi kiri. Pil status memakai latar tipis + teks berwarna (kontras AA).
- Fokus keyboard selalu terlihat: cincin biru `#58a6ff` 2 px (outline untuk tombol/baris, ring untuk kontrol formulir). Pada baris metrik cincin digambar ke dalam (`outline-offset:-2px`) karena induknya `overflow:hidden`.
- Gerak 80 ms untuk hover/fokus, 200 ms untuk panel; tanpa animasi saat muat.

### Tray (permukaan utama)

- Ikon: hijau / kuning / merah / abu-abu (`?` jika semua sensor gagal). Warna **selalu** disertai teks.
- Tooltip 3 baris: status + alasan, suhu CPU, RAM%.
- Klik kiri → jendela menempel di tray (`tauri-plugin-positioner`). Klik kanan → Buka / Pengaturan / Keluar.

### Jendela detail

Ukuran ±360×520, satu kolom, dekorasi jendela standar:

1. **Banner status** — warna + satu kalimat + satu saran tindakan.
2. **Baris metrik** — lima baris padat: label awam, angka monospace, pil status, bar tipis. Grup diberi heading tak terlihat "Ringkasan metrik" agar bisa dijangkau navigasi per heading.
3. **Panel "Kenapa?"** — penjelasan awam; di sinilah materi Arsitektur Komputer (RAM, disk, panas) disampaikan.
4. **Footer** — waktu update terakhir + sumber suhu (LHM / ACPI / tidak tersedia).

### Alur & state

- Tutup jendela = sembunyi ke tray; keluar hanya lewat menu.
- First run: satu layar penjelasan singkat (bukan wizard).
- Pengaturan: bahasa (ID/EN), interval (1/2/5 s), sumber suhu (Otomatis/ACPI/LHM/Nonaktif), mulai terminimalkan.
- Empty state dirancang: baris "Tidak terdeteksi" + penjelasan.

### Aksesibilitas

Kontras AA · fokus keyboard terlihat · status tidak hanya warna · teks ≥ 14 px · animasi minimal.

Detail yang dipegang implementasi (lihat `docs/a11y.md` untuk bukti ukur):

- Landmark `main`; heading `h1` per tampilan + heading tak terlihat untuk grup metrik.
- Baris metrik memakai `aria-current` (bukan `aria-pressed`, karena pilihan tidak bisa dibatalkan) dan `aria-controls` ke panel "Kenapa?"; detail angka ikut masuk nama aksesibel.
- `role="status"` pada banner (tanpa `aria-live` ganda); galat simpan memakai `role="alert"` yang dipasang ulang tiap kegagalan.
- Catatan di bawah kontrol formulir ditautkan lewat `aria-describedby`; opsi bahasa membawa `lang` masing-masing.
- Fokus tidak hilang saat kartu perkenalan ditutup atau saat jendela dimunculkan dari tray.

---

## 9. Error handling & edge cases

| Skenario | Perilaku |
|---|---|
| WMI lambat/error/timeout | Anggaran 300 ms per provider; hasil yang lebih lambat tetap dipakai bila ada, tetapi dicatat sebagai kegagalan (D21). Error → metrik "tidak tersedia" |
| Provider gagal 5x berturut | Circuit breaker: nonaktif 5 menit, lalu coba lagi |
| LHM tidak berjalan | `LhmWmiProvider` gagal senyap → fallback ACPI/NVML |
| Butuh admin | Tidak pernah minta elevasi; fitur dianggap tidak ada |
| Sleep/resume | Gap > 10 s → reset penghitung sustained, ambil sampel baru |
| Dua instance | Instance kedua memunculkan jendela yang ada, lalu keluar |
| Tray di monitor kedua / DPI beda | Posisi jendela dihitung ulang saat diklik |
| Berjalan berhari-hari | Tanpa histori → tidak ada pertumbuhan memori |
| Aplikasi jadi beban | sysinfo 2 s, suhu 5 s, di luar thread UI |

---

## 10. Strategi pengujian

1. **Black-box** — tabel skenario: buka/tutup, tray, ganti bahasa, setting tersimpan, sensor hilang.
2. **Akurasi** — 5 sampel per metrik vs Task Manager & HWiNFO, catat selisih % (target < 5%).
3. **Uji pemahaman (klaim inti)** — 2 kondisi: angka mentah vs aplikasi; ukur ketepatan + waktu.
4. **SUS** — 13 responden, 10 pertanyaan standar.
5. **Fault injection** — flag `--simulate-missing-temp` untuk demo "sensor hilang" saat sidang.
6. **Soak test** — 30 menit; catat CPU & RAM aplikasi sendiri.
7. **Pilot** — 2 orang sebelum uji penuh, untuk menangkap masalah pemahaman lebih awal.

Instrumen siap pakai (skenario tugas, stimulus, formulir SUS/akurasi, etika,
rekap) ada di `docs/responden.md`. Alat stimulus: `docs/ramfill.ps1`
(memicu Waspada/Bahaya lewat RAM), `docs/kartu-angka.html` (pembanding angka
mentah dengan nilai yang sama), `docs/load.ps1` (beban CPU, tidak dipakai di
mesin uji karena dibatasi ±60–70%).

---

## 11. Risiko & mitigasi

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Suhu core tidak terbaca di laptop demo | Konsep melemah | LHM opsional + `--simulate-missing-temp` + narasi jujur |
| WMI hang / butuh admin | App freeze | Timeout + circuit breaker + tanpa elevasi |
| Toolchain belum siap (Rust, MSVC, WebView2) | Makan 1 hari | Pre-flight di Hari 1 |
| 13 responden dalam 3 minggu | Data SUS kurang | Pilot minggu ke-2, uji penuh minggu ke-3 |
| SUS < 68 | Target meleset | Perbaikan hasil pilot sebelum uji penuh |
| Waktu habis | Fitur setengah | Urutan prioritas: sensor → status → tray → UI → bilingual → pengujian |

---

## 12. Decision Log

| # | Keputusan | Alternatif ditolak | Alasan |
|---|---|---|---|
| D1 | Dinilai sebagai gelar karya | Produk pasar | Jawaban user |
| D2 | Tauri v2 + Rust + React/TS/Vite/Tailwind, Windows-only | Electron, .NET | Ringan, permintaan user |
| D3 | 8 mahasiswa non-IT + 5 keluarga non-teknis | Satu kelompok | Klaim "awam" defensible |
| D4 | Mode 3 minggu (deadline 4 Des) | — | Jawaban user |
| D5 | Prototype/Iterative + SUS + black-box + akurasi + pemahaman | Waterfall, RAD | Paling cocok untuk IMK |
| D6 | 4 Des = aplikasi; laporan menyusul | Deadline laporan | Jawaban user |
| D7 | Bilingual ID + EN via dictionary | ID saja | Permintaan user |
| D8 | EN tersedia, pengujian dalam ID | Uji dua bahasa | Data bersih |
| D9 | Rust poller + event push | Frontend polling, sidecar | Tray andal, satu sumber logika |
| D10 | Hibrida: sensor sendiri + LHM opsional | Sidecar penuh, driver sendiri | Akurasi tanpa melanggar NFR |
| D11 | Satu `Snapshot`, 3 level + `unknown` | 5 level, angka mentah | Beban kognitif awam |
| D12 | Status berubah setelah 3 sampel bertahan | Realtime | Mencegah ikon berkedip |
| D13 | Tray utama + jendela detail | Dashboard-only | Nilai < 3 detik |
| D14 | Warna selalu + teks | Warna saja | Aksesibilitas |
| D15 | Timeout 300 ms + circuit breaker per provider | Satu sensor gagal = app gagal | Reliabilitas |
| D16 | sysinfo 2 s, suhu 5 s | Semua 1 s | WMI mahal |
| D17 | Ambang batas di §7 | Angka karangan | Dapat dipertahankan di sidang |
| D18 | Suhu CPU best-effort + LHM dipasang di mesin demo | Paksa driver sendiri, ACPI dengan admin | Hasil probe: ACPI `0x80041003`, LHM opsional terbukti jalurnya |
| D19 | Baterai dikeluarkan dari MVP | Menambah dependensi baterai | Mesin uji desktop tanpa baterai; metrik tak teruji = risiko tak perlu |
| D20 | Settings JSON polos di `%APPDATA%` | `tauri-plugin-store` | Satu dependensi lebih sedikit; format cukup 4 field |
| D21 | Anggaran waktu 300 ms bersifat "lembut" | Bunuh thread provider | Koneksi WMI tidak `Send`; membunuh thread akan membocorkan COM. Thread poller di luar UI, jadi UI tetap responsif |
| D22 | Jendela detail ditempatkan manual di pojok kanan bawah | `tauri-plugin-positioner` | Perilaku yang sama (menempel ke tepi tray) tanpa satu dependensi tambahan; margin 12 px dari kanan dan 56 px dari bawah dikalikan `scale_factor` |
| D23 | Label menu & tooltip tray dikirim dari React | Teks tray ditulis di Rust | Menjaga satu sumber terjemahan; Rust hanya mengirim `level` dan `reasonKey` |
| D24 | Single instance memakai `tauri-plugin-single-instance` | Mutex Win32 buatan sendiri | Plugin resmi Tauri v2: mutex bernama + saluran IPC sudah diuji, cukup 3 baris; instance kedua memunculkan jendela yang ada lalu keluar (exit 0). Mutex manual berarti menulis ulang IPC dan menangani kasus balapan start |
| D25 | `--simulate-missing-temp` mematikan semua provider suhu (LHM, ACPI, NVML) saat runtime, tanpa menulis berkas settings; flag yang dibawa instance kedua juga mengaktifkan mode ini pada instance yang sudah berjalan | Mengubah setelan `temperatureSource` ke `off` | Setelan pengguna tidak boleh berubah karena flag demo. Karena NVML dimatikan seluruhnya, nama GPU juga tidak tampil — itu konsekuensi yang diterima dan dicatat di `docs/black-box.md` |
| D26 | Nama produk **ArxseusMonitor**: `productName` = `ArxseusMonitor`, `identifier` = `com.arxseus.monitor`, folder settings `%APPDATA%\ArxseusMonitor`, binary `arxseus-monitor.exe` | Tetap `hwmonitor` / `com.hwmonitor.app` | Nama final ditetapkan pemilik proyek (16 Nov) sebelum repo `Kizter/ArxseusMonitor` dipublikasikan. Settings lama di `%APPDATA%\hwmonitor` tidak dimigrasikan karena belum ada pengguna; berkas bukti historis di `docs/` tetap memakai nama lama dan tidak diubah |
| D27 | Uji responden (SUS + uji pemahaman 13 orang) **tidak dijalankan**; evaluasi memakai black-box, soak test, audit aksesibilitas, dan akurasi vs Task Manager | Menjalankan 8 mahasiswa + 5 keluarga | Keputusan pemilik proyek (16 Nov). Konsekuensi dicatat jujur: kriteria §5 nomor 3 dan 4 tidak punya data. Instrumen tetap disimpan di `docs/responden.md` sebagai rancangan metode, bukan sebagai bukti |
| D28 | Akurasi dibuktikan terhadap counter Windows `% Processor Time` / `Available MBytes` / `Get-Volume`, bukan GUI Task Manager; basis CPU = `% Processor Time` (sama dengan `sysinfo`) | Membandingkan dengan angka CPU Task Manager apa adanya | GUI tidak bisa diotomasi, dan Task Manager Win11 memakai `% Processor Utility` (berbobot turbo). Pada beban yang sama Utility 82,8% vs Time/aplikasi ~52%; membandingkan keduanya akan terlihat seperti galat padahal beda definisi metrik. Bukti: `docs/akurasi.md` |
| D29 | NFR "jaringan: 0 request" ditulis dua bagian: backend Rust + frontend 0 request, runtime WebView2 2 koneksi TLS ke Microsoft saat start. `additionalBrowserArgs` tidak dipakai | Menulis "0 request" apa adanya; memaksa flag Chromium | Pengukuran (`docs/jaringan.md`) menunjukkan 8 flag `--disable-*` terpasang tetapi jumlah koneksi tetap 2, dan `--host-resolver-rules` dibuang WebView2. Klaim harus sesuai yang terukur; demo offline dilakukan dengan adaptor jaringan dimatikan |

---

## 13. Pertanyaan terbuka

1. Verifikasi jalur LHM di mesin demo (butuh pemasangan LHM + WMI provider aktif, serta admin sekali saat memasang) — `docs/sensor-probe.md` masih menandainya belum terverifikasi.
2. Identitas visual selain nama: ikon aplikasi/tray masih memakai ikon bawaan Tauri; belum ada keputusan untuk membuat ikon sendiri.
3. Publikasi: repo `Kizter/ArxseusMonitor` berisi kode + dokumen bukti; lisensi belum ditentukan.
4. Angka CPU Task Manager perlu dibaca manual sekali saat demo untuk mengonfirmasi bahwa selisihnya memang karena `% Processor Utility` (bukan `% Processor Time`) — `docs/akurasi.md`.
5. Koneksi TLS bawaan runtime WebView2 (2 buah ke Microsoft) belum bisa dimatikan dari konfigurasi aplikasi; bila klaim "tanpa jaringan" dituntut mutlak, demo dilakukan dengan adaptor jaringan dimatikan (`docs/jaringan.md`).

