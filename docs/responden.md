# Instrumen uji responden (Fase 5)

Alat dan prosedur untuk menguji klaim inti: **status terjemahan (aman/waspada/
bahaya + saran) membuat orang awam lebih cepat dan lebih tepat menilai kondisi
komputer daripada angka mentah** (DESIGN.md §1), plus SUS dan akurasi angka.

Semua ambang diambil dari DESIGN.md §7; semua target dari DESIGN.md §5 dan §10.
Instrumen ini belum dipakai pada responden. Kalibrasi stimulus sudah diukur pada
mesin uji (16 Nov) dan hasilnya tercatat di §5.4.

> **Status (D27, 16 Nov): tidak dijalankan.** Uji responden dilewati atas
> keputusan pemilik proyek, jadi berkas ini adalah rancangan metode, bukan bukti.
> Kriteria SUS ≥ 68 dan pemahaman ≥ 80% karena itu tidak punya data di laporan.
> Nama produk sudah final (D26): instruksi di bawah memakai `ArxseusMonitor.exe`
> dan `%APPDATA%\ArxseusMonitor`.

---

## 1. Yang diukur dan batas lulus

| # | Instrumen | Cara ukur | Batas lulus |
|---|---|---|---|
| 1 | Pemahaman (klaim inti) | 4 kondisi × 2 penyajian, jawaban + waktu | ≥ 80% responden (≥ 11 dari 13) benar dalam < 10 detik pada penyajian aplikasi |
| 2 | Usability | SUS 10 item, skala 1–5 | rata-rata SUS ≥ 68 |
| 3 | Akurasi | 5 sampel per metrik vs Task Manager (suhu: HWiNFO bila ada) | selisih median < 5% untuk CPU, RAM, disk |
| 4 | Tugas terpandu | 7 tugas, keberhasilan + bantuan | ≥ 80% tugas per responden berhasil dengan bantuan kode 0–1 |

Pemahaman adalah klaim yang diuji; SUS dan akurasi adalah pendukung. Angka mentah
bukan pembanding yang harus "kalah" — ia hanya baseline agar klaim bisa diukur.

---

## 2. Responden

| Kelompok | Jumlah | Kriteria |
|---|---|---|
| Mahasiswa non-IT | 8 | Aktif kuliah, bukan mahasiswa informatika/ilmu komputer, memakai komputer sehari-hari |
| Keluarga non-teknis | 5 | Tidak bekerja di bidang IT, usia 25–65 |
| Pilot | 2 | Rekan di luar 13 responden; bukan target analisis |

- Total 13 responden, kode **R01–R13** (R01–R08 mahasiswa, R09–R13 keluarga).
- Eksklusi: pernah memakai aplikasi monitor hardware, atau terlibat dalam
  pengembangan aplikasi ini.
- Rekrutmen: ajakan lisan/chat, tanpa menyebut detail fitur. Kalimat pembuka:
  "Saya sedang menyelesaikan tugas akhir tentang aplikasi yang menjelaskan
  kondisi komputer. Boleh minta 30–40 menit untuk mencoba dan menilai?"
- Tidak ada insentif uang. Boleh diberi camilan/minuman; catat bila ada.
- Jadwal: pilot minggu ke-2, uji penuh minggu ke-3 (DESIGN.md §11).

---

## 3. Etika dan persetujuan

Dibacakan sebelum mulai, lalu responden mencentang lembar persetujuan (Lampiran A).

Poin naskah:

1. Tujuan: menilai apakah tampilan aplikasi mudah dipahami, bukan menilai
   kemampuan responden.
2. Sukarela; boleh berhenti atau melewati pertanyaan kapan saja tanpa alasan.
3. Durasi ±30–40 menit.
4. Data yang dicatat: jawaban, waktu, skor SUS, dan tangkapan layar aplikasi.
   Tidak ada nama, suara, atau rekaman wajah. Responden hanya dikenali lewat kode
   R01–R13.
