//! Probe sensor: bukti bahwa pembacaan hardware benar-benar jalan di mesin ini.
//!
//! Jalankan dengan:
//! `cargo run --example probe --manifest-path src-tauri/Cargo.toml`
//!
//! Dua siklus dibaca: "cold" (koneksi baru, seperti saat aplikasi dibuka) dan
//! "warm" (provider dipakai ulang, seperti saat polling 2 detik). Angka warm
//! yang dipakai untuk memutuskan interval polling.

use arxseus_monitor_lib::sensors::{
    collect_once, default_providers, PartialMetrics, Probe, TempSource,
};

fn main() {
    println!("== ArxseusMonitor sensor probe ==");
    println!("platform: {}\n", std::env::consts::OS);

    let mut providers = default_providers();

    let cold = collect_once(&mut providers);
    print_reports("cold (startup)", &cold);

    let warm = collect_once(&mut providers);
    print_reports("warm (polling)", &warm);

    println!("\n== gabungan (warm) ==");
    print_metrics(&warm.merged);

    println!("\n== keputusan strategi suhu ==");
    match warm.merged.cpu_temp_source {
        Some(TempSource::Lhm) => println!("cpu: LHM  -> suhu core akurat tersedia"),
        Some(TempSource::Acpi) => {
            println!("cpu: ACPI -> thermal zone sistem, bisa bukan suhu core")
        }
        Some(TempSource::Nvml) => println!("cpu: NVML -> tidak lazim untuk CPU"),
        None => println!("cpu: TIDAK TERDETEKSI -> kunci mode best-effort"),
    }
    if warm.merged.gpu_temp_c.is_some() {
        println!("gpu: NVML -> suhu GPU tersedia");
    } else {
        println!("gpu: TIDAK TERDETEKSI");
    }
}

fn print_reports(title: &str, probe: &Probe) {
    println!("-- provider: {title} --");
    let mut total_ms = 0.0;

    for report in &probe.reports {
        let ms = report.elapsed.as_secs_f64() * 1000.0;
        total_ms += ms;
        let state = if report.result.is_ok() { "ok  " } else { "fail" };
        println!("[{state}] {:<12} {ms:>7.1} ms", report.name);

        if let Err(error) = &report.result {
            println!("       -> {error}");
        }
    }

    println!("       total        {total_ms:>7.1} ms\n");
}

fn print_metrics(metrics: &PartialMetrics) {
    line("cpu_brand", metrics.cpu_brand.as_deref());
    line("cpu_cores", metrics.cpu_cores.map(|v| v.to_string()).as_deref());
    line(
        "cpu_usage",
        metrics.cpu_usage_pct.map(|v| format!("{v:.1} %")).as_deref(),
    );
    line(
        "ram",
        match (metrics.ram_used_bytes, metrics.ram_total_bytes) {
            (Some(used), Some(total)) if total > 0 => Some(format!(
                "{} / {} ({:.0}%)",
                human_bytes(used),
                human_bytes(total),
                used as f64 / total as f64 * 100.0
            )),
            _ => None,
        }
        .as_deref(),
    );
    line("uptime", metrics.uptime_secs.map(human_uptime).as_deref());
    line(
        "cpu_temp",
        metrics
            .cpu_temp_c
            .map(|v| format!("{v:.1} C ({})", source_label(metrics.cpu_temp_source)))
            .as_deref(),
    );
    line("gpu_name", metrics.gpu_name.as_deref());
    line(
        "gpu_temp",
        metrics.gpu_temp_c.map(|v| format!("{v:.1} C")).as_deref(),
    );

    for disk in &metrics.disks {
        let free_pct = if disk.total_bytes > 0 {
            disk.free_bytes as f64 / disk.total_bytes as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "       disk {:<6} {:>9} bebas dari {:>9} ({free_pct:>3.0}%)",
            disk.mount,
            human_bytes(disk.free_bytes),
            human_bytes(disk.total_bytes)
        );
    }
}

fn line(label: &str, value: Option<&str>) {
    match value {
        Some(value) if !value.is_empty() => println!("       {label:<14}: {value}"),
        _ => println!("       {label:<14}: -"),
    }
}

fn source_label(source: Option<TempSource>) -> &'static str {
    match source {
        Some(TempSource::Lhm) => "lhm",
        Some(TempSource::Acpi) => "acpi",
        Some(TempSource::Nvml) => "nvml",
        None => "?",
    }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

fn human_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if days > 0 {
        format!("{days}h {hours}j")
    } else {
        format!("{hours}j {minutes}m")
    }
}
