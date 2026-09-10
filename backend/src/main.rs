mod collectors;

use std::{fs, path::Path, thread, time::Duration};

use nix::ifaddrs::getifaddrs;

use collectors::cpu::{calculate_cpu_usage, read_total_cpu};
use collectors::memory::read_memory_usage;
use collectors::disk::{read_disk_sample, read_filesystem_usage};


#[derive(Debug, Clone, Copy)]
struct GpuSample {
    utilization_percent: f64,
    vram_used: u64,
    vram_total: u64,
}

#[derive(Debug, Clone)]
struct ThermalZone {
    name: String,
    temperature_celsius: f64,
}

#[derive(Debug, Clone)]
struct ServiceStatus {
    name: String,
    active: bool,
    failed: bool,
    status: String,
}

#[derive(Debug, Clone)]
struct NetworkSample {
    name: String,
    rx_bytes: u64,
    tx_bytes: u64,
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

fn read_interface_addresses() -> Vec<(String, String)> {
    let mut addresses = Vec::new();

    let Ok(interfaces) = getifaddrs() else {
        return addresses;
    };

    for interface in interfaces {
        let Some(address) = interface.address else {
            continue;
        };

        let address = if let Some(inet) = address.as_sockaddr_in() {
            inet.ip().to_string()
        } else if let Some(inet6) = address.as_sockaddr_in6() {
            inet6.ip().to_string()
        } else {
            continue;
        };

        addresses.push((interface.interface_name, address));
    }

    addresses
}

fn read_amd_gpu() -> Option<GpuSample> {
    let device = Path::new("/sys/class/drm/card2/device");

    let utilization = fs::read_to_string(device.join("gpu_busy_percent"))
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()?;

    let vram_used = fs::read_to_string(device.join("mem_info_vram_used"))
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;

    let vram_total = fs::read_to_string(device.join("mem_info_vram_total"))
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;

    Some(GpuSample {
        utilization_percent: utilization,
        vram_used,
        vram_total,
    })
}

fn read_service_statuses() -> Vec<ServiceStatus> {
    let services = [
        ("NetworkManager.service", false),
        ("pipewire.service", true),
        ("pipewire-pulse.service", true),
        ("wireplumber.service", true),
    ];

    let mut statuses = Vec::new();

    for (service, user_service) in services {
        let mut command = std::process::Command::new("systemctl");

        if user_service {
            command.arg("--user");
        }

        let output = command
            .args([
                "show",
                service,
                "-p",
                "LoadState",
                "-p",
                "ActiveState",
                "-p",
                "SubState",
            ])
            .output();

        let Ok(output) = output else {
            continue;
        };

        if !output.status.success() {
            continue;
        }

        let text = String::from_utf8_lossy(&output.stdout);

        let mut load_state = "";
        let mut active_state = "";
        let mut sub_state = "";

        for line in text.lines() {
            if let Some(value) = line.strip_prefix("LoadState=") {
                load_state = value;
            } else if let Some(value) = line.strip_prefix("ActiveState=") {
                active_state = value;
            } else if let Some(value) = line.strip_prefix("SubState=") {
                sub_state = value;
            }
        }

        statuses.push(ServiceStatus {
            name: service.to_string(),
            active: active_state == "active",
            failed: load_state == "not-found" || active_state == "failed",
            status: sub_state.to_string(),
        });
    }

    statuses
}

fn read_thermal_zones() -> Vec<ThermalZone> {
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

fn read_network_samples() -> Vec<NetworkSample> {
    let contents =
        fs::read_to_string("/proc/net/dev").expect("failed to read /proc/net/dev");

    let mut interfaces = Vec::new();

    for line in contents.lines().skip(2) {
        let Some((name, values)) = line.split_once(':') else {
            continue;
        };

        let name = name.trim().to_string();
        let fields: Vec<&str> = values.split_whitespace().collect();

        if fields.len() < 9 {
            continue;
        }

        let rx_bytes = fields[0].parse::<u64>().unwrap_or(0);
        let tx_bytes = fields[8].parse::<u64>().unwrap_or(0);

        interfaces.push(NetworkSample {
            name,
            rx_bytes,
            tx_bytes,
        });
    }

    interfaces
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

fn main() {
    let previous_cpu = read_total_cpu();
    let previous_processes = read_processes();
    let previous_disk = read_disk_sample();
    let previous_network = read_network_samples();

    thread::sleep(Duration::from_secs(1));

    let current_cpu = read_total_cpu();
    let current_processes = read_processes();
    let current_disk = read_disk_sample();
    let current_network = read_network_samples();

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

    let previous_network_by_name: std::collections::HashMap<&str, &NetworkSample> =
        previous_network
            .iter()
            .map(|interface| (interface.name.as_str(), interface))
            .collect();

    let mut network_rates = Vec::new();

    for current in &current_network {
        if let Some(previous) = previous_network_by_name.get(current.name.as_str()) {
            let rx_bytes = current.rx_bytes.saturating_sub(previous.rx_bytes);
            let tx_bytes = current.tx_bytes.saturating_sub(previous.tx_bytes);

            network_rates.push((current.name.as_str(), rx_bytes, tx_bytes));
        }
    }

    let memory = read_memory_usage();
    let memory_total = memory.total_bytes;
    let memory_used = memory.used_bytes;
    let memory_usage = memory.usage_percent;
    let filesystem = read_filesystem_usage("/");
    let filesystem_total = filesystem.total_bytes;
    let filesystem_used = filesystem.used_bytes;
    let filesystem_available = filesystem.available_bytes;
    let filesystem_usage = filesystem.usage_percent;
    let interface_addresses = read_interface_addresses();
    let thermal_zones = read_thermal_zones();
    let gpu = read_amd_gpu();
    let services = read_service_statuses();

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

    println!("\n=== NETWORK ===");

    for (name, rx_bytes, tx_bytes) in &network_rates {
        println!(
            "{:<8} download {:>8.2} KB/s upload {:>8.2} KB/s",
            name,
            *rx_bytes as f64 / 1024.0,
            *tx_bytes as f64 / 1024.0
        );

        for (interface_name, address) in &interface_addresses {
            if interface_name == name {
                println!("         IP: {}", address);
            }
        }
    }

    println!("\n=== THERMAL ===");

    for zone in &thermal_zones {
        println!(
            "{:<12} {:>6.1}°C",
            zone.name,
            zone.temperature_celsius
        );
    }

    println!("\n=== GPU ===");

    if let Some(gpu) = gpu {
        let vram_used_mb = gpu.vram_used as f64 / 1024.0 / 1024.0;
        let vram_total_mb = gpu.vram_total as f64 / 1024.0 / 1024.0;

        println!("AMD GPU usage: {:.1}%", gpu.utilization_percent);
        println!(
            "VRAM: {:.0} / {:.0} MB",
            vram_used_mb,
            vram_total_mb
        );
    } else {
        println!("AMD GPU telemetry unavailable");
    }

    println!("\n=== SYSTEM HEALTH ===");

    for service in &services {
        let state = if service.failed {
            "FAILED"
        } else if service.active {
            "HEALTHY"
        } else {
            "INACTIVE"
        };

        println!(
            "{:<26} {:<8} ({})",
            service.name,
            state,
            service.status
        );
    }

    println!("\n=== TOP PROCESSES BY CPU ===");

    for (cpu, process) in processes.iter().take(10) {
        let memory_percentage = if memory_total == 0 {
            0.0
        } else {
            (process.memory_bytes as f64 / memory_total as f64) * 100.0
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
    println!("Used: {} MB", memory_used / 1024 / 1024);
    println!("Total: {} MB", memory_total / 1024 / 1024);
}