5. Rekaman layar hanya bila responden setuju; bila tidak, pencatat menulis manual.
6. Data dipakai hanya untuk laporan tugas akhir dan tidak dibagikan.
7. Pertanyaan boleh kapan saja; kontak pencatat ditulis di lembar persetujuan.

Catatan untuk responden keluarga: tegaskan "ini bukan ujian" dan tidak ada
jawaban salah.

---

## 4. Perlengkapan dan persiapan mesin

| Item | Keterangan |
|---|---|
| Build uji | `npm run tauri build`, versi terpasang dicatat (tanggal build + ukuran `ArxseusMonitor.exe`) |
| Mesin | Laptop/PC uji yang sama untuk semua responden; catat spesifikasi (CPU, RAM, resolusi, skala) |
| Aplikasi pembanding | Task Manager (bawaan); HWiNFO **hanya bila** LHM terpasang dan WMI provider aktif |
| Alat waktu | Stopwatch ponsel, resolusi 0,1 detik |
| Alat stimulus | `docs/ramfill.ps1`, flag `--simulate-missing-temp`, `docs/kartu-angka.html` |
| Formulir | Lampiran A + B dicetak, atau spreadsheet dengan kolom yang sama |
| Pencatat | Minimal 1 orang yang tidak membantu responden (mencatat waktu & bantuan) |

Persiapan sebelum responden masuk:

1. Set bahasa **Indonesia**, interval **2 detik**, sumber suhu **Otomatis**.
2. Responden pertama: `introSeen=false` (kartu selamat datang tampil). Untuk
   responden berikutnya, kembalikan ke `false` agar alur sama untuk semua.
   Caranya: tutup aplikasi, hapus `%APPDATA%\ArxseusMonitor\settings.json`, jalankan
   ulang (settings dibuat default).
3. Tutup aplikasi berat (browser banyak tab, build, game) agar kondisi "aman"
   wajar. Catat aplikasi yang berjalan.
4. Rekam baseline aplikasi (angka + status) sebagai baris pertama lembar sesi.
5. Letakkan jendela aplikasi di posisi yang sama tiap sesi (±360×520, kanan
   bawah), dan pastikan tooltip tray bisa dibaca responden.

---

## 5. Stimulus uji pemahaman

Empat kondisi, masing-masing disajikan dua kali: **aplikasi** (dengan status) dan
**angka mentah** (nilai sama, tanpa kata status). Tujuannya membandingkan
penyajian, bukan hardware.

### 5.1 Kondisi dan cara membuatnya

| Kode | Kondisi | Cara membuat | Status acuan | Penyajian angka mentah |
|---|---|---|---|---|
| S1 | Aman | Diamkan mesin ≥ 30 detik tanpa beban | Aman | Angka dari snapshot yang sama |
| S2 | Waspada | `docs\ramfill.ps1 -TargetPercent 82 -HoldSeconds 40` | Waspada (RAM 80–92%) | Angka dari snapshot yang sama |
| S3 | Bahaya | `docs\ramfill.ps1 -TargetPercent 93 -HoldSeconds 40` | Bahaya (RAM > 92%) | Angka dari snapshot yang sama |
| S4 | Tanpa data | Jalankan `ArxseusMonitor.exe --simulate-missing-temp` (bila aplikasi sudah berjalan, jalankan exe dengan flag yang sama: instance kedua memaksa mode simulasi lalu keluar, D25) | Aman, suhu CPU/GPU tidak terbaca | Angka dengan suhu "tidak tersedia" |

Bukti acuan (DESIGN.md §7): CPU < 70 / 70–90 / > 90%; RAM < 80 / 80–92 / > 92%;
disk > 15% kosong / 8–15% / < 8%.

Catatan alat: `cargo run --example snapshot` **tidak** meneruskan flag ini (contoh
memanggil `Monitor::new(&settings, false)`), jadi S4 harus dibuat lewat instance
aplikasi. Jangan memakai contoh snapshot sebagai bukti "tanpa data".

### 5.2 Langkah menangkap stimulus (per kondisi)

1. Siapkan kondisi (§5.1), tunggu sampai **banner aplikasi berubah** ke status
   yang dimaksud. Status berubah setelah **3 sampel bertahan** (±6 detik), jadi
   jangan menangkap pada detik pertama.
