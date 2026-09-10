use serde::Serialize;
use std::fs;
use std::path::Path;

use nix::sys::statvfs::statvfs;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct DiskSample {
    pub read_sectors: u64,
    pub write_sectors: u64,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct FilesystemUsage {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

pub fn read_disk_sample() -> DiskSample {
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

pub fn read_filesystem_usage(path: &str) -> FilesystemUsage {
    let stat = statvfs(Path::new(path))
        .expect("failed to read filesystem statistics");

    let block_size = stat.fragment_size() as u64;
    let total_bytes = stat.blocks() as u64 * block_size;
    let available_bytes = stat.blocks_available() as u64 * block_size;
    let used_bytes = total_bytes.saturating_sub(available_bytes);

    let usage_percent = if total_bytes == 0 {
        0.0
    } else {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    };

    FilesystemUsage {
        total_bytes,
        used_bytes,
        available_bytes,
        usage_percent,
    }
}
