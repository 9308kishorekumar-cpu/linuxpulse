use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct MemoryUsage {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f64,
}

pub fn read_memory_usage() -> MemoryUsage {
    let contents =
        fs::read_to_string("/proc/meminfo").expect("failed to read /proc/meminfo");

    let mut total_kb = 0;
    let mut available_kb = 0;

    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            total_kb = value
                .split_whitespace()
                .next()
                .expect("MemTotal value missing")
                .parse::<u64>()
                .expect("invalid MemTotal value");
        } else if let Some(value) = line.strip_prefix("MemAvailable:") {
            available_kb = value
                .split_whitespace()
                .next()
                .expect("MemAvailable value missing")
                .parse::<u64>()
                .expect("invalid MemAvailable value");
        }
    }

    if total_kb == 0 {
        return MemoryUsage {
            total_bytes: 0,
            used_bytes: 0,
            usage_percent: 0.0,
        };
    }

    let total_bytes = total_kb * 1024;
    let available_bytes = available_kb * 1024;
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    let usage_percent = (used_bytes as f64 / total_bytes as f64) * 100.0;

    MemoryUsage {
        total_bytes,
        used_bytes,
        usage_percent,
    }
}
