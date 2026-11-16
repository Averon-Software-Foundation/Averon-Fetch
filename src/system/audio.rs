use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct AudioInfo {
    pub devices: Vec<String>,
    pub default_sink: Option<String>,
}

fn parse_asound_cards(content: &str) -> Vec<String> {
    let mut devices = Vec::new();

    for line in content.lines() {
        let Some((_, rest)) = line.split_once("]: ") else {
            continue;
        };
        let rest = rest.trim();
        if rest.is_empty() {
            continue;
        }
        let name = rest
            .split_once(" - ")
            .map(|(_, name)| name.trim())
            .unwrap_or(rest);
        if !name.is_empty() && !devices.iter().any(|existing| existing == name) {
            devices.push(name.to_string());
        }
    }

    devices
}

fn parse_pactl_default_sink(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        line.trim()
            .strip_prefix("Default Sink:")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn default_sink() -> Option<String> {
    let output = Command::new("pactl").arg("info").output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_pactl_default_sink(&String::from_utf8_lossy(&output.stdout))
}

pub fn collect() -> AudioInfo {
    let devices = fs::read_to_string("/proc/asound/cards")
        .map(|content| parse_asound_cards(&content))
        .unwrap_or_default();

    AudioInfo {
        devices,
        default_sink: default_sink(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_asound_cards() {
        let content = " 0 [PCH            ]: HDA-Intel - HDA Intel PCH\n                      HDA Intel PCH at 0x irq 1\n 1 [HDMI           ]: HDA-Intel - HDA ATI HDMI\n";
        assert_eq!(
            parse_asound_cards(content),
            vec!["HDA Intel PCH", "HDA ATI HDMI"]
        );
    }

    #[test]
    fn test_parse_pactl_default_sink() {
        let content = "Server Name: PulseAudio\nDefault Sink: alsa_output.pci-0000_00_1f.3.analog-stereo\nDefault Source: alsa_input\n";
        assert_eq!(
            parse_pactl_default_sink(content),
            Some(String::from("alsa_output.pci-0000_00_1f.3.analog-stereo"))
        );
    }

    #[test]
    fn test_default_audio_is_empty() {
        let audio = AudioInfo::default();
        assert!(audio.devices.is_empty());
        assert!(audio.default_sink.is_none());
    }
}
