//! Cetak beberapa `Snapshot` berurutan: bukti Fase 2 bahwa model, health
//! engine, dan poller bekerja bersama.
//!
//! Jalankan dengan:
//! `cargo run --example snapshot --manifest-path src-tauri/Cargo.toml`
//!
//! Sampel terakhir dicetak sebagai JSON — itulah kontrak yang nanti diterima
//! frontend lewat event `snapshot`.

use arxseus_monitor_lib::health::Level;
use arxseus_monitor_lib::monitor::model::{Metric, Snapshot};
use arxseus_monitor_lib::monitor::poller::Monitor;
use arxseus_monitor_lib::settings::Settings;
use std::thread;
use std::time::Duration;

const SAMPLES: usize = 4;

fn main() {
    let settings = Settings::default();
    let interval = Duration::from_millis(settings.interval_ms());
    let mut monitor = Monitor::new(&settings, false);

    println!(
        "== ArxseusMonitor snapshot ==\ninterval: {} ms, sumber suhu: {:?}\n",
        settings.interval_ms(),
        settings.temperature_source
    );

    let mut last: Option<Snapshot> = None;

    for index in 1..=SAMPLES {
        let snapshot = monitor.tick();
        print_row(index, &snapshot);
        last = Some(snapshot);

        if index < SAMPLES {
            thread::sleep(interval);
        }
    }

    if let Some(snapshot) = last {
        println!("\n== JSON yang diterima frontend (sampel terakhir) ==");
        match serde_json::to_string_pretty(&snapshot) {
            Ok(json) => println!("{json}"),
            Err(error) => println!("gagal menyerialkan snapshot: {error}"),
        }
    }
}

fn print_row(index: usize, snapshot: &Snapshot) {
    let m = &snapshot.metrics;

    println!(
        "#{index} overall={:<7} cpu {:>5} {} | ram {:>5} {} | disk {:>5} {} | cpuTemp {:>5} {} | gpuTemp {:>5} {}",
        label(snapshot.overall),
        value(&m.cpu),
        label(m.cpu.level),
        value(&m.ram),
        label(m.ram.level),
        value(&m.disk),
        label(m.disk.level),
        value(&m.cpu_temp),
        label(m.cpu_temp.level),
        value(&m.gpu_temp),
        label(m.gpu_temp.level),
    );

    if let Some(temp) = &snapshot.cpu_temp_source {
        println!("   sumber suhu CPU: {temp:?}");
    }
}

fn value(metric: &Metric) -> String {
    match metric.value {
        Some(value) => format!("{value:.1}{}", metric.unit),
        None => "—".to_string(),
    }
}

fn label(level: Level) -> &'static str {
    match level {
        Level::Safe => "safe",
        Level::Warn => "warn",
        Level::Danger => "danger",
        Level::Unknown => "unknown",
    }
}
