use crate::collectors::{
    disk::FilesystemUsage,
    gpu::GpuSample,
    memory::MemoryUsage,
    network::NetworkSample,
    processes::ProcessInfo,
    services::ServiceStatus,
    temperature::ThermalZone,
};

#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    pub cpu_percent: f64,
    pub process: ProcessInfo,
}

#[derive(Debug, Clone)]
pub struct NetworkRate {
    pub name: String,
    pub rx_bytes_per_second: u64,
    pub tx_bytes_per_second: u64,
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub cpu_usage_percent: f64,
    pub memory: MemoryUsage,
    pub filesystem: FilesystemUsage,
    pub network: Vec<NetworkRate>,
    pub network_samples: Vec<NetworkSample>,
    pub temperatures: Vec<ThermalZone>,
    pub gpu: Option<GpuSample>,
    pub services: Vec<ServiceStatus>,
    pub processes: Vec<ProcessSnapshot>,
}
