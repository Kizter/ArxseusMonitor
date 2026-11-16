# Hasil Probe Sensor — Fase 1

Tanggal: 13 November 2026
Mesin uji: Intel Core i5-13400F (16 thread), 15,9 GB RAM, NVIDIA GeForce GTX 1650, Windows
Perintah:

```bash
cargo run --example probe --manifest-path src-tauri/Cargo.toml
```

Dua siklus diukur: **cold** (koneksi baru, seperti saat aplikasi dibuka) dan
**warm** (provider dipakai ulang, seperti saat polling 2 detik).

---

## 1. Ringkasan per provider

| Provider | Sumber | Status | Cold | Warm | Catatan |
|---|---|---|---|---|---|
| `sysinfo` | crate `sysinfo` | **OK** | 327,0 ms | 0,6 ms | CPU, RAM, disk, uptime lengkap |
| `lhm_wmi` | `ROOT\LibreHardwareMonitor` | gagal | 7,6 ms | 1,0 ms | `0x8004100E` namespace tidak ada — LHM tidak berjalan |
| `thermal_wmi` | `MSAcpi_ThermalZoneTemperature` | gagal | 10,1 ms | 3,4 ms | `0x80041003` akses ditolak — butuh administrator |
| `nvml` | NVML | **OK** | 11,3 ms | 0,0 ms | GTX 1650, suhu GPU 48 °C |
| **Total** | | | **355,9 ms** | **5,1 ms** | |

## 2. Metrik yang benar-benar terbaca

| Metrik | Nilai | Sumber |
|---|---|---|
| CPU brand | 13th Gen Intel(R) Core(TM) i5-13400F | sysinfo |
| CPU thread | 16 | sysinfo |
| CPU usage | 4,6 % | sysinfo |
| RAM | 10,9 / 15,9 GB (69 %) | sysinfo |
| Uptime | 3 jam 15 menit | sysinfo |
| Disk C: | 81,8 GB bebas dari 465,1 GB (18 %) | sysinfo |
| Disk D: | 251,6 GB bebas dari 465,7 GB (54 %) | sysinfo |
| Disk E: | 172,4 GB bebas dari 419,2 GB (41 %) | sysinfo |
| **Suhu CPU** | **tidak terdeteksi** | — |
| Nama GPU | NVIDIA GeForce GTX 1650 | NVML |
| Suhu GPU | 48,0 °C | NVML |

## 3. Kesimpulan

1. **Gerbang Fase 1 tercapai lebih awal (13 Nov, batas 16 Nov).** Suhu CPU tidak
   tersedia tanpa hak administrator. Sesuai rencana, strategi suhu dikunci ke
   **best-effort** dan pekerjaan lanjut ke Fase 2. Tidak ada pengejaran driver.
2. **Performa aman.** Warm 5,1 ms per siklus berarti polling 2 detik hanya
   memakai ~0,25 % dari satu inti untuk pembacaan sensor. Cold 356 ms masih di
   bawah target startup < 2 detik.
3. **`sysinfo` adalah fondasi yang andal.** Semua metrik non-suhu terbaca tanpa
   admin, tanpa driver, tanpa jaringan.
4. **ACPI thermal zone bukan jalan keluar.** Di mesin ini ia ada tetapi
   menolak akses tanpa elevasi. Karena aplikasi tidak pernah meminta admin,
   sumber ini praktis tidak terpakai — dan itu konsisten dengan desain.
5. **NVML memberi suhu GPU tanpa admin.** Metrik "panas" tetap ada di UI,
   walau bukan dari CPU.
6. **Jalur hibrida LHM belum terverifikasi** — LHM tidak terpasang di mesin ini.
   Ini satu-satunya cara mendapat suhu core CPU dan masih perlu diuji manual.

## 4. Dampak ke desain (tidak ada perubahan arsitektur)

- Kartu suhu CPU **wajib** punya state "tidak terdeteksi" dengan penjelasan,
  bukan 0 °C. Sudah dirancang di `DESIGN.md` §9, sekarang terbukti perlu.
- Status keseluruhan dihitung tanpa metrik suhu bila tidak tersedia
  (`unknown` tidak dihitung). Sudah sesuai `DESIGN.md` §7.
- Footer menampilkan sumber suhu: `lhm` / `acpi` / `tidak tersedia`.
  Di mesin ini akan tampil "tidak tersedia" untuk CPU dan angka untuk GPU.
- Narasi skripsi: keterbatasan ini bukan bug, melainkan **temuan** yang
  didokumentasikan (Windows tidak mengekspos suhu core tanpa driver ring0).

## 5. Tindak lanjut sebelum sidang

**Keputusan (13 Nov): opsi B** — terima UI jujur "tidak terdeteksi" untuk suhu
CPU, dan buktikan jalur hibrida dengan memasang LibreHardwareMonitor di mesin
demo. LHM berjalan terpisah dan butuh admin; aplikasi kita tetap tanpa elevasi.

- [ ] Pasang LHM (`winget install --id LibreHardwareMonitor.LibreHardwareMonitor -e`),
      aktifkan *Options → WMI Provider* (atau opsi serupa), jalankan LHM sebagai
      administrator → jalankan probe lagi → pastikan `lhm_wmi` `[ok]`.
- [ ] Opsional: jalankan probe sekali sebagai administrator untuk mendokumentasikan
      apakah ACPI benar-benar memberi suhu CPU yang masuk akal (hanya bukti,
      bukan ketergantungan aplikasi).
- [ ] Simpan output `probe` sebagai lampiran bab pengujian.
