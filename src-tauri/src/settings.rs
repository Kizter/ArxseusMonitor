//! Preferensi pengguna yang disimpan antar sesi.
//!
//! JSON polos di `%APPDATA%\<produk>\settings.json`. Kesalahan baca **tidak
//! pernah** membuat aplikasi gagal jalan: file rusak atau hilang berarti
//! kembali ke default. Tidak ada database, tidak ada jaringan.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Nama folder di `%APPDATA%`, mengikuti nama produk final (D26).
pub const APP_DIR: &str = "ArxseusMonitor";
pub const FILE_NAME: &str = "settings.json";

pub const DEFAULT_INTERVAL_SECS: u64 = 2;
/// Hanya tiga pilihan ini yang ditawarkan UI (`DESIGN.md` §8).
pub const ALLOWED_INTERVALS: [u64; 3] = [1, 2, 5];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Id,
    En,
}

/// Sumber suhu CPU yang dipilih pengguna.
///
/// `Auto` = coba LHM dulu, lalu ACPI. `Off` mematikan pembacaan suhu CPU
/// sepenuhnya (kartu suhu CPU menjadi "tidak terdeteksi").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TempPreference {
    #[default]
    Auto,
    Acpi,
    Lhm,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub language: Language,
    pub interval_secs: u64,
    pub temperature_source: TempPreference,
    pub start_minimized: bool,
    /// Apakah layar penjelasan pertama kali sudah ditutup pengguna.
    pub intro_seen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::Id,
            interval_secs: DEFAULT_INTERVAL_SECS,
            temperature_source: TempPreference::Auto,
            start_minimized: false,
            intro_seen: false,
        }
    }
}

impl Settings {
    /// Paksa nilai yang masuk akal. Dipanggil setiap kali settings dibaca,
    /// termasuk untuk file yang diedit manual.
    pub fn sanitized(mut self) -> Self {
        if !ALLOWED_INTERVALS.contains(&self.interval_secs) {
            self.interval_secs = DEFAULT_INTERVAL_SECS;
        }
        self
    }

    pub fn interval_ms(&self) -> u64 {
        self.interval_secs.saturating_mul(1000)
    }
}

/// Lokasi default: `%APPDATA%\ArxseusMonitor\settings.json`, dengan fallback ke
/// direktori kerja kalau `APPDATA` tidak ada.
pub fn default_path() -> PathBuf {
    match std::env::var_os("APPDATA") {
        Some(appdata) => PathBuf::from(appdata).join(APP_DIR).join(FILE_NAME),
        None => PathBuf::from(APP_DIR).join(FILE_NAME),
    }
}

/// Baca settings. File tidak ada atau isinya rusak → default, tanpa panic.
pub fn load(path: &Path) -> Settings {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str::<Settings>(&text)
            .map(Settings::sanitized)
            .unwrap_or_else(|error| {
                eprintln!(
                    "[settings] {} tidak bisa dibaca sebagai JSON ({error}); memakai default",
                    path.display()
                );
                Settings::default()
            }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Settings::default(),
        Err(error) => {
            eprintln!(
                "[settings] gagal membaca {} ({error}); memakai default",
                path.display()
            );
            Settings::default()
        }
    }
}

/// Tulis settings. Folder dibuat bila belum ada.
pub fn save(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut text = serde_json::to_string_pretty(&settings.clone().sanitized())
        .map_err(io::Error::other)?;
    text.push('\n');
    fs::write(path, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "arxseus-settings-test-{}-{tag}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn roundtrip_through_disk() {
        let dir = temp_dir("roundtrip");
        let path = dir.join(FILE_NAME);
        let settings = Settings {
            language: Language::En,
            interval_secs: 5,
            temperature_source: TempPreference::Lhm,
            start_minimized: true,
            intro_seen: true,
        };

        save(&path, &settings).expect("tulis settings");
        assert_eq!(load(&path), settings);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_means_defaults() {
        let path = temp_dir("missing").join(FILE_NAME);
        assert_eq!(load(&path), Settings::default());
    }

    #[test]
    fn corrupt_file_means_defaults_not_crash() {
        let dir = temp_dir("corrupt");
        fs::create_dir_all(&dir).expect("buat folder");
        let path = dir.join(FILE_NAME);
        fs::write(&path, "{ ini bukan json").expect("tulis file rusak");

        assert_eq!(load(&path), Settings::default());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_interval_falls_back_to_default() {
        let dir = temp_dir("sanitize");
        fs::create_dir_all(&dir).expect("buat folder");
        let path = dir.join(FILE_NAME);
        fs::write(&path, r#"{"intervalSecs": 17}"#).expect("tulis file");

        let settings = load(&path);
        assert_eq!(settings.interval_secs, DEFAULT_INTERVAL_SECS);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn partial_json_is_filled_with_defaults() {
        let dir = temp_dir("partial");
        fs::create_dir_all(&dir).expect("buat folder");
        let path = dir.join(FILE_NAME);
        fs::write(&path, r#"{"language": "en"}"#).expect("tulis file");

        let settings = load(&path);
        assert_eq!(settings.language, Language::En);
        assert_eq!(settings.interval_secs, DEFAULT_INTERVAL_SECS);
        assert_eq!(settings.temperature_source, TempPreference::Auto);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_clamps_invalid_interval() {
        let dir = temp_dir("save-sanitize");
        let path = dir.join(FILE_NAME);

        let settings = Settings {
            interval_secs: 99,
            ..Default::default()
        };
        save(&path, &settings).expect("tulis settings");

        assert_eq!(load(&path).interval_secs, DEFAULT_INTERVAL_SECS);

        let _ = fs::remove_dir_all(&dir);
    }
}
