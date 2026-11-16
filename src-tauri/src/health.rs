//! Ambang batas dan aturan perubahan status.
//!
//! Modul ini sengaja bebas dari I/O dan dari tipe sensor supaya bisa diuji
//! murni. Semua angka berasal dari `DESIGN.md` §7.

use serde::Serialize;

/// Tiga level yang bisa ditindaklanjuti pengguna awam, plus `Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Safe,
    Warn,
    Danger,
    Unknown,
}

impl Level {
    fn rank(self) -> u8 {
        match self {
            Level::Unknown => 0,
            Level::Safe => 1,
            Level::Warn => 2,
            Level::Danger => 3,
        }
    }

    fn worst(self, other: Level) -> Level {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }
}

/// Level terburuk dari beberapa metrik. `Unknown` diabaikan; kalau semuanya
/// `Unknown`, hasilnya `Unknown`.
pub fn overall(levels: impl IntoIterator<Item = Level>) -> Level {
    levels
        .into_iter()
        .filter(|level| *level != Level::Unknown)
        .reduce(Level::worst)
        .unwrap_or(Level::Unknown)
}

/// Arah bahaya sebuah metrik.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Makin tinggi makin buruk (beban, suhu, pemakaian RAM).
    HigherIsWorse,
    /// Makin rendah makin buruk (sisa ruang disk).
    LowerIsWorse,
}

#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    pub cpu_warn: f64,
    pub cpu_danger: f64,
    pub ram_warn: f64,
    pub ram_danger: f64,
    pub disk_warn_free: f64,
    pub disk_danger_free: f64,
    pub cpu_temp_warn: f64,
    pub cpu_temp_danger: f64,
    pub gpu_temp_warn: f64,
    pub gpu_temp_danger: f64,
}

impl Default for Thresholds {
    /// Sumber: `DESIGN.md` §7 (Tjmax CPU modern, batas aman GPU konsumen,
    /// Windows mulai bermasalah di bawah 10% ruang disk).
    fn default() -> Self {
        Self {
            cpu_warn: 70.0,
            cpu_danger: 90.0,
            ram_warn: 80.0,
            ram_danger: 92.0,
            disk_warn_free: 15.0,
            disk_danger_free: 8.0,
            cpu_temp_warn: 75.0,
            cpu_temp_danger: 90.0,
            gpu_temp_warn: 80.0,
            gpu_temp_danger: 90.0,
        }
    }
}

/// Klasifikasi satu nilai. Batas atas termasuk kategori yang lebih ringan,
/// sesuai tabel di desain (mis. 90 °C masih `Warn`, 90,1 °C `Danger`).
pub fn classify(value: Option<f64>, warn: f64, danger: f64, direction: Direction) -> Level {
    let Some(value) = value.filter(|v| v.is_finite()) else {
        return Level::Unknown;
    };

    match direction {
        Direction::HigherIsWorse => {
            if value > danger {
                Level::Danger
            } else if value >= warn {
                Level::Warn
            } else {
                Level::Safe
            }
        }
        Direction::LowerIsWorse => {
            if value < danger {
                Level::Danger
            } else if value <= warn {
                Level::Warn
            } else {
                Level::Safe
            }
        }
    }
}

/// Jumlah sampel berturut-turut sebelum status berubah.
pub const SUSTAIN_SAMPLES: u8 = 3;

/// Menahan perubahan status sampai kondisi bertahan beberapa sampel,
/// supaya ikon tray tidak berkedip saat ada lonjakan sesaat.
///
/// Sampel pertama selalu diterima apa adanya — monitor tidak boleh menampilkan
/// "aman" hanya karena belum ada data.
#[derive(Debug, Clone, Copy)]
pub struct Latch {
    current: Level,
    pending: Option<(Level, u8)>,
    initialized: bool,
}

impl Default for Latch {
    fn default() -> Self {
        Self::new()
    }
}

