use std::fs;

use nix::ifaddrs::getifaddrs;

#[derive(Debug, Clone)]
pub struct NetworkSample {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

pub fn read_interface_addresses() -> Vec<(String, String)> {
    let mut addresses = Vec::new();

    let Ok(interfaces) = getifaddrs() else {
        return addresses;
    };

    for interface in interfaces {
        let Some(address) = interface.address else {
            continue;
        };

        let address = if let Some(inet) = address.as_sockaddr_in() {
            inet.ip().to_string()
        } else if let Some(inet6) = address.as_sockaddr_in6() {
            inet6.ip().to_string()
        } else {
            continue;
        };

        addresses.push((interface.interface_name, address));
    }

    addresses
}

pub fn read_network_samples() -> Vec<NetworkSample> {
    let contents =
        fs::read_to_string("/proc/net/dev").expect("failed to read /proc/net/dev");

    let mut interfaces = Vec::new();

    for line in contents.lines().skip(2) {
        let Some((name, values)) = line.split_once(':') else {
            continue;
        };

        let name = name.trim().to_string();
        let fields: Vec<&str> = values.split_whitespace().collect();

        if fields.len() < 9 {
            continue;
        }

        let rx_bytes = fields[0].parse::<u64>().unwrap_or(0);
        let tx_bytes = fields[8].parse::<u64>().unwrap_or(0);

        interfaces.push(NetworkSample {
            name,
            rx_bytes,
            tx_bytes,
        });
    }

    interfaces
}
