use serde::Serialize;
#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub active: bool,
    pub failed: bool,
    pub status: String,
}

pub fn read_service_statuses() -> Vec<ServiceStatus> {
    let services = [
        ("NetworkManager.service", false),
        ("pipewire.service", true),
        ("pipewire-pulse.service", true),
        ("wireplumber.service", true),
    ];

    let mut statuses = Vec::new();

    for (service, user_service) in services {
        let mut command = std::process::Command::new("systemctl");

        if user_service {
            command.arg("--user");
        }

        let output = command
            .args([
                "show",
                service,
                "-p",
                "LoadState",
                "-p",
                "ActiveState",
                "-p",
                "SubState",
            ])
            .output();

        let Ok(output) = output else {
            continue;
        };

        if !output.status.success() {
            continue;
        }

        let text = String::from_utf8_lossy(&output.stdout);

        let mut load_state = "";
        let mut active_state = "";
        let mut sub_state = "";

        for line in text.lines() {
            if let Some(value) = line.strip_prefix("LoadState=") {
                load_state = value;
            } else if let Some(value) = line.strip_prefix("ActiveState=") {
                active_state = value;
            } else if let Some(value) = line.strip_prefix("SubState=") {
                sub_state = value;
            }
        }

        statuses.push(ServiceStatus {
            name: service.to_string(),
            active: active_state == "active",
            failed: load_state == "not-found" || active_state == "failed",
            status: sub_state.to_string(),
        });
    }

    statuses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_valid_service_statuses() {
        let services = read_service_statuses();

        for service in services {
            assert!(!service.name.is_empty());

            if service.active {
                assert!(!service.status.is_empty());
            }

            assert!(
                service.status.is_empty()
                    || service
                        .status
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-')
            );
        }
    }
}
