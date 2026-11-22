// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_v4: Vec<String>,
    pub ip_v6: Vec<String>,
    pub is_up: bool,
    pub speed_mbps: Option<u64>,
}

type AddressMap = HashMap<String, (Vec<String>, Vec<String>)>;

fn push_address(map: &mut AddressMap, name: &str, ip_v6: bool, address: String) {
    let entry = map.entry(name.to_string()).or_default();
    let addresses = if ip_v6 { &mut entry.1 } else { &mut entry.0 };

    if !addresses.contains(&address) {
        addresses.push(address);
    }
}

/// Reject loopback, link-local (`fe80::/10`), and deprecated site-local
/// (`fec0::/10`) addresses so the IPv6 fallback shows a routable address.
fn ipv6_is_reportable(ip: &Ipv6Addr) -> bool {
    if ip.is_loopback() {
        return false;
    }

    let first = ip.segments()[0];
    (first & 0xffc0) != 0xfe80 && (first & 0xffc0) != 0xfec0
}

#[cfg(unix)]
fn ipv4_from_sockaddr(address: *const libc::sockaddr) -> Option<Ipv4Addr> {
    let storage = unsafe { &*(address as *const libc::sockaddr_in) };
    if i32::from(storage.sin_family) != libc::AF_INET {
        return None;
    }

    // s_addr is stored in network byte order; Ipv4Addr::from expects the
    // numeric/host representation, hence the byte-order swap.
    Some(Ipv4Addr::from(u32::from_be(storage.sin_addr.s_addr)))
}

#[cfg(unix)]
fn ipv6_from_sockaddr(address: *const libc::sockaddr) -> Option<Ipv6Addr> {
    let storage = unsafe { &*(address as *const libc::sockaddr_in6) };
    if i32::from(storage.sin6_family) != libc::AF_INET6 {
        return None;
    }

    Some(Ipv6Addr::from(storage.sin6_addr.s6_addr))
}

