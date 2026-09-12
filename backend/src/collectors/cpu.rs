use std::fs;

#[derive(Debug, Clone, Copy)]
pub struct CpuSample {
    pub total: u64,
    pub idle: u64,
}

fn parse_cpu_line(line: &str) -> CpuSample {
    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse::<u64>().expect("invalid CPU value"))
        .collect();

    let idle = values[3] + values[4];
    let total = values.iter().sum();

    CpuSample { total, idle }
}

pub fn read_total_cpu() -> CpuSample {
    let contents = fs::read_to_string("/proc/stat").expect("failed to read /proc/stat");

    let line = contents
        .lines()
        .find(|line| line.starts_with("cpu "))
        .expect("CPU line not found");

    parse_cpu_line(line)
}

pub fn calculate_cpu_usage(previous: CpuSample, current: CpuSample) -> f64 {
    let total_delta = current.total - previous.total;
    let idle_delta = current.idle - previous.idle;

    if total_delta == 0 {
        return 0.0;
    }

    (1.0 - (idle_delta as f64 / total_delta as f64)) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_valid_cpu_sample() {
        let sample = read_total_cpu();

        assert!(sample.total > 0);
        assert!(sample.idle <= sample.total);
    }

    #[test]
    fn calculates_cpu_usage() {
        let previous = CpuSample {
            total: 100,
            idle: 40,
        };
        let current = CpuSample {
            total: 200,
            idle: 80,
        };

        let usage = calculate_cpu_usage(previous, current);

        assert!((usage - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_delta_returns_zero_usage() {
        let sample = CpuSample {
            total: 100,
            idle: 40,
        };

        assert_eq!(calculate_cpu_usage(sample, sample), 0.0);
    }
}