2. Catat angka yang tampil (CPU, RAM, disk, suhu) di lembar sesi.
3. Tangkap jendela aplikasi (Win+Shift+S, region jendela saja).
4. Buka `docs/kartu-angka.html` dengan angka yang sama, mis.
   `kartu-angka.html?cpu=8&ram=82&used=13.1&total=15.9&disk=16.7&gputemp=46`,
   lalu tangkap region yang sama (lebar 322 px).
5. Lepas kondisi: hentikan `ramfill.ps1` (Ctrl+C) atau tutup instance
   `--simulate-missing-temp`. Pastikan RAM tersedia kembali > 2 GB sebelum
   kondisi berikutnya.

Simpan hasil di `docs/data/bukti/S1-app.png`, `S1-angka.png`, dst.

### 5.3 Aturan keselamatan saat mengisi RAM

- `ramfill.ps1` berhenti sendiri bila RAM tersedia < 600 MB, pemakaian ≥ 96%,
  atau alokasi mencapai batas (60% RAM total; bisa diatur `-MaxTotalMB`).
- Jangan menjalankan S2/S3 saat ada proses lain yang sedang menulis data besar
  (build, unduhan, render) — risiko paging berat.
- Hentikan segera bila layar membeku > 5 detik; RAM dilepas otomatis saat proses
  dihentikan.
- Antara S2 dan S3, tunggu sampai pemakaian kembali ke baseline (±1 menit).

### 5.4 Kalibrasi terukur (16 Nov; baris S4 dari bukti instance 15 Nov)

| Yang dicoba | Hasil pada aplikasi | Kesimpulan |
|---|---|---|
| Idle (baseline) | cpu 4–10% safe, ram 56,9–57,7% safe, disk 16,7% safe, gpuTemp 45–47 °C safe, cpuTemp unknown, overall **safe** | S1 tercapai |
| `ramfill -TargetPercent 82` | ram **81,9% warn**, overall **warn** (sampel ke-4 setelah latch) | S2 tercapai |
| `ramfill -TargetPercent 93` | ram **93,1% danger**, overall **danger** (4/4 sampel) | S3 tercapai |
| 16 thread sibuk (`docs\load.ps1`) | total 56–73%, per-core 57–72%, **tidak pernah > 70% stabil** | Beban CPU **tidak dipakai** untuk memicu Waspada/Bahaya di mesin ini |
| `--simulate-missing-temp` (instance aplikasi, 15 Nov) | cpuTemp & gpuTemp `null`, dua baris "Menunggu data", tanpa 0 °C, overall **safe** (`docs/black-box.md`) | S4 tercapai |

Catatan yang memengaruhi prosedur:

1. **Latch 3 sampel terlihat di data.** Saat RAM naik 77,8 → 83,9 → 90,0 → 92,9%,
   status masih `safe` pada sampel ke-4 karena tiap sampel berganti level dan
   penghitung 3-sampel selalu direset. Karena itu §5.2 menuntut konfirmasi banner,
   bukan sekadar angka target.
2. **Mesin uji dibatasi ±60–70% CPU** (`HypervisorPresent=True`, skema daya
   "Razer Cortex Power Plan"). Di mesin lain `load.ps1` bisa dipakai, tetapi
   instrumen ini tidak bergantung padanya.
3. **Disk C: hanya 16,7% kosong**, yaitu 1,7 poin di atas batas Waspada (15%).
   Periksa ulang sebelum sesi; bila turun < 15%, kondisi S1 tidak lagi "Aman" dan
   harus dicatat sebagai penyimpangan.

---

## 6. Urutan sesi