impl Latch {
    pub const fn new() -> Self {
        Self {
            current: Level::Unknown,
            pending: None,
            initialized: false,
        }
    }

    pub const fn current(&self) -> Level {
        self.current
    }

    /// Lupakan riwayat. Dipakai setelah sleep/resume: sampel sebelum dan
    /// sesudah tidur tidak boleh dihitung sebagai kondisi yang "bertahan".
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn apply(&mut self, raw: Level) -> Level {
        if !self.initialized {
            self.initialized = true;
            self.current = raw;
            return self.current;
        }

        // Hilangnya data tidak perlu ditahan: lebih baik jujur "tidak
        // terdeteksi" daripada menampilkan angka lama.
        if raw == Level::Unknown {
            self.current = Level::Unknown;
            self.pending = None;
            return self.current;
        }

        if raw == self.current {
            self.pending = None;
            return self.current;
        }

        self.pending = match self.pending {
            Some((level, count)) if level == raw => Some((level, count.saturating_add(1))),
            _ => Some((raw, 1)),
        };

        if let Some((level, count)) = self.pending
            && count >= SUSTAIN_SAMPLES
        {
            self.current = level;
            self.pending = None;
        }

        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_high_side_with_inclusive_boundary() {
        let warn = 70.0;
        let danger = 90.0;
        let dir = Direction::HigherIsWorse;

        assert_eq!(classify(Some(10.0), warn, danger, dir), Level::Safe);
        assert_eq!(classify(Some(70.0), warn, danger, dir), Level::Warn);
        assert_eq!(classify(Some(90.0), warn, danger, dir), Level::Warn);
        assert_eq!(classify(Some(90.1), warn, danger, dir), Level::Danger);
        assert_eq!(classify(None, warn, danger, dir), Level::Unknown);
    }

    #[test]
    fn classifies_low_side_for_disk_space() {
        let warn_free = 15.0;
        let danger_free = 8.0;
        let dir = Direction::LowerIsWorse;

        assert_eq!(classify(Some(40.0), warn_free, danger_free, dir), Level::Safe);
        assert_eq!(classify(Some(15.0), warn_free, danger_free, dir), Level::Warn);
        assert_eq!(classify(Some(8.0), warn_free, danger_free, dir), Level::Warn);
        assert_eq!(classify(Some(7.9), warn_free, danger_free, dir), Level::Danger);
    }

    #[test]
    fn overall_takes_worst_and_ignores_unknown() {
        assert_eq!(
            overall([Level::Safe, Level::Warn, Level::Unknown]),
            Level::Warn
        );
        assert_eq!(
            overall([Level::Warn, Level::Danger]),
            Level::Danger
        );
        assert_eq!(overall([Level::Unknown, Level::Unknown]), Level::Unknown);
    }

    #[test]
    fn first_sample_is_adopted_immediately() {
        let mut latch = Latch::new();
        assert_eq!(latch.apply(Level::Danger), Level::Danger);
    }

    #[test]
    fn change_needs_three_samples() {
        let mut latch = Latch::new();
        assert_eq!(latch.apply(Level::Safe), Level::Safe);

        assert_eq!(latch.apply(Level::Danger), Level::Safe);
        assert_eq!(latch.apply(Level::Danger), Level::Safe);
        assert_eq!(latch.apply(Level::Danger), Level::Danger);
    }

    #[test]
    fn flapping_resets_the_counter() {
        let mut latch = Latch::new();
        latch.apply(Level::Safe);

        latch.apply(Level::Warn);
        latch.apply(Level::Warn);
        // Satu sampel kembali aman membatalkan hitungan.
        latch.apply(Level::Safe);
        assert_eq!(latch.apply(Level::Warn), Level::Safe);
    }

    #[test]
    fn missing_data_is_immediate() {
        let mut latch = Latch::new();
        latch.apply(Level::Safe);
        assert_eq!(latch.apply(Level::Unknown), Level::Unknown);
    }
}
