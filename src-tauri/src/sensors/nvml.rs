//! Suhu dan nama GPU lewat NVML (hanya mesin NVIDIA).
//!
//! NVML tidak butuh admin dan tidak butuh driver kita sendiri; DLL-nya datang
//! bersama driver NVIDIA. Di mesin non-NVIDIA, `NVML::init` gagal dan provider
//! ini melaporkan "tidak tersedia".

use crate::sensors::{PartialMetrics, SensorError, SensorProvider, SensorResult};
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::Nvml;

pub struct NvmlProvider {
    nvml: Option<Nvml>,
}

impl NvmlProvider {
    pub fn new() -> Self {
        Self {
            nvml: Nvml::init().ok(),
        }
    }
}

impl Default for NvmlProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SensorProvider for NvmlProvider {
    fn name(&self) -> &'static str {
        "nvml"
    }

    fn read(&mut self) -> SensorResult {
        let nvml = self.nvml.as_ref().ok_or_else(|| {
            SensorError::Unavailable("NVML tidak tersedia (tidak ada GPU NVIDIA / driver)".into())
        })?;

        let device = nvml.device_by_index(0)?;
        let name = device.name().ok();
        let temperature = device
            .temperature(TemperatureSensor::Gpu)
            .ok()
            .map(|t| t as f32);

        if name.is_none() && temperature.is_none() {
            return Err(SensorError::Unavailable(
                "GPU NVIDIA terdeteksi, tetapi nama/suhu tidak terbaca".into(),
            ));
        }

        Ok(PartialMetrics {
            gpu_name: name,
            gpu_temp_c: temperature,
            ..Default::default()
        })
    }
}