| Menit | Kegiatan | Catatan |
|---|---|---|
| 0–3 | Pembukaan, tujuan, persetujuan | Baca §3, centang Lampiran A |
| 3–5 | Perkenalan aplikasi | Responden klik "Mengerti" pada kartu selamat datang |
| 5–15 | Tugas T1–T7 | Bantuan dicatat, bukan diberikan diam-diam |
| 15–22 | Uji pemahaman (8 stimulus) | Urutan diseimbangkan (§6.1) |
| 22–27 | Kuesioner SUS | Aplikasi ditutup agar tidak menjadi petunjuk |
| 27–32 | Wawancara singkat | 2 pertanyaan: bagian paling membingungkan; apakah status dipercaya |
| 32–35 | Penutup, terima kasih | Simpan bukti, cek kelengkapan Lampiran A/B |

### 6.1 Penyeimbangan urutan

- Responden ganjil (R01, R03, …): penyajian **angka mentah** lebih dulu.
- Responden genap: penyajian **aplikasi** lebih dulu.
- Urutan kondisi di dalam tiap blok dirotasi agar tidak selalu S1 lebih dulu:
  - blok ganjil: S1, S2, S3, S4
  - blok genap: S3, S1, S4, S2
- Jangan mengganti urutan di tengah sesi; bila terjadi, catat di lembar sesi.

---

## 7. Tugas terpandu

Responden memakai aplikasi yang sedang berjalan. T1–T5 dan T7 memakai kondisi
nyata saat itu; **T6 memakai kondisi S4** (`--simulate-missing-temp`) dan
dijalankan sekali saat stimulus S4 disiapkan, bukan di tengah tugas lain.
Pencatat memulai stopwatch saat instruksi selesai diucapkan.

| # | Instruksi (dibacakan) | Kriteria berhasil | Target waktu |
|---|---|---|---|
| T1 | "Buka jendela aplikasi ini dari ikon di pojok kanan bawah." | Jendela tampil, tanpa bantuan | ≤ 15 s |
| T2 | "Menurut aplikasi, bagaimana kondisi komputer ini? Bagian mana yang membuat Anda menjawab begitu?" | Menyebut status yang sama dengan banner + minimal satu metrik pendukung | ≤ 20 s |
| T3 | "Buka penjelasan untuk memori (RAM), lalu ceritakan dengan kata sendiri." | Panel "Kenapa?" terbuka; parafrase benar (RAM = tempat kerja sementara, penuh → lambat) | ≤ 40 s |
| T4 | "Ubah bahasa aplikasi ke Inggris, lalu kembalikan ke Indonesia." | Dua kali ganti, semua teks ikut berubah | ≤ 30 s |
| T5 | "Ubah pembaruan menjadi tiap 5 detik, lalu pastikan pilihan itu tersimpan." | Kembali ke tampilan utama, buka lagi pengaturan, nilai tetap 5 detik | ≤ 40 s |
| T6 | "Di layar ada suhu yang tidak terbaca. Apakah itu tanda komputer rusak?" (pakai S4) | Menjawab bukan kerusakan / keterbatasan, bukan "rusak/panas" | ≤ 20 s |
| T7 | "Tanpa membuka jendela, apa status komputer dari ikon tray?" | Menyebut status + minimal satu angka dari tooltip | ≤ 15 s |

Definisi bantuan:

| Kode | Arti |
|---|---|
| 0 | Mandiri |
| 1 | Satu petunjuk verbal (mis. "coba lihat bagian atas") |
| 2 | Diarahkan/dibantu klik |

Berhasil = kode 0 atau 1. Tugas dengan kode 2 dicatat gagal. Jangan membantu
sebelum 30 detik tanpa kemajuan, kecuali responden meminta.

---

## 8. Uji pemahaman

### 8.1 Pertanyaan baku

- S1–S3: **"Menurut tampilan ini, apakah komputer ini sehat? Sebutkan bagian mana
  yang membuat Anda menjawab begitu."**
- S4: **"Apakah ada yang perlu dikhawatirkan dari tampilan ini?"**

Aturan:

1. Tampilkan stimulus, mulai stopwatch saat kartu terlihat penuh.
2. Hentikan saat responden selesai menjawab (status + alasan).
3. Jangan mengoreksi, jangan mengangguk berlebihan, jangan bertanya lanjutan
   yang mengarahkan.
