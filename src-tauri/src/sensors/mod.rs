//! Lapisan sensor.
//!
//! Setiap provider membaca satu kelompok metrik dan boleh gagal sendiri-sendiri.
//! Hasilnya digabung dengan prioritas tetap: provider yang lebih dipercaya
//! dijalankan lebih dulu, dan field yang sudah terisi tidak ditimpa.

pub mod breaker;
pub mod lhm_wmi;
pub mod nvml;
pub mod sysinfo_provider;
pub mod thermal_wmi;

use serde::Serialize;
use std::time::{Duration, Instant};

/// Sumber data suhu. Ditampilkan di footer UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TempSource {
    Lhm,
    Acpi,
    Nvml,
}

/// Satu partisi / volume yang dilaporkan ke UI.
#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub removable: bool,
}

/// Metrik parsial. Semua field opsional: `None` berarti "tidak terdeteksi",
/// bukan nol.
#[derive(Debug, Default, Clone, Serialize)]
pub struct PartialMetrics {
    pub cpu_brand: Option<String>,
    pub cpu_cores: Option<usize>,
    pub cpu_usage_pct: Option<f32>,
    pub ram_used_bytes: Option<u64>,
    pub ram_total_bytes: Option<u64>,
    pub uptime_secs: Option<u64>,
    pub cpu_temp_c: Option<f32>,
    pub cpu_temp_source: Option<TempSource>,
    pub gpu_name: Option<String>,
    pub gpu_temp_c: Option<f32>,
    pub disks: Vec<DiskInfo>,
}

