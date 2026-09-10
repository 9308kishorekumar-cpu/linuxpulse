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
                message: format!(
                    "{} is reporting state '{}'.",
                    service.name, service.status
                ),
            });
        }
    }

    insights
}
