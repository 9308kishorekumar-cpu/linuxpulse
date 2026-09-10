use std::{fs, thread, time::Duration};

#[derive(Debug, Clone, Copy)]
struct CpuSample {
    total: u64,
    idle: u64,
}

fn read_cpu_sample() -> CpuSample {
    let contents =
        fs::read_to_string("/proc/stat").expect("failed to read /proc/stat");

    let line = contents
        .lines()
        .find(|line| line.starts_with("cpu "))
        .expect("CPU line not found");

    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse::<u64>().expect("invalid CPU value"))
        .collect();

    let idle = values[3] + values[4];
    let total = values.iter().sum();

    CpuSample { total, idle }
}

fn calculate_cpu_usage(previous: CpuSample, current: CpuSample) -> f64 {
    let total_delta = current.total - previous.total;
    let idle_delta = current.idle - previous.idle;

    if total_delta == 0 {
        return 0.0;
    }

    (1.0 - (idle_delta as f64 / total_delta as f64)) * 100.0
}

fn read_memory_usage() -> (u64, u64, f64) {
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
        return (0, 0, 0.0);
    }

    let used_kb = total_kb.saturating_sub(available_kb);
    let usage = (used_kb as f64 / total_kb as f64) * 100.0;

    (total_kb, used_kb, usage)
}

fn main() {
    let previous = read_cpu_sample();

    thread::sleep(Duration::from_secs(1));

    let current = read_cpu_sample();

    let cpu_usage = calculate_cpu_usage(previous, current);
    let (memory_total, memory_used, memory_usage) = read_memory_usage();

    println!("CPU usage: {:.2}%", cpu_usage);
    println!("Memory usage: {:.2}%", memory_usage);
    println!("Memory used: {} MB", memory_used / 1024);
    println!("Memory total: {} MB", memory_total / 1024);
}
