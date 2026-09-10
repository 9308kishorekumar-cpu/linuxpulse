use std::{fs, path::Path};

#[derive(Debug, Clone, Copy)]
pub struct GpuSample {
    pub utilization_percent: f64,
    pub vram_used: u64,
    pub vram_total: u64,
}

pub fn read_amd_gpu() -> Option<GpuSample> {
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
