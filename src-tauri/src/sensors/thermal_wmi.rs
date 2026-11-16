//! Suhu sistem lewat WMI `MSAcpi_ThermalZoneTemperature` (namespace `ROOT\WMI`).
//!
//! Best-effort: banyak laptop tidak mengekspos thermal zone, atau hanya
//! mengekspos suhu motherboard, bukan suhu core CPU. Nilai dari WMI dalam
//! sepersepuluh Kelvin.

use crate::sensors::{
    wmi_message, PartialMetrics, SensorError, SensorProvider, SensorResult, TempSource,
};
use serde::Deserialize;
use wmi::WMIConnection;

#[derive(Debug, Deserialize)]
#[serde(
    rename = "MSAcpi_ThermalZoneTemperature",
    rename_all = "PascalCase"
)]
struct ThermalZone {
    current_temperature: u32,
}

/// WMI mengembalikan suhu dalam sepersepuluh Kelvin.
pub fn deci_kelvin_to_celsius(raw: u32) -> f32 {
    raw as f32 / 10.0 - 273.15
}

pub struct ThermalWmiProvider;

impl ThermalWmiProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThermalWmiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SensorProvider for ThermalWmiProvider {
    fn name(&self) -> &'static str {
        "thermal_wmi"
    }

    fn read(&mut self) -> SensorResult {
        let connection = WMIConnection::with_namespace_path("ROOT\\WMI")
            .map_err(|error| SensorError::Unavailable(wmi_message("buka ROOT\\WMI", &error)))?;

        let rows: Vec<ThermalZone> = connection
            .raw_query("SELECT CurrentTemperature FROM MSAcpi_ThermalZoneTemperature")
            .map_err(|error| {
                SensorError::Unavailable(wmi_message(
                    "baca MSAcpi_ThermalZoneTemperature",
                    &error,
                ))
            })?;

        if rows.is_empty() {
            return Err(SensorError::Unavailable(
                "tidak ada thermal zone ACPI di perangkat ini".into(),
            ));
        }

        let hottest = rows
            .iter()
            .map(|row| deci_kelvin_to_celsius(row.current_temperature))
            .filter(|celsius| celsius.is_finite() && (-40.0..150.0).contains(celsius))
            .reduce(f32::max)
            .ok_or_else(|| {
                SensorError::Unavailable(format!(
                    "{} thermal zone ACPI ditemukan, tetapi nilainya tidak masuk akal",
                    rows.len()
                ))
            })?;

        Ok(PartialMetrics {
            cpu_temp_c: Some(hottest),
            cpu_temp_source: Some(TempSource::Acpi),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_deci_kelvin_to_celsius() {
        assert!((deci_kelvin_to_celsius(3000) - 26.85).abs() < 0.01);
        assert!((deci_kelvin_to_celsius(2732) - 0.05).abs() < 0.01);
    }
}
