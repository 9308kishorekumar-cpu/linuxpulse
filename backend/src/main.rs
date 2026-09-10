use std::{fs, path::Path};

#[derive(Debug)]
struct ProcessInfo {
    pid: u32,
    name: String,
    state: char,
    memory_bytes: u64,
}

fn read_process(pid: u32) -> Option<ProcessInfo> {
    let proc_dir = format!("/proc/{pid}");

    let stat = fs::read_to_string(format!("{proc_dir}/stat")).ok()?;

    let open_paren = stat.find('(')?;
    let close_paren = stat.rfind(')')?;

    let name = stat[open_paren + 1..close_paren].to_string();

    let after_name = stat.get(close_paren + 2..)?;
    let mut fields = after_name.split_whitespace();

    let state = fields.next()?.chars().next()?;

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

    processes.sort_by_key(|process| std::cmp::Reverse(process.memory_bytes));

    processes
}

fn main() {
    let processes = read_processes();

    println!("Processes found: {}", processes.len());
    println!();

    for process in processes.iter().take(10) {
        println!(
            "{:<8} {:<25} state={} memory={} MB",
            process.pid,
            process.name,
            process.state,
            process.memory_bytes / 1024 / 1024
        );
    }
}
