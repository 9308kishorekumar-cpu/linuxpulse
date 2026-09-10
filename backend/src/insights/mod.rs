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

    insights
}
