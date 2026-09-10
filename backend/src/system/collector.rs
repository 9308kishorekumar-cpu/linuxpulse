use std::collections::HashMap;

use crate::collectors::{
    cpu::{calculate_cpu_usage, read_total_cpu, CpuSample},
    disk::{read_disk_sample, read_filesystem_usage, DiskSample},
    gpu::read_amd_gpu,
    memory::read_memory_usage,
    network::{read_interface_addresses, read_network_samples, NetworkSample},
    processes::{calculate_process_cpu, read_processes, ProcessInfo},
    services::read_service_statuses,
    temperature::read_thermal_zones,
};

use super::snapshot::{DiskRate, NetworkRate, ProcessSnapshot, SystemSnapshot};

pub struct SnapshotCollector {
    previous_cpu: CpuSample,
    previous_disk: DiskSample,
    previous_network: Vec<NetworkSample>,
    previous_processes: Vec<ProcessInfo>,
}

impl SnapshotCollector {
    pub fn new() -> Self {
        Self {
            previous_cpu: read_total_cpu(),
            previous_disk: read_disk_sample(),
            previous_network: read_network_samples(),
            previous_processes: read_processes(),
        }
    }

    pub fn collect(&mut self) -> SystemSnapshot {
        let current_cpu = read_total_cpu();
        let current_disk = read_disk_sample();
        let current_network = read_network_samples();
        let current_processes = read_processes();

        let cpu_usage_percent = calculate_cpu_usage(self.previous_cpu, current_cpu);
        let system_cpu_delta = current_cpu.total.saturating_sub(self.previous_cpu.total);

        let disk = DiskRate {
            read_bytes_per_second: current_disk
                .read_sectors
                .saturating_sub(self.previous_disk.read_sectors)
                * 512,
            write_bytes_per_second: current_disk
                .write_sectors
                .saturating_sub(self.previous_disk.write_sectors)
                * 512,
        };

        let previous_network_by_name: HashMap<&str, &NetworkSample> = self
            .previous_network
            .iter()
            .map(|interface| (interface.name.as_str(), interface))
            .collect();

        let network = current_network
            .iter()
            .filter_map(|current| {
                let previous = previous_network_by_name.get(current.name.as_str())?;

                Some(NetworkRate {
                    name: current.name.clone(),
                    rx_bytes_per_second: current.rx_bytes.saturating_sub(previous.rx_bytes),
                    tx_bytes_per_second: current.tx_bytes.saturating_sub(previous.tx_bytes),
                })
            })
            .collect();

        let previous_processes_by_pid: HashMap<u32, &ProcessInfo> = self
            .previous_processes
            .iter()
            .map(|process| (process.pid, process))
            .collect();

        let mut processes = current_processes
            .iter()
            .filter_map(|current| {
                let previous = previous_processes_by_pid.get(&current.pid)?;

                Some(ProcessSnapshot {
                    cpu_percent: calculate_process_cpu(
                        previous,
                        current,
                        system_cpu_delta,
                    ),
                    process: current.clone(),
                })
            })
            .collect::<Vec<_>>();

        processes.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent));
        processes.truncate(10);

        self.previous_cpu = current_cpu;
        self.previous_disk = current_disk;
        self.previous_network = current_network.clone();
        self.previous_processes = current_processes;

        SystemSnapshot {
            cpu_usage_percent,
            memory: read_memory_usage(),
            filesystem: read_filesystem_usage("/"),
            disk,
            network,
            network_addresses: read_interface_addresses(),
            temperatures: read_thermal_zones(),
            gpu: read_amd_gpu(),
            services: read_service_statuses(),
            processes,
        }
    }
}
