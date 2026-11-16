# Pemakaian jaringan: apa yang benar-benar dibuka

Menguji NFR DESIGN.md §4 ("Jaringan: 0 request"). Diukur 16 Nov pada build rilis
`arxseus-monitor.exe`.

## Metode

1. Jalankan `arxseus-monitor.exe`, tunggu 6 detik.
2. Susun pohon proses (aplikasi + seluruh anak `msedgewebview2.exe`).
3. `Get-NetTCPConnection` difilter per PID pohon itu, state `Established`,
   alamat remote di luar loopback.
4. Diulang tiap 5 detik selama 20 detik.

## Hasil

| Pengukuran | Koneksi eksternal | Endpoint |
|---|---|---|
| Tanpa argumen tambahan | 2 | `40.99.9.114:443`, `40.99.10.18:443`, `52.98.40.34:443`, `52.98.189.242:443` (beda tiap kali jalan) |
| Dengan 8 flag background networking | 2 | tetap ada |
| Dengan flag + `--host-resolver-rules` | 2 | flag itu **dibuang WebView2**, tidak muncul di command line proses browser |

- Koneksi bertahan minimal 21 detik (diamati pada 4 titik waktu) dan dimiliki
  proses `msedgewebview2.exe`, bukan proses aplikasi.
- Sertifikat endpoint: `CN=outlook.com, O=Microsoft Corporation` dengan SAN
  Outlook/Office/Office365 -> infrastruktur Microsoft (Azure Front Door), bukan
  server proyek.
- Proses aplikasi (`arxseus-monitor.exe`) sendiri tidak punya koneksi eksternal.

## Arti untuk klaim NFR

Kode aplikasi **tidak** melakukan request apa pun: tidak ada klien HTTP di
Cargo.toml, tidak ada `fetch` di frontend, dan UI dilayani dari
`tauri.localhost` lokal. Yang membuka koneksi adalah runtime WebView2 milik
Microsoft yang menempel pada aplikasi, dengan endpoint yang tidak bisa
dikendalikan dari konfigurasi Tauri.

Karena itu klaim "0 request" diubah menjadi dua bagian yang bisa dibuktikan
(lihat DESIGN.md §4):

1. **Backend Rust dan frontend aplikasi: 0 request** (tidak ada kode jaringan).
2. **Runtime WebView2: 2 koneksi TLS ke Microsoft saat start** (di luar kendali
   aplikasi; terukur, bukan asumsi).

## Yang dicoba dan tidak berhasil

| Upaya | Hasil |
|---|---|
| `--disable-background-networking`, `--disable-component-update`, `--disable-domain-reliability`, `--disable-sync`, `--disable-client-side-phishing-detection`, `--no-pings`, `--disable-breakpad`, `--disable-features=...` | Flag terpasang (diverifikasi ada di command line proses browser), koneksi tetap 2 |
| `--host-resolver-rules="MAP * ~NOTFOUND, ..."` | Dibuang WebView2; tidak sampai ke proses browser |
| Menghapus flag (kembali ke konfigurasi bersih) | Tetap 2, jadi flag tidak berpengaruh pada jumlah koneksi |

Konfigurasi `additionalBrowserArgs` karena itu **tidak dipakai**: tidak ada bukti
ia mengurangi apa pun, dan menambah knop yang tidak teruji hanya menyesatkan.

## Bila demo harus benar-benar offline

Matikan adaptor jaringan, atau jalankan demo di mesin tanpa jaringan. Dengan
adaptor mati, alamat IP yang sudah di-cache gagal dihubungi dan tidak ada
koneksi `Established` sama sekali; aplikasi tetap berfungsi penuh karena tidak
butuh jaringan.
