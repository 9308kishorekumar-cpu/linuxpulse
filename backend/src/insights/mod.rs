use serde::Serialize;

use crate::system::snapshot::SystemSnapshot;

#[derive(Debug, Clone, Serialize)]
pub struct Insight {
    pub severity: String,
    pub title: String,
    pub message: String,
}

pub fn analyze(snapshot: &SystemSnapshot) -> Vec<Insight> {
    let mut insights = Vec::new();

    if snapshot.cpu_usage_percent >= 85.0 {
        insights.push(Insight {
            severity: "warning".to_string(),
            title: "High CPU usage".to_string(),
            message: format!(
                "CPU usage is {:.1}%. One or more processes may be causing sustained system load.",
                snapshot.cpu_usage_percent
            ),
        });
    }

    if snapshot.memory.usage_percent >= 85.0 {
        insights.push(Insight {
            severity: "warning".to_string(),
            title: "High memory usage".to_string(),
            message: format!(
                "Memory usage is {:.1}%. Applications may be consuming most available RAM.",
                snapshot.memory.usage_percent
            ),
        });
    }

    if snapshot.filesystem.usage_percent >= 90.0 {
        insights.push(Insight {
            severity: "warning".to_string(),
            title: "Disk space is low".to_string(),
            message: format!(
                "Filesystem usage is {:.1}%. Only {:.1} GB is available.",
                snapshot.filesystem.usage_percent,
                snapshot.filesystem.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0
            ),
        });
    }

    for zone in &snapshot.temperatures {
        if zone.temperature_celsius >= 85.0 {
            insights.push(Insight {
                severity: "critical".to_string(),
                title: format!("High temperature: {}", zone.name),
                message: format!(
                    "{} is at {:.1}°C. Check system load and cooling.",
                    zone.name, zone.temperature_celsius
                ),
            });
        }
    }

    for service in &snapshot.services {
        if service.failed {
            insights.push(Insight {
                severity: "critical".to_string(),
                title: format!("Service failed: {}", service.name),
                message: format!("{} is reporting state '{}'.", service.name, service.status),
            });
        }
    }

    insights
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collectors::{
        disk::FilesystemUsage, memory::MemoryUsage, services::ServiceStatus,
        temperature::ThermalZone,
    };
    use crate::system::snapshot::{DiskRate, NetworkRate, ProcessSnapshot};

    fn base_snapshot() -> SystemSnapshot {
        SystemSnapshot {
            cpu_usage_percent: 20.0,
            memory: MemoryUsage {
                total_bytes: 16 * 1024 * 1024 * 1024,
                used_bytes: 4 * 1024 * 1024 * 1024,
                usage_percent: 25.0,
            },
            filesystem: FilesystemUsage {
                total_bytes: 100 * 1024 * 1024 * 1024,
                used_bytes: 50 * 1024 * 1024 * 1024,
                available_bytes: 50 * 1024 * 1024 * 1024,
                usage_percent: 50.0,
            },
            disk: DiskRate {
                read_bytes_per_second: 0,
                write_bytes_per_second: 0,
            },
            network: Vec::<NetworkRate>::new(),
            network_addresses: Vec::new(),
            temperatures: Vec::new(),
            gpu: None,
            services: Vec::new(),
            processes: Vec::<ProcessSnapshot>::new(),
        }
    }

    #[test]
    fn detects_high_cpu_usage() {
        let mut snapshot = base_snapshot();
        snapshot.cpu_usage_percent = 85.0;

        let insights = analyze(&snapshot);

        assert_eq!(insights.len(), 1);
        assert_eq!(insights[0].severity, "warning");
        assert_eq!(insights[0].title, "High CPU usage");
    }

    #[test]
    fn detects_high_memory_usage() {
        let mut snapshot = base_snapshot();
        snapshot.memory.usage_percent = 85.0;

        let insights = analyze(&snapshot);

        assert_eq!(insights.len(), 1);
        assert_eq!(insights[0].severity, "warning");
        assert_eq!(insights[0].title, "High memory usage");
    }

    #[test]
    fn detects_low_disk_space() {
        let mut snapshot = base_snapshot();
        snapshot.filesystem.usage_percent = 90.0;

        let insights = analyze(&snapshot);

        assert_eq!(insights.len(), 1);
        assert_eq!(insights[0].severity, "warning");
        assert_eq!(insights[0].title, "Disk space is low");
    }

    #[test]
    fn detects_high_temperature() {
        let mut snapshot = base_snapshot();
        snapshot.temperatures.push(ThermalZone {
            name: "cpu".to_string(),
            temperature_celsius: 85.0,
        });

        let insights = analyze(&snapshot);

        assert_eq!(insights.len(), 1);
        assert_eq!(insights[0].severity, "critical");
        assert_eq!(insights[0].title, "High temperature: cpu");
    }

    #[test]
    fn detects_failed_service() {
        let mut snapshot = base_snapshot();
        snapshot.services.push(ServiceStatus {
            name: "example.service".to_string(),
            active: false,
            failed: true,
            status: "failed".to_string(),
        });

        let insights = analyze(&snapshot);

        assert_eq!(insights.len(), 1);
        assert_eq!(insights[0].severity, "critical");
        assert_eq!(insights[0].title, "Service failed: example.service");
    }

    #[test]
    fn healthy_snapshot_has_no_insights() {
        let snapshot = base_snapshot();

        let insights = analyze(&snapshot);

        assert!(insights.is_empty());
    }
}
