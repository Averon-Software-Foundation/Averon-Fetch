use crate::config::Config;
use crate::modules::{Line, Module};
use crate::system::network::format_link_speed;
use crate::system::SystemInfo;

pub struct NetworkModule;

impl Module for NetworkModule {
    fn id(&self) -> &'static str {
        "network"
    }

    fn label(&self) -> &'static str {
        "Local IP"
    }

    fn description(&self) -> &'static str {
        "Local IP addresses"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        let mut lines = Vec::new();

        for interface in &info.network {
            if !interface.ip_v4.is_empty() {
                for ip in &interface.ip_v4 {
                    lines.push(Line::new(
                        self.id(),
                        self.label(),
                        format!("{ip} ({})", interface.name),
                    ));
                }
            } else if let Some(ip) = interface.ip_v6.first() {
                lines.push(Line::new(
                    self.id(),
                    self.label(),
                    format!("{ip} ({})", interface.name),
                ));
            }
        }

        lines
    }
}

pub struct NetworkSpeedModule;

impl Module for NetworkSpeedModule {
    fn id(&self) -> &'static str {
        "network_speed"
    }

    fn label(&self) -> &'static str {
        "Network"
    }

    fn description(&self) -> &'static str {
        "Network interface link speeds"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        info.network
            .iter()
            .filter_map(|interface| {
                interface.speed_mbps.map(|speed| {
                    Line::with_suffix(
                        self.id(),
                        self.label(),
                        interface.name.clone(),
                        format_link_speed(speed),
                    )
                })
            })
            .collect()
    }
}
