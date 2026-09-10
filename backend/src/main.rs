use std::fs;

fn main() {
    let cpu = fs::read_to_string("/proc/stat")
        .expect("failed to read /proc/stat");

    let memory = fs::read_to_string("/proc/meminfo")
        .expect("failed to read /proc/meminfo");

    println!("=== CPU ===");
    println!(
        "{}",
        cpu.lines()
            .find(|line| line.starts_with("cpu "))
            .expect("CPU line not found")
    );

    println!("\n=== MEMORY ===");
    for line in memory.lines().take(8) {
        println!("{line}");
    }
}
