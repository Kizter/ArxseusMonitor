//! Bentuk data yang dikirim ke frontend.
//!
//! Frontend tidak pernah menghitung status sendiri: semua `level` dan
//! `reasonKey` sudah diputuskan di sini. Yang dilakukan React hanya
//! menerjemahkan `reasonKey` menjadi kalimat dan memilih warna.
//!
//! Nama field di JSON memakai camelCase agar cocok dengan TypeScript.

use crate::health::{classify, Direction, Latch, Level, Thresholds};
use crate::sensors::{PartialMetrics, TempSource};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// Satu metrik siap tampil. `value == None` berarti sensor tidak tersedia —
/// bukan nol, dan UI wajib menampilkan "tidak terdeteksi".
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub value: Option<f64>,
    pub unit: &'static str,
    pub level: Level,
    /// Kunci i18n, mis. `"ram.high"`. Frontend yang menulis kalimatnya.
    pub reason_key: &'static str,
}

impl Metric {
    fn new(value: Option<f64>, unit: &'static str, level: Level, id: MetricId) -> Self {
        Self {
            value: value.map(|v| (v * 10.0).round() / 10.0),
            unit,
            level,
            reason_key: id.reason_key(level),
        }
    }

    fn unavailable(unit: &'static str, id: MetricId) -> Self {
        Self::new(None, unit, Level::Unknown, id)
    }
}

#[derive(Debug, Clone, Copy)]
enum MetricId {
    Cpu,
    Ram,
    Disk,
    CpuTemp,
    GpuTemp,
}

impl MetricId {
    fn reason_key(self, level: Level) -> &'static str {
        use Level::{Danger, Safe, Unknown, Warn};

        match (self, level) {
            (MetricId::Cpu, Safe) => "cpu.ok",
            (MetricId::Cpu, Warn) => "cpu.high",
            (MetricId::Cpu, Danger) => "cpu.critical",
            (MetricId::Cpu, Unknown) => "cpu.unavailable",

            (MetricId::Ram, Safe) => "ram.ok",
            (MetricId::Ram, Warn) => "ram.high",
            (MetricId::Ram, Danger) => "ram.critical",
            (MetricId::Ram, Unknown) => "ram.unavailable",

            (MetricId::Disk, Safe) => "disk.ok",
            (MetricId::Disk, Warn) => "disk.low",
            (MetricId::Disk, Danger) => "disk.critical",
            (MetricId::Disk, Unknown) => "disk.unavailable",

            (MetricId::CpuTemp, Safe) => "temp.cpu.ok",
            (MetricId::CpuTemp, Warn) => "temp.cpu.warm",
            (MetricId::CpuTemp, Danger) => "temp.cpu.critical",
            (MetricId::CpuTemp, Unknown) => "temp.cpu.unavailable",

            (MetricId::GpuTemp, Safe) => "temp.gpu.ok",
            (MetricId::GpuTemp, Warn) => "temp.gpu.warm",
            (MetricId::GpuTemp, Danger) => "temp.gpu.critical",
            (MetricId::GpuTemp, Unknown) => "temp.gpu.unavailable",
        }
    }
}

/// Lima metrik inti. Baterai tidak termasuk MVP (lihat `DESIGN.md` D19).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub cpu: Metric,
    pub ram: Metric,
    pub disk: Metric,
    pub cpu_temp: Metric,
    pub gpu_temp: Metric,
}

/// Informasi perangkat yang tidak berubah-ubah: dipakai untuk label kartu dan
/// baris "10,9 dari 15,9 GB".
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub cpu_brand: Option<String>,
    pub cpu_cores: Option<usize>,
    pub gpu_name: Option<String>,
    pub ram_used_bytes: Option<u64>,
    pub ram_total_bytes: Option<u64>,
    pub uptime_secs: Option<u64>,
}

/// Satu partisi dengan persentase yang sudah dihitung di Rust.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskView {
    pub name: String,
    pub mount: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub free_pct: f64,
}

/// Satu potret keadaan mesin. Dikirim utuh setiap siklus; frontend tidak
/// menambal state lama.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    /// Milidetik sejak epoch (UTC).
    pub at: u64,
    pub overall: Level,
    pub metrics: Metrics,
    pub device: DeviceInfo,
    pub disks: Vec<DiskView>,
    pub cpu_temp_source: Option<TempSource>,
    pub interval_ms: u64,
}

/// Mesin status: menerima metrik mentah, mengembalikan `Snapshot`.
///
/// Setiap metrik punya `Latch` sendiri sehingga lonjakan sesaat tidak langsung
/// mengubah status. Nilai angka tetap yang terbaru — yang ditahan hanya
/// levelnya.
pub struct Engine {
    thresholds: Thresholds,
    interval_ms: u64,
    cpu: Latch,
    ram: Latch,
    disk: Latch,
    cpu_temp: Latch,
    gpu_temp: Latch,
}

