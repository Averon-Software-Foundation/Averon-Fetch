# Averon Fetch

A modern, lightweight system information utility written in Rust.

Developed by [Averon Software Foundation](https://github.com/AveronSoftwareFoundation).

Averon Fetch displays detailed system information in the terminal with a clean, structured layout. It is designed as an independent alternative to tools such as Fastfetch, with its own aesthetic and configuration system built from the ground up using Rust.

---

## Features

- Native Rust implementation with no external interpreters
- Modular information system: every row is produced by an independent module
- Enable, disable, reorder, and relabel modules through TOML configuration
- Automatic logo scaling and narrow-terminal handling
- 24-bit true colour, 256-colour fallback, and plain-text output
- Respects `NO_COLOR` and never emits ANSI sequences when piped by default
- JSON output for scripting and integration
- Linux-first design with cross-platform support planned
- Cached system queries within a single execution
- Lightweight startup and minimal subprocess use

---

## Terminal Output Example

```
         ▄▄▄▄▄▄                josh@hostname
       ▄█████████▄▄            ────────────────────
      ██████  ████████         OS              : Averon Linux x86_64
     ████      ████████        Host            : B550 EAGLE
    ████        ▐████████      Kernel          : Linux 6.11.3
   ████           ████████     Uptime          : 4 hours, 23 minutes
  ████             ████████    Packages        : 1450 (pacman), 20 (flatpak)
 ████               ████████   Shell           : fish 3.7.0
 ████▄▄▄▄▄▄▄▄▄▄▄▄▄▄████████    Terminal        : konsole
  ████████████████████████     Display         : Acer XB273 GX 2560x1440 @ 165Hz
   ██████         ████████     Desktop         : KDE Plasma 6.2
    ████           ███████     WM              : KWin
     ██             ██████     Display Server  : Wayland
                   █████       CPU             : AMD Ryzen 7 5800X (8C/16T) @ 3.80 GHz
                    ████       CPU Temp        : 45.0 °C
                               GPU             : NVIDIA GeForce RTX 5060 Ti
                               Memory          : 8.51 GiB / 15.53 GiB (55%)
                               Swap            : 1.88 GiB / 15.53 GiB (12%)
                               Disk (/)        : 282.00 GiB / 620.00 GiB (45%)
                               Local IP        : 192.168.1.100 (eth0)
                               BIOS            : F14 (B550 EAGLE)
                               Locale          : en_GB.UTF-8
```

---

## Installation

### From Source

```sh
git clone https://github.com/AveronSoftwareFoundation/Averon-Fetch.git
cd Averon-Fetch
cargo build --release
sudo install -Dm755 target/release/averon-fetch /usr/local/bin/averon-fetch
```

Two binaries are built: `averon-fetch` and the shorter `av-fetch`.

### Cargo Install

```sh
cargo install averon-fetch
```

### Arch Linux - AUR (planned)

```sh
yay -S averon-fetch
```

---

## Usage

```
averon-fetch                                     Display system information
averon-fetch --help                              Show help
averon-fetch --version                           Show version
averon-fetch --logo-only                         Display only the logo
averon-fetch --logo                              Alias for --logo-only
averon-fetch --no-logo                           Display information without the logo
averon-fetch --list-modules                      List modules and enabled state
averon-fetch --module cpu                        Show only CPU information
averon-fetch --module cpu --module memory        Show CPU and memory in that order
averon-fetch --all                               Show all modules, ignoring config visibility
averon-fetch --format json                       Structured JSON output
averon-fetch --format text                       Terminal text output (default)
averon-fetch --no-color                          Disable colour
averon-fetch --color always                      Force colour, even when piped
averon-fetch --color never                       Force plain output
averon-fetch --pipe                              Plain text output (no colour)
averon-fetch --noip                              Hide local IP addresses
averon-fetch --config                            Show configuration path and contents
averon-fetch --config /path/to/config.toml       Use a specific configuration file
averon-fetch --config-path                       Print the effective configuration path
averon-fetch --generate-config                   Write a documented default configuration
averon-fetch --show-errors                       Print collector diagnostics to stderr
```

Module names are case-insensitive. Common aliases are accepted, including
`window_manager` for `wm`, `ip` and `local_ip` for `network`, `motherboard` for
`bios`, `gpu_util` for `gpu_usage`, and `vram` for `gpu_vram`.

---

## Configuration

The default configuration file is:

```
$XDG_CONFIG_HOME/averon-fetch/config.toml
```

If `XDG_CONFIG_HOME` is not set, the path is:

```
~/.config/averon-fetch/config.toml
```

Generate a documented default file with:

```sh
averon-fetch --generate-config
```

A complete example:

```toml
[general]
logo = true
colour = true
separator = ": "
show_errors = false

[logo]
width = 28
scale = 1.0
# custom_ascii = """
#   /\
#  /  \
# /____\
# """

[colours]
label = "#5B53EB"
value = "#DCDCDC"
separator = "#64D8F5"
accent = "#5B53EB"
header_user = "#DCDCDC"
header_host = "#5B53EB"
logo_primary = "#DCDCDC"
logo_accent = "#5B53EB"

[terminal]
# auto, always, or never
colour = "auto"
truecolour = true

[modules]
username = true
hostname = true
os = true
arch = true              # include the architecture suffix on the OS row
host = true
kernel = true
uptime = true
packages = true
shell = true
terminal = true
display = true
desktop = true
window_manager = true
display_server = true
theme = true
cpu = true
cpu_usage = false
cpu_temperature = true
gpu = true
gpu_usage = false
gpu_temperature = false
gpu_vram = false
memory = true
swap = true
disk = true
network = true
network_speed = false
bios = true
battery = true
audio = false
locale = true

# Modules not listed keep their built-in order after the listed ones.
order = ["os", "host", "kernel", "uptime", "cpu", "memory", "disk"]

[labels]
# Custom labels may use module ids or default labels.
os = "System"
"Local IP" = "IP"
Icons = "Icon Theme"
```

Logo width and scale are both supported. `scale = 2.0` doubles the built-in
logo, while `width = 40` sets a target width directly. A custom `custom_ascii`
block replaces the built-in mark and is used verbatim.

### Colour behaviour

- `auto` (default) uses colour only on an interactive terminal.
- `NO_COLOR`, `TERM=dumb`, and non-tty output disable colour.
- `--no-color` and `--pipe` always force plain output.
- `--color always` is the explicit opt-in that permits ANSI output when piped.
- `truecolour = false` uses a 256-colour approximation.
- If the terminal does not advertise 24-bit support, `auto` uses 256 colours.

---

## Supported Information

| Module | Information |
|--------|-------------|
| General | Username, hostname |
| OS | Distribution, architecture |
| Kernel | Linux kernel version |
| Uptime | System uptime |
| Packages | Installed package counts per manager |
| Shell | Shell name and version |
| Terminal | Terminal emulator |
| Display | Every connected monitor, resolution, refresh rate |
| Desktop | Desktop environment and version |
| WM | Window manager |
| Display Server | Wayland or X11 |
| Theme | GTK theme, icon theme, cursor theme, font |
| CPU | Model, cores, threads, frequency |
| CPU Usage | Current CPU utilisation |
| CPU Temperature | CPU temperature from hwmon or thermal zones |
| GPU | Model, type, driver |
| GPU Usage | Utilisation from sysfs or `nvidia-smi` |
| GPU Temperature | Temperature from sysfs or `nvidia-smi` |
| GPU VRAM | Video memory usage |
| Memory | RAM usage |
| Swap | Swap usage |
| Disk | Mounted disk usage per partition |
| Network | Local IP addresses |
| Network Speed | Interface link speeds from sysfs |
| BIOS | Motherboard BIOS version |
| Battery | Charge level and status |
| Audio | ALSA devices and default PulseAudio/PipeWire sink |
| Locale | System locale and keyboard layout |

Modules only emit rows when the required information is available. Missing
hardware, unsupported drivers, and unreadable files are omitted rather than
shown with placeholder values.

---

## Performance

Averon Fetch keeps work proportional to what is displayed:

- A single `sysinfo::System` snapshot is shared within an execution.
- Expensive collectors run only when a selected module needs them.
- Packages, GPU details, display, desktop, shell, locale, and audio are not
  queried when their modules are disabled.
- `nvidia-smi` is called at most once, and only when a selected GPU detail
  module needs data that sysfs could not provide.
- Network addresses are collected with one `ip` invocation.
- Terminal width is read with an ioctl instead of a `stty` subprocess.
- CPU usage uses a short `/proc/stat` sample and is disabled by default.

---

## JSON Output

```sh
averon-fetch --format json
```

```json
{
  "header": {
    "username": "josh",
    "hostname": "hostname"
  },
  "modules": [
    {
      "module": "os",
      "label": "OS",
      "value": "Averon Linux x86_64"
    },
    {
      "module": "memory",
      "label": "Memory",
      "value": "8.51 GiB / 15.53 GiB (55%)"
    }
  ]
}
```

JSON output contains no ANSI sequences and never includes logo artwork.

---

## Supported Platforms

| Platform | Status |
|----------|--------|
| Linux x86_64 | Supported |
| Linux ARM64 | Planned |
| Windows x86_64 | Planned |
| macOS ARM64 | Planned |
| macOS x86_64 | Planned |

---

## Package Manager Support

| Manager | Detection Method |
|---------|-----------------|
| pacman | `/var/lib/pacman/local/` |
| apt/dpkg | `/var/lib/dpkg/status` |
| rpm | `rpm -qa` |
| flatpak | `flatpak list` |
| snap | `/snap/` directory |

---

## Building from Source

**Requirements:** Rust 1.74 or later, Cargo

```sh
cargo build              # debug build
cargo build --release    # optimised release build
cargo test               # run tests
cargo fmt --check        # check formatting
cargo clippy -- -D warnings
```

---

## Contributing

Contributions are welcome. Please follow the Averon Software Foundation project conventions when submitting patches or pull requests. Ensure all code is formatted with `cargo fmt` and passes `cargo clippy` before submission. Open an issue to discuss significant changes before beginning work.

---

## Licence

Averon Fetch is licensed under the **GNU General Public License v3.0**.

See the [LICENSE](LICENSE) file for full terms.