4. Bila responden bertanya "ini sehat?", jawab: "Menurut tampilan itu, bagaimana?"
5. Catat jawaban apa adanya (kutipan singkat) + kode benar/salah + waktu 0,1 s.
6. Waktu > 10 detik tetap dicatat; status benar tetap dicatat terpisah.

### 8.2 Kode jawaban benar

| Kondisi | Jawaban benar (kode B) | Jawaban salah (kode S) |
|---|---|---|
| S1 | aman, sehat, normal, baik, tidak ada masalah | waspada/perlu perhatian, bahaya, rusak |
| S2 | waspada, perlu dipantau, agak tinggi, mulai penuh | aman/sehat, bahaya, rusak |
| S3 | bahaya, bermasalah, sangat penuh, harus segera ditangani | aman, waspada, "tidak tahu" |
| S4 | tidak ada yang perlu dikhawatirkan, suhu tidak terbaca karena keterbatasan Windows, bukan kerusakan | "komputernya rusak/panas", "sensor rusak", "harus diperbaiki" |

Aturan pengodean: satu kode per jawaban; bila responden menyebut dua hal
berlawanan, ambil kesimpulan akhirnya dan catat kutipannya. Untuk S2/S3, jawaban
yang benar tetapi menyebut metrik yang salah tetap B, dengan catatan.

### 8.3 Formulir (8 baris per responden)

| Kondisi | Penyajian | Jawaban (kutipan) | Kode | Waktu (s) | Lulus (< 10 s & B) |
|---|---|---|---|---|---|
| S1 | angka | | | | |
| S1 | aplikasi | | | | |
| S2 | angka | | | | |
| S2 | aplikasi | | | | |
| S3 | angka | | | | |
| S3 | aplikasi | | | | |
| S4 | angka | | | | |
| S4 | aplikasi | | | | |

Lulus per responden pada penyajian aplikasi bila **keempat** kondisi benar dan
< 10 detik. Batas lulus agregat: ≥ 11 dari 13 responden.

---

## 9. Kuesioner SUS

Skala 1–5: 1 = sangat tidak setuju, 2 = tidak setuju, 3 = netral,
4 = setuju, 5 = sangat setuju. Item ganjil bersifat positif, item genap negatif.

| # | Pernyataan |
|---|---|
| 1 | Saya ingin menggunakan aplikasi ini sesering mungkin. |
| 2 | Saya merasa aplikasi ini terlalu rumit padahal semestinya bisa lebih sederhana. |
| 3 | Saya merasa aplikasi ini mudah digunakan. |
| 4 | Saya rasa saya butuh bantuan orang teknis untuk bisa memakai aplikasi ini. |
| 5 | Saya merasa bagian-bagian aplikasi ini sudah terpadu dengan baik. |
| 6 | Saya merasa terlalu banyak hal yang tidak konsisten di aplikasi ini. |
| 7 | Saya rasa kebanyakan orang akan cepat belajar memakai aplikasi ini. |
| 8 | Saya merasa aplikasi ini kikuk (tidak nyaman) untuk digunakan. |
| 9 | Saya merasa percaya diri saat memakai aplikasi ini. |
| 10 | Saya perlu belajar banyak hal dulu sebelum bisa memakai aplikasi ini. |

Cara menghitung (Brooke, 1996):

1. Item ganjil (1, 3, 5, 7, 9): skor − 1.
2. Item genap (2, 4, 6, 8, 10): 5 − skor.
3. Jumlahkan 10 nilai (rentang 0–40), lalu **× 2,5** → skor 0–100.

Contoh: skor mentah [5, 1, 5, 1, 4, 2, 5, 1, 4, 2] → ganjil 4+4+3+4+3 = 18,
genap 4+4+3+4+3 = 18, total 36 → **SUS 90**.

Pelaporan: skor tiap responden, rata-rata, median, simpangan baku, min–maks.
Batas lulus rata-rata **≥ 68**. Bila < 68, tulis penyebab dari catatan tugas dan
wawancara, bukan hanya angkanya.

Item genap (2, 4, 6, 8, 10) bersifat negatif — jangan lupa membalik arahnya saat
menghitung; item ganjil tidak dibalik.

