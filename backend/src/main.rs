mod collectors;
mod system;

use std::{thread, time::Duration};

use system::collector::SnapshotCollector;


fn main() {
    let mut collector = SnapshotCollector::new();

    thread::sleep(Duration::from_secs(1));

    let snapshot = collector.collect();

    println!("=== CPU ===");
    println!("Total usage: {:.2}%", snapshot.cpu_usage_percent);

    println!("\n=== DISK I/O ===");
    println!(
        "Read: {:.2} MB/s",
        snapshot.disk.read_bytes_per_second as f64 / 1024.0 / 1024.0
    );
    println!(
        "Write: {:.2} MB/s",
        snapshot.disk.write_bytes_per_second as f64 / 1024.0 / 1024.0
    );

    println!("\n=== FILESYSTEM ===");
    println!(
        "Total: {:.2} GB",
        snapshot.filesystem.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!(
        "Used: {:.2} GB",
        snapshot.filesystem.used_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!(
        "Available: {:.2} GB",
        snapshot.filesystem.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!("Usage: {:.2}%", snapshot.filesystem.usage_percent);

    println!("\n=== NETWORK ===");

    let addresses = collectors::network::read_interface_addresses();

    for interface in &snapshot.network {
        println!(
            "{:<8} download {:>8.2} KB/s upload {:>8.2} KB/s",
            interface.name,
            interface.rx_bytes_per_second as f64 / 1024.0,
            interface.tx_bytes_per_second as f64 / 1024.0
        );

        for (interface_name, address) in &addresses {
            if interface_name == &interface.name {
                println!("         IP: {}", address);
            }
        }
    }

    println!("\n=== THERMAL ===");

    for zone in &snapshot.temperatures {
        println!(
            "{:<12} {:>6.1}°C",
            zone.name,
            zone.temperature_celsius
        );
    }

    println!("\n=== GPU ===");

    if let Some(gpu) = &snapshot.gpu {
        println!(
            "AMD GPU usage: {:.1}%",
            gpu.utilization_percent
        );
        println!(
            "VRAM: {:.0} / {:.0} MB",
            gpu.vram_used as f64 / 1024.0 / 1024.0,
            gpu.vram_total as f64 / 1024.0 / 1024.0
        );
    } else {
        println!("AMD GPU telemetry unavailable");
    }

    println!("\n=== SYSTEM HEALTH ===");

    for service in &snapshot.services {
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

    for process in &snapshot.processes {
        let memory_percentage = if snapshot.memory.total_bytes == 0 {
            0.0
        } else {
            (process.process.memory_bytes as f64 / snapshot.memory.total_bytes as f64) * 100.0
        };

        println!(
            "{:<8} {:<25} CPU {:>6.2}% memory {:>4} MB ({:>5.2}%) state={}",
            process.process.pid,
            process.process.name,
            process.cpu_percent,
            process.process.memory_bytes / 1024 / 1024,
            memory_percentage,
            process.process.state
        );

        if !process.process.command.is_empty() {
            println!("         command: {}", process.process.command);
        }
    }

    println!("\n=== MEMORY ===");
    println!("Usage: {:.2}%", snapshot.memory.usage_percent);
    println!("Used: {} MB", snapshot.memory.used_bytes / 1024 / 1024);
    println!("Total: {} MB", snapshot.memory.total_bytes / 1024 / 1024);
}