impl PartialMetrics {
    /// Isi hanya field yang masih kosong. Provider yang dipanggil lebih dulu
    /// menang.
    pub fn merge(&mut self, other: PartialMetrics) {
        fn fill<T>(target: &mut Option<T>, value: Option<T>) {
            if target.is_none() {
                *target = value;
            }
        }

        fill(&mut self.cpu_brand, other.cpu_brand);
        fill(&mut self.cpu_cores, other.cpu_cores);
        fill(&mut self.cpu_usage_pct, other.cpu_usage_pct);
        fill(&mut self.ram_used_bytes, other.ram_used_bytes);
        fill(&mut self.ram_total_bytes, other.ram_total_bytes);
        fill(&mut self.uptime_secs, other.uptime_secs);
        fill(&mut self.cpu_temp_c, other.cpu_temp_c);
        fill(&mut self.cpu_temp_source, other.cpu_temp_source);
        fill(&mut self.gpu_name, other.gpu_name);
        fill(&mut self.gpu_temp_c, other.gpu_temp_c);

        if self.disks.is_empty() {
            self.disks = other.disks;
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SensorError {
    #[error("WMI: {0}")]
    Wmi(#[from] wmi::WMIError),
    #[error("NVML: {0}")]
    Nvml(#[from] nvml_wrapper::error::NvmlError),
    #[error("sysinfo: {0}")]
    Sysinfo(String),
    #[error("{0}")]
    Unavailable(String),
}

pub type SensorResult = Result<PartialMetrics, SensorError>;

/// Terjemahkan HRESULT WMI yang sering muncul menjadi penyebab yang bisa
/// ditindaklanjuti. `None` berarti tidak ada pemetaan khusus.
pub fn wmi_error_hint(error: &wmi::WMIError) -> Option<&'static str> {
    use wmi::WMIError::HResultError;

    match error {
        HResultError { hres } => match *hres as u32 {
            0x8004_1003 => Some("butuh izin administrator"),
            0x8004_100E => Some("namespace tidak tersedia"),
            0x8004_1010 => Some("class tidak tersedia"),
            0x8004_1011 => Some("provider tidak bisa dijalankan"),
            _ => None,
        },
        _ => None,
    }
}

/// Pesan error WMI lengkap dengan petunjuk penyebab.
pub fn wmi_message(context: &str, error: &wmi::WMIError) -> String {
    match wmi_error_hint(error) {
        Some(hint) => format!("{context}: {hint} ({error})"),
        None => format!("{context}: {error}"),
    }
}

/// Kontrak semua provider. `&mut self` karena sebagian provider menyimpan
/// koneksi/handle.
pub trait SensorProvider {
    fn name(&self) -> &'static str;
    fn read(&mut self) -> SensorResult;
}

/// Hasil satu provider, termasuk durasi — dipakai untuk memutuskan interval
/// polling di Fase 2.
#[derive(Debug)]
pub struct ProviderReport {
    pub name: &'static str,
    pub elapsed: Duration,
    pub result: SensorResult,
}

#[derive(Debug)]
pub struct Probe {
    pub merged: PartialMetrics,
    pub reports: Vec<ProviderReport>,
}

/// Urutan = prioritas penggabungan.
pub fn default_providers() -> Vec<Box<dyn SensorProvider>> {
    vec![
        Box::new(sysinfo_provider::SysinfoProvider::new()),
        Box::new(lhm_wmi::LhmWmiProvider::new()),
        Box::new(thermal_wmi::ThermalWmiProvider::new()),
        Box::new(nvml::NvmlProvider::new()),
    ]
}

/// Satu siklus baca. Menerima provider yang sudah ada supaya bisa dipakai
/// berulang tanpa membangun ulang koneksi (penting untuk poller).
pub fn collect_once(providers: &mut [Box<dyn SensorProvider>]) -> Probe {
    let mut merged = PartialMetrics::default();
    let mut reports = Vec::with_capacity(providers.len());

    for provider in providers.iter_mut() {
        let start = Instant::now();
        let result = provider.read();
        let elapsed = start.elapsed();

        if let Ok(metrics) = &result {
            merged.merge(metrics.clone());
        }

        reports.push(ProviderReport {
            name: provider.name(),
            elapsed,
            result,
        });
    }

    Probe { merged, reports }
}

/// Probe sekali pakai (cold start).
pub fn probe() -> Probe {
    collect_once(&mut default_providers())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_keeps_first_provider_value() {
        let mut merged = PartialMetrics {
            cpu_temp_c: Some(61.0),
            cpu_temp_source: Some(TempSource::Lhm),
            ..Default::default()
        };

        merged.merge(PartialMetrics {
            cpu_temp_c: Some(80.0),
            cpu_temp_source: Some(TempSource::Acpi),
            cpu_usage_pct: Some(12.5),
            ..Default::default()
        });

        assert_eq!(merged.cpu_temp_c, Some(61.0));
        assert_eq!(merged.cpu_temp_source, Some(TempSource::Lhm));
        assert_eq!(merged.cpu_usage_pct, Some(12.5));
    }

    #[test]
    fn merge_does_not_overwrite_disks() {
        let mut merged = PartialMetrics {
            disks: vec![DiskInfo {
                name: "C:".into(),
                mount: "C:\\".into(),
                total_bytes: 100,
                free_bytes: 50,
                removable: false,
            }],
            ..Default::default()
        };

        merged.merge(PartialMetrics {
            disks: vec![DiskInfo {
                name: "D:".into(),
                mount: "D:\\".into(),
                total_bytes: 200,
                free_bytes: 20,
                removable: false,
            }],
            ..Default::default()
        });

        assert_eq!(merged.disks.len(), 1);
        assert_eq!(merged.disks[0].name, "C:");
    }

    #[test]
    fn maps_common_wmi_hresults() {
        let access_denied = wmi::WMIError::HResultError {
            hres: 0x8004_1003u32 as i32,
        };
        assert_eq!(
            wmi_error_hint(&access_denied),
            Some("butuh izin administrator")
        );

        let invalid_namespace = wmi::WMIError::HResultError {
            hres: 0x8004_100Eu32 as i32,
        };
        assert_eq!(
            wmi_error_hint(&invalid_namespace),
            Some("namespace tidak tersedia")
        );
        assert!(wmi_message("buka ROOT\\WMI", &access_denied).contains("administrator"));
    }
}