---

## 10. Lembar akurasi

Metode: 5 sampel per metrik, jeda 10 detik, aplikasi dan Task Manager terlihat
bersamaan. Untuk suhu, HWiNFO hanya dipakai bila LHM terpasang dan WMI provider
aktif; bila tidak, tulis "tidak diuji (batas mesin)".

| Metrik | Aplikasi | Acuan (Task Manager / HWiNFO) | Selisih % |
|---|---|---|---|
| CPU % | | | |
| RAM % | | | |
| Disk % kosong (C:) | | | |
| Suhu CPU °C (opsional) | | | |
| Suhu GPU °C (opsional) | | | |

Rumus: `selisih = |app − acuan| / acuan × 100`. Catat 5 baris per metrik, lalu
ambil **median**. Target < 5% untuk CPU, RAM, dan disk.

Catatan metode yang harus ditulis di laporan:

- CPU aplikasi = rata-rata sejak sampel sebelumnya (2 detik, DESIGN.md §4),
  sedangkan Task Manager punya interval dan ikut menghitung dirinya sendiri.
  Karena itu bandingkan pada kondisi relatif stabil (idle dan beban tetap).
- RAM: bandingkan persentase, bukan byte, karena acuan Task Manager membulatkan.
- Disk: acuan persen kosong dari Explorer/`Get-Volume`, bukan aktivitas disk.
- Suhu CPU tidak tersedia tanpa admin di mesin uji (`0x80041003`), jadi target
  akurasi §5 hanya mencakup CPU/RAM/disk.

---

## 11. Pilot 2 orang

Dijalankan sebelum 13 responden, dengan orang di luar sampel. Dicatat terpisah,
tidak masuk analisis.

Yang dinilai:

1. Apakah pertanyaan §8.1 dipahami tanpa penjelasan tambahan?
2. Apakah kartu angka (§5.2) terbaca sebagai "angka biasa", bukan petunjuk status?
3. Apakah 8 stimulus terlalu melelahkan? Kalau ya, kurangi jadi 6 (S1–S3 aplikasi
   + S1–S3 angka) dan catat alasannya.
4. Durasi nyata vs perkiraan 30–40 menit.
5. Tugas mana yang paling sering butuh bantuan.

Kriteria lanjut ke uji penuh: tidak ada pertanyaan yang perlu diubah maknanya,
durasi ≤ 45 menit, dan semua kondisi stimulus bisa dibuat ulang. Perubahan
instrumen dicatat sebagai versi (v1.0 → v1.1) beserta alasannya.

---

## 12. Rekap dan pelaporan

### 12.1 SUS

| Responden | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | Total (0–40) | SUS |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R01 | | | | | | | | | | | | |
| … | | | | | | | | | | | | |
| R13 | | | | | | | | | | | | |

Ringkasan: rata-rata, median, SD, min–maks, jumlah ≥ 68.

### 12.2 Pemahaman

| Responden | App benar | App < 10 s | App lulus 4/4 | Angka benar | Angka < 10 s | Angka lulus 4/4 |
|---|---|---|---|---|---|---|
| R01 | | | | | | |
| … | | | | | | |

Ringkasan: persentase lulus per penyajian; selisih rata-rata waktu (detik);
jumlah jawaban "tidak tahu".

### 12.3 Akurasi

| Metrik | Median selisih % | Maks selisih % | Lulus (< 5%) |
|---|---|---|---|
| CPU | | | |
| RAM | | | |
| Disk | | | |

### 12.4 Penyimpanan data

- Data mentah: `docs/data/responden.csv` (hanya kode R01–R13, tanpa nama).
- Bukti gambar: `docs/data/bukti/` (8 stimulus + tangkapan layar tugas penting).
- Jangan menyimpan kontak atau nama responden di dalam repo.
- Rekap akhir masuk laporan sebagai tabel, bukan tangkapan layar spreadsheet.

---

## 13. Batas pengujian

