//! Poller: satu-satunya tempat pembacaan hardware berjalan.
//!
//! Berjalan di thread sendiri (bukan `setInterval` di React) karena WebView2
//! men-throttle timer saat jendela tersembunyi — dan jendela kita memang
//! sebagian besar tersembunyi di tray.
//!
//! Kebijakan yang diterapkan di sini:
//! - anggaran 300 ms per provider; hasil yang lebih lambat dianggap tidak sehat
//!   dan dihitung sebagai kegagalan circuit breaker;
//! - provider yang gagal 5x berturut-turut dilewati selama 5 menit;
//! - jeda > 10 detik (sleep/resume) menghapus riwayat level.

use crate::monitor::model::{now_ms, Engine, Snapshot};
use crate::sensors::breaker::Breaker;
use crate::sensors::{default_providers, PartialMetrics, SensorProvider};
use crate::settings::{Settings, TempPreference};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Anggaran waktu satu provider (`DESIGN.md` §9).
pub const READ_BUDGET: Duration = Duration::from_millis(300);
/// Anggaran pembacaan pertama tiap provider. Koneksi WMI/NVML baru dibuat di
/// sini, jadi wajar lebih lambat; batas ini sejalan dengan target startup < 2 s.
pub const STARTUP_BUDGET: Duration = Duration::from_secs(2);
/// Jeda yang dianggap sleep/resume.
pub const SLEEP_GAP: Duration = Duration::from_secs(10);
/// Panjang potongan tidur agar `stop()` tidak menunggu lama.
const SLEEP_SLICE: Duration = Duration::from_millis(100);

struct Slot {
    provider: Box<dyn SensorProvider>,
    breaker: Breaker,
    /// Sudah pernah dibaca sekali? Pembacaan pertama memakai `STARTUP_BUDGET`.
    warmed: bool,
    /// Sudah dilaporkan bermasalah, agar log tidak berulang tiap 2 detik.
    was_failing: bool,
}

pub struct Monitor {
    slots: Vec<Slot>,
    engine: Engine,
    interval: Duration,
    last_tick: Option<Instant>,
}

impl Monitor {
    pub fn new(settings: &Settings, simulate_missing_temp: bool) -> Self {
        let slots = default_providers()
            .into_iter()
            .filter(|provider| {
                provider_enabled(
                    provider.name(),
                    settings.temperature_source,
                    simulate_missing_temp,
                )
            })
            .map(|provider| Slot {
                provider,
                breaker: Breaker::new(),
                warmed: false,
                was_failing: false,
            })
            .collect();

        Self {
            slots,
            engine: Engine::new(settings.interval_ms()),
            interval: Duration::from_millis(settings.interval_ms()),
            last_tick: None,
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Nama provider yang benar-benar dipakai. Dipakai test supaya filter
    /// preferensi suhu dan mode simulasi diuji lewat jalur `new`, bukan hanya
    /// lewat `provider_enabled`.
    #[cfg(test)]
    fn provider_names(&self) -> Vec<&'static str> {
        self.slots
            .iter()
            .map(|slot| slot.provider.name())
            .collect()
    }

    /// Satu siklus baca. Tidak pernah panic: provider yang gagal hanya
    /// meninggalkan metriknya `None`.
    pub fn tick(&mut self) -> Snapshot {
        let started = Instant::now();

        if gap_requires_reset(self.last_tick.map(|last| started.duration_since(last))) {
            self.engine.reset();
        }
        self.last_tick = Some(started);

        let mut merged = PartialMetrics::default();

        for slot in &mut self.slots {
            if !slot.breaker.allows(started) {
                continue;
            }

            let read_start = Instant::now();
            let result = slot.provider.read();
            let elapsed = read_start.elapsed();
            let budget = if slot.warmed {
                READ_BUDGET
            } else {
                STARTUP_BUDGET
            };
            let healthy = result.is_ok() && elapsed <= budget;

            match &result {
                Ok(metrics) => merged.merge(metrics.clone()),
                Err(error) if !slot.was_failing => {
                    eprintln!("[monitor] {} tidak tersedia: {error}", slot.provider.name());
                }
                Err(_) => {}
            }

            if !healthy && elapsed > budget && !slot.was_failing {
                eprintln!(
                    "[monitor] {} lambat: {:.0} ms (anggaran {:.0} ms)",
                    slot.provider.name(),
                    elapsed.as_secs_f64() * 1000.0,
                    budget.as_secs_f64() * 1000.0
                );
            }

            slot.warmed = true;
            slot.was_failing = !healthy;
            slot.breaker.record(healthy, read_start + elapsed);
        }

        self.engine.update(&merged, now_ms())
    }
}

/// Provider mana yang ikut dipanggil untuk preferensi suhu tertentu.
/// NVML selalu jalan karena ia satu-satunya sumber suhu GPU.
///
/// Mode simulasi (`--simulate-missing-temp`) mematikan **semua** sumber suhu,
/// termasuk NVML, supaya demo "sensor hilang" memperlihatkan dua kartu suhu
/// tanpa angka palsu.
pub fn provider_enabled(
    name: &str,
    preference: TempPreference,
    simulate_missing_temp: bool,
) -> bool {
    if simulate_missing_temp && matches!(name, "lhm_wmi" | "thermal_wmi" | "nvml") {
        return false;
    }

    match name {
        "lhm_wmi" => matches!(preference, TempPreference::Auto | TempPreference::Lhm),
        "thermal_wmi" => matches!(preference, TempPreference::Auto | TempPreference::Acpi),
        _ => true,
    }
}

fn gap_requires_reset(gap: Option<Duration>) -> bool {
    gap.is_some_and(|gap| gap > SLEEP_GAP)
}

/// Pegangan untuk menghentikan thread poller.
pub struct MonitorHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl MonitorHandle {
    pub fn stop(&mut self) {
        self.signal();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }

