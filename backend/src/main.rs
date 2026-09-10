use std::{fs, path::Path, thread, time::Duration};

#[derive(Debug, Clone, Copy)]
struct CpuSample {
    total: u64,
    idle: u64,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    state: char,
    memory_bytes: u64,
    cpu_time: u64,
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

fn read_total_cpu() -> CpuSample {
    let contents =
        fs::read_to_string("/proc/stat").expect("failed to read /proc/stat");

    let line = contents
        .lines()
        .find(|line| line.starts_with("cpu "))
        .expect("CPU line not found");

    parse_cpu_line(line)
}

fn read_process(pid: u32) -> Option<ProcessInfo> {
    let proc_dir = format!("/proc/{pid}");

    let stat = fs::read_to_string(format!("{proc_dir}/stat")).ok()?;

    let open_paren = stat.find('(')?;
    let close_paren = stat.rfind(')')?;

    let name = stat[open_paren + 1..close_paren].to_string();

    let after_name = stat.get(close_paren + 2..)?;
    let fields: Vec<&str> = after_name.split_whitespace().collect();

    let state = fields.first()?.chars().next()?;

    // After the closing ')' the first field is state (field 3).
    // utime is field 14 and stime is field 15, so they are
    // indexes 11 and 12 in this slice.
    let utime = fields.get(11)?.parse::<u64>().ok()?;
    let stime = fields.get(12)?.parse::<u64>().ok()?;

    let status = fs::read_to_string(format!("{proc_dir}/status")).ok()?;

    let memory_kb = status
        .lines()
        .find_map(|line| {
            let value = line.strip_prefix("VmRSS:")?;
            value.split_whitespace().next()?.parse::<u64>().ok()
        })
        .unwrap_or(0);

    Some(ProcessInfo {
        pid,
        name,
        state,
        memory_bytes: memory_kb * 1024,
        cpu_time: utime + stime,
    })
}

fn read_processes() -> Vec<ProcessInfo> {
    let mut processes = Vec::new();

    let entries = fs::read_dir(Path::new("/proc")).expect("failed to read /proc");

    for entry in entries.flatten() {
        let file_name = entry.file_name();

        let Some(name) = file_name.to_str() else {
            continue;
        };

        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };

        if let Some(process) = read_process(pid) {
            processes.push(process);
        }
    }

    processes
}

fn calculate_cpu_usage(previous: CpuSample, current: CpuSample) -> f64 {
    let total_delta = current.total - previous.total;
    let idle_delta = current.idle - previous.idle;

    if total_delta == 0 {
        return 0.0;
    }

    (1.0 - (idle_delta as f64 / total_delta as f64)) * 100.0
}

fn calculate_process_cpu(
    previous: &ProcessInfo,
    current: &ProcessInfo,
    system_delta: u64,
) -> f64 {
    if system_delta == 0 || current.cpu_time < previous.cpu_time {
        return 0.0;
    }

    let process_delta = current.cpu_time - previous.cpu_time;

    (process_delta as f64 / system_delta as f64) * 100.0
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
    let previous_cpu = read_total_cpu();
    let previous_processes = read_processes();

    thread::sleep(Duration::from_secs(1));

    let current_cpu = read_total_cpu();
    let current_processes = read_processes();

    let total_cpu = calculate_cpu_usage(previous_cpu, current_cpu);
    let system_cpu_delta = current_cpu.total - previous_cpu.total;

    let previous_by_pid: std::collections::HashMap<u32, &ProcessInfo> = previous_processes
        .iter()
        .map(|process| (process.pid, process))
        .collect();

    let mut processes = Vec::new();

    for current in current_processes {
        if let Some(previous) = previous_by_pid.get(&current.pid) {
            let cpu = calculate_process_cpu(previous, &current, system_cpu_delta);

            processes.push((cpu, current));
        }
    }

    processes.sort_by(|a, b| b.0.total_cmp(&a.0));

    println!("=== CPU ===");
    println!("Total usage: {:.2}%", total_cpu);

    println!("\n=== TOP PROCESSES BY CPU ===");

    for (cpu, process) in processes.iter().take(10) {
        println!(
            "{:<8} {:<25} CPU {:>6.2}% memory {:>4} MB state={}",
            process.pid,
            process.name,
            cpu,
            process.memory_bytes / 1024 / 1024,
            process.state
        );
    }

    let (memory_total, memory_used, memory_usage) = read_memory_usage();

    println!("\n=== MEMORY ===");
    println!("Usage: {:.2}%", memory_usage);
    println!("Used: {} MB", memory_used / 1024);
    println!("Total: {} MB", memory_total / 1024);
}