impl Engine {
    pub fn new(interval_ms: u64) -> Self {
        Self::with_thresholds(interval_ms, Thresholds::default())
    }

    pub fn with_thresholds(interval_ms: u64, thresholds: Thresholds) -> Self {
        Self {
            thresholds,
            interval_ms,
            cpu: Latch::new(),
            ram: Latch::new(),
            disk: Latch::new(),
            cpu_temp: Latch::new(),
            gpu_temp: Latch::new(),
        }
    }

    /// Untuk sleep/resume: riwayat level dibuang, sampel berikutnya menjadi
    /// patokan baru.
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ram.reset();
        self.disk.reset();
        self.cpu_temp.reset();
        self.gpu_temp.reset();
    }

    pub fn update(&mut self, partial: &PartialMetrics, at: u64) -> Snapshot {
        let t = self.thresholds;

        let cpu_pct = partial.cpu_usage_pct.map(f64::from);
        let ram_pct = match (partial.ram_used_bytes, partial.ram_total_bytes) {
            (Some(used), Some(total)) if total > 0 => {
                Some(used as f64 / total as f64 * 100.0)
            }
            _ => None,
        };
        let disk_pct = worst_disk_free_pct(partial);

        let cpu_level = self.cpu.apply(classify(
            cpu_pct,
            t.cpu_warn,
            t.cpu_danger,
            Direction::HigherIsWorse,
        ));
        let ram_level = self.ram.apply(classify(
            ram_pct,
            t.ram_warn,
            t.ram_danger,
            Direction::HigherIsWorse,
        ));
        let disk_level = self.disk.apply(classify(
            disk_pct,
            t.disk_warn_free,
            t.disk_danger_free,
            Direction::LowerIsWorse,
        ));
        let cpu_temp_level = self.cpu_temp.apply(classify(
            partial.cpu_temp_c.map(f64::from),
            t.cpu_temp_warn,
            t.cpu_temp_danger,
            Direction::HigherIsWorse,
        ));
        let gpu_temp_level = self.gpu_temp.apply(classify(
            partial.gpu_temp_c.map(f64::from),
            t.gpu_temp_warn,
            t.gpu_temp_danger,
            Direction::HigherIsWorse,
        ));

        let metrics = Metrics {
            cpu: Metric::new(cpu_pct, "%", cpu_level, MetricId::Cpu),
            ram: Metric::new(ram_pct, "%", ram_level, MetricId::Ram),
            disk: Metric::new(disk_pct, "%", disk_level, MetricId::Disk),
            cpu_temp: match partial.cpu_temp_c {
                Some(celsius) => Metric::new(
                    Some(f64::from(celsius)),
                    "°C",
                    cpu_temp_level,
                    MetricId::CpuTemp,
                ),
                None => Metric::unavailable("°C", MetricId::CpuTemp),
            },
            gpu_temp: match partial.gpu_temp_c {
                Some(celsius) => Metric::new(
                    Some(f64::from(celsius)),
                    "°C",
                    gpu_temp_level,
                    MetricId::GpuTemp,
                ),
                None => Metric::unavailable("°C", MetricId::GpuTemp),
            },
        };

        let overall = crate::health::overall([
            metrics.cpu.level,
            metrics.ram.level,
            metrics.disk.level,
            metrics.cpu_temp.level,
            metrics.gpu_temp.level,
        ]);

        let disks = partial
            .disks
            .iter()
            .filter(|disk| disk.total_bytes > 0)
            .map(|disk| DiskView {
                name: disk.name.clone(),
                mount: disk.mount.clone(),
                total_bytes: disk.total_bytes,
                free_bytes: disk.free_bytes,
                free_pct: free_pct(disk.free_bytes, disk.total_bytes),
            })
            .collect();

        Snapshot {
            at,
            overall,
            metrics,
            device: DeviceInfo {
                cpu_brand: partial.cpu_brand.clone(),
                cpu_cores: partial.cpu_cores,
                gpu_name: partial.gpu_name.clone(),
                ram_used_bytes: partial.ram_used_bytes,
                ram_total_bytes: partial.ram_total_bytes,
                uptime_secs: partial.uptime_secs,
            },
            disks,
            cpu_temp_source: partial.cpu_temp_source,
            interval_ms: self.interval_ms,
        }
    }
}

/// Kartu "Ruang penyimpanan" mewakili disk yang paling penuh — itu yang paling
/// mungkin membuat pengguna kena masalah lebih dulu.
fn worst_disk_free_pct(partial: &PartialMetrics) -> Option<f64> {
    partial
        .disks
        .iter()
        .filter(|disk| disk.total_bytes > 0)
        .map(|disk| free_pct(disk.free_bytes, disk.total_bytes))
        .reduce(f64::min)
}

