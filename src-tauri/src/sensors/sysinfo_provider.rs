//! Metrik dasar dari crate `sysinfo`: CPU, RAM, disk, uptime.
//!
//! Provider ini adalah satu-satunya yang diharapkan selalu berhasil. Kalau
//! `System` gagal dibuat, ia melaporkan error dan provider lain tetap jalan.

use crate::sensors::{DiskInfo, PartialMetrics, SensorError, SensorProvider, SensorResult};
use sysinfo::{Disks, System, MINIMUM_CPU_UPDATE_INTERVAL};

pub struct SysinfoProvider {
    sys: Option<System>,
    primed: bool,
}

impl SysinfoProvider {
    pub fn new() -> Self {
        Self {
            sys: System::new().ok(),
            primed: false,
        }
    }
}

impl Default for SysinfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SensorProvider for SysinfoProvider {
    fn name(&self) -> &'static str {
        "sysinfo"
    }

    fn read(&mut self) -> SensorResult {
        let sys = self
            .sys
            .as_mut()
            .ok_or_else(|| SensorError::Sysinfo("System tidak dapat diinisialisasi".into()))?;

        if !self.primed {
            sys.refresh_cpu_all();
            // Penggunaan CPU dihitung dari selisih dua sampel.
            std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
            self.primed = true;
        }
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let cpus = sys.cpus();
        let cpu_brand = cpus
            .iter()
            .find_map(|cpu| {
                let brand = cpu.brand().trim();
                (!brand.is_empty()).then(|| brand.to_string())
            })
            .or_else(|| {
                let arch = System::cpu_arch();
                (!arch.trim().is_empty()).then_some(arch)
            });

        let disks = Disks::new_with_refreshed_list()
            .map(|list| {
                list.iter()
                    .filter(|disk| !disk.is_removable() && disk.total_space() > 0)
                    .map(|disk| DiskInfo {
                        name: disk.name().to_string_lossy().to_string(),
                        mount: disk.mount_point().to_string_lossy().to_string(),
                        total_bytes: disk.total_space(),
                        free_bytes: disk.available_space(),
                        removable: disk.is_removable(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(PartialMetrics {
            cpu_brand,
            cpu_cores: (!cpus.is_empty()).then_some(cpus.len()),
            cpu_usage_pct: Some(sys.global_cpu_usage()),
            ram_used_bytes: Some(sys.used_memory()),
            ram_total_bytes: Some(sys.total_memory()),
            uptime_secs: System::uptime().ok(),
            disks,
            ..Default::default()
        })
    }
}