/// Read addresses through `getifaddrs(3)`. This has no external command
/// dependency, so it also works on minimal ARM/musl systems without
/// `iproute2` installed.
#[cfg(unix)]
fn collect_addresses() -> AddressMap {
    let mut map = AddressMap::new();
    let mut list: *mut libc::ifaddrs = std::ptr::null_mut();

    if unsafe { libc::getifaddrs(&mut list) } != 0 {
        return collect_addresses_from_ip();
    }

    let mut cursor = list;
    while !cursor.is_null() {
        let entry = unsafe { &*cursor };

        if !entry.ifa_name.is_null() && !entry.ifa_addr.is_null() {
            let name = unsafe { std::ffi::CStr::from_ptr(entry.ifa_name) }
                .to_string_lossy()
                .into_owned();
            let family = i32::from(unsafe { (*entry.ifa_addr).sa_family });

            match family {
                libc::AF_INET => {
                    if let Some(ip) = ipv4_from_sockaddr(entry.ifa_addr) {
                        push_address(&mut map, &name, false, ip.to_string());
                    }
                }
                libc::AF_INET6 => {
                    if let Some(ip) = ipv6_from_sockaddr(entry.ifa_addr) {
                        if ipv6_is_reportable(&ip) {
                            push_address(&mut map, &name, true, ip.to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        cursor = unsafe { (*cursor).ifa_next };
    }

    unsafe { libc::freeifaddrs(list) };
    sort_addresses(&mut map);
    map
}

#[cfg(not(unix))]
fn collect_addresses() -> AddressMap {
    collect_addresses_from_ip()
}

fn sort_addresses(map: &mut AddressMap) {
    for (ip_v4, ip_v6) in map.values_mut() {
        ip_v4.sort();
        ip_v6.sort();
    }
}

/// Fallback for the unlikely case that `getifaddrs` fails. Requires the
/// `iproute2` command, so it is not the primary source.
fn collect_addresses_from_ip() -> AddressMap {
    let mut map: AddressMap = AddressMap::new();

    let Ok(output) = Command::new("ip").args(["-o", "addr", "show"]).output() else {
        return map;
    };

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }

        let iface = cols[1].trim_end_matches(':');

        if let Some(pos) = cols.iter().position(|&col| col == "inet") {
            if let Some(addr) = cols.get(pos + 1) {
                let ip = addr.split('/').next().unwrap_or(addr);
                push_address(&mut map, iface, false, ip.to_string());
            }
        } else if let Some(pos) = cols.iter().position(|&col| col == "inet6") {
            if let Some(addr) = cols.get(pos + 1) {
                let ip = addr.split('/').next().unwrap_or(addr);
                let reportable = ip
                    .parse::<Ipv6Addr>()
                    .map(|address| ipv6_is_reportable(&address))
                    .unwrap_or(true);
                if reportable {
                    push_address(&mut map, iface, true, ip.to_string());
                }
            }
        }
    }

    sort_addresses(&mut map);
    map
}

fn interface_names(addresses: &AddressMap) -> Vec<String> {
    let mut names = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    // /sys/class/net is always present on Linux, but getifaddrs still gives us
    // a usable list if the sysfs mount is restricted (containers, chroots).
    if names.is_empty() {
        names.extend(addresses.keys().cloned());
    }

    names.retain(|name| name != "lo");
    names.sort();
    names.dedup();
    names
}

fn is_up(name: &str) -> bool {
    fs::read_to_string(format!("/sys/class/net/{name}/operstate"))
        .map(|state| state.trim() == "up")
        .unwrap_or(false)
}

fn link_speed(name: &str) -> Option<u64> {
    let raw = fs::read_to_string(format!("/sys/class/net/{name}/speed")).ok()?;
    let value = raw.trim().parse::<i64>().ok()?;
    if value > 0 {
        Some(value as u64)
    } else {
        None
    }
}

pub fn collect() -> Vec<NetworkInterface> {
    let addresses = collect_addresses();
    let names = interface_names(&addresses);

    let mut interfaces: Vec<NetworkInterface> = names
        .into_iter()
        .map(|name| {
            let (ip_v4, ip_v6) = addresses.get(&name).cloned().unwrap_or_default();
            NetworkInterface {
                is_up: is_up(&name),
                speed_mbps: link_speed(&name),
                name,
                ip_v4,
                ip_v6,
            }
        })
        .collect();

    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    interfaces
}

pub fn format_link_speed(mbps: u64) -> String {
    if mbps >= 1000 {
        let whole_gb = mbps / 1000;
        if mbps == whole_gb * 1000 {
            format!("{whole_gb} Gb/s")
        } else {
            format!("{:.1} Gb/s", mbps as f64 / 1000.0)
        }
    } else {
        format!("{mbps} Mb/s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_link_speed() {
        assert_eq!(format_link_speed(100), "100 Mb/s");
        assert_eq!(format_link_speed(1000), "1 Gb/s");
        assert_eq!(format_link_speed(2500), "2.5 Gb/s");
        assert_eq!(format_link_speed(10000), "10 Gb/s");
    }

    #[test]
    fn test_default_interface_is_empty() {
        let iface = NetworkInterface::default();
        assert!(iface.ip_v4.is_empty());
        assert!(iface.ip_v6.is_empty());
        assert!(iface.speed_mbps.is_none());
    }

    #[test]
    fn test_ipv6_link_local_and_loopback_are_hidden() {
        let parse = |value: &str| value.parse::<Ipv6Addr>().unwrap();

        assert!(!ipv6_is_reportable(&parse("::1")));
        assert!(!ipv6_is_reportable(&parse("fe80::1")));
        assert!(!ipv6_is_reportable(&parse("febf::1")));
        assert!(!ipv6_is_reportable(&parse("fec0::1")));
        assert!(ipv6_is_reportable(&parse("2001:db8::1")));
        assert!(ipv6_is_reportable(&parse("fd00::1")));
    }

    #[test]
    fn test_push_address_deduplicates() {
        let mut map = AddressMap::new();
        push_address(&mut map, "eth0", false, String::from("192.0.2.2"));
        push_address(&mut map, "eth0", false, String::from("192.0.2.2"));
        push_address(&mut map, "eth0", false, String::from("192.0.2.3"));

        let (ip_v4, ip_v6) = map.get("eth0").expect("interface entry");
        assert_eq!(ip_v4, &["192.0.2.2", "192.0.2.3"]);
        assert!(ip_v6.is_empty());
    }
}