    fn signal(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Drop for MonitorHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Jalankan poller di thread bernama `monitor`.
///
/// `Settings` yang dipindahkan (bukan `Monitor`) supaya inisialisasi NVML/WMI
/// terjadi di thread poller — koneksi WMI tidak boleh dibuat di thread lain.
pub fn spawn<F>(settings: Settings, simulate_missing_temp: bool, on_snapshot: F) -> MonitorHandle
where
    F: Fn(Snapshot) + Send + 'static,
{
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);

    let thread = thread::Builder::new()
        .name("monitor".into())
        .spawn(move || {
            let mut monitor = Monitor::new(&settings, simulate_missing_temp);

            while !flag.load(Ordering::Relaxed) {
                let snapshot = monitor.tick();
                on_snapshot(snapshot);

                if !sleep_unless_stopped(monitor.interval, &flag) {
                    break;
                }
            }
        })
        .expect("thread poller tidak bisa dibuat");

    MonitorHandle {
        stop,
        thread: Some(thread),
    }
}

/// Tidur dalam potongan kecil. `true` berarti masih boleh lanjut.
fn sleep_unless_stopped(interval: Duration, stop: &AtomicBool) -> bool {
    let deadline = Instant::now() + interval;

    while !stop.load(Ordering::Relaxed) {
        let now = Instant::now();
        if now >= deadline {
            return true;
        }
        thread::sleep((deadline - now).min(SLEEP_SLICE));
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_preference_filters_providers() {
        use TempPreference::{Acpi, Auto, Lhm, Off};

        assert!(provider_enabled("sysinfo", Off, false));
        assert!(provider_enabled("nvml", Off, false));

        assert!(provider_enabled("lhm_wmi", Auto, false));
        assert!(provider_enabled("lhm_wmi", Lhm, false));
        assert!(!provider_enabled("lhm_wmi", Acpi, false));
        assert!(!provider_enabled("lhm_wmi", Off, false));

        assert!(provider_enabled("thermal_wmi", Auto, false));
        assert!(provider_enabled("thermal_wmi", Acpi, false));
        assert!(!provider_enabled("thermal_wmi", Lhm, false));
        assert!(!provider_enabled("thermal_wmi", Off, false));
    }

    #[test]
    fn simulation_removes_every_temperature_source() {
        use TempPreference::Auto;

        for name in ["lhm_wmi", "thermal_wmi", "nvml"] {
            assert!(
                !provider_enabled(name, Auto, true),
                "{name} harus dilewati saat simulasi"
            );
        }

        // Metrik non-suhu tetap dibaca supaya aplikasi masih berguna.
        assert!(provider_enabled("sysinfo", Auto, true));
    }

    #[test]
    fn monitor_wiring_matches_the_simulation_flag() {
        use TempPreference::Off;

        let simulated = Monitor::new(&Settings::default(), true);
        assert_eq!(simulated.provider_names(), vec!["sysinfo"]);

        let normal = Monitor::new(&Settings::default(), false);
        let mut names = normal.provider_names();
        names.sort_unstable();
        assert_eq!(
            names,
            vec!["lhm_wmi", "nvml", "sysinfo", "thermal_wmi"],
            "settings default (auto) harus memakai keempat provider"
        );

        let settings = Settings {
            temperature_source: Off,
            ..Settings::default()
        };
        let without_cpu_temp = Monitor::new(&settings, false);
        let mut names = without_cpu_temp.provider_names();
        names.sort_unstable();
        assert_eq!(names, vec!["nvml", "sysinfo"]);
    }

    #[test]
    fn long_gap_requires_reset() {
        assert!(!gap_requires_reset(None));
        assert!(!gap_requires_reset(Some(Duration::from_secs(2))));
        assert!(!gap_requires_reset(Some(SLEEP_GAP)));
        assert!(gap_requires_reset(Some(Duration::from_secs(11))));
    }

    /// Jalur `tick()` nyata tanpa WMI: mode simulasi hanya memakai `sysinfo`,
    /// jadi test ini tidak bergantung pada sensor suhu mesin uji.
    #[test]
    fn tick_without_temperature_sources_produces_a_snapshot() {
        let mut monitor = Monitor::new(&Settings::default(), true);
        let snapshot = monitor.tick();

        assert!(snapshot.at > 0);
        assert_eq!(snapshot.interval_ms, 2000);
        // CPU/RAM hampir pasti terbaca di mesin Windows mana pun.
        assert!(snapshot.metrics.cpu.value.is_some());
        assert!(snapshot.metrics.ram.value.is_some());
        // Sumber suhu dimatikan: tidak boleh ada angka palsu 0 °C.
        assert!(snapshot.metrics.cpu_temp.value.is_none());
        assert!(snapshot.metrics.gpu_temp.value.is_none());
    }

    /// Sama seperti di atas tetapi dengan seluruh provider nyata (LHM/ACPI/NVML).
    /// Ditandai `ignore` karena hasilnya bergantung pada mesin (WMI bisa lambat,
    /// butuh admin, atau tidak ada GPU NVIDIA). Jalankan eksplisit di mesin demo:
    /// `cargo test -- --ignored tick_with_real_providers`.
    #[test]
    #[ignore = "butuh WMI/NVML nyata di mesin uji"]
    fn tick_with_real_providers_does_not_panic() {
        let mut monitor = Monitor::new(&Settings::default(), false);
        let snapshot = monitor.tick();

        assert!(snapshot.at > 0);
        assert!(snapshot.metrics.cpu.value.is_some());
        assert!(snapshot.metrics.ram.value.is_some());
    }
}
