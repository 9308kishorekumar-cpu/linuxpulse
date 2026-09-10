use serde::Serialize;

use crate::collectors::{
    disk::FilesystemUsage,
    gpu::GpuSample,
    memory::MemoryUsage,
    processes::ProcessInfo,
    services::ServiceStatus,
    temperature::ThermalZone,
};

#[derive(Debug, Clone, Serialize)]
pub struct ProcessSnapshot {
    pub cpu_percent: f64,
    pub process: ProcessInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskRate {
    pub read_bytes_per_second: u64,
    pub write_bytes_per_second: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkRate {
    pub name: String,
    pub rx_bytes_per_second: u64,
    pub tx_bytes_per_second: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemSnapshot {
    pub cpu_usage_percent: f64,
    pub memory: MemoryUsage,
    pub filesystem: FilesystemUsage,
    pub disk: DiskRate,
    pub network: Vec<NetworkRate>,
    pub network_addresses: Vec<(String, String)>,
    pub temperatures: Vec<ThermalZone>,
    pub gpu: Option<GpuSample>,
    pub services: Vec<ServiceStatus>,
    pub processes: Vec<ProcessSnapshot>,
}
