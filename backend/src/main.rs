mod collectors;
mod system;

use std::{thread, time::Duration};

use collectors::cpu::{calculate_cpu_usage, read_total_cpu};
use collectors::memory::read_memory_usage;
use collectors::disk::{read_disk_sample, read_filesystem_usage};
use collectors::network::{read_interface_addresses, read_network_samples, NetworkSample};
use collectors::processes::{calculate_process_cpu, read_processes, ProcessInfo};
use collectors::temperature::read_thermal_zones;
use collectors::gpu::read_amd_gpu;
use collectors::services::read_service_statuses;


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