fn free_pct(free: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        ((free as f64 / total as f64 * 1000.0).round() / 10.0).max(0.0)
    }
}

/// Milidetik sejak epoch. Dipakai UI untuk menampilkan "diperbarui 3 detik lalu".
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::DiskInfo;

    fn disk(name: &str, total: u64, free: u64) -> DiskInfo {
        DiskInfo {
            name: name.into(),
            mount: format!("{name}\\"),
            total_bytes: total,
            free_bytes: free,
            removable: false,
        }
    }

    #[test]
    fn missing_sensors_are_unknown_not_zero() {
        let mut engine = Engine::new(2000);
        let snapshot = engine.update(&PartialMetrics::default(), 1);

        assert_eq!(snapshot.overall, Level::Unknown);
        assert_eq!(snapshot.metrics.cpu.value, None);
        assert_eq!(snapshot.metrics.cpu.level, Level::Unknown);
        assert_eq!(snapshot.metrics.cpu.reason_key, "cpu.unavailable");
        assert_eq!(snapshot.metrics.cpu_temp.reason_key, "temp.cpu.unavailable");
        assert!(snapshot.disks.is_empty());
    }

    #[test]
    fn ram_percentage_and_level() {
        let mut engine = Engine::new(2000);
        let partial = PartialMetrics {
            ram_used_bytes: Some(13_000_000_000),
            ram_total_bytes: Some(16_000_000_000),
            ..Default::default()
        };

        let snapshot = engine.update(&partial, 1);
        let ram = &snapshot.metrics.ram;

        assert_eq!(ram.level, Level::Warn);
        assert_eq!(ram.reason_key, "ram.high");
        assert_eq!(ram.value, Some(81.3));
    }

    #[test]
    fn disk_metric_uses_the_fullest_disk() {
        let mut engine = Engine::new(2000);
        let partial = PartialMetrics {
            disks: vec![
                disk("C:", 1000, 400),
                disk("D:", 1000, 50),
                disk("E:", 0, 0),
            ],
            ..Default::default()
        };

        let snapshot = engine.update(&partial, 1);

        assert_eq!(snapshot.metrics.disk.value, Some(5.0));
        assert_eq!(snapshot.metrics.disk.level, Level::Danger);
        assert_eq!(snapshot.metrics.disk.reason_key, "disk.critical");
        // Disk tanpa ukuran tidak ditampilkan sama sekali.
        assert_eq!(snapshot.disks.len(), 2);
    }

    #[test]
    fn level_change_waits_for_three_samples_but_value_is_live() {
        let mut engine = Engine::new(2000);
        let hot = PartialMetrics {
            cpu_usage_pct: Some(97.0),
            ..Default::default()
        };
        let cool = PartialMetrics {
            cpu_usage_pct: Some(10.0),
            ..Default::default()
        };

        let first = engine.update(&cool, 1);
        assert_eq!(first.metrics.cpu.level, Level::Safe);

        assert_eq!(
            engine.update(&hot, 2).metrics.cpu.level,
            Level::Safe,
            "satu sampel panas belum cukup"
        );
        assert_eq!(engine.update(&hot, 3).metrics.cpu.level, Level::Safe);

        let third = engine.update(&hot, 4);
        assert_eq!(third.metrics.cpu.level, Level::Danger);
        assert_eq!(third.metrics.cpu.value, Some(97.0));
        assert_eq!(third.overall, Level::Danger);
    }

    #[test]
    fn reset_forgets_the_previous_level() {
        let mut engine = Engine::new(2000);
        let cool = PartialMetrics {
            cpu_usage_pct: Some(5.0),
            ..Default::default()
        };
        let hot = PartialMetrics {
            cpu_usage_pct: Some(95.0),
            ..Default::default()
        };

        engine.update(&cool, 1);
        engine.reset();

        // Sampel pertama sesudah reset langsung diterima, tidak menunggu 3x.
        assert_eq!(
            engine.update(&hot, 2).metrics.cpu.level,
            Level::Danger
        );
    }

    #[test]
    fn gpu_temperature_is_reported_with_its_own_reason_key() {
        let mut engine = Engine::new(2000);
        let partial = PartialMetrics {
            gpu_temp_c: Some(84.0),
            ..Default::default()
        };

        let snapshot = engine.update(&partial, 1);
        assert_eq!(snapshot.metrics.gpu_temp.level, Level::Warn);
        assert_eq!(snapshot.metrics.gpu_temp.reason_key, "temp.gpu.warm");
        assert_eq!(snapshot.metrics.gpu_temp.value, Some(84.0));
    }
}
