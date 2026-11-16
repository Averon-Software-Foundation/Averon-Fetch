use crate::config::Config;
use crate::modules::{Line, Module};
use crate::system::SystemInfo;

pub struct UsernameModule;

impl Module for UsernameModule {
    fn id(&self) -> &'static str {
        "username"
    }

    fn label(&self) -> &'static str {
        "User"
    }

    fn description(&self) -> &'static str {
        "Current username"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        match &info.general.username {
            Some(username) => vec![Line::new(self.id(), self.label(), username)],
            None => Vec::new(),
        }
    }
}

pub struct HostnameModule;

impl Module for HostnameModule {
    fn id(&self) -> &'static str {
        "hostname"
    }

    fn label(&self) -> &'static str {
        "Hostname"
    }

    fn description(&self) -> &'static str {
        "Machine hostname"
    }

    fn render(&self, info: &SystemInfo, _config: &Config) -> Vec<Line> {
        match &info.general.hostname {
            Some(hostname) => vec![Line::new(self.id(), self.label(), hostname)],
            None => Vec::new(),
        }
    }
}
