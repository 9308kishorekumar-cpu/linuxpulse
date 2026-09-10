use std::{fs, thread, time::Duration};

#[derive(Debug, Clone, Copy)]
struct CpuSample {
    total: u64,
    idle: u64,
}

fn parse_cpu_line(line: &str) -> CpuSample {
    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse::<u64>().expect("invalid CPU value"))
        .collect();

    let idle = values[3] + values[4];
    let total = values.iter().sum();

    CpuSample { total, idle }
}

fn read_cpu_samples() -> Vec<(String, CpuSample)> {
    let contents =
        fs::read_to_string("/proc/stat").expect("failed to read /proc/stat");

    contents
        .lines()
        .filter(|line| {
            let mut parts = line.split_whitespace();

            match parts.next() {
                Some(name) if name == "cpu" => true,
                Some(name) => name.starts_with("cpu") && name[3..].parse::<usize>().is_ok(),
                None => false,
            }
        })
        .map(|line| {
            let name = line
                .split_whitespace()
                .next()
                .expect("CPU name missing")
                .to_string();

            (name, parse_cpu_line(line))
        })
        .collect()
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
    let previous = read_cpu_samples();

    thread::sleep(Duration::from_secs(1));

    let current = read_cpu_samples();

    let total_cpu = calculate_cpu_usage(previous[0].1, current[0].1);

    println!("=== CPU ===");
    println!("Total usage: {:.2}%", total_cpu);

    for ((name, previous_sample), (_, current_sample)) in
        previous.iter().skip(1).zip(current.iter().skip(1))
    {
        let usage = calculate_cpu_usage(*previous_sample, *current_sample);
        println!("{name}: {:.2}%", usage);
    }

    let (memory_total, memory_used, memory_usage) = read_memory_usage();

    println!("\n=== MEMORY ===");
    println!("Usage: {:.2}%", memory_usage);
    println!("Used: {} MB", memory_used / 1024);
    println!("Total: {} MB", memory_total / 1024);
}
