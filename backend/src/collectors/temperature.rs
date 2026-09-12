use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize)]
pub struct ThermalZone {
    pub name: String,
    pub temperature_celsius: f64,
}

pub fn read_thermal_zones() -> Vec<ThermalZone> {
    let mut zones = Vec::new();

    let Ok(entries) = fs::read_dir("/sys/class/thermal") else {
        return zones;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();

        let Some(name) = file_name.to_str() else {
            continue;
        };

        if !name.starts_with("thermal_zone") {
            continue;
        }

        let zone_path = entry.path();

        let Ok(zone_type) = fs::read_to_string(zone_path.join("type")) else {
            continue;
        };

        let Ok(raw_temperature) = fs::read_to_string(zone_path.join("temp")) else {
            continue;
        };

        let Ok(raw_temperature) = raw_temperature.trim().parse::<f64>() else {
            continue;
        };

        zones.push(ThermalZone {
            name: zone_type.trim().to_string(),
            temperature_celsius: raw_temperature / 1000.0,
        });
    }

    zones
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_valid_thermal_zones() {
        let zones = read_thermal_zones();

        for zone in zones {
            assert!(!zone.name.is_empty());
            assert!(zone.temperature_celsius > -273.15);
            assert!(zone.temperature_celsius < 200.0);
        }
    }
}

