use serde::Serialize;
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub state: char,
    pub memory_bytes: u64,
    pub cpu_time: u64,
    pub command: String,
}

pub fn read_process(pid: u32) -> Option<ProcessInfo> {
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

pub fn read_processes() -> Vec<ProcessInfo> {
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

pub fn calculate_process_cpu(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_current_process() {
        let pid = std::process::id();
        let process = read_process(pid).expect("current process should be readable");

        assert_eq!(process.pid, pid);
        assert!(!process.name.is_empty());
        assert!(!process.command.is_empty());
    }

    #[test]
    fn calculates_process_cpu() {
        let previous = ProcessInfo {
            pid: 1,
            name: "test".to_string(),
            state: 'S',
            memory_bytes: 0,
            cpu_time: 100,
            command: "test".to_string(),
        };

        let current = ProcessInfo {
            cpu_time: 130,
            ..previous.clone()
        };

        let usage = calculate_process_cpu(&previous, &current, 100);

        assert!((usage - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_system_delta_returns_zero_process_cpu() {
        let process = ProcessInfo {
            pid: 1,
            name: "test".to_string(),
            state: 'S',
            memory_bytes: 0,
            cpu_time: 100,
            command: "test".to_string(),
        };

        assert_eq!(calculate_process_cpu(&process, &process, 0), 0.0);
    }

    #[test]
    fn decreasing_process_cpu_returns_zero() {
        let previous = ProcessInfo {
            pid: 1,
            name: "test".to_string(),
            state: 'S',
            memory_bytes: 0,
            cpu_time: 200,
            command: "test".to_string(),
        };

        let current = ProcessInfo {
            cpu_time: 100,
            ..previous.clone()
        };

        assert_eq!(calculate_process_cpu(&previous, &current, 100), 0.0);
    }
}
