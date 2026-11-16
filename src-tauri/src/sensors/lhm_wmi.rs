//! Suhu core CPU dari namespace WMI `ROOT\LibreHardwareMonitor`.
//!
//! Hanya tersedia bila LibreHardwareMonitor sedang berjalan dan WMI provider-nya
//! diaktifkan. Aplikasi kita tidak membundel LHM dan tidak memasang driver —
//! provider ini murni opsional.

use crate::sensors::{PartialMetrics, SensorError, SensorProvider, SensorResult, TempSource};
use serde::Deserialize;
use wmi::WMIConnection;

const NAMESPACE: &str = "ROOT\\LibreHardwareMonitor";

#[derive(Debug, Deserialize)]
#[serde(rename = "Sensor", rename_all = "PascalCase")]
struct LhmSensor {
    name: String,
    sensor_type: String,
    value: f32,
}

pub struct LhmWmiProvider;

impl LhmWmiProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LhmWmiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SensorProvider for LhmWmiProvider {
    fn name(&self) -> &'static str {
        "lhm_wmi"
    }

    fn read(&mut self) -> SensorResult {
        let connection = WMIConnection::with_namespace_path(NAMESPACE).map_err(|err| {
            SensorError::Unavailable(format!(
                "namespace {NAMESPACE} tidak tersedia (LHM tidak berjalan / WMI provider mati): {err}"
            ))
        })?;

        let sensors: Vec<LhmSensor> =
            connection.raw_query("SELECT Name, SensorType, Value FROM Sensor")?;

        let temperatures: Vec<&LhmSensor> = sensors
            .iter()
            .filter(|sensor| sensor.sensor_type.eq_ignore_ascii_case("temperature"))
            .filter(|sensor| sensor.value.is_finite() && (-40.0..150.0).contains(&sensor.value))
            .collect();

        let cpu_temp_c = temperatures
            .iter()
            .filter(|sensor| {
                let name = sensor.name.to_ascii_lowercase();
                name.contains("cpu") || name.contains("core") || name.contains("package")
            })
            .map(|sensor| sensor.value)
            .reduce(f32::max)
            .or_else(|| {
                temperatures
                    .iter()
                    .map(|sensor| sensor.value)
                    .reduce(f32::max)
            })
            .ok_or_else(|| {
                SensorError::Unavailable("LHM berjalan, tetapi tidak ada sensor suhu CPU".into())
            })?;

        Ok(PartialMetrics {
            cpu_temp_c: Some(cpu_temp_c),
            cpu_temp_source: Some(TempSource::Lhm),
            ..Default::default()
        })
    }
}