1. Sampel 13 orang non-probability; hasil tidak digeneralisasi ke populasi luas.
2. Satu mesin, satu konfigurasi layar (2560×1440 @150%), satu build.
3. Hanya bahasa Indonesia yang diuji; versi Inggris tersedia tetapi tidak dinilai
   (D8).
4. Suhu CPU tidak tersedia tanpa admin di mesin uji; klaim suhu diuji lewat
   kondisi "tanpa data" (S4) dan suhu GPU via NVML.
5. Waspada/Bahaya dipicu dengan mengisi RAM (`ramfill.ps1`), bukan beban kerja
   nyata pengguna. Ini disengaja agar metrik lain tetap stabil; sebutkan di
   laporan bahwa angkanya nyata tetapi pemicunya buatan.
6. Beban CPU tidak bisa memicu Waspada di mesin uji (terukur 56–73%), sehingga
   kondisi CPU tinggi tidak diuji.
7. NVDA, mode kontras paksa, dan pembesaran 200% berada di luar cakupan
   (lihat `docs/a11y.md`), karena uji ini fokus pada pemahaman orang awam.

---

## Lampiran A. Lembar sesi per responden

```
Kode responden : R__        Kelompok: mahasiswa / keluarga
Tanggal, jam   : ____       Durasi total: ____ menit
Build aplikasi : ____       Mesin: ____ (CPU/RAM/layar/skala)
Aplikasi lain  : ____

PERSETUJUAN
[ ] Responden membaca/mendengar ringkasan §3 dan setuju.
[ ] Setuju direkam layar (opsional).   [ ] Tidak.

BASELINE APLIKASI
Status: ____   CPU __%   RAM __%   Disk __%   Suhu CPU ____   Suhu GPU ____

TUGAS
T1 ____ s  bantuan 0/1/2  berhasil Y/T   catatan: ____
T2 ____ s  bantuan 0/1/2  berhasil Y/T   catatan: ____
T3 ____ s  bantuan 0/1/2  berhasil Y/T   catatan: ____
T4 ____ s  bantuan 0/1/2  berhasil Y/T   catatan: ____
T5 ____ s  bantuan 0/1/2  berhasil Y/T   catatan: ____
T6 ____ s  bantuan 0/1/2  berhasil Y/T   catatan: ____
T7 ____ s  bantuan 0/1/2  berhasil Y/T   catatan: ____

PEMAHAMAN (8 baris, lihat §8.3)
Urutan penyajian: angka dulu / aplikasi dulu

SUS: total ____ (0-40) -> skor ____

WAWANCARA
1. Bagian paling membingungkan: ____
2. Apakah statusnya dipercaya? Mengapa: ____

CATATAN PENCATAT (gangguan, penyimpangan urutan, kondisi mesin): ____
```

## Lampiran B. Lembar SUS (dicetak)

```
Kode: R__     Tanggal: ____

Lingkari: 1 = sangat tidak setuju ... 5 = sangat setuju

1. Saya ingin menggunakan aplikasi ini sesering mungkin.        1 2 3 4 5
2. Saya merasa aplikasi ini terlalu rumit padahal semestinya
   bisa lebih sederhana.                                        1 2 3 4 5
3. Saya merasa aplikasi ini mudah digunakan.                    1 2 3 4 5
4. Saya rasa saya butuh bantuan orang teknis untuk bisa
   memakai aplikasi ini.                                        1 2 3 4 5
5. Saya merasa bagian-bagian aplikasi ini sudah terpadu
   dengan baik.                                                 1 2 3 4 5
6. Saya merasa terlalu banyak hal yang tidak konsisten
   di aplikasi ini.                                             1 2 3 4 5
7. Saya rasa kebanyakan orang akan cepat belajar memakai
   aplikasi ini.                                                1 2 3 4 5
8. Saya merasa aplikasi ini kikuk (tidak nyaman) digunakan.     1 2 3 4 5
9. Saya merasa percaya diri saat memakai aplikasi ini.           1 2 3 4 5
10. Saya perlu belajar banyak hal dulu sebelum bisa
    memakai aplikasi ini.                                       1 2 3 4 5
```
