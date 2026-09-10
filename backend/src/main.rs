use std::{fs, path::Path, thread, time::Duration};


#[derive(Debug, Clone, Copy)]
struct CpuSample {
    total: u64,
    idle: u64,
}

#[derive(Debug, Clone, Copy)]
struct DiskSample {
    read_sectors: u64,
    write_sectors: u64,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    state: char,
    memory_bytes: u64,
    cpu_time: u64,
    command: String,
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

fn read_disk_sample() -> DiskSample {
    let contents =
        fs::read_to_string("/proc/diskstats").expect("failed to read /proc/diskstats");

    for line in contents.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();

        if fields.len() < 10 || fields[2] != "nvme0n1" {
            continue;
        }

        return DiskSample {
            read_sectors: fields[5].parse::<u64>().expect("invalid read sector count"),
            write_sectors: fields[9].parse::<u64>().expect("invalid write sector count"),
        };
    }

    panic!("nvme0n1 not found in /proc/diskstats");
}

fn read_filesystem_usage(path: &str) -> (u64, u64, u64, f64) {
    let stat = nix::sys::statvfs::statvfs(path)
        .expect("failed to read filesystem statistics");

    let block_size = stat.fragment_size() as u64;
    let total_bytes = stat.blocks() as u64 * block_size;
    let available_bytes = stat.blocks_available() as u64 * block_size;
    let used_bytes = total_bytes.saturating_sub(available_bytes);

    let usage = if total_bytes == 0 {
        0.0
    } else {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    };

    (total_bytes, used_bytes, available_bytes, usage)
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

    let command = fs::read(format!("{proc_dir}/cmdline"))
        .ok()
        .map(|bytes| {
            bytes
                .split(|byte| *byte == 0)
                .filter(|part| !part.is_empty())
                .map(|part| String::from_utf8_lossy(part).into_owned())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();

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
        command,
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
    let previous_disk = read_disk_sample();

    thread::sleep(Duration::from_secs(1));

    let current_cpu = read_total_cpu();
    let current_processes = read_processes();
    let current_disk = read_disk_sample();

    let total_cpu = calculate_cpu_usage(previous_cpu, current_cpu);
    let system_cpu_delta = current_cpu.total - previous_cpu.total;

    let read_bytes = current_disk
        .read_sectors
        .saturating_sub(previous_disk.read_sectors)
        * 512;

    let write_bytes = current_disk
        .write_sectors
        .saturating_sub(previous_disk.write_sectors)
        * 512;

    let (memory_total, memory_used, memory_usage) = read_memory_usage();
    let (filesystem_total, filesystem_used, filesystem_available, filesystem_usage) =
        read_filesystem_usage("/");

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

    println!("\n=== DISK I/O ===");
    println!("Read: {:.2} MB/s", read_bytes as f64 / 1024.0 / 1024.0);
    println!("Write: {:.2} MB/s", write_bytes as f64 / 1024.0 / 1024.0);

    println!("\n=== FILESYSTEM ===");
    println!("Total: {:.2} GB", filesystem_total as f64 / 1024.0 / 1024.0 / 1024.0);
    println!("Used: {:.2} GB", filesystem_used as f64 / 1024.0 / 1024.0 / 1024.0);
    println!("Available: {:.2} GB", filesystem_available as f64 / 1024.0 / 1024.0 / 1024.0);
    println!("Usage: {:.2}%", filesystem_usage);

    println!("\n=== TOP PROCESSES BY CPU ===");

    for (cpu, process) in processes.iter().take(10) {
        let memory_percentage = if memory_total == 0 {
            0.0
        } else {
            (process.memory_bytes as f64 / (memory_total * 1024) as f64) * 100.0
        };

        println!(
            "{:<8} {:<25} CPU {:>6.2}% memory {:>4} MB ({:>5.2}%) state={}",
            process.pid,
            process.name,
            cpu,
            process.memory_bytes / 1024 / 1024,
            memory_percentage,
            process.state
        );

        if !process.command.is_empty() {
            println!("         command: {}", process.command);
        }
    }

    println!("\n=== MEMORY ===");
    println!("Usage: {:.2}%", memory_usage);
    println!("Used: {} MB", memory_used / 1024);
    println!("Total: {} MB", memory_total / 1024);
}
